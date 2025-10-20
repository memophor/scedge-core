// Copyright 2025 Memophor Labs
// SPDX-License-Identifier: Apache-2.0

//! Configuration management for Scedge Core.
//!
//! Loads configuration from environment variables and files. Supports:
//! - Server binding configuration
//! - Redis connection settings
//! - TTL defaults
//! - Tenant authentication
//! - Feature flags (metrics, event bus)

use std::env;
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::policy::TenantConfig;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub listen_addr: SocketAddr,
    pub default_ttl: Duration,
    pub redis_url: String,
    pub tenant_keys_path: Option<PathBuf>,
    pub jwt_secret: Option<String>,
    pub event_bus_enabled: bool,
    pub event_bus_channel: String,
    pub event_bus_url: String,
    pub metrics_enabled: bool,
    pub upstream: Option<UpstreamConfig>,
}

#[derive(Debug, Deserialize)]
struct TenantsFile {
    tenants: Vec<TenantConfig>,
}

#[derive(Debug, Clone)]
pub struct UpstreamConfig {
    pub base_url: String,
    pub timeout: Duration,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        let listen_addr: SocketAddr = env::var("SCEDGE_ADDR")
            .or_else(|_| env::var("SCEDGE_PORT").map(|p| format!("0.0.0.0:{}", p)))
            .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
            .parse()
            .context("invalid SCEDGE_ADDR or SCEDGE_PORT")?;

        let default_ttl = parse_duration("SCEDGE_DEFAULT_TTL", 86400)?;

        let redis_url =
            env::var("SCEDGE_REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        let tenant_keys_path = env::var("SCEDGE_TENANT_KEYS_PATH").ok().map(PathBuf::from);

        let jwt_secret = env::var("SCEDGE_JWT_SECRET").ok();

        let event_bus_enabled = env::var("SCEDGE_EVENT_BUS_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        let event_bus_channel =
            env::var("SCEDGE_EVENT_BUS_CHANNEL").unwrap_or_else(|_| "synagraph.cache".to_string());

        let event_bus_url = env::var("SCEDGE_EVENT_BUS_URL")
            .unwrap_or_else(|_| "nats://127.0.0.1:4222".to_string());

        let metrics_enabled = env::var("SCEDGE_METRICS_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        let upstream = match env::var("SCEDGE_UPSTREAM_URL") {
            Ok(url) if !url.trim().is_empty() => {
                let timeout = parse_duration("SCEDGE_UPSTREAM_TIMEOUT_SECS", 5)?;
                Some(UpstreamConfig {
                    base_url: url,
                    timeout,
                })
            }
            _ => None,
        };

        Ok(Self {
            listen_addr,
            default_ttl,
            redis_url,
            tenant_keys_path,
            jwt_secret,
            event_bus_enabled,
            event_bus_channel,
            event_bus_url,
            metrics_enabled,
            upstream,
        })
    }

    pub fn listen_addr(&self) -> SocketAddr {
        self.listen_addr
    }

    pub fn default_ttl(&self) -> Duration {
        self.default_ttl
    }

    /// Load tenant configurations from file
    pub fn load_tenants(&self) -> Result<Vec<TenantConfig>> {
        if let Some(path) = &self.tenant_keys_path {
            let content = fs::read_to_string(path)
                .with_context(|| format!("Failed to read tenant keys file: {:?}", path))?;

            let file: TenantsFile = serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse tenant keys file: {:?}", path))?;

            Ok(file.tenants)
        } else {
            Ok(Vec::new())
        }
    }
}

fn parse_duration(env_key: &str, default_secs: u64) -> Result<Duration> {
    let raw = env::var(env_key).unwrap_or_else(|_| default_secs.to_string());
    let secs: u64 = raw
        .parse()
        .with_context(|| format!("{env_key} must be an integer number of seconds"))?;

    Ok(Duration::from_secs(secs))
}
