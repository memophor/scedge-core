use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use tokio::sync::RwLock;

use crate::model::{ArtifactPayload, CachedArtifact};

#[derive(Clone)]
pub struct CacheState {
    inner: Arc<RwLock<HashMap<String, CacheEntry>>>,
    default_ttl: Duration,
}

#[derive(Clone)]
struct CacheEntry {
    payload: ArtifactPayload,
    stored_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
}

pub struct CacheWriteOutcome {
    pub record: CachedArtifact,
    pub created: bool,
}

impl CacheState {
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
        }
    }

    pub fn default_ttl(&self) -> Duration {
        self.default_ttl
    }

    pub async fn set(&self, key: String, mut payload: ArtifactPayload) -> CacheWriteOutcome {
        let now = Utc::now();
        let (expires_at, ttl_seconds) = normalize_ttl(&mut payload, self.default_ttl, now);
        let entry = CacheEntry {
            payload: payload.clone(),
            stored_at: now,
            expires_at,
        };

        let mut guard = self.inner.write().await;
        let created = !guard.contains_key(&key);
        guard.insert(key.clone(), entry);
        drop(guard);

        let record = CachedArtifact {
            key,
            artifact: payload,
            stored_at: now,
            expires_at,
        };

        if let Some(ttl) = ttl_seconds {
            tracing::debug!(seconds = ttl, "stored artifact with ttl");
        }

        CacheWriteOutcome { record, created }
    }

    pub async fn get(&self, key: &str) -> Option<CachedArtifact> {
        let mut guard = self.inner.write().await;
        let now = Utc::now();

        if let Some(entry) = guard.get(key) {
            if entry.is_expired(now) {
                guard.remove(key);
                return None;
            }

            return Some(entry.as_record(key));
        }

        None
    }

    pub async fn purge(&self, keys: &[String]) -> usize {
        let mut guard = self.inner.write().await;
        let mut removed = 0;
        for key in keys {
            if guard.remove(key).is_some() {
                removed += 1;
            }
        }
        removed
    }

    pub async fn purge_expired(&self) -> usize {
        let mut guard = self.inner.write().await;
        let now = Utc::now();
        let before = guard.len();
        guard.retain(|_, entry| !entry.is_expired(now));
        before - guard.len()
    }
}

impl CacheEntry {
    fn is_expired(&self, now: DateTime<Utc>) -> bool {
        self.expires_at.map(|deadline| deadline <= now).unwrap_or(false)
    }

    fn as_record(&self, key: &str) -> CachedArtifact {
        CachedArtifact {
            key: key.to_string(),
            artifact: self.payload.clone(),
            stored_at: self.stored_at,
            expires_at: self.expires_at,
        }
    }
}

fn normalize_ttl(
    payload: &mut ArtifactPayload,
    default_ttl: Duration,
    now: DateTime<Utc>,
) -> (Option<DateTime<Utc>>, Option<u64>) {
    let requested = payload.ttl_seconds.filter(|secs| *secs > 0);
    let duration = match requested {
        Some(secs) => Duration::from_secs(secs),
        None => default_ttl,
    };

    if duration.is_zero() {
        payload.ttl_seconds = None;
        return (None, None);
    }

    let chrono_duration = match ChronoDuration::from_std(duration) {
        Ok(value) => value,
        Err(_) => {
            const MAX_SECS: u64 = i64::MAX as u64;
            let capped = duration.as_secs().min(MAX_SECS);
            ChronoDuration::seconds(capped as i64)
        }
    };
    let expires_at = now + chrono_duration;
    payload.ttl_seconds = Some(duration.as_secs());

    (Some(expires_at), Some(duration.as_secs()))
}
