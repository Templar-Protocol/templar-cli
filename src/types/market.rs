//! Lending market configuration and snapshot types.
//!
//! These types mirror the Templar Protocol market contract state. A
//! [`MarketConfiguration`] describes the static parameters of a lending market,
//! while a [`Snapshot`] captures its time-series state at a particular block.

use serde::{Deserialize, Serialize};

// Re-export FungibleAsset so downstream code can access it via `types::market::FungibleAsset`.
pub use super::number::FungibleAsset;

use super::number::{Decimal, U128, U64};

// ---------------------------------------------------------------------------
// MarketConfiguration
// ---------------------------------------------------------------------------

/// Static configuration of a Templar lending market.
///
/// Returned by `get_configuration()` on the market contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketConfiguration {
    /// The asset that suppliers deposit and borrowers receive.
    pub borrow_asset: FungibleAsset,

    /// The asset that borrowers pledge as collateral.
    pub collateral_asset: FungibleAsset,

    /// Maintenance collateral ratio for the market (decimal string).
    pub borrow_mcr_maintenance: Decimal,

    /// Liquidation collateral ratio threshold (decimal string).
    pub borrow_mcr_liquidation: Decimal,

    /// Maximum fraction of deposited borrow asset that can be borrowed.
    pub borrow_asset_maximum_usage_ratio: Decimal,

    /// Oracle price identifier for the borrow asset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub borrow_asset_oracle_id: Option<String>,

    /// Oracle price identifier for the collateral asset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collateral_asset_oracle_id: Option<String>,

    /// Decimals of the borrow asset token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub borrow_asset_decimals: Option<u8>,

    /// Decimals of the collateral asset token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collateral_asset_decimals: Option<u8>,

    /// Interest rate model configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interest_rate_model: Option<InterestRateModel>,

    /// Fee configuration for the market.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fee_config: Option<MarketFeeConfig>,

    /// Maximum borrow duration in nanoseconds (0 = unlimited).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_borrow_duration_ns: Option<U64>,

    /// Yield distribution weights.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub yield_weights: Option<YieldWeights>,
}

/// Interest rate model parameters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterestRateModel {
    /// Base interest rate (decimal string).
    pub base_rate: Decimal,

    /// Rate multiplier applied to utilization (decimal string).
    pub rate_multiplier: Decimal,

    /// Jump multiplier applied above optimal utilization (decimal string).
    pub jump_multiplier: Decimal,

    /// Optimal utilization ratio (decimal string).
    pub optimal_utilization: Decimal,
}

/// Fee configuration for a lending market.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketFeeConfig {
    /// Protocol fee rate on interest (decimal string).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol_fee_rate: Option<Decimal>,

    /// Liquidation penalty rate (decimal string).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub liquidation_penalty_rate: Option<Decimal>,

    /// Liquidation protocol share (decimal string).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub liquidation_protocol_share: Option<Decimal>,
}

/// Yield distribution weights determining how interest is split.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YieldWeights {
    /// Weight allocated to suppliers.
    pub supplier: Decimal,

    /// Weight allocated to the protocol treasury.
    pub treasury: Decimal,

    /// Weight allocated to the insurance fund.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub insurance: Option<Decimal>,
}

// ---------------------------------------------------------------------------
// BorrowAssetMetrics
// ---------------------------------------------------------------------------

/// Aggregate metrics for the borrow asset side of a market.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BorrowAssetMetrics {
    /// Total borrow asset deposited by suppliers (active, non-borrowed).
    pub deposited_active: U128,

    /// Total borrow asset currently borrowed.
    pub borrowed: U128,

    /// Total collateral asset deposited by borrowers.
    pub collateral_deposited: U128,

    /// Current utilization ratio (decimal string).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub utilization: Option<Decimal>,

    /// Current interest rate (decimal string).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interest_rate: Option<Decimal>,
}

// ---------------------------------------------------------------------------
// Snapshot
// ---------------------------------------------------------------------------

/// A time-series snapshot of a lending market's state.
///
/// Snapshots are emitted at regular intervals (`time_chunk`) and capture
/// aggregate metrics at `end_timestamp_ms`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    /// The time chunk index.
    pub time_chunk: U64,

    /// End timestamp of this snapshot in milliseconds.
    pub end_timestamp_ms: U64,

    /// Total borrow asset deposited in active (non-borrowed) state.
    pub borrow_asset_deposited_active: U128,

    /// Total borrow asset currently borrowed.
    pub borrow_asset_borrowed: U128,

    /// Total collateral asset deposited.
    pub collateral_asset_deposited: U128,

    /// Yield distribution for this period.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub yield_distribution: Option<YieldDistribution>,

    /// Interest rate at the time of the snapshot (decimal string).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interest_rate: Option<Decimal>,
}

/// Yield distribution within a snapshot period.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YieldDistribution {
    /// Total yield generated in this period.
    pub total: U128,

    /// Yield allocated to suppliers.
    pub supplier: U128,

    /// Yield allocated to the protocol treasury.
    pub treasury: U128,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_market_config() -> MarketConfiguration {
        MarketConfiguration {
            borrow_asset: FungibleAsset::Nep141 {
                account_id: "usdc.near".into(),
            },
            collateral_asset: FungibleAsset::Nep141 {
                account_id: "wrap.near".into(),
            },
            borrow_mcr_maintenance: Decimal::new("1.5"),
            borrow_mcr_liquidation: Decimal::new("1.2"),
            borrow_asset_maximum_usage_ratio: Decimal::new("0.8"),
            borrow_asset_oracle_id: Some("pyth-usdc".into()),
            collateral_asset_oracle_id: Some("pyth-near".into()),
            borrow_asset_decimals: Some(6),
            collateral_asset_decimals: Some(24),
            interest_rate_model: Some(InterestRateModel {
                base_rate: Decimal::new("0.02"),
                rate_multiplier: Decimal::new("0.1"),
                jump_multiplier: Decimal::new("3.0"),
                optimal_utilization: Decimal::new("0.8"),
            }),
            fee_config: Some(MarketFeeConfig {
                protocol_fee_rate: Some(Decimal::new("0.1")),
                liquidation_penalty_rate: Some(Decimal::new("0.05")),
                liquidation_protocol_share: Some(Decimal::new("0.5")),
            }),
            max_borrow_duration_ns: Some(U64(86_400_000_000_000)),
            yield_weights: Some(YieldWeights {
                supplier: Decimal::new("0.8"),
                treasury: Decimal::new("0.2"),
                insurance: None,
            }),
        }
    }

    fn sample_snapshot() -> Snapshot {
        Snapshot {
            time_chunk: U64(100),
            end_timestamp_ms: U64(1_700_000_000_000),
            borrow_asset_deposited_active: U128(1_000_000_000_000),
            borrow_asset_borrowed: U128(500_000_000_000),
            collateral_asset_deposited: U128(2_000_000_000_000_000_000_000_000_000),
            yield_distribution: Some(YieldDistribution {
                total: U128(1_000_000),
                supplier: U128(800_000),
                treasury: U128(200_000),
            }),
            interest_rate: Some(Decimal::new("0.05")),
        }
    }

    #[test]
    fn market_config_round_trip() {
        let original = sample_market_config();
        let json = serde_json::to_string(&original).unwrap();
        let parsed: MarketConfiguration = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn market_config_json_format() {
        let config = sample_market_config();
        let value = serde_json::to_value(&config).unwrap();
        assert_eq!(value["borrow_mcr_maintenance"], "1.5");
        assert_eq!(value["borrow_mcr_liquidation"], "1.2");
        assert_eq!(value["borrow_asset_maximum_usage_ratio"], "0.8");
        assert_eq!(value["borrow_asset"]["account_id"], "usdc.near");
        assert_eq!(value["collateral_asset"]["account_id"], "wrap.near");
    }

    #[test]
    fn market_config_minimal() {
        let json = r#"{
            "borrow_asset": {"account_id": "usdc.near"},
            "collateral_asset": {"account_id": "wrap.near"},
            "borrow_mcr_maintenance": "1.5",
            "borrow_mcr_liquidation": "1.2",
            "borrow_asset_maximum_usage_ratio": "0.8"
        }"#;
        let config: MarketConfiguration = serde_json::from_str(json).unwrap();
        assert_eq!(config.borrow_mcr_maintenance.0, "1.5");
        assert!(config.interest_rate_model.is_none());
        assert!(config.fee_config.is_none());
        assert!(config.yield_weights.is_none());
    }

    #[test]
    fn market_config_nep245_asset() {
        let config = MarketConfiguration {
            borrow_asset: FungibleAsset::Nep245 {
                contract_id: "multi.near".into(),
                token_id: "usdc-1".into(),
            },
            collateral_asset: FungibleAsset::Nep141 {
                account_id: "wrap.near".into(),
            },
            borrow_mcr_maintenance: Decimal::new("1.5"),
            borrow_mcr_liquidation: Decimal::new("1.2"),
            borrow_asset_maximum_usage_ratio: Decimal::new("0.8"),
            borrow_asset_oracle_id: None,
            collateral_asset_oracle_id: None,
            borrow_asset_decimals: None,
            collateral_asset_decimals: None,
            interest_rate_model: None,
            fee_config: None,
            max_borrow_duration_ns: None,
            yield_weights: None,
        };
        let json = serde_json::to_string(&config).unwrap();
        let parsed: MarketConfiguration = serde_json::from_str(&json).unwrap();
        assert_eq!(config, parsed);
    }

    #[test]
    fn snapshot_round_trip() {
        let original = sample_snapshot();
        let json = serde_json::to_string(&original).unwrap();
        let parsed: Snapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn snapshot_json_format() {
        let snap = sample_snapshot();
        let value = serde_json::to_value(&snap).unwrap();
        assert_eq!(value["time_chunk"], "100");
        assert_eq!(value["end_timestamp_ms"], "1700000000000");
        assert_eq!(value["borrow_asset_deposited_active"], "1000000000000");
        assert_eq!(value["interest_rate"], "0.05");
    }

    #[test]
    fn snapshot_minimal() {
        let json = r#"{
            "time_chunk": "1",
            "end_timestamp_ms": "1700000000000",
            "borrow_asset_deposited_active": "0",
            "borrow_asset_borrowed": "0",
            "collateral_asset_deposited": "0"
        }"#;
        let snap: Snapshot = serde_json::from_str(json).unwrap();
        assert_eq!(snap.time_chunk.0, 1);
        assert!(snap.yield_distribution.is_none());
        assert!(snap.interest_rate.is_none());
    }

    #[test]
    fn borrow_asset_metrics_round_trip() {
        let metrics = BorrowAssetMetrics {
            deposited_active: U128(1_000_000),
            borrowed: U128(500_000),
            collateral_deposited: U128(2_000_000),
            utilization: Some(Decimal::new("0.5")),
            interest_rate: Some(Decimal::new("0.03")),
        };
        let json = serde_json::to_string(&metrics).unwrap();
        let parsed: BorrowAssetMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(metrics, parsed);
    }

    #[test]
    fn interest_rate_model_round_trip() {
        let model = InterestRateModel {
            base_rate: Decimal::new("0.02"),
            rate_multiplier: Decimal::new("0.1"),
            jump_multiplier: Decimal::new("3.0"),
            optimal_utilization: Decimal::new("0.8"),
        };
        let json = serde_json::to_string(&model).unwrap();
        let parsed: InterestRateModel = serde_json::from_str(&json).unwrap();
        assert_eq!(model, parsed);
    }

    #[test]
    fn yield_weights_round_trip() {
        let weights = YieldWeights {
            supplier: Decimal::new("0.7"),
            treasury: Decimal::new("0.2"),
            insurance: Some(Decimal::new("0.1")),
        };
        let json = serde_json::to_string(&weights).unwrap();
        let parsed: YieldWeights = serde_json::from_str(&json).unwrap();
        assert_eq!(weights, parsed);
    }
}
