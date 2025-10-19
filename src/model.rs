use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

fn default_confidence() -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactPayload {
    pub content: serde_json::Value,
    #[serde(default)]
    pub provenance: Option<serde_json::Value>,
    #[serde(default)]
    pub policy: Option<serde_json::Value>,
    #[serde(default = "default_confidence")]
    pub confidence: f32,
    #[serde(default)]
    pub ttl_seconds: Option<u64>,
    pub etag: String,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
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
    pub etag: String,
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
}

#[derive(Debug, Deserialize)]
pub struct PurgeRequest {
    pub keys: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PurgeResponse {
    pub purged: usize,
}

#[derive(Debug, Clone)]
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
