//! Pyth/Hermes oracle price feed client.
//!
//! Fetches latest price updates from the Hermes API and formats
//! oracle responses for contract calls.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::CliError;

/// Client for the Pyth Hermes API.
pub struct PythClient {
    client: reqwest::Client,
    base_url: String,
}

/// A price update from Hermes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HermesPriceUpdate {
    /// Price feed ID (hex).
    pub id: String,
    /// The price data.
    pub price: HermesPrice,
}

/// Price data from Hermes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HermesPrice {
    /// Price value as string.
    pub price: String,
    /// Confidence interval as string.
    pub conf: String,
    /// Price exponent.
    pub expo: i32,
    /// Publish time (Unix timestamp).
    pub publish_time: i64,
}

impl PythClient {
    /// Create a new Pyth client.
    pub fn new(hermes_url: &str) -> Result<Self, CliError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;
        Ok(Self {
            client,
            base_url: hermes_url.trim_end_matches('/').to_string(),
        })
    }

    /// Fetch latest prices for the given price feed IDs.
    pub async fn get_latest_prices(
        &self,
        ids: &[String],
    ) -> Result<HashMap<String, Option<HermesPrice>>, CliError> {
        let mut query_params: Vec<(&str, &str)> = Vec::new();
        for id in ids {
            query_params.push(("ids[]", id));
        }

        let resp: Vec<HermesPriceUpdate> = self
            .client
            .get(format!("{}/v2/updates/price/latest", self.base_url))
            .query(&query_params)
            .send()
            .await?
            .json()
            .await?;

        let mut result = HashMap::new();
        for update in resp {
            result.insert(update.id.clone(), Some(update.price));
        }
        // Add None for any requested IDs not in the response
        for id in ids {
            result.entry(id.clone()).or_insert(None);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pyth_client_creation() {
        let client = PythClient::new("https://hermes.pyth.network");
        assert!(client.is_ok());
    }

    #[test]
    fn hermes_price_deserialization() {
        let json = r#"{"price": "12345", "conf": "100", "expo": -8, "publish_time": 1700000000}"#;
        let price: HermesPrice = serde_json::from_str(json).unwrap();
        assert_eq!(price.price, "12345");
        assert_eq!(price.expo, -8);
    }
}
