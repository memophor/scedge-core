// Copyright 2025 Memophor Labs
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! HTTP API handlers for Scedge Core.
//!
//! This module implements all REST API endpoints for the caching service:
//!
//! - `GET /healthz` - Service health check
//! - `GET /metrics` - Prometheus metrics export
//! - `GET /lookup` - Retrieve cached artifacts
//! - `POST /store` - Store new artifacts
//! - `POST /purge` - Remove cached artifacts
//!
//! All handlers enforce tenant isolation, policy validation, and observability.

use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::Json;
use chrono::{Duration, Utc};
use tokio::time::Instant;

use crate::cache::Cache;
use crate::error::AppError;
use crate::metrics::Metrics;
use crate::model::{
    LookupQuery, LookupResponse, PurgeRequest, PurgeResponse, StoreRequest, StoreResponse,
    StoreStatus,
};
use crate::policy::PolicyEngine;
use crate::upstream::UpstreamClient;

#[derive(Clone)]
pub struct AppState {
    pub cache: Cache,
    pub metrics: Metrics,
    pub policy: PolicyEngine,
    pub default_ttl_seconds: u64,
    pub upstream: Option<UpstreamClient>,
}

/// Health check endpoint
pub async fn health() -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "status": "healthy",
        "service": "scedge-core",
        "version": env!("CARGO_PKG_VERSION"),
    })))
}

/// Metrics endpoint
pub async fn metrics(State(state): State<AppState>) -> Result<String, AppError> {
    state.metrics.export()
}

/// Store an artifact in the cache
pub async fn handle_store(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<StoreRequest>,
) -> Result<Json<StoreResponse>, AppError> {
    // Validate inputs
    if request.key.trim().is_empty() {
        return Err(AppError::bad_request("key is required"));
    }

    if request.artifact.hash.trim().is_empty() {
        return Err(AppError::bad_request("artifact hash is required"));
    }

    let tenant_id = &request.artifact.policy.tenant;

    // Validate API key if provided
    if let Some(api_key) = headers.get("x-api-key").and_then(|h| h.to_str().ok()) {
        state.policy.validate_api_key(tenant_id, api_key).await?;
    }

    // Validate TTL against tenant limits
    state
        .policy
        .validate_ttl(tenant_id, request.artifact.ttl_seconds)
        .await?;

    // Validate region access
    state
        .policy
        .validate_region(tenant_id, request.artifact.policy.region.as_deref())
        .await?;

    // Validate compliance requirements
    state
        .policy
        .validate_compliance(
            tenant_id,
            request.artifact.policy.phi,
            request.artifact.policy.pii,
        )
        .await?;

    // Calculate expiration
    let ttl_seconds = request
        .artifact
        .ttl_seconds
        .unwrap_or(state.default_ttl_seconds);
    let expires_at = if ttl_seconds > 0 {
        Some(Utc::now() + Duration::seconds(ttl_seconds as i64))
    } else {
        None
    };

    // Store in cache
    let cached = state
        .cache
        .set(request.key.clone(), request.artifact, expires_at)
        .await?;

    // Record metrics
    state.metrics.record_cache_store();

    let response = StoreResponse {
        key: cached.key,
        status: StoreStatus::Created,
        hash: cached.artifact.hash.clone(),
        expires_at: cached.expires_at,
    };

    Ok(Json(response))
}

/// Lookup an artifact from the cache
pub async fn handle_lookup(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<LookupQuery>,
) -> Result<Json<LookupResponse>, AppError> {
    if query.key.trim().is_empty() {
        return Err(AppError::bad_request("key query parameter is required"));
    }

    // Attempt to get from cache
    match state.cache.get(&query.key).await? {
        Some(record) => {
            let tenant_id = &record.artifact.policy.tenant;

            // If tenant is specified in query, validate it matches
            if let Some(requested_tenant) = &query.tenant {
                if requested_tenant != tenant_id {
                    state.metrics.record_cache_miss();
                    return Err(AppError::not_found("cache miss"));
                }
            }

            // Validate API key if provided
            if let Some(api_key) = headers.get("x-api-key").and_then(|h| h.to_str().ok()) {
                state.policy.validate_api_key(tenant_id, api_key).await?;
            }

            state.metrics.record_cache_hit();

            let now = Utc::now();
            let ttl_remaining = record.ttl_remaining_seconds(now);

            let response = LookupResponse {
                key: record.key,
                artifact: record.artifact,
                expires_at: record.expires_at,
                ttl_remaining_seconds: ttl_remaining,
            };

            Ok(Json(response))
        }
        None => {
            state.metrics.record_cache_miss();
            if let Some(upstream) = &state.upstream {
                state.metrics.record_upstream_request();
                let start = Instant::now();

                match upstream.lookup(&query.key, query.tenant.as_deref()).await {
                    Ok(Some(upstream_record)) => {
                        state
                            .metrics
                            .record_upstream_latency(start.elapsed().as_secs_f64());

                        let tenant_id = &upstream_record.artifact.policy.tenant;

                        if let Some(requested_tenant) = &query.tenant {
                            if requested_tenant != tenant_id {
                                tracing::warn!(
                                    requested = %requested_tenant,
                                    upstream = %tenant_id,
                                    key = %query.key,
                                    "Tenant mismatch between request and upstream response",
                                );
                                state.metrics.record_upstream_failure();
                                return Err(AppError::not_found("cache miss"));
                            }
                        }

                        if let Some(api_key) =
                            headers.get("x-api-key").and_then(|h| h.to_str().ok())
                        {
                            state.policy.validate_api_key(tenant_id, api_key).await?;
                        }

                        let mut expires_at = upstream_record.expires_at;

                        if expires_at.is_none() {
                            if let Some(ttl_remaining) = upstream_record.ttl_remaining_seconds {
                                if ttl_remaining > 0 {
                                    expires_at =
                                        Some(Utc::now() + Duration::seconds(ttl_remaining as i64));
                                }
                            }
                        }

                        if expires_at.is_none() {
                            if let Some(ttl) = upstream_record.artifact.ttl_seconds {
                                if ttl > 0 {
                                    expires_at = Some(Utc::now() + Duration::seconds(ttl as i64));
                                }
                            }
                        }

                        if expires_at.is_none() && state.default_ttl_seconds > 0 {
                            expires_at = Some(
                                Utc::now() + Duration::seconds(state.default_ttl_seconds as i64),
                            );
                        }

                        let cached = state
                            .cache
                            .set(query.key.clone(), upstream_record.artifact, expires_at)
                            .await?;

                        state.metrics.record_cache_store();
                        tracing::debug!(key = %cached.key, "cached artifact from upstream");

                        let now = Utc::now();
                        let ttl_remaining = cached.ttl_remaining_seconds(now);

                        let response = LookupResponse {
                            key: cached.key,
                            artifact: cached.artifact,
                            expires_at: cached.expires_at,
                            ttl_remaining_seconds: ttl_remaining,
                        };

                        return Ok(Json(response));
                    }
                    Ok(None) => {
                        state
                            .metrics
                            .record_upstream_latency(start.elapsed().as_secs_f64());
                    }
                    Err(err) => {
                        state.metrics.record_upstream_failure();
                        state
                            .metrics
                            .record_upstream_latency(start.elapsed().as_secs_f64());
                        return Err(err);
                    }
                }
            }

            Err(AppError::not_found("cache miss"))
        }
    }
}

/// Purge artifacts from the cache
pub async fn handle_purge(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<PurgeRequest>,
) -> Result<Json<PurgeResponse>, AppError> {
    let purged;

    // Validate API key for tenant if specified
    if let Some(tenant_id) = &request.tenant {
        if let Some(api_key) = headers.get("x-api-key").and_then(|h| h.to_str().ok()) {
            state.policy.validate_api_key(tenant_id, api_key).await?;
        }
    }

    // Purge by explicit keys
    if !request.keys.is_empty() {
        purged = state.cache.delete_many(&request.keys).await?;
    }
    // Purge by tenant
    else if let Some(tenant_id) = &request.tenant {
        let pattern = format!("{}:*", tenant_id);
        let keys = state.cache.scan_by_pattern(&pattern).await?;
        purged = state.cache.delete_many(&keys).await?;
    }
    // Purge by provenance hash
    else if let Some(prov_hash) = &request.provenance_hash {
        // Scan all keys and check provenance
        let keys = state.cache.scan_by_pattern("*").await?;
        let mut to_purge = Vec::new();

        for key in keys {
            if let Ok(Some(artifact)) = state.cache.get(&key).await {
                let has_hash = artifact
                    .artifact
                    .provenance
                    .iter()
                    .any(|p| p.hash.as_deref() == Some(prov_hash.as_str()))
                    || artifact.artifact.hash == *prov_hash;

                if has_hash {
                    to_purge.push(key);
                }
            }
        }

        purged = state.cache.delete_many(&to_purge).await?;
    } else {
        return Err(AppError::bad_request(
            "must specify keys, tenant, or provenance_hash",
        ));
    }

    state.metrics.record_cache_purge(purged);

    Ok(Json(PurgeResponse { purged }))
}
