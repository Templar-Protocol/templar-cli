//! Templar backend REST API client.
//!
//! Provides an alternative path for read operations when the backend
//! service is available, avoiding direct RPC calls.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::CliError;

/// Client for the Templar backend REST API.
pub struct BackendClient {
    client: reqwest::Client,
    base_url: String,
}

/// Health check response.
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Service status.
    pub status: String,
}

/// Market summary from the backend.
#[derive(Debug, Serialize, Deserialize)]
pub struct MarketSummary {
    /// Market contract account ID.
    pub account_id: String,
    /// Market display name.
    #[serde(default)]
    pub name: String,
    /// Total value locked.
    #[serde(default)]
    pub tvl: String,
    /// Current borrow APR.
    #[serde(default)]
    pub borrow_apr: String,
    /// Current supply APY.
    #[serde(default)]
    pub supply_apy: String,
    /// Market status.
    #[serde(default)]
    pub status: String,
}

/// Price response from the backend.
#[derive(Debug, Serialize, Deserialize)]
pub struct PriceResponse {
    /// Asset prices keyed by asset ID.
    pub prices: HashMap<String, f64>,
}

impl BackendClient {
    /// Create a new backend client.
    pub fn new(base_url: &str) -> Result<Self, CliError> {
        let client =
            reqwest::Client::builder().timeout(std::time::Duration::from_secs(30)).build()?;
        Ok(Self { client, base_url: base_url.trim_end_matches('/').to_string() })
    }

    /// Check backend health.
    pub async fn health(&self) -> Result<HealthResponse, CliError> {
        let resp =
            self.client.get(format!("{}/v1/health", self.base_url)).send().await?.json().await?;
        Ok(resp)
    }

    /// List all markets.
    pub async fn list_markets(&self) -> Result<Vec<MarketSummary>, CliError> {
        let resp =
            self.client.get(format!("{}/v1/markets", self.base_url)).send().await?.json().await?;
        Ok(resp)
    }

    /// Get a specific market.
    pub async fn get_market(&self, market_id: &str) -> Result<MarketSummary, CliError> {
        let resp = self
            .client
            .get(format!("{}/v1/markets/{market_id}", self.base_url))
            .send()
            .await?
            .json()
            .await?;
        Ok(resp)
    }

    /// Get prices for the given asset IDs.
    pub async fn get_prices(&self, asset_ids: &[String]) -> Result<PriceResponse, CliError> {
        let ids = asset_ids.join(",");
        let resp = self
            .client
            .get(format!("{}/v1/prices", self.base_url))
            .query(&[("assetIds", ids)])
            .send()
            .await?
            .json()
            .await?;
        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_client_creation() {
        let client = BackendClient::new("https://api.templarfi.org");
        assert!(client.is_ok());
    }

    #[test]
    fn base_url_trimmed() {
        let client = BackendClient::new("https://api.templarfi.org/").unwrap();
        assert_eq!(client.base_url, "https://api.templarfi.org");
    }
}
