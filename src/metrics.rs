// Copyright 2025 Memophor Labs
// SPDX-License-Identifier: Apache-2.0

//! Prometheus metrics collection for Scedge Core.
//!
//! Tracks cache performance, request patterns, and system health.

use prometheus::{Counter, Histogram, HistogramOpts, IntCounter, IntGauge, Opts, Registry};
use std::sync::Arc;

use crate::error::AppError;

/// Metrics collector for Scedge
#[derive(Clone)]
pub struct Metrics {
    pub registry: Arc<Registry>,

    // Cache metrics
    pub cache_hits: IntCounter,
    pub cache_misses: IntCounter,
    pub cache_stores: IntCounter,
    pub cache_purges: IntCounter,
    pub cache_size: IntGauge,

    // Request metrics
    pub requests_total: Counter,
    pub request_duration: Histogram,

    // Upstream hydration metrics
    pub upstream_requests: IntCounter,
    pub upstream_failures: IntCounter,
    pub upstream_latency: Histogram,

    // Artifact metrics
    pub artifacts_stored: IntCounter,
    pub artifacts_expired: IntCounter,
}

impl Metrics {
    pub fn new() -> Result<Self, AppError> {
        let registry = Registry::new();

        // Cache hit/miss counters
        let cache_hits = IntCounter::with_opts(Opts::new(
            "scedge_cache_hits_total",
            "Total number of cache hits",
        ))
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create metric: {}", e)))?;

        let cache_misses = IntCounter::with_opts(Opts::new(
            "scedge_cache_misses_total",
            "Total number of cache misses",
        ))
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create metric: {}", e)))?;

        let cache_stores = IntCounter::with_opts(Opts::new(
            "scedge_cache_stores_total",
            "Total number of cache stores",
        ))
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create metric: {}", e)))?;

        let cache_purges = IntCounter::with_opts(Opts::new(
            "scedge_cache_purges_total",
            "Total number of cache purges",
        ))
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create metric: {}", e)))?;

        let cache_size = IntGauge::with_opts(Opts::new(
            "scedge_cache_size",
            "Current number of cached artifacts",
        ))
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create metric: {}", e)))?;

        // Request metrics
        let requests_total = Counter::with_opts(Opts::new(
            "scedge_requests_total",
            "Total number of HTTP requests",
        ))
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create metric: {}", e)))?;

        let request_duration = Histogram::with_opts(
            HistogramOpts::new(
                "scedge_request_duration_seconds",
                "HTTP request duration in seconds",
            )
            .buckets(vec![
                0.001, 0.005, 0.010, 0.025, 0.050, 0.100, 0.250, 0.500, 1.0,
            ]),
        )
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create metric: {}", e)))?;

        // Upstream hydration metrics
        let upstream_requests = IntCounter::with_opts(Opts::new(
            "scedge_upstream_requests_total",
            "Total number of cache miss hydrations attempted against upstream",
        ))
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create metric: {}", e)))?;

        let upstream_failures = IntCounter::with_opts(Opts::new(
            "scedge_upstream_failures_total",
            "Total number of upstream hydration attempts that resulted in an error",
        ))
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create metric: {}", e)))?;

        let upstream_latency = Histogram::with_opts(
            HistogramOpts::new(
                "scedge_upstream_latency_seconds",
                "Duration of upstream hydration requests in seconds",
            )
            .buckets(vec![
                0.005, 0.010, 0.025, 0.050, 0.100, 0.250, 0.500, 1.0, 2.0, 5.0,
            ]),
        )
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create metric: {}", e)))?;

        // Artifact metrics
        let artifacts_stored = IntCounter::with_opts(Opts::new(
            "scedge_artifacts_stored_total",
            "Total number of artifacts stored",
        ))
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create metric: {}", e)))?;

        let artifacts_expired = IntCounter::with_opts(Opts::new(
            "scedge_artifacts_expired_total",
            "Total number of artifacts expired",
        ))
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create metric: {}", e)))?;

        // Register all metrics
        registry
            .register(Box::new(cache_hits.clone()))
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to register metric: {}", e)))?;
        registry
            .register(Box::new(cache_misses.clone()))
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to register metric: {}", e)))?;
        registry
            .register(Box::new(cache_stores.clone()))
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to register metric: {}", e)))?;
        registry
            .register(Box::new(cache_purges.clone()))
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to register metric: {}", e)))?;
        registry
            .register(Box::new(cache_size.clone()))
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to register metric: {}", e)))?;
        registry
            .register(Box::new(requests_total.clone()))
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to register metric: {}", e)))?;
        registry
            .register(Box::new(request_duration.clone()))
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to register metric: {}", e)))?;
        registry
            .register(Box::new(upstream_requests.clone()))
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to register metric: {}", e)))?;
        registry
            .register(Box::new(upstream_failures.clone()))
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to register metric: {}", e)))?;
        registry
            .register(Box::new(upstream_latency.clone()))
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to register metric: {}", e)))?;
        registry
            .register(Box::new(artifacts_stored.clone()))
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to register metric: {}", e)))?;
        registry
            .register(Box::new(artifacts_expired.clone()))
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to register metric: {}", e)))?;

        Ok(Self {
            registry: Arc::new(registry),
            cache_hits,
            cache_misses,
            cache_stores,
            cache_purges,
            cache_size,
            requests_total,
            request_duration,
            upstream_requests,
            upstream_failures,
            upstream_latency,
            artifacts_stored,
            artifacts_expired,
        })
    }

    /// Record a cache hit
    pub fn record_cache_hit(&self) {
        self.cache_hits.inc();
    }

    /// Record a cache miss
    pub fn record_cache_miss(&self) {
        self.cache_misses.inc();
    }

    /// Record a cache store operation
    pub fn record_cache_store(&self) {
        self.cache_stores.inc();
        self.artifacts_stored.inc();
    }

    /// Record a cache purge operation
    pub fn record_cache_purge(&self, count: usize) {
        self.cache_purges.inc_by(count as u64);
    }

    /// Update the cache size gauge
    pub fn update_cache_size(&self, size: i64) {
        self.cache_size.set(size);
    }

    /// Record an artifact expiration
    pub fn record_artifact_expired(&self) {
        self.artifacts_expired.inc();
    }

    /// Record an upstream hydration attempt
    pub fn record_upstream_request(&self) {
        self.upstream_requests.inc();
    }

    /// Record an upstream hydration failure
    pub fn record_upstream_failure(&self) {
        self.upstream_failures.inc();
    }

    /// Observe latency for an upstream hydration attempt in seconds
    pub fn record_upstream_latency(&self, seconds: f64) {
        self.upstream_latency.observe(seconds);
    }

    /// Export metrics in Prometheus format
    pub fn export(&self) -> Result<String, AppError> {
        use prometheus::Encoder;

        let encoder = prometheus::TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();

        encoder
            .encode(&metric_families, &mut buffer)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to encode metrics: {}", e)))?;

        String::from_utf8(buffer).map_err(|e| {
            AppError::Internal(anyhow::anyhow!(
                "Failed to convert metrics to string: {}",
                e
            ))
        })
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new().expect("Failed to create default metrics")
    }
}
