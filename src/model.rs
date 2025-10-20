// Copyright 2025 Memophor Labs
// SPDX-License-Identifier: Apache-2.0

//! Data models and schemas for knowledge artifacts.
//!
//! Defines the structure of cached artifacts with policy, provenance, and metrics.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

fn default_confidence() -> f32 {
    1.0
}

/// Policy context for an artifact - defines access control and compliance requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyContext {
    pub tenant: String,
    #[serde(default)]
    pub phi: bool,
    #[serde(default)]
    pub pii: bool,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub compliance_tags: Vec<String>,
}

/// Provenance information - tracks the source and lineage of knowledge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceInfo {
    pub source: String,
    #[serde(default)]
    pub hash: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub generated_at: Option<DateTime<Utc>>,
}

/// Artifact metrics - confidence scores and quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetrics {
    #[serde(default = "default_confidence")]
    pub score: f32,
    #[serde(default)]
    pub generated_at: Option<DateTime<Utc>>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

/// The core artifact payload containing answer/knowledge and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactPayload {
    /// The actual answer or knowledge content
    #[serde(alias = "content")]
    pub answer: serde_json::Value,

    /// Policy context for access control
    pub policy: PolicyContext,

    /// Provenance tracking
    #[serde(default)]
    pub provenance: Vec<ProvenanceInfo>,

    /// Quality and confidence metrics
    #[serde(default)]
    pub metrics: Option<ArtifactMetrics>,

    /// Time-to-live in seconds
    #[serde(default, alias = "ttl_sec")]
    pub ttl_seconds: Option<u64>,

    /// Hash/ETag for versioning
    pub hash: String,

    /// Additional metadata
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

impl Default for ArtifactMetrics {
    fn default() -> Self {
        Self {
            score: 1.0,
            generated_at: None,
            extra: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreRequest {
    pub key: String,
    pub artifact: ArtifactPayload,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StoreStatus {
    Created,
    Updated,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoreResponse {
    pub key: String,
    pub status: StoreStatus,
    pub hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LookupResponse {
    pub key: String,
    pub artifact: ArtifactPayload,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl_remaining_seconds: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct LookupQuery {
    pub key: String,
    #[serde(default)]
    pub tenant: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PurgeRequest {
    #[serde(default)]
    pub keys: Vec<String>,
    #[serde(default)]
    pub tenant: Option<String>,
    #[serde(default)]
    pub provenance_hash: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PurgeResponse {
    pub purged: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedArtifact {
    pub key: String,
    pub artifact: ArtifactPayload,
    pub stored_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl CachedArtifact {
    pub fn ttl_remaining_seconds(&self, now: DateTime<Utc>) -> Option<u64> {
        self.expires_at.map(|deadline| {
            let remaining = (deadline - now).num_seconds();
            if remaining <= 0 {
                0
            } else {
                remaining as u64
            }
        })
    }
}
