//! Transaction construction helpers for NEAR.
//!
//! [`TransactionBuilder`] provides a fluent API for building NEAR transactions
//! with function-call actions, gas estimation, and deposit formatting.
//!
//! ## Gas conventions
//!
//! - Default gas per function call: 100 `TGas` (100 * 10^12).
//! - The builder allows overriding gas per action.
//!
//! ## Deposit conventions
//!
//! - All deposits are specified in yoctoNEAR (10^-24 NEAR).
//! - Helper constants are provided for common amounts.

use near_primitives::action::{Action, FunctionCallAction};
use near_primitives::hash::CryptoHash;
use near_primitives::transaction::Transaction;
use near_primitives::types::{AccountId, Nonce};

use crate::error::CliError;

// ---------------------------------------------------------------------------
// Gas constants
// ---------------------------------------------------------------------------

/// 1 `TGas` = 10^12 gas units.
pub const TGAS: u64 = 1_000_000_000_000;

/// Default gas attached to each function call: 100 `TGas`.
pub const DEFAULT_GAS: u64 = 100 * TGAS;

/// Maximum gas per transaction (300 `TGas`).
pub const MAX_GAS: u64 = 300 * TGAS;

// ---------------------------------------------------------------------------
// Deposit constants
// ---------------------------------------------------------------------------

/// 1 NEAR in yoctoNEAR.
pub const ONE_NEAR: u128 = 1_000_000_000_000_000_000_000_000;

/// 1 yoctoNEAR — the minimum deposit for payable methods.
pub const ONE_YOCTO: u128 = 1;

/// Zero deposit — for non-payable calls.
pub const ZERO_DEPOSIT: u128 = 0;

/// Storage deposit for registering a new token account (0.00125 NEAR).
pub const STORAGE_DEPOSIT_AMOUNT: u128 = 1_250_000_000_000_000_000_000;

// ---------------------------------------------------------------------------
// TransactionBuilder
// ---------------------------------------------------------------------------

/// A single function-call action to include in a transaction.
#[derive(Debug, Clone)]
pub struct FunctionCall {
    /// Contract method name.
    pub method_name: String,
    /// JSON-serialized arguments.
    pub args: Vec<u8>,
    /// Gas to attach (in gas units).
    pub gas: u64,
    /// Deposit to attach (in yoctoNEAR).
    pub deposit: u128,
}

/// Fluent builder for constructing NEAR transactions.
///
/// # Example
///
/// ```rust,ignore
/// use templar_cli::near::tx_builder::{TransactionBuilder, DEFAULT_GAS, ONE_YOCTO};
///
/// let tx = TransactionBuilder::new("alice.near".parse()?, "contract.near".parse()?)
///     .function_call("do_something", br#"{"key":"value"}"#.to_vec(), DEFAULT_GAS, ONE_YOCTO)
///     .function_call("do_another", b"{}".to_vec(), DEFAULT_GAS, 0)
///     .build(42, public_key, block_hash)?;
/// ```
#[derive(Debug, Clone)]
pub struct TransactionBuilder {
    signer_id: AccountId,
    receiver_id: AccountId,
    actions: Vec<FunctionCall>,
}

impl TransactionBuilder {
    /// Create a new transaction builder.
    ///
    /// # Arguments
    ///
    /// * `signer_id` — the account signing and paying for the transaction.
    /// * `receiver_id` — the contract account receiving the call(s).
    pub fn new(signer_id: AccountId, receiver_id: AccountId) -> Self {
        Self { signer_id, receiver_id, actions: Vec::new() }
    }

    /// Add a function-call action with explicit gas and deposit.
    pub fn function_call(
        mut self,
        method_name: &str,
        args: Vec<u8>,
        gas: u64,
        deposit: u128,
    ) -> Self {
        self.actions.push(FunctionCall {
            method_name: method_name.to_string(),
            args,
            gas,
            deposit,
        });
        self
    }

    /// Add a function-call action with default gas and zero deposit.
    pub fn call(self, method_name: &str, args: Vec<u8>) -> Self {
        self.function_call(method_name, args, DEFAULT_GAS, ZERO_DEPOSIT)
    }

    /// Add a function-call action with default gas and 1 yoctoNEAR deposit.
    pub fn call_with_one_yocto(self, method_name: &str, args: Vec<u8>) -> Self {
        self.function_call(method_name, args, DEFAULT_GAS, ONE_YOCTO)
    }

    /// Returns the number of actions currently in the builder.
    pub fn action_count(&self) -> usize {
        self.actions.len()
    }

    /// Returns the total gas required across all actions.
    pub fn total_gas(&self) -> u64 {
        self.actions.iter().map(|a| a.gas).sum()
    }

    /// Returns the total deposit across all actions.
    pub fn total_deposit(&self) -> u128 {
        self.actions.iter().map(|a| a.deposit).sum()
    }

    /// Build the unsigned [`Transaction`].
    ///
    /// # Arguments
    ///
    /// * `nonce` — the access key nonce (will be incremented by 1 for the tx).
    /// * `public_key` — the signer's public key.
    /// * `block_hash` — a recent block hash for the transaction reference.
    ///
    /// # Errors
    ///
    /// Returns an error if no actions have been added or if total gas exceeds
    /// the per-transaction limit.
    pub fn build(
        self,
        nonce: Nonce,
        public_key: near_crypto::PublicKey,
        block_hash: CryptoHash,
    ) -> Result<Transaction, CliError> {
        if self.actions.is_empty() {
            return Err(CliError::InvalidInput("transaction must have at least one action".into()));
        }

        let total_gas = self.total_gas();
        if total_gas > MAX_GAS {
            return Err(CliError::InvalidInput(format!(
                "total gas {total_gas} exceeds maximum {MAX_GAS} (300 TGas)"
            )));
        }

        let actions: Vec<Action> = self
            .actions
            .into_iter()
            .map(|fc| {
                Action::FunctionCall(Box::new(FunctionCallAction {
                    method_name: fc.method_name,
                    args: fc.args,
                    gas: fc.gas,
                    deposit: fc.deposit,
                }))
            })
            .collect();

        Ok(Transaction::V0(near_primitives::transaction::TransactionV0 {
            signer_id: self.signer_id,
            public_key,
            nonce: nonce + 1,
            receiver_id: self.receiver_id,
            block_hash,
            actions,
        }))
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Format a yoctoNEAR amount as a human-readable NEAR string.
///
/// # Examples
///
/// ```rust,ignore
/// assert_eq!(format_near(1_000_000_000_000_000_000_000_000), "1.000000 NEAR");
/// assert_eq!(format_near(0), "0.000000 NEAR");
/// ```
pub fn format_near(yocto: u128) -> String {
    let whole = yocto / ONE_NEAR;
    let frac = yocto % ONE_NEAR;
    // Show 6 decimal places.
    let frac_str = format!("{frac:024}");
    let frac_6 = &frac_str[..6];
    format!("{whole}.{frac_6} NEAR")
}

/// Parse a NEAR amount string (e.g., "1.5") into yoctoNEAR.
///
/// Accepts integer or decimal notation. The string should NOT include
/// the "NEAR" suffix.
pub fn parse_near(amount: &str) -> Result<u128, CliError> {
    let trimmed = amount.trim();
    if trimmed.is_empty() {
        return Err(CliError::InvalidInput("empty NEAR amount".into()));
    }

    let parts: Vec<&str> = trimmed.split('.').collect();
    match parts.len() {
        1 => {
            let whole: u128 = parts[0]
                .parse()
                .map_err(|e| CliError::InvalidInput(format!("invalid NEAR amount: {e}")))?;
            Ok(whole * ONE_NEAR)
        }
        2 => {
            let whole: u128 = parts[0]
                .parse()
                .map_err(|e| CliError::InvalidInput(format!("invalid NEAR amount: {e}")))?;

            let frac_str = parts[1];
            if frac_str.len() > 24 {
                return Err(CliError::InvalidInput(
                    "NEAR amount has too many decimal places (max 24)".into(),
                ));
            }

            // Pad to 24 decimal places.
            let padded = format!("{frac_str:0<24}");
            let frac: u128 = padded
                .parse()
                .map_err(|e| CliError::InvalidInput(format!("invalid NEAR fraction: {e}")))?;

            Ok(whole * ONE_NEAR + frac)
        }
        _ => Err(CliError::InvalidInput(
            "invalid NEAR amount format (multiple decimal points)".into(),
        )),
    }
}

/// Build JSON arguments for a simple function call.
///
/// This is a convenience wrapper around `serde_json::to_vec`.
pub fn json_args<T: serde::Serialize>(args: &T) -> Result<Vec<u8>, CliError> {
    serde_json::to_vec(args).map_err(|e| {
        CliError::Serialization(format!("failed to serialize function call args: {e}"))
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_public_key() -> near_crypto::PublicKey {
        "ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp".parse().unwrap()
    }

    fn test_block_hash() -> CryptoHash {
        CryptoHash::default()
    }

    #[test]
    fn gas_constants_are_correct() {
        assert_eq!(TGAS, 1_000_000_000_000);
        assert_eq!(DEFAULT_GAS, 100_000_000_000_000);
        assert_eq!(MAX_GAS, 300_000_000_000_000);
    }

    #[test]
    fn deposit_constants_are_correct() {
        assert_eq!(ONE_NEAR, 1_000_000_000_000_000_000_000_000);
        assert_eq!(ONE_YOCTO, 1);
        assert_eq!(ZERO_DEPOSIT, 0);
    }

    #[test]
    fn builder_single_action() {
        let signer: AccountId = "alice.near".parse().unwrap();
        let receiver: AccountId = "contract.near".parse().unwrap();

        let tx = TransactionBuilder::new(signer.clone(), receiver.clone())
            .call("do_something", b"{}".to_vec())
            .build(41, test_public_key(), test_block_hash())
            .unwrap();

        match &tx {
            Transaction::V0(v0) => {
                assert_eq!(v0.signer_id, signer);
                assert_eq!(v0.receiver_id, receiver);
                assert_eq!(v0.nonce, 42);
                assert_eq!(v0.actions.len(), 1);
            }
            _ => panic!("expected Transaction::V0"),
        }
    }

    #[test]
    fn builder_multi_action() {
        let signer: AccountId = "alice.near".parse().unwrap();
        let receiver: AccountId = "contract.near".parse().unwrap();

        let builder = TransactionBuilder::new(signer, receiver)
            .call("method_a", b"{}".to_vec())
            .call_with_one_yocto("method_b", b"{}".to_vec())
            .function_call("method_c", b"{}".to_vec(), 50 * TGAS, 100);

        assert_eq!(builder.action_count(), 3);
        assert_eq!(builder.total_gas(), DEFAULT_GAS + DEFAULT_GAS + 50 * TGAS);
        assert_eq!(builder.total_deposit(), ONE_YOCTO + 100);

        let tx = builder.build(0, test_public_key(), test_block_hash()).unwrap();
        match &tx {
            Transaction::V0(v0) => {
                assert_eq!(v0.actions.len(), 3);
            }
            _ => panic!("expected Transaction::V0"),
        }
    }

    #[test]
    fn builder_empty_actions_error() {
        let signer: AccountId = "alice.near".parse().unwrap();
        let receiver: AccountId = "contract.near".parse().unwrap();

        let result = TransactionBuilder::new(signer, receiver).build(
            0,
            test_public_key(),
            test_block_hash(),
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, CliError::InvalidInput(_)));
        assert!(err.to_string().contains("at least one action"));
    }

    #[test]
    fn builder_exceeds_max_gas_error() {
        let signer: AccountId = "alice.near".parse().unwrap();
        let receiver: AccountId = "contract.near".parse().unwrap();

        let result = TransactionBuilder::new(signer, receiver)
            .function_call("a", b"{}".to_vec(), MAX_GAS, 0)
            .function_call("b", b"{}".to_vec(), 1, 0) // Exceeds by 1
            .build(0, test_public_key(), test_block_hash());

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("exceeds maximum"));
    }

    #[test]
    fn format_near_whole() {
        assert_eq!(format_near(ONE_NEAR), "1.000000 NEAR");
    }

    #[test]
    fn format_near_zero() {
        assert_eq!(format_near(0), "0.000000 NEAR");
    }

    #[test]
    fn format_near_fractional() {
        let amount = ONE_NEAR + ONE_NEAR / 2;
        assert_eq!(format_near(amount), "1.500000 NEAR");
    }

    #[test]
    fn format_near_large() {
        assert_eq!(format_near(100 * ONE_NEAR), "100.000000 NEAR");
    }

    #[test]
    fn format_near_tiny() {
        assert_eq!(format_near(1), "0.000000 NEAR");
    }

    #[test]
    fn parse_near_whole() {
        assert_eq!(parse_near("1").unwrap(), ONE_NEAR);
    }

    #[test]
    fn parse_near_decimal() {
        assert_eq!(parse_near("1.5").unwrap(), ONE_NEAR + ONE_NEAR / 2);
    }

    #[test]
    fn parse_near_zero() {
        assert_eq!(parse_near("0").unwrap(), 0);
    }

    #[test]
    fn parse_near_zero_decimal() {
        assert_eq!(parse_near("0.0").unwrap(), 0);
    }

    #[test]
    fn parse_near_full_precision() {
        // 1 yoctoNEAR
        assert_eq!(parse_near("0.000000000000000000000001").unwrap(), 1);
    }

    #[test]
    fn parse_near_empty_error() {
        assert!(parse_near("").is_err());
    }

    #[test]
    fn parse_near_invalid_error() {
        assert!(parse_near("not_a_number").is_err());
    }

    #[test]
    fn parse_near_multiple_dots_error() {
        assert!(parse_near("1.2.3").is_err());
    }

    #[test]
    fn parse_near_too_many_decimals_error() {
        let s = format!("0.{}", "1".repeat(25));
        assert!(parse_near(&s).is_err());
    }

    #[test]
    fn json_args_success() {
        #[derive(serde::Serialize)]
        struct Args {
            account_id: String,
        }
        let args = Args { account_id: "alice.near".into() };
        let bytes = json_args(&args).unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(parsed["account_id"], "alice.near");
    }

    #[test]
    fn json_args_empty_object() {
        let args = serde_json::json!({});
        let bytes = json_args(&args).unwrap();
        assert_eq!(bytes, b"{}");
    }

    #[test]
    fn storage_deposit_amount_reasonable() {
        // 0.00125 NEAR
        const { assert!(STORAGE_DEPOSIT_AMOUNT > 0) };
        const { assert!(STORAGE_DEPOSIT_AMOUNT < ONE_NEAR) };
    }

    #[test]
    fn builder_nonce_increments() {
        let signer: AccountId = "alice.near".parse().unwrap();
        let receiver: AccountId = "contract.near".parse().unwrap();

        let tx = TransactionBuilder::new(signer, receiver)
            .call("test", b"{}".to_vec())
            .build(99, test_public_key(), test_block_hash())
            .unwrap();

        match &tx {
            Transaction::V0(v0) => {
                assert_eq!(v0.nonce, 100);
            }
            _ => panic!("expected Transaction::V0"),
        }
    }

    #[test]
    fn builder_preserves_function_call_details() {
        let signer: AccountId = "alice.near".parse().unwrap();
        let receiver: AccountId = "contract.near".parse().unwrap();
        let args = br#"{"amount":"1000"}"#.to_vec();

        let tx = TransactionBuilder::new(signer, receiver)
            .function_call("transfer", args.clone(), 50 * TGAS, 42)
            .build(0, test_public_key(), test_block_hash())
            .unwrap();

        match &tx {
            Transaction::V0(v0) => {
                let action = &v0.actions[0];
                if let Action::FunctionCall(fc) = action {
                    assert_eq!(fc.method_name, "transfer");
                    assert_eq!(fc.args, args);
                    assert_eq!(fc.gas, 50 * TGAS);
                    assert_eq!(fc.deposit, 42);
                } else {
                    panic!("expected FunctionCall action");
                }
            }
            _ => panic!("expected Transaction::V0"),
        }
    }
}
