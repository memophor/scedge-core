use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use anyhow::{Context, Result};

pub struct AppConfig {
    pub listen_addr: SocketAddr,
    pub default_ttl: Duration,
    pub janitor_interval: Duration,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        let listen_addr: SocketAddr = env::var("SCEDGE_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:9090".to_string())
            .parse()
            .context("invalid SCEDGE_ADDR")?;

        let default_ttl = parse_duration("SCEDGE_DEFAULT_TTL", 300)?;
        let janitor_interval = parse_duration("SCEDGE_JANITOR_SECONDS", 30)?;

        Ok(Self {
            listen_addr,
            default_ttl,
            janitor_interval,
        })
    }

    pub fn listen_addr(&self) -> SocketAddr {
        self.listen_addr
    }

    pub fn default_ttl(&self) -> Duration {
        self.default_ttl
    }

    pub fn janitor_interval(&self) -> Duration {
        self.janitor_interval
    }
}

fn parse_duration(env_key: &str, default_secs: u64) -> Result<Duration> {
    let raw = env::var(env_key).unwrap_or_else(|_| default_secs.to_string());
    let secs: u64 = raw
        .parse()
        .with_context(|| format!("{env_key} must be an integer number of seconds"))?;

    Ok(Duration::from_secs(secs))
}
