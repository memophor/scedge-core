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

//! Scedge Core - Smart Cache on the Edge
//!
//! This is the main entrypoint for the Scedge Core service, a policy-aware edge cache
//! designed for distributed AI systems. It provides semantic caching of knowledge artifacts
//! with sub-50ms latency while reducing GPU compute costs.
//!
//! # Architecture
//!
//! Scedge Core consists of:
//! - **Cache Layer**: Redis-backed storage with pluggable backend trait
//! - **Policy Engine**: Multi-tenant authentication and authorization
//! - **Event Bus**: Real-time invalidation from graph updates
//! - **Metrics**: Prometheus-compatible observability
//! - **REST API**: HTTP endpoints for artifact lifecycle management
//!
//! # Usage
//!
//! Configure via environment variables (see .env.example) and run:
//!
//! ```bash
//! cargo run
//! ```
//!
//! See QUICKSTART.md for detailed setup instructions.

mod api;
mod cache;
mod config;
mod error;
mod events;
mod metrics;
mod model;
mod policy;

use axum::routing::{get, post};
use axum::Router;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use crate::api::{handle_lookup, handle_purge, handle_store, health, metrics as metrics_handler, AppState};
use crate::cache::{Cache, RedisCache};
use crate::config::AppConfig;
use crate::events::{EventBus, EventBusConfig};
use crate::metrics::Metrics;
use crate::policy::PolicyEngine;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env file if it exists
    dotenvy::dotenv().ok();

    // Initialize tracing/logging
    init_tracing();

    tracing::info!("Starting Scedge Core v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = AppConfig::from_env()?;
    tracing::info!(
        redis_url = %config.redis_url,
        listen_addr = %config.listen_addr,
        "Configuration loaded"
    );

    // Initialize Redis cache
    tracing::info!("Connecting to Redis...");
    let redis_cache = RedisCache::new(&config.redis_url)?;
    redis_cache.ping().await?;
    tracing::info!("Redis connection established");

    let cache = Cache::new(redis_cache);

    // Initialize metrics
    let metrics = if config.metrics_enabled {
        tracing::info!("Metrics enabled");
        Metrics::new()?
    } else {
        tracing::info!("Metrics disabled");
        Metrics::default()
    };

    // Initialize policy engine
    let policy_engine = PolicyEngine::new(config.jwt_secret.clone());

    // Load tenant configurations
    match config.load_tenants() {
        Ok(tenants) => {
            if tenants.is_empty() {
                tracing::warn!("No tenant configurations loaded - API key validation will fail");
            } else {
                tracing::info!(count = tenants.len(), "Loading tenant configurations");
                for tenant in tenants {
                    tracing::debug!(tenant_id = %tenant.tenant_id, "Loaded tenant");
                    policy_engine.add_tenant(tenant).await;
                }
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "Failed to load tenant configurations - continuing without tenant auth");
        }
    }

    // Initialize event bus
    if config.event_bus_enabled {
        tracing::info!(channel = %config.event_bus_channel, "Starting event bus");
        let event_config = EventBusConfig {
            redis_url: config.redis_url.clone(),
            channel: config.event_bus_channel.clone(),
        };
        let mut event_bus = EventBus::new(event_config, cache.clone());
        event_bus.start().await?;
    } else {
        tracing::info!("Event bus disabled");
    }

    // Create application state
    let state = AppState {
        cache: cache.clone(),
        metrics: metrics.clone(),
        policy: policy_engine,
        default_ttl_seconds: config.default_ttl().as_secs(),
    };

    // Build router
    let app = Router::new()
        .route("/healthz", get(health))
        .route("/health", get(health))
        .route("/metrics", get(metrics_handler))
        .route("/lookup", get(handle_lookup))
        .route("/store", post(handle_store))
        .route("/purge", post(handle_purge))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Start server
    let listen_addr = config.listen_addr();
    let listener = tokio::net::TcpListener::bind(listen_addr).await?;

    tracing::info!(%listen_addr, "Scedge Core is running");
    tracing::info!("Endpoints:");
    tracing::info!("  GET  /healthz        - Health check");
    tracing::info!("  GET  /metrics        - Prometheus metrics");
    tracing::info!("  GET  /lookup?key=... - Lookup artifact");
    tracing::info!("  POST /store          - Store artifact");
    tracing::info!("  POST /purge          - Purge artifacts");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Scedge Core shut down cleanly");

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .init();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::warn!(%error, "Failed to install Ctrl+C handler");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        match signal(SignalKind::terminate()) {
            Ok(mut term_signal) => {
                term_signal.recv().await;
            }
            Err(error) => {
                tracing::warn!(%error, "Failed to install SIGTERM handler");
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C signal");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM signal");
        },
    }

    tracing::info!("Initiating graceful shutdown...");
}
