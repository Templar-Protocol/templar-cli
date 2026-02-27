//! Borrow position types.
//!
//! These types represent an active or historical borrow position in a Templar
//! lending market. A [`BorrowPosition`] tracks the collateral deposited, the
//! principal borrowed, accumulated interest, and in-flight bridge amounts.

use serde::{Deserialize, Serialize};

use super::number::{Accumulator, U128, U64};

// ---------------------------------------------------------------------------
// BorrowPosition
// ---------------------------------------------------------------------------

/// A borrow position in a Templar lending market.
///
/// Serialization matches the contract's JSON output, including the `serde(alias)`
/// and `serde(default)` annotations from the contract code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BorrowPosition {
    /// Block timestamp (milliseconds) when the position was opened.
    pub started_at_block_timestamp_ms: U64,

    /// Amount of collateral asset deposited.
    pub collateral_asset_deposit: U128,

    /// Principal amount of borrow asset borrowed.
    pub borrow_asset_principal: U128,

    /// Accumulated interest tracked via an accumulator.
    ///
    /// The contract originally named this field `borrow_asset_fees`, so we
    /// accept both names during deserialization.
    #[serde(alias = "borrow_asset_fees")]
    pub interest: Accumulator,

    /// Accrued fees owed on this position.
    #[serde(default)]
    pub fees: U128,

    /// Borrow asset amount currently in flight via cross-chain bridge.
    #[serde(default)]
    pub borrow_asset_in_flight: U128,

    /// Collateral asset amount currently in flight via cross-chain bridge.
    #[serde(default)]
    pub collateral_asset_in_flight: U128,
}

// ---------------------------------------------------------------------------
// BorrowStatus
// ---------------------------------------------------------------------------

/// Health status of a borrow position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", content = "reason")]
pub enum BorrowStatus {
    /// Position is healthy — collateral ratio is above maintenance threshold.
    Healthy,

    /// Position has dropped below the maintenance collateral ratio but is not
    /// yet eligible for liquidation.
    MaintenanceRequired,

    /// Position is eligible for liquidation.
    Liquidation(LiquidationReason),
}

// ---------------------------------------------------------------------------
// LiquidationReason
// ---------------------------------------------------------------------------

/// Reason a borrow position became eligible for liquidation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LiquidationReason {
    /// Collateral value fell below the liquidation threshold.
    Undercollateralization,

    /// The borrow duration exceeded the maximum allowed time.
    Expiration,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_borrow_position() -> BorrowPosition {
        BorrowPosition {
            started_at_block_timestamp_ms: U64(1_700_000_000_000),
            collateral_asset_deposit: U128(10_000_000_000_000_000_000_000_000),
            borrow_asset_principal: U128(5_000_000),
            interest: Accumulator::new("1.000000001"),
            fees: U128(1000),
            borrow_asset_in_flight: U128(0),
            collateral_asset_in_flight: U128(0),
        }
    }

    #[test]
    fn borrow_position_round_trip() {
        let original = sample_borrow_position();
        let json = serde_json::to_string(&original).unwrap();
        let parsed: BorrowPosition = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn borrow_position_json_format() {
        let pos = sample_borrow_position();
        let value = serde_json::to_value(&pos).unwrap();
        assert_eq!(value["started_at_block_timestamp_ms"], "1700000000000");
        assert_eq!(value["collateral_asset_deposit"], "10000000000000000000000000");
        assert_eq!(value["borrow_asset_principal"], "5000000");
        assert_eq!(value["interest"], "1.000000001");
        assert_eq!(value["fees"], "1000");
        assert_eq!(value["borrow_asset_in_flight"], "0");
        assert_eq!(value["collateral_asset_in_flight"], "0");
    }

    #[test]
    fn borrow_position_alias_interest() {
        // Contract may send "borrow_asset_fees" instead of "interest"
        let json = r#"{
            "started_at_block_timestamp_ms": "1700000000000",
            "collateral_asset_deposit": "1000",
            "borrow_asset_principal": "500",
            "borrow_asset_fees": "1.0001"
        }"#;
        let pos: BorrowPosition = serde_json::from_str(json).unwrap();
        assert_eq!(pos.interest.0 .0, "1.0001");
    }

    #[test]
    fn borrow_position_defaults() {
        // fees and in_flight fields default to 0 if missing
        let json = r#"{
            "started_at_block_timestamp_ms": "1700000000000",
            "collateral_asset_deposit": "1000",
            "borrow_asset_principal": "500",
            "interest": "1.0"
        }"#;
        let pos: BorrowPosition = serde_json::from_str(json).unwrap();
        assert_eq!(pos.fees.0, 0);
        assert_eq!(pos.borrow_asset_in_flight.0, 0);
        assert_eq!(pos.collateral_asset_in_flight.0, 0);
    }

    #[test]
    fn borrow_status_healthy_round_trip() {
        let status = BorrowStatus::Healthy;
        let json = serde_json::to_string(&status).unwrap();
        let parsed: BorrowStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, parsed);
    }

    #[test]
    fn borrow_status_maintenance_round_trip() {
        let status = BorrowStatus::MaintenanceRequired;
        let json = serde_json::to_string(&status).unwrap();
        let parsed: BorrowStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, parsed);
    }

    #[test]
    fn borrow_status_liquidation_undercollateralization() {
        let status = BorrowStatus::Liquidation(LiquidationReason::Undercollateralization);
        let json = serde_json::to_string(&status).unwrap();
        let parsed: BorrowStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, parsed);
    }

    #[test]
    fn borrow_status_liquidation_expiration() {
        let status = BorrowStatus::Liquidation(LiquidationReason::Expiration);
        let json = serde_json::to_string(&status).unwrap();
        let parsed: BorrowStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, parsed);
    }

    #[test]
    fn borrow_status_json_format_healthy() {
        let status = BorrowStatus::Healthy;
        let value = serde_json::to_value(&status).unwrap();
        assert_eq!(value["status"], "Healthy");
    }

    #[test]
    fn borrow_status_json_format_liquidation() {
        let status = BorrowStatus::Liquidation(LiquidationReason::Undercollateralization);
        let value = serde_json::to_value(&status).unwrap();
        assert_eq!(value["status"], "Liquidation");
        assert_eq!(value["reason"], "Undercollateralization");
    }

    #[test]
    fn liquidation_reason_round_trip() {
        for reason in [LiquidationReason::Undercollateralization, LiquidationReason::Expiration] {
            let json = serde_json::to_string(&reason).unwrap();
            let parsed: LiquidationReason = serde_json::from_str(&json).unwrap();
            assert_eq!(reason, parsed);
        }
    }

    #[test]
    fn liquidation_reason_json_strings() {
        let u = serde_json::to_string(&LiquidationReason::Undercollateralization).unwrap();
        assert_eq!(u, "\"Undercollateralization\"");
        let e = serde_json::to_string(&LiquidationReason::Expiration).unwrap();
        assert_eq!(e, "\"Expiration\"");
    }
}
