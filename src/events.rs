// Copyright 2025 Memophor Labs
// SPDX-License-Identifier: Apache-2.0

//! Event bus integration for graph-aware cache invalidation.
//!
//! Listens to Redis Pub/Sub events from SynaGraph for intelligent cache invalidation:
//! - SUPERSEDED_BY: Invalidate artifacts with old provenance hashes
//! - REVOKE_CAPSULE: Remove all artifacts from a revoked knowledge capsule
//! - INVALIDATE_TENANT: Clear all cache entries for a tenant
//! - UPDATE_TTL: Adjust TTL for matching artifacts

use async_nats::{Client, Subscriber};
use futures_util::StreamExt;
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
    RevokeCapsule { capsule_id: String, tenant: String },
    /// Invalidate all artifacts for a tenant
    InvalidateTenant { tenant: String },
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
    pub url: String,
    pub channel: String,
}

impl Default for EventBusConfig {
    fn default() -> Self {
        Self {
            url: "nats://127.0.0.1:4222".to_string(),
            channel: "synagraph.cache".to_string(),
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
    pub async fn start(&mut self) -> Result<mpsc::Sender<()>, AppError> {
        let client = async_nats::connect(self.config.url.as_str())
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to connect to NATS: {}", e)))?;

        let subscriber = client
            .subscribe(self.config.channel.clone())
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to subscribe: {}", e)))?;

        let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx.clone());

        let cache = self.cache.clone();
        let subject = self.config.channel.clone();

        tokio::spawn(async move {
            if let Err(e) = Self::listen_loop(client, subscriber, cache, shutdown_rx).await {
                tracing::error!(error = %e, subject = %subject, "Event bus listener error");
            }
        });

        tracing::info!(subject = %self.config.channel, "Event bus started");
        Ok(shutdown_tx)
    }

    async fn listen_loop(
        client: Client,
        mut subscriber: Subscriber,
        cache: Cache,
        mut shutdown_rx: mpsc::Receiver<()>,
    ) -> Result<(), AppError> {
        let _client_guard = client;

        loop {
            tokio::select! {
                maybe_msg = subscriber.next() => {
                    match maybe_msg {
                        Some(msg) => {
                            let payload_bytes = msg.payload;
                            let payload = match std::str::from_utf8(&payload_bytes) {
                                Ok(text) => text,
                                Err(error) => {
                                    tracing::error!(%error, "Received non-UTF8 event payload");
                                    continue;
                                }
                            };

                            if let Err(err) = Self::handle_event(payload, &cache).await {
                                tracing::error!(error = %err, payload, "Failed to handle event");
                            }
                        }
                        None => {
                            tracing::warn!("Event bus subscription closed");
                            break;
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
            GraphEvent::SupersededBy {
                old_hash,
                new_hash,
                tenant,
            } => {
                tracing::info!(old_hash, new_hash, tenant, "Handling SUPERSEDED_BY event");

                // Find all artifacts with the old hash and purge them
                let pattern = format!("{}:*", tenant);
                let keys = cache.scan_by_pattern(&pattern).await?;

                let mut purged = 0;
                for key in keys {
                    if let Ok(Some(artifact)) = cache.get(&key).await {
                        // Check if any provenance hash matches
                        let has_old_hash = artifact
                            .artifact
                            .provenance
                            .iter()
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
                        let has_capsule = artifact
                            .artifact
                            .provenance
                            .iter()
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

            GraphEvent::UpdateTtl {
                pattern,
                tenant,
                new_ttl_seconds,
            } => {
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
pub async fn publish_event(
    bus_url: &str,
    channel: &str,
    event: &GraphEvent,
) -> Result<(), AppError> {
    let client = async_nats::connect(bus_url)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to connect to NATS: {}", e)))?;

    let payload = serde_json::to_string(event)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to serialize event: {}", e)))?;

    let subject = channel.to_string();

    client
        .publish(subject, payload.into_bytes().into())
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to publish event: {}", e)))?;

    client
        .flush()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to flush NATS client: {}", e)))?;

    Ok(())
}
