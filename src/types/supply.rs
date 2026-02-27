//! Supply position types.
//!
//! These types represent a supplier's position in a Templar lending market.
//! A [`SupplyPosition`] tracks the deposited borrow asset, any incoming or
//! outgoing amounts, and accumulated yield.

use serde::{Deserialize, Serialize};

use super::number::{Accumulator, U128, U64};

// ---------------------------------------------------------------------------
// SupplyPosition
// ---------------------------------------------------------------------------

/// A supply (lender) position in a Templar lending market.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupplyPosition {
    /// Block timestamp (milliseconds) when the position was opened.
    pub started_at_block_timestamp_ms: U64,

    /// The deposit breakdown (active, incoming, outgoing).
    pub borrow_asset_deposit: Deposit,

    /// Accumulated yield tracked via an accumulator.
    pub borrow_asset_yield: Accumulator,
}

// ---------------------------------------------------------------------------
// Deposit
// ---------------------------------------------------------------------------

/// Breakdown of a supply deposit's lifecycle states.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deposit {
    /// Amount currently active (earning yield).
    pub active: U128,

    /// Deposits that are pending activation (e.g., awaiting next epoch).
    #[serde(default)]
    pub incoming: Vec<IncomingDeposit>,

    /// Amount queued for withdrawal.
    #[serde(default)]
    pub outgoing: U128,
}

/// A pending incoming deposit awaiting activation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncomingDeposit {
    /// Amount of the incoming deposit.
    pub amount: U128,

    /// Block timestamp (milliseconds) when this deposit will become active.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activation_timestamp_ms: Option<U64>,
}

// ---------------------------------------------------------------------------
// WithdrawalRequestStatus
// ---------------------------------------------------------------------------

/// Status of a withdrawal request from a supply position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum WithdrawalRequestStatus {
    /// The withdrawal is queued and waiting to be processed.
    Pending {
        /// Amount requested for withdrawal.
        amount: U128,
        /// Timestamp when the request was created.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        requested_at_ms: Option<U64>,
    },

    /// The withdrawal is ready to be claimed.
    Ready {
        /// Amount available to claim.
        amount: U128,
    },

    /// The withdrawal has been completed.
    Completed {
        /// Amount that was withdrawn.
        amount: U128,
        /// Timestamp when the withdrawal was completed.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        completed_at_ms: Option<U64>,
    },

    /// The withdrawal request was cancelled.
    Cancelled,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_supply_position() -> SupplyPosition {
        SupplyPosition {
            started_at_block_timestamp_ms: U64(1_700_000_000_000),
            borrow_asset_deposit: Deposit {
                active: U128(10_000_000),
                incoming: vec![IncomingDeposit {
                    amount: U128(1_000_000),
                    activation_timestamp_ms: Some(U64(1_700_001_000_000)),
                }],
                outgoing: U128(500_000),
            },
            borrow_asset_yield: Accumulator::new("1.05"),
        }
    }

    #[test]
    fn supply_position_round_trip() {
        let original = sample_supply_position();
        let json = serde_json::to_string(&original).unwrap();
        let parsed: SupplyPosition = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn supply_position_json_format() {
        let pos = sample_supply_position();
        let value = serde_json::to_value(&pos).unwrap();
        assert_eq!(value["started_at_block_timestamp_ms"], "1700000000000");
        assert_eq!(value["borrow_asset_deposit"]["active"], "10000000");
        assert_eq!(value["borrow_asset_deposit"]["outgoing"], "500000");
        assert_eq!(value["borrow_asset_yield"], "1.05");
        let incoming = value["borrow_asset_deposit"]["incoming"].as_array().unwrap();
        assert_eq!(incoming.len(), 1);
        assert_eq!(incoming[0]["amount"], "1000000");
    }

    #[test]
    fn supply_position_empty_incoming() {
        let pos = SupplyPosition {
            started_at_block_timestamp_ms: U64(1_700_000_000_000),
            borrow_asset_deposit: Deposit {
                active: U128(10_000_000),
                incoming: vec![],
                outgoing: U128(0),
            },
            borrow_asset_yield: Accumulator::new("1.0"),
        };
        let json = serde_json::to_string(&pos).unwrap();
        let parsed: SupplyPosition = serde_json::from_str(&json).unwrap();
        assert_eq!(pos, parsed);
    }

    #[test]
    fn deposit_defaults() {
        // incoming and outgoing should default when missing
        let json = r#"{"active": "1000"}"#;
        let deposit: Deposit = serde_json::from_str(json).unwrap();
        assert_eq!(deposit.active.0, 1000);
        assert!(deposit.incoming.is_empty());
        assert_eq!(deposit.outgoing.0, 0);
    }

    #[test]
    fn deposit_round_trip() {
        let deposit = Deposit {
            active: U128(5_000_000),
            incoming: vec![
                IncomingDeposit {
                    amount: U128(1_000_000),
                    activation_timestamp_ms: Some(U64(1_700_000_000_000)),
                },
                IncomingDeposit {
                    amount: U128(2_000_000),
                    activation_timestamp_ms: None,
                },
            ],
            outgoing: U128(100_000),
        };
        let json = serde_json::to_string(&deposit).unwrap();
        let parsed: Deposit = serde_json::from_str(&json).unwrap();
        assert_eq!(deposit, parsed);
    }

    #[test]
    fn incoming_deposit_round_trip() {
        let incoming = IncomingDeposit {
            amount: U128(1_000_000),
            activation_timestamp_ms: Some(U64(1_700_000_000_000)),
        };
        let json = serde_json::to_string(&incoming).unwrap();
        let parsed: IncomingDeposit = serde_json::from_str(&json).unwrap();
        assert_eq!(incoming, parsed);
    }

    #[test]
    fn withdrawal_status_pending_round_trip() {
        let status = WithdrawalRequestStatus::Pending {
            amount: U128(1_000_000),
            requested_at_ms: Some(U64(1_700_000_000_000)),
        };
        let json = serde_json::to_string(&status).unwrap();
        let parsed: WithdrawalRequestStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, parsed);
    }

    #[test]
    fn withdrawal_status_ready_round_trip() {
        let status = WithdrawalRequestStatus::Ready {
            amount: U128(1_000_000),
        };
        let json = serde_json::to_string(&status).unwrap();
        let parsed: WithdrawalRequestStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, parsed);
    }

    #[test]
    fn withdrawal_status_completed_round_trip() {
        let status = WithdrawalRequestStatus::Completed {
            amount: U128(1_000_000),
            completed_at_ms: Some(U64(1_700_001_000_000)),
        };
        let json = serde_json::to_string(&status).unwrap();
        let parsed: WithdrawalRequestStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, parsed);
    }

    #[test]
    fn withdrawal_status_cancelled_round_trip() {
        let status = WithdrawalRequestStatus::Cancelled;
        let json = serde_json::to_string(&status).unwrap();
        let parsed: WithdrawalRequestStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, parsed);
    }

    #[test]
    fn withdrawal_status_json_format() {
        let status = WithdrawalRequestStatus::Pending {
            amount: U128(5000),
            requested_at_ms: None,
        };
        let value = serde_json::to_value(&status).unwrap();
        assert_eq!(value["status"], "Pending");
        assert_eq!(value["amount"], "5000");
    }

    #[test]
    fn withdrawal_status_cancelled_json() {
        let status = WithdrawalRequestStatus::Cancelled;
        let value = serde_json::to_value(&status).unwrap();
        assert_eq!(value["status"], "Cancelled");
    }
}
