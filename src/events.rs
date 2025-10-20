// Copyright 2025 Memophor Labs
// SPDX-License-Identifier: Apache-2.0

//! Event bus integration for graph-aware cache invalidation.
//!
//! Listens to Redis Pub/Sub events from SynaGraph for intelligent cache invalidation:
//! - SUPERSEDED_BY: Invalidate artifacts with old provenance hashes
//! - REVOKE_CAPSULE: Remove all artifacts from a revoked knowledge capsule
//! - INVALIDATE_TENANT: Clear all cache entries for a tenant
//! - UPDATE_TTL: Adjust TTL for matching artifacts

use futures_util::stream::StreamExt;
use redis::aio::Connection;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::cache::Cache;
use crate::error::AppError;

/// Event types from SynaGraph
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GraphEvent {
    /// An artifact has been superseded by a newer version
    SupersededBy {
        old_hash: String,
        new_hash: String,
        tenant: String,
    },
    /// Revoke/delete a capsule and all its artifacts
    RevokeCapsule {
        capsule_id: String,
        tenant: String,
    },
    /// Invalidate all artifacts for a tenant
    InvalidateTenant {
        tenant: String,
    },
    /// Update TTL for artifacts matching a pattern
    UpdateTtl {
        pattern: String,
        tenant: String,
        new_ttl_seconds: u64,
    },
}

/// Event bus configuration
#[derive(Clone)]
pub struct EventBusConfig {
    pub redis_url: String,
    pub channel: String,
}

impl Default for EventBusConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://127.0.0.1:6379".to_string(),
            channel: "scedge:events".to_string(),
        }
    }
}

/// Event bus for receiving invalidation events from SynaGraph
pub struct EventBus {
    config: EventBusConfig,
    cache: Cache,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl EventBus {
    pub fn new(config: EventBusConfig, cache: Cache) -> Self {
        Self {
            config,
            cache,
            shutdown_tx: None,
        }
    }

    /// Start listening for events
    pub async fn start(&mut self) -> Result<(), AppError> {
        let client = redis::Client::open(self.config.redis_url.as_str())
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create Redis client: {}", e)))?;

        let conn = client.get_async_connection().await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to connect to Redis: {}", e)))?;

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        let cache = self.cache.clone();
        let channel = self.config.channel.clone();

        tokio::spawn(async move {
            if let Err(e) = Self::listen_loop(conn, channel, cache, &mut shutdown_rx).await {
                tracing::error!(error = %e, "Event bus listener error");
            }
        });

        tracing::info!(channel = %self.config.channel, "Event bus started");
        Ok(())
    }

    async fn listen_loop(
        conn: Connection,
        channel: String,
        cache: Cache,
        shutdown_rx: &mut mpsc::Receiver<()>,
    ) -> Result<(), AppError> {
        let mut pubsub = conn.into_pubsub();
        pubsub.subscribe(&channel).await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to subscribe: {}", e)))?;

        let mut msg_stream = pubsub.on_message();

        loop {
            tokio::select! {
                msg = msg_stream.next() => {
                    if let Some(msg) = msg {
                        let payload: String = msg.get_payload()
                            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to get payload: {}", e)))?;

                        if let Err(e) = Self::handle_event(&payload, &cache).await {
                            tracing::error!(error = %e, payload, "Failed to handle event");
                        }
                    }
                }
                _ = shutdown_rx.recv() => {
                    tracing::info!("Event bus shutting down");
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_event(payload: &str, cache: &Cache) -> Result<(), AppError> {
        let event: GraphEvent = serde_json::from_str(payload)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to parse event: {}", e)))?;

        match event {
            GraphEvent::SupersededBy { old_hash, new_hash, tenant } => {
                tracing::info!(
                    old_hash,
                    new_hash,
                    tenant,
                    "Handling SUPERSEDED_BY event"
                );

                // Find all artifacts with the old hash and purge them
                let pattern = format!("{}:*", tenant);
                let keys = cache.scan_by_pattern(&pattern).await?;

                let mut purged = 0;
                for key in keys {
                    if let Ok(Some(artifact)) = cache.get(&key).await {
                        // Check if any provenance hash matches
                        let has_old_hash = artifact.artifact.provenance.iter()
                            .any(|p| p.hash.as_deref() == Some(&old_hash));

                        if has_old_hash || artifact.artifact.hash == old_hash {
                            cache.delete(&key).await?;
                            purged += 1;
                        }
                    }
                }

                tracing::info!(purged, "Purged artifacts with superseded hash");
            }

            GraphEvent::RevokeCapsule { capsule_id, tenant } => {
                tracing::info!(capsule_id, tenant, "Handling REVOKE_CAPSULE event");

                // Purge all artifacts related to this capsule
                let pattern = format!("{}:*", tenant);
                let keys = cache.scan_by_pattern(&pattern).await?;

                let mut purged = 0;
                for key in keys {
                    if let Ok(Some(artifact)) = cache.get(&key).await {
                        // Check if any provenance source contains the capsule_id
                        let has_capsule = artifact.artifact.provenance.iter()
                            .any(|p| p.source.contains(&capsule_id));

                        if has_capsule {
                            cache.delete(&key).await?;
                            purged += 1;
                        }
                    }
                }

                tracing::info!(purged, "Purged artifacts for revoked capsule");
            }

            GraphEvent::InvalidateTenant { tenant } => {
                tracing::info!(tenant, "Handling INVALIDATE_TENANT event");

                // Purge all artifacts for this tenant
                let pattern = format!("{}:*", tenant);
                let keys = cache.scan_by_pattern(&pattern).await?;

                let purged = cache.delete_many(&keys).await?;
                tracing::info!(purged, "Purged all artifacts for tenant");
            }

            GraphEvent::UpdateTtl { pattern, tenant, new_ttl_seconds } => {
                tracing::info!(
                    pattern,
                    tenant,
                    new_ttl_seconds,
                    "Handling UPDATE_TTL event"
                );

                // This would require re-storing artifacts with new TTL
                // For now, just log it
                tracing::warn!("UPDATE_TTL not fully implemented yet");
            }
        }

        Ok(())
    }

    /// Stop the event bus
    pub async fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
    }
}

/// Publish an event to the event bus (for testing or internal use)
pub async fn publish_event(redis_url: &str, channel: &str, event: &GraphEvent) -> Result<(), AppError> {
    let client = redis::Client::open(redis_url)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create Redis client: {}", e)))?;

    let mut conn = client.get_multiplexed_async_connection().await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to connect to Redis: {}", e)))?;

    let payload = serde_json::to_string(event)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to serialize event: {}", e)))?;

    conn.publish(channel, payload).await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to publish event: {}", e)))?;

    Ok(())
}
