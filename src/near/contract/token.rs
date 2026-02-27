//! NEP-141 fungible token contract wrappers.
//!
//! Provides typed methods for interacting with NEP-141 fungible tokens:
//!
//! - `ft_balance_of` — query token balance
//! - `ft_metadata` — query token metadata (name, symbol, decimals)
//! - `ft_transfer_call` — transfer tokens with a message (used for supply,
//!   collateralize, repay, and liquidate operations)
//! - `storage_deposit` — register an account for token storage

use std::sync::Arc;

use near_primitives::types::AccountId;
use near_primitives::views::FinalExecutionOutcomeView;
use serde::{Deserialize, Serialize};

use crate::error::CliError;
use crate::near::contract::ContractClient;
use crate::near::rpc::NearRpcClient;
use crate::near::signer::NearSigner;
use crate::near::tx_builder::{DEFAULT_GAS, ONE_YOCTO, STORAGE_DEPOSIT_AMOUNT};

// ---------------------------------------------------------------------------
// Argument types
// ---------------------------------------------------------------------------

/// Arguments for `ft_balance_of`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceOfArgs {
    /// The account to query.
    pub account_id: AccountId,
}

/// Arguments for `ft_transfer_call`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtTransferCallArgs {
    /// The receiver contract.
    pub receiver_id: AccountId,
    /// The amount to transfer (string-encoded u128).
    pub amount: String,
    /// The message to pass to the receiver's `ft_on_transfer`.
    pub msg: String,
}

/// Arguments for `storage_deposit`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDepositArgs {
    /// The account to register. If `None`, registers the caller.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<AccountId>,
    /// Whether to register only (do not over-deposit).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration_only: Option<bool>,
}

/// NEP-148 fungible token metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtMetadata {
    /// Human-readable token name.
    pub name: String,
    /// Token ticker symbol.
    pub symbol: String,
    /// Number of decimals for display.
    pub decimals: u8,
    /// Optional icon (data URI).
    #[serde(default)]
    pub icon: Option<String>,
    /// Optional reference URL to off-chain metadata.
    #[serde(default)]
    pub reference: Option<String>,
    /// Optional hash of the reference content.
    #[serde(default)]
    pub reference_hash: Option<String>,
    /// Token metadata spec version.
    pub spec: String,
}

// ---------------------------------------------------------------------------
// TokenClient
// ---------------------------------------------------------------------------

/// Typed client for a NEP-141 fungible token contract.
pub struct TokenClient {
    inner: ContractClient,
}

impl TokenClient {
    /// Create a new token client.
    pub fn new(
        rpc: Arc<dyn NearRpcClient>,
        contract_id: AccountId,
        signer: Option<NearSigner>,
    ) -> Self {
        Self { inner: ContractClient::new(rpc, contract_id, signer) }
    }

    /// Returns the contract account ID.
    pub fn contract_id(&self) -> &AccountId {
        self.inner.contract_id()
    }

    // -----------------------------------------------------------------------
    // View calls
    // -----------------------------------------------------------------------

    /// Get the fungible token balance for an account.
    pub async fn ft_balance_of(&self, account_id: &AccountId) -> Result<String, CliError> {
        self.inner.view("ft_balance_of", &BalanceOfArgs { account_id: account_id.clone() }).await
    }

    /// Get the token metadata (NEP-148).
    pub async fn ft_metadata(&self) -> Result<FtMetadata, CliError> {
        self.inner.view("ft_metadata", &serde_json::json!({})).await
    }

    // -----------------------------------------------------------------------
    // Write calls
    // -----------------------------------------------------------------------

    /// Transfer tokens to a receiver contract with a message.
    ///
    /// This is the primary method for interacting with Templar contracts:
    /// supply, collateralize, repay, and liquidate operations all use
    /// `ft_transfer_call` with different `msg` values.
    pub async fn ft_transfer_call(
        &self,
        receiver_id: &AccountId,
        amount: &str,
        msg: &str,
    ) -> Result<FinalExecutionOutcomeView, CliError> {
        self.inner
            .call(
                "ft_transfer_call",
                &FtTransferCallArgs {
                    receiver_id: receiver_id.clone(),
                    amount: amount.to_string(),
                    msg: msg.to_string(),
                },
                DEFAULT_GAS,
                ONE_YOCTO,
            )
            .await
    }

    /// Register an account for storage on this token contract.
    ///
    /// Attaches the standard storage deposit amount (0.00125 NEAR).
    pub async fn storage_deposit(
        &self,
        account_id: Option<&AccountId>,
    ) -> Result<FinalExecutionOutcomeView, CliError> {
        self.inner
            .call(
                "storage_deposit",
                &StorageDepositArgs {
                    account_id: account_id.cloned(),
                    registration_only: Some(true),
                },
                DEFAULT_GAS,
                STORAGE_DEPOSIT_AMOUNT,
            )
            .await
    }
}

impl std::fmt::Debug for TokenClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenClient").field("contract_id", self.inner.contract_id()).finish()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::near::rpc::{MockNearRpcClient, ViewCallResult};
    use near_primitives::hash::CryptoHash;

    fn mock_view_rpc(response: serde_json::Value) -> Arc<MockNearRpcClient> {
        let mut mock = MockNearRpcClient::new();
        let bytes = serde_json::to_vec(&response).unwrap();
        mock.expect_view_function().returning(move |_, _, _| {
            Ok(ViewCallResult {
                result: bytes.clone(),
                logs: vec![],
                block_height: 400,
                block_hash: CryptoHash::default(),
            })
        });
        Arc::new(mock)
    }

    fn token_client(rpc: Arc<MockNearRpcClient>) -> TokenClient {
        let contract_id: AccountId = "usdc.near".parse().unwrap();
        TokenClient::new(rpc, contract_id, None)
    }

    #[tokio::test]
    async fn ft_balance_of() {
        let rpc = mock_view_rpc(serde_json::json!("1000000000"));
        let client = token_client(rpc);
        let account_id: AccountId = "alice.near".parse().unwrap();
        let balance = client.ft_balance_of(&account_id).await.unwrap();
        assert_eq!(balance, "1000000000");
    }

    #[tokio::test]
    async fn ft_metadata() {
        let rpc = mock_view_rpc(serde_json::json!({
            "spec": "ft-1.0.0",
            "name": "USD Coin",
            "symbol": "USDC",
            "decimals": 6,
            "icon": null,
            "reference": null,
            "reference_hash": null
        }));
        let client = token_client(rpc);
        let metadata = client.ft_metadata().await.unwrap();
        assert_eq!(metadata.name, "USD Coin");
        assert_eq!(metadata.symbol, "USDC");
        assert_eq!(metadata.decimals, 6);
        assert_eq!(metadata.spec, "ft-1.0.0");
    }

    #[tokio::test]
    async fn ft_transfer_call_without_signer_errors() {
        let rpc: Arc<MockNearRpcClient> = Arc::new(MockNearRpcClient::new());
        let client = token_client(rpc);
        let receiver: AccountId = "market.tmplr.near".parse().unwrap();
        let result = client.ft_transfer_call(&receiver, "1000000", "\"Supply\"").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CliError::Signing(_)));
    }

    #[tokio::test]
    async fn storage_deposit_without_signer_errors() {
        let rpc: Arc<MockNearRpcClient> = Arc::new(MockNearRpcClient::new());
        let client = token_client(rpc);
        let account: AccountId = "alice.near".parse().unwrap();
        let result = client.storage_deposit(Some(&account)).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CliError::Signing(_)));
    }

    #[test]
    fn token_client_contract_id() {
        let rpc: Arc<MockNearRpcClient> = Arc::new(MockNearRpcClient::new());
        let contract_id: AccountId = "usdc.near".parse().unwrap();
        let client = TokenClient::new(rpc, contract_id.clone(), None);
        assert_eq!(client.contract_id(), &contract_id);
    }

    #[test]
    fn token_client_debug() {
        let rpc: Arc<MockNearRpcClient> = Arc::new(MockNearRpcClient::new());
        let contract_id: AccountId = "usdc.near".parse().unwrap();
        let client = TokenClient::new(rpc, contract_id, None);
        let debug = format!("{client:?}");
        assert!(debug.contains("usdc.near"));
    }

    #[test]
    fn balance_of_args_serialize() {
        let args = BalanceOfArgs { account_id: "alice.near".parse().unwrap() };
        let json = serde_json::to_value(&args).unwrap();
        assert_eq!(json["account_id"], "alice.near");
    }

    #[test]
    fn ft_transfer_call_args_serialize() {
        let args = FtTransferCallArgs {
            receiver_id: "market.tmplr.near".parse().unwrap(),
            amount: "1000000".to_string(),
            msg: "\"Supply\"".to_string(),
        };
        let json = serde_json::to_value(&args).unwrap();
        assert_eq!(json["receiver_id"], "market.tmplr.near");
        assert_eq!(json["amount"], "1000000");
        assert_eq!(json["msg"], "\"Supply\"");
    }

    #[test]
    fn storage_deposit_args_with_account() {
        let args = StorageDepositArgs {
            account_id: Some("alice.near".parse().unwrap()),
            registration_only: Some(true),
        };
        let json = serde_json::to_value(&args).unwrap();
        assert_eq!(json["account_id"], "alice.near");
        assert_eq!(json["registration_only"], true);
    }

    #[test]
    fn storage_deposit_args_without_account() {
        let args = StorageDepositArgs { account_id: None, registration_only: None };
        let json = serde_json::to_value(&args).unwrap();
        assert!(json.get("account_id").is_none());
        assert!(json.get("registration_only").is_none());
    }

    #[test]
    fn ft_metadata_deserialize() {
        let json = serde_json::json!({
            "spec": "ft-1.0.0",
            "name": "Wrapped Bitcoin",
            "symbol": "WBTC",
            "decimals": 8,
            "icon": "data:image/svg+xml,...",
            "reference": "https://example.com/wbtc.json",
            "reference_hash": "abc123"
        });
        let metadata: FtMetadata = serde_json::from_value(json).unwrap();
        assert_eq!(metadata.name, "Wrapped Bitcoin");
        assert_eq!(metadata.symbol, "WBTC");
        assert_eq!(metadata.decimals, 8);
        assert!(metadata.icon.is_some());
        assert!(metadata.reference.is_some());
        assert!(metadata.reference_hash.is_some());
    }

    #[test]
    fn ft_metadata_deserialize_minimal() {
        let json = serde_json::json!({
            "spec": "ft-1.0.0",
            "name": "Test Token",
            "symbol": "TEST",
            "decimals": 18
        });
        let metadata: FtMetadata = serde_json::from_value(json).unwrap();
        assert_eq!(metadata.name, "Test Token");
        assert!(metadata.icon.is_none());
        assert!(metadata.reference.is_none());
    }
}
