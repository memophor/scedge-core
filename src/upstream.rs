// Copyright 2025 Memophor Labs
// SPDX-License-Identifier: Apache-2.0

//! Upstream fetcher for Scedge Core.
//!
//! Handles cache miss hydration by calling a configured SynaGraph endpoint
//! and translating the response into the local cache format.

use anyhow::anyhow;
use reqwest::{Client, StatusCode};

use crate::config::UpstreamConfig;
use crate::error::AppError;
use crate::model::LookupResponse;

/// HTTP client wrapper for talking to the upstream knowledge graph.
#[derive(Clone)]
pub struct UpstreamClient {
    base_url: String,
    client: Client,
}

impl UpstreamClient {
    /// Construct a new upstream client using the provided configuration.
    pub fn try_new(config: UpstreamConfig) -> Result<Self, AppError> {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|e| AppError::Internal(anyhow!("Failed to build upstream client: {}", e)))?;

        Ok(Self {
            base_url: config.base_url,
            client,
        })
    }

    /// Fetch an artifact from the upstream graph.
    pub async fn lookup(
        &self,
        key: &str,
        tenant: Option<&str>,
    ) -> Result<Option<LookupResponse>, AppError> {
        let url = format!("{}/lookup", self.base_url.trim_end_matches('/'));

        let mut request = self.client.get(url).query(&[("key", key)]);
        if let Some(tenant) = tenant {
            request = request.query(&[("tenant", tenant)]);
        }

        let response = request
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow!("Upstream request failed: {}", e)))?;

        let status = response.status();

        if status == StatusCode::NOT_FOUND {
            tracing::debug!(key, "Upstream returned miss");
            return Ok(None);
        }

        if !status.is_success() {
            return Err(AppError::Internal(anyhow!(
                "Upstream returned unexpected status {}",
                status
            )));
        }

        let payload = response
            .json::<LookupResponse>()
            .await
            .map_err(|e| AppError::Internal(anyhow!("Failed to parse upstream response: {}", e)))?;

        Ok(Some(payload))
    }
}
