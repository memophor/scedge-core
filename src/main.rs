mod config;
mod error;
mod model;
mod state;

use std::time::Duration;

use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use config::AppConfig;
use error::AppError;
use model::{LookupQuery, LookupResponse, PurgeRequest, PurgeResponse, StoreRequest, StoreResponse, StoreStatus};
use state::{CacheState, CacheWriteOutcome};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let cfg = AppConfig::from_env()?;
    let state = CacheState::new(cfg.default_ttl());

    spawn_janitor(state.clone(), cfg.janitor_interval());

    let app = Router::new()
        .route("/lookup", get(handle_lookup))
        .route("/store", post(handle_store))
        .route("/purge", post(handle_purge))
        .with_state(state.clone());

    let listen_addr = cfg.listen_addr();
    let listener = tokio::net::TcpListener::bind(listen_addr).await?;

    tracing::info!(%listen_addr, "starting scedge edge cache");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("scedge exited cleanly");

    Ok(())
}

async fn handle_store(
    State(state): State<CacheState>,
    Json(request): Json<StoreRequest>,
) -> Result<Json<StoreResponse>, AppError> {
    if request.key.trim().is_empty() {
        return Err(AppError::bad_request("key is required"));
    }

    if request.artifact.etag.trim().is_empty() {
        return Err(AppError::bad_request("artifact etag is required"));
    }

    let StoreRequest { key, artifact } = request;
    let CacheWriteOutcome { record, created } = state.set(key, artifact).await;

    let response = StoreResponse {
        key: record.key,
        status: if created {
            StoreStatus::Created
        } else {
            StoreStatus::Updated
        },
        etag: record.artifact.etag.clone(),
        expires_at: record.expires_at,
    };

    Ok(Json(response))
}

async fn handle_lookup(
    State(state): State<CacheState>,
    Query(query): Query<LookupQuery>,
) -> Result<Json<LookupResponse>, AppError> {
    if query.key.trim().is_empty() {
        return Err(AppError::bad_request("key query parameter is required"));
    }

    let Some(record) = state.get(&query.key).await else {
        return Err(AppError::not_found("cache miss"));
    };

    let now = chrono::Utc::now();
    let ttl_remaining = record.ttl_remaining_seconds(now);

    let response = LookupResponse {
        key: record.key,
        artifact: record.artifact,
        expires_at: record.expires_at,
        ttl_remaining_seconds: ttl_remaining,
    };

    Ok(Json(response))
}

async fn handle_purge(
    State(state): State<CacheState>,
    Json(request): Json<PurgeRequest>,
) -> Result<Json<PurgeResponse>, AppError> {
    if request.keys.is_empty() {
        return Err(AppError::bad_request("keys cannot be empty"));
    }

    let purged = state.purge(&request.keys).await;

    Ok(Json(PurgeResponse { purged }))
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

fn spawn_janitor(state: CacheState, interval: Duration) {
    if interval.is_zero() {
        tracing::warn!("janitor interval disabled; expired entries will linger");
        return;
    }

    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        loop {
            ticker.tick().await;
            let purged = state.purge_expired().await;
            if purged > 0 {
                tracing::debug!(purged, "purged expired artifacts");
            }
        }
    });
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::warn!(%error, "failed to install Ctrl+C handler");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        match signal(SignalKind::terminate()) {
            Ok(mut term_signal) => term_signal.recv().await,
            Err(error) => {
                tracing::warn!(%error, "failed to install SIGTERM handler");
                None
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("shutdown signal received");
}
