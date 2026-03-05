//! Numeric wrapper types for NEAR-compatible serialization.
//!
//! NEAR contracts serialize `u64` and `u128` values as quoted decimal strings
//! in JSON (e.g., `"1000000"`). The [`U64`] and [`U128`] newtypes handle
//! this transparently via serde. [`Decimal`] wraps a string-encoded decimal
//! number used for ratios and rates.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::CliError;

// ---------------------------------------------------------------------------
// U64 — NEAR-compatible u64 wrapper (JSON string)
// ---------------------------------------------------------------------------

/// A `u64` that serializes as a quoted decimal string in JSON.
///
/// Mirrors `near_sdk::json_types::U64`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct U64(pub u64);

impl Serialize for U64 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for U64 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        let val = s.parse::<u64>().map_err(serde::de::Error::custom)?;
        Ok(U64(val))
    }
}

impl fmt::Display for U64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for U64 {
    fn from(v: u64) -> Self {
        Self(v)
    }
}

impl From<U64> for u64 {
    fn from(v: U64) -> Self {
        v.0
    }
}

// ---------------------------------------------------------------------------
// U128 — NEAR-compatible u128 wrapper (JSON string)
// ---------------------------------------------------------------------------

/// A `u128` that serializes as a quoted decimal string in JSON.
///
/// Mirrors `near_sdk::json_types::U128`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct U128(pub u128);

impl Serialize for U128 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for U128 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        let val = s.parse::<u128>().map_err(serde::de::Error::custom)?;
        Ok(U128(val))
    }
}

impl fmt::Display for U128 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u128> for U128 {
    fn from(v: u128) -> Self {
        Self(v)
    }
}

impl From<U128> for u128 {
    fn from(v: U128) -> Self {
        v.0
    }
}

/// Type alias for [`U128`] used by some contract interfaces.
pub type U128String = U128;

// ---------------------------------------------------------------------------
// Decimal — String-encoded decimal number
// ---------------------------------------------------------------------------

/// A decimal number serialized as a string.
///
/// Used for ratios, rates, and other non-integer numeric values in Templar
/// contracts. The inner value is kept as a raw string to preserve the exact
/// representation from the contract.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Decimal(pub String);

impl Decimal {
    /// Create a new `Decimal` from a string value.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Attempt to parse the inner string as `f64`.
    ///
    /// Returns a [`CliError::Serialization`] if the string is not a valid number.
    pub fn to_f64(&self) -> Result<f64, CliError> {
        self.0
            .parse::<f64>()
            .map_err(|e| CliError::Serialization(format!("invalid decimal '{}': {e}", self.0)))
    }
}

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for Decimal {
    fn default() -> Self {
        Self("0".into())
    }
}

impl FromStr for Decimal {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl From<&str> for Decimal {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

// ---------------------------------------------------------------------------
// Amount newtypes
// ---------------------------------------------------------------------------

/// Amount of a borrow asset, stored as a [`U128`].
///
/// This newtype provides semantic clarity when dealing with multiple asset
/// amounts that share the same underlying representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BorrowAssetAmount(pub U128);

impl BorrowAssetAmount {
    /// Create a new amount from a raw `u128`.
    pub fn new(value: u128) -> Self {
        Self(U128(value))
    }

    /// Returns the inner `u128` value.
    pub fn as_u128(&self) -> u128 {
        self.0 .0
    }
}

impl fmt::Display for BorrowAssetAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u128> for BorrowAssetAmount {
    fn from(v: u128) -> Self {
        Self::new(v)
    }
}

/// Amount of a collateral asset, stored as a [`U128`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CollateralAssetAmount(pub U128);

impl CollateralAssetAmount {
    /// Create a new amount from a raw `u128`.
    pub fn new(value: u128) -> Self {
        Self(U128(value))
    }

    /// Returns the inner `u128` value.
    pub fn as_u128(&self) -> u128 {
        self.0 .0
    }
}

impl fmt::Display for CollateralAssetAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u128> for CollateralAssetAmount {
    fn from(v: u128) -> Self {
        Self::new(v)
    }
}

// ---------------------------------------------------------------------------
// Accumulator — used for interest tracking
// ---------------------------------------------------------------------------

/// An accumulator value used for tracking compounding interest.
///
/// Serialized as a string-encoded decimal, matching the contract representation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Accumulator(pub Decimal);

impl Accumulator {
    /// Create a new accumulator from a string value.
    pub fn new(value: impl Into<String>) -> Self {
        Self(Decimal::new(value))
    }
}

impl fmt::Display for Accumulator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ---------------------------------------------------------------------------
// FungibleAsset
// ---------------------------------------------------------------------------

/// A fungible asset identifier, either NEP-141 or NEP-245.
///
/// **Variant order matters**: `Nep245` must come before `Nep141` because serde
/// tries untagged variants in declaration order. `Nep245` has two required
/// fields (`contract_id` + `token_id`) while `Nep141` only requires
/// `account_id`, so JSON with both fields would incorrectly match `Nep141`
/// if it came first.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FungibleAsset {
    /// A NEP-245 multi-token.
    Nep245 {
        /// The multi-token contract account ID.
        contract_id: String,
        /// The token ID within the multi-token contract.
        token_id: String,
    },
    /// A standard NEP-141 fungible token.
    Nep141 {
        /// The token contract account ID.
        account_id: String,
    },
}

impl fmt::Display for FungibleAsset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nep141 { account_id } => write!(f, "{account_id}"),
            Self::Nep245 { contract_id, token_id } => write!(f, "{contract_id}:{token_id}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- U64 tests --

    #[test]
    fn u64_serializes_as_string() {
        let val = U64(42);
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(json, "\"42\"");
    }

    #[test]
    fn u64_deserializes_from_string() {
        let val: U64 = serde_json::from_str("\"42\"").unwrap();
        assert_eq!(val.0, 42);
    }

    #[test]
    fn u64_round_trip() {
        let original = U64(u64::MAX);
        let json = serde_json::to_string(&original).unwrap();
        let parsed: U64 = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn u64_display() {
        assert_eq!(U64(100).to_string(), "100");
    }

    #[test]
    fn u64_from_conversions() {
        let val: U64 = 42u64.into();
        assert_eq!(val.0, 42);
        let back: u64 = val.into();
        assert_eq!(back, 42);
    }

    // -- U128 tests --

    #[test]
    fn u128_serializes_as_string() {
        let val = U128(1_000_000);
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(json, "\"1000000\"");
    }

    #[test]
    fn u128_deserializes_from_string() {
        let val: U128 = serde_json::from_str("\"1000000\"").unwrap();
        assert_eq!(val.0, 1_000_000);
    }

    #[test]
    fn u128_round_trip() {
        let original = U128(u128::MAX);
        let json = serde_json::to_string(&original).unwrap();
        let parsed: U128 = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn u128_display() {
        assert_eq!(U128(999).to_string(), "999");
    }

    #[test]
    fn u128_from_conversions() {
        let val: U128 = 100u128.into();
        assert_eq!(val.0, 100);
        let back: u128 = val.into();
        assert_eq!(back, 100);
    }

    // -- Decimal tests --

    #[test]
    fn decimal_serializes_as_string() {
        let val = Decimal::new("1.5");
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(json, "\"1.5\"");
    }

    #[test]
    fn decimal_deserializes_from_string() {
        let val: Decimal = serde_json::from_str("\"3.14\"").unwrap();
        assert_eq!(val.0, "3.14");
    }

    #[test]
    fn decimal_round_trip() {
        let original = Decimal::new("123456789.987654321");
        let json = serde_json::to_string(&original).unwrap();
        let parsed: Decimal = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn decimal_to_f64() {
        let val = Decimal::new("3.14");
        let f = val.to_f64().unwrap();
        assert!((f - 3.14_f64).abs() < 0.001);
    }

    #[test]
    fn decimal_to_f64_invalid() {
        let val = Decimal::new("not_a_number");
        assert!(val.to_f64().is_err());
    }

    #[test]
    fn decimal_display() {
        assert_eq!(Decimal::new("1.23").to_string(), "1.23");
    }

    #[test]
    fn decimal_from_str() {
        let val: Decimal = "2.71828".parse().unwrap();
        assert_eq!(val.0, "2.71828");
    }

    // -- Amount newtypes --

    #[test]
    fn borrow_asset_amount_round_trip() {
        let original = BorrowAssetAmount::new(1_000_000);
        let json = serde_json::to_string(&original).unwrap();
        assert_eq!(json, "\"1000000\"");
        let parsed: BorrowAssetAmount = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn collateral_asset_amount_round_trip() {
        let original = CollateralAssetAmount::new(500_000);
        let json = serde_json::to_string(&original).unwrap();
        assert_eq!(json, "\"500000\"");
        let parsed: CollateralAssetAmount = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn borrow_asset_amount_accessors() {
        let amt = BorrowAssetAmount::new(42);
        assert_eq!(amt.as_u128(), 42);
        let from: BorrowAssetAmount = 42u128.into();
        assert_eq!(from, amt);
    }

    #[test]
    fn collateral_asset_amount_accessors() {
        let amt = CollateralAssetAmount::new(99);
        assert_eq!(amt.as_u128(), 99);
        let from: CollateralAssetAmount = 99u128.into();
        assert_eq!(from, amt);
    }

    // -- Accumulator tests --

    #[test]
    fn accumulator_round_trip() {
        let original = Accumulator::new("1.000000001");
        let json = serde_json::to_string(&original).unwrap();
        assert_eq!(json, "\"1.000000001\"");
        let parsed: Accumulator = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    // -- FungibleAsset tests --

    #[test]
    fn fungible_asset_nep141_round_trip() {
        let asset = FungibleAsset::Nep141 { account_id: "usdc.near".into() };
        let json = serde_json::to_string(&asset).unwrap();
        let parsed: FungibleAsset = serde_json::from_str(&json).unwrap();
        assert_eq!(asset, parsed);
    }

    #[test]
    fn fungible_asset_nep245_round_trip() {
        let asset =
            FungibleAsset::Nep245 { contract_id: "multi.near".into(), token_id: "token-1".into() };
        let json = serde_json::to_string(&asset).unwrap();
        let parsed: FungibleAsset = serde_json::from_str(&json).unwrap();
        assert_eq!(asset, parsed);
    }

    #[test]
    fn fungible_asset_nep141_json_format() {
        let asset = FungibleAsset::Nep141 { account_id: "usdc.near".into() };
        let json = serde_json::to_value(&asset).unwrap();
        assert_eq!(json["account_id"], "usdc.near");
        // Should NOT have contract_id or token_id
        assert!(json.get("contract_id").is_none());
    }

    #[test]
    fn fungible_asset_nep245_json_format() {
        let asset =
            FungibleAsset::Nep245 { contract_id: "multi.near".into(), token_id: "token-1".into() };
        let json = serde_json::to_value(&asset).unwrap();
        assert_eq!(json["contract_id"], "multi.near");
        assert_eq!(json["token_id"], "token-1");
    }

    #[test]
    fn fungible_asset_display() {
        let nep141 = FungibleAsset::Nep141 { account_id: "usdc.near".into() };
        assert_eq!(nep141.to_string(), "usdc.near");

        let nep245 =
            FungibleAsset::Nep245 { contract_id: "multi.near".into(), token_id: "token-1".into() };
        assert_eq!(nep245.to_string(), "multi.near:token-1");
    }
}
