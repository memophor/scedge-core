use async_trait::async_trait;
use chrono::{DateTime, Utc};
use redis::AsyncCommands;
use std::sync::Arc;

use crate::error::AppError;
use crate::model::{ArtifactPayload, CachedArtifact};

/// Trait for cache backends
#[async_trait]
pub trait CacheBackend: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<CachedArtifact>, AppError>;
    async fn set(&self, key: String, artifact: ArtifactPayload, expires_at: Option<DateTime<Utc>>) -> Result<CachedArtifact, AppError>;
    async fn delete(&self, key: &str) -> Result<bool, AppError>;
    async fn delete_many(&self, keys: &[String]) -> Result<usize, AppError>;
    async fn scan_by_pattern(&self, pattern: &str) -> Result<Vec<String>, AppError>;
}

/// Redis-based cache backend
#[derive(Clone)]
pub struct RedisCache {
    client: redis::Client,
}

impl RedisCache {
    pub fn new(redis_url: &str) -> Result<Self, AppError> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create Redis client: {}", e)))?;

        Ok(Self { client })
    }

    /// Test the Redis connection
    pub async fn ping(&self) -> Result<(), AppError> {
        let mut conn = self.client.get_multiplexed_async_connection().await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to connect to Redis: {}", e)))?;

        redis::cmd("PING")
            .query_async::<_, String>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis PING failed: {}", e)))?;

        Ok(())
    }

    fn build_redis_key(&self, key: &str) -> String {
        format!("scedge:artifact:{}", key)
    }
}

#[async_trait]
impl CacheBackend for RedisCache {
    async fn get(&self, key: &str) -> Result<Option<CachedArtifact>, AppError> {
        let mut conn = self.client.get_multiplexed_async_connection().await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis connection failed: {}", e)))?;

        let redis_key = self.build_redis_key(key);
        let data: Option<String> = conn.get(&redis_key).await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis GET failed: {}", e)))?;

        match data {
            Some(json) => {
                let artifact: CachedArtifact = serde_json::from_str(&json)
                    .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to deserialize artifact: {}", e)))?;

                // Check if expired
                if let Some(expires_at) = artifact.expires_at {
                    if expires_at <= Utc::now() {
                        // Delete expired entry
                        let _ = self.delete(key).await;
                        return Ok(None);
                    }
                }

                Ok(Some(artifact))
            }
            None => Ok(None),
        }
    }

    async fn set(&self, key: String, artifact: ArtifactPayload, expires_at: Option<DateTime<Utc>>) -> Result<CachedArtifact, AppError> {
        let mut conn = self.client.get_multiplexed_async_connection().await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis connection failed: {}", e)))?;

        let now = Utc::now();
        let cached = CachedArtifact {
            key: key.clone(),
            artifact,
            stored_at: now,
            expires_at,
        };

        let json = serde_json::to_string(&cached)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to serialize artifact: {}", e)))?;

        let redis_key = self.build_redis_key(&key);

        if let Some(exp) = expires_at {
            let ttl = (exp - now).num_seconds();
            if ttl > 0 {
                conn.set_ex(&redis_key, json, ttl as u64).await
                    .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis SETEX failed: {}", e)))?;
            } else {
                // Already expired, don't store
                return Err(AppError::bad_request("Artifact already expired"));
            }
        } else {
            conn.set(&redis_key, json).await
                .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis SET failed: {}", e)))?;
        }

        Ok(cached)
    }

    async fn delete(&self, key: &str) -> Result<bool, AppError> {
        let mut conn = self.client.get_multiplexed_async_connection().await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis connection failed: {}", e)))?;

        let redis_key = self.build_redis_key(key);
        let deleted: i32 = conn.del(&redis_key).await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis DEL failed: {}", e)))?;

        Ok(deleted > 0)
    }

    async fn delete_many(&self, keys: &[String]) -> Result<usize, AppError> {
        if keys.is_empty() {
            return Ok(0);
        }

        let mut conn = self.client.get_multiplexed_async_connection().await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis connection failed: {}", e)))?;

        let redis_keys: Vec<String> = keys.iter()
            .map(|k| self.build_redis_key(k))
            .collect();

        let deleted: usize = conn.del(&redis_keys).await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis DEL failed: {}", e)))?;

        Ok(deleted)
    }

    async fn scan_by_pattern(&self, pattern: &str) -> Result<Vec<String>, AppError> {
        let mut conn = self.client.get_multiplexed_async_connection().await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis connection failed: {}", e)))?;

        let search_pattern = format!("scedge:artifact:{}", pattern);
        let mut keys = Vec::new();
        let mut cursor = 0;

        loop {
            let (new_cursor, batch): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&search_pattern)
                .arg("COUNT")
                .arg(100)
                .query_async(&mut conn)
                .await
                .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis SCAN failed: {}", e)))?;

            // Strip the "scedge:artifact:" prefix
            for key in batch {
                if let Some(stripped) = key.strip_prefix("scedge:artifact:") {
                    keys.push(stripped.to_string());
                }
            }

            cursor = new_cursor;
            if cursor == 0 {
                break;
            }
        }

        Ok(keys)
    }
}

/// Cache wrapper that can use different backends
#[derive(Clone)]
pub struct Cache {
    backend: Arc<dyn CacheBackend>,
}

impl Cache {
    pub fn new(backend: impl CacheBackend + 'static) -> Self {
        Self {
            backend: Arc::new(backend),
        }
    }

    pub async fn get(&self, key: &str) -> Result<Option<CachedArtifact>, AppError> {
        self.backend.get(key).await
    }

    pub async fn set(&self, key: String, artifact: ArtifactPayload, expires_at: Option<DateTime<Utc>>) -> Result<CachedArtifact, AppError> {
        self.backend.set(key, artifact, expires_at).await
    }

    pub async fn delete(&self, key: &str) -> Result<bool, AppError> {
        self.backend.delete(key).await
    }

    pub async fn delete_many(&self, keys: &[String]) -> Result<usize, AppError> {
        self.backend.delete_many(keys).await
    }

    pub async fn scan_by_pattern(&self, pattern: &str) -> Result<Vec<String>, AppError> {
        self.backend.scan_by_pattern(pattern).await
    }
}
