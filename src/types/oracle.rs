//! Oracle and price feed types.
//!
//! Templar uses the Pyth Network oracle for price feeds. These types mirror
//! the on-chain oracle response format and the Pyth price structure.

use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

// ---------------------------------------------------------------------------
// PriceIdentifier
// ---------------------------------------------------------------------------

/// A 32-byte Pyth price feed identifier, serialized as a hex string.
///
/// Example: `"e62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43"`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PriceIdentifier(pub [u8; 32]);

impl PriceIdentifier {
    /// Create a `PriceIdentifier` from a hex string.
    pub fn from_hex(hex_str: &str) -> Result<Self, crate::error::CliError> {
        let bytes = hex::decode(hex_str).map_err(|e| {
            crate::error::CliError::Serialization(format!("invalid hex price id: {e}"))
        })?;
        if bytes.len() != 32 {
            return Err(crate::error::CliError::Serialization(format!(
                "price identifier must be 32 bytes, got {}",
                bytes.len()
            )));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }

    /// Returns the hex string representation (no `0x` prefix).
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl fmt::Display for PriceIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl Serialize for PriceIdentifier {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for PriceIdentifier {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        let hex = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")).unwrap_or(&s);
        Self::from_hex(hex).map_err(serde::de::Error::custom)
    }
}

// ---------------------------------------------------------------------------
// PythPrice
// ---------------------------------------------------------------------------

/// A price value from the Pyth Network oracle.
///
/// The actual price is `price * 10^expo`. Confidence interval is represented
/// by `conf * 10^expo`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PythPrice {
    /// The price value as a string-encoded signed integer.
    pub price: String,

    /// The confidence interval as a string-encoded unsigned integer.
    pub conf: String,

    /// The exponent (signed integer, typically negative).
    pub expo: i32,

    /// Unix timestamp when the price was published.
    pub publish_time: i64,
}

impl PythPrice {
    /// Parse the price field as `i64`.
    pub fn price_i64(&self) -> Result<i64, crate::error::CliError> {
        self.price.parse::<i64>().map_err(|e| {
            crate::error::CliError::Serialization(format!("invalid price '{}': {e}", self.price))
        })
    }

    /// Parse the confidence field as `u64`.
    pub fn conf_u64(&self) -> Result<u64, crate::error::CliError> {
        self.conf.parse::<u64>().map_err(|e| {
            crate::error::CliError::Serialization(format!(
                "invalid confidence '{}': {e}",
                self.conf
            ))
        })
    }

    /// Compute the price as `f64` by applying the exponent.
    pub fn price_f64(&self) -> Result<f64, crate::error::CliError> {
        let p = self.price_i64()? as f64;
        let exp = 10f64.powi(self.expo);
        Ok(p * exp)
    }
}

// ---------------------------------------------------------------------------
// OracleResponse
// ---------------------------------------------------------------------------

/// Response from the on-chain oracle, mapping price identifiers to optional
/// Pyth prices.
///
/// A `None` value indicates the price feed had no valid update within the
/// acceptable staleness window.
pub type OracleResponse = HashMap<PriceIdentifier, Option<PythPrice>>;

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_HEX: &str = "e62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43";

    fn sample_price_id() -> PriceIdentifier {
        PriceIdentifier::from_hex(SAMPLE_HEX).unwrap()
    }

    fn sample_pyth_price() -> PythPrice {
        PythPrice {
            price: "310000".into(),
            conf: "150".into(),
            expo: -5,
            publish_time: 1_700_000_000,
        }
    }

    // -- PriceIdentifier tests --

    #[test]
    fn price_id_from_hex_valid() {
        let id = PriceIdentifier::from_hex(SAMPLE_HEX).unwrap();
        assert_eq!(id.to_hex(), SAMPLE_HEX);
    }

    #[test]
    fn price_id_from_hex_wrong_length() {
        let result = PriceIdentifier::from_hex("aabb");
        assert!(result.is_err());
    }

    #[test]
    fn price_id_from_hex_invalid_chars() {
        let result = PriceIdentifier::from_hex(&"zz".repeat(32));
        assert!(result.is_err());
    }

    #[test]
    fn price_id_round_trip() {
        let original = sample_price_id();
        let json = serde_json::to_string(&original).unwrap();
        assert_eq!(json, format!("\"{SAMPLE_HEX}\""));
        let parsed: PriceIdentifier = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn price_id_display() {
        let id = sample_price_id();
        assert_eq!(id.to_string(), SAMPLE_HEX);
    }

    // -- PythPrice tests --

    #[test]
    fn pyth_price_round_trip() {
        let original = sample_pyth_price();
        let json = serde_json::to_string(&original).unwrap();
        let parsed: PythPrice = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn pyth_price_json_format() {
        let price = sample_pyth_price();
        let value = serde_json::to_value(&price).unwrap();
        assert_eq!(value["price"], "310000");
        assert_eq!(value["conf"], "150");
        assert_eq!(value["expo"], -5);
        assert_eq!(value["publish_time"], 1_700_000_000);
    }

    #[test]
    fn pyth_price_i64() {
        let price = sample_pyth_price();
        assert_eq!(price.price_i64().unwrap(), 310_000);
    }

    #[test]
    fn pyth_price_conf_u64() {
        let price = sample_pyth_price();
        assert_eq!(price.conf_u64().unwrap(), 150);
    }

    #[test]
    fn pyth_price_f64() {
        let price = sample_pyth_price();
        let f = price.price_f64().unwrap();
        assert!((f - 3.1).abs() < 1e-10);
    }

    #[test]
    fn pyth_price_negative_price() {
        let price =
            PythPrice { price: "-100".into(), conf: "10".into(), expo: -2, publish_time: 0 };
        assert_eq!(price.price_i64().unwrap(), -100);
        let f = price.price_f64().unwrap();
        assert!((f - (-1.0)).abs() < 1e-10);
    }

    // -- OracleResponse tests --

    #[test]
    fn oracle_response_round_trip() {
        let mut response: OracleResponse = HashMap::new();
        response.insert(sample_price_id(), Some(sample_pyth_price()));

        let zeros = PriceIdentifier([0u8; 32]);
        response.insert(zeros, None);

        let json = serde_json::to_string(&response).unwrap();
        let parsed: OracleResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response, parsed);
    }

    #[test]
    fn oracle_response_empty() {
        let response: OracleResponse = HashMap::new();
        let json = serde_json::to_string(&response).unwrap();
        assert_eq!(json, "{}");
        let parsed: OracleResponse = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_empty());
    }

    #[test]
    fn oracle_response_with_none_value() {
        let json = format!("{{\"{SAMPLE_HEX}\":null}}");
        let parsed: OracleResponse = serde_json::from_str(&json).unwrap();
        let id = sample_price_id();
        assert!(parsed.contains_key(&id));
        assert!(parsed[&id].is_none());
    }
}
