//! NEP-245 multi-token contract wrappers.
//!
//! Provides typed methods for interacting with NEP-245 multi-token contracts.
//! This is the primary token standard used by Templar for cross-chain assets
//! bridged through the NEAR Intents system (`intents.near`).
//!
//! ## Token model
//!
//! Cross-chain assets are represented as NEP-245 multi-tokens:
//! `Nep245 { contract_id: "intents.near", token_id: "nep141:<asset>.omft.near" }`
//!
//! Market contracts receive these via `mt_on_transfer` (NEP-245).

use std::sync::Arc;

use near_primitives::types::AccountId;
use near_primitives::views::FinalExecutionOutcomeView;
use serde::{Deserialize, Serialize};

use crate::error::CliError;
use crate::near::contract::ContractClient;
use crate::near::rpc::NearRpcClient;
use crate::near::signer::NearSigner;
use crate::near::tx_builder::{DEFAULT_GAS, ONE_YOCTO};

// ---------------------------------------------------------------------------
// Argument types
// ---------------------------------------------------------------------------

/// Arguments for `mt_balance_of`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtBalanceOfArgs {
    /// The account to query.
    pub account_id: AccountId,
    /// The token ID within the multi-token contract.
    pub token_id: String,
}

/// Arguments for `mt_transfer_call`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtTransferCallArgs {
    /// The receiver contract.
    pub receiver_id: AccountId,
    /// The token ID to transfer.
    pub token_id: String,
    /// The amount to transfer (string-encoded u128).
    pub amount: String,
    /// The message to pass to the receiver's `mt_on_transfer`.
    pub msg: String,
    /// Optional memo for on-chain logging.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
}

/// Arguments for `mt_metadata` (base metadata query).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtMetadataArgs {
    /// The token ID to query metadata for.
    pub token_id: String,
}

/// NEP-245 token metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtTokenMetadata {
    /// Human-readable token name.
    pub name: String,
    /// Token ticker symbol.
    pub symbol: String,
    /// Number of decimals for display.
    #[serde(default)]
    pub decimals: Option<u8>,
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
    #[serde(default)]
    pub spec: Option<String>,
}

// ---------------------------------------------------------------------------
// MultiTokenClient
// ---------------------------------------------------------------------------

/// Typed client for a NEP-245 multi-token contract.
pub struct MultiTokenClient {
    inner: ContractClient,
}

impl MultiTokenClient {
    /// Create a new multi-token client.
    pub fn new(
        rpc: Arc<dyn NearRpcClient>,
        contract_id: AccountId,
        signer: Option<NearSigner>,
    ) -> Self {
        Self {
            inner: ContractClient::new(rpc, contract_id, signer),
        }
    }

    /// Returns the contract account ID.
    pub fn contract_id(&self) -> &AccountId {
        self.inner.contract_id()
    }

    // -----------------------------------------------------------------------
    // View calls
    // -----------------------------------------------------------------------

    /// Get the balance of a specific token for an account.
    pub async fn mt_balance_of(
        &self,
        account_id: &AccountId,
        token_id: &str,
    ) -> Result<String, CliError> {
        self.inner
            .view(
                "mt_balance_of",
                &MtBalanceOfArgs {
                    account_id: account_id.clone(),
                    token_id: token_id.to_string(),
                },
            )
            .await
    }

    /// Get token metadata for a specific token ID.
    pub async fn mt_metadata(
        &self,
        token_id: &str,
    ) -> Result<serde_json::Value, CliError> {
        self.inner
            .view(
                "mt_metadata",
                &MtMetadataArgs {
                    token_id: token_id.to_string(),
                },
            )
            .await
    }

    // -----------------------------------------------------------------------
    // Write calls
    // -----------------------------------------------------------------------

    /// Transfer multi-tokens to a receiver contract with a message.
    ///
    /// This is the primary method for interacting with Templar market
    /// contracts for cross-chain assets. The `msg` parameter encodes the
    /// market action (supply, collateralize, repay, liquidate).
    pub async fn mt_transfer_call(
        &self,
        receiver_id: &AccountId,
        token_id: &str,
        amount: &str,
        msg: &str,
        memo: Option<&str>,
    ) -> Result<FinalExecutionOutcomeView, CliError> {
        self.inner
            .call(
                "mt_transfer_call",
                &MtTransferCallArgs {
                    receiver_id: receiver_id.clone(),
                    token_id: token_id.to_string(),
                    amount: amount.to_string(),
                    msg: msg.to_string(),
                    memo: memo.map(ToString::to_string),
                },
                DEFAULT_GAS,
                ONE_YOCTO,
            )
            .await
    }
}

impl std::fmt::Debug for MultiTokenClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MultiTokenClient")
            .field("contract_id", self.inner.contract_id())
            .finish()
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
        mock.expect_view_function()
            .returning(move |_, _, _| {
                Ok(ViewCallResult {
                    result: bytes.clone(),
                    logs: vec![],
                    block_height: 500,
                    block_hash: CryptoHash::default(),
                })
            });
        Arc::new(mock)
    }

    fn mt_client(rpc: Arc<MockNearRpcClient>) -> MultiTokenClient {
        let contract_id: AccountId = "intents.near".parse().unwrap();
        MultiTokenClient::new(rpc, contract_id, None)
    }

    #[tokio::test]
    async fn mt_balance_of() {
        let rpc = mock_view_rpc(serde_json::json!("500000000"));
        let client = mt_client(rpc);
        let account_id: AccountId = "alice.near".parse().unwrap();
        let balance = client
            .mt_balance_of(&account_id, "nep141:wbtc.omft.near")
            .await
            .unwrap();
        assert_eq!(balance, "500000000");
    }

    #[tokio::test]
    async fn mt_metadata() {
        let rpc = mock_view_rpc(serde_json::json!({
            "name": "Wrapped Bitcoin",
            "symbol": "WBTC",
            "decimals": 8
        }));
        let client = mt_client(rpc);
        let metadata = client
            .mt_metadata("nep141:wbtc.omft.near")
            .await
            .unwrap();
        assert_eq!(metadata["name"], "Wrapped Bitcoin");
        assert_eq!(metadata["symbol"], "WBTC");
        assert_eq!(metadata["decimals"], 8);
    }

    #[tokio::test]
    async fn mt_transfer_call_without_signer_errors() {
        let rpc: Arc<MockNearRpcClient> = Arc::new(MockNearRpcClient::new());
        let client = mt_client(rpc);
        let receiver: AccountId = "market.tmplr.near".parse().unwrap();
        let result = client
            .mt_transfer_call(
                &receiver,
                "nep141:wbtc.omft.near",
                "100000000",
                "\"Collateralize\"",
                None,
            )
            .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CliError::Signing(_)));
    }

    #[tokio::test]
    async fn mt_transfer_call_with_memo_without_signer_errors() {
        let rpc: Arc<MockNearRpcClient> = Arc::new(MockNearRpcClient::new());
        let client = mt_client(rpc);
        let receiver: AccountId = "market.tmplr.near".parse().unwrap();
        let result = client
            .mt_transfer_call(
                &receiver,
                "nep141:wbtc.omft.near",
                "100000000",
                "\"Supply\"",
                Some("templar-cli supply"),
            )
            .await;
        assert!(result.is_err());
    }

    #[test]
    fn multi_token_client_contract_id() {
        let rpc: Arc<MockNearRpcClient> = Arc::new(MockNearRpcClient::new());
        let contract_id: AccountId = "intents.near".parse().unwrap();
        let client = MultiTokenClient::new(rpc, contract_id.clone(), None);
        assert_eq!(client.contract_id(), &contract_id);
    }

    #[test]
    fn multi_token_client_debug() {
        let rpc: Arc<MockNearRpcClient> = Arc::new(MockNearRpcClient::new());
        let contract_id: AccountId = "intents.near".parse().unwrap();
        let client = MultiTokenClient::new(rpc, contract_id, None);
        let debug = format!("{client:?}");
        assert!(debug.contains("intents.near"));
    }

    #[test]
    fn mt_balance_of_args_serialize() {
        let args = MtBalanceOfArgs {
            account_id: "alice.near".parse().unwrap(),
            token_id: "nep141:wbtc.omft.near".to_string(),
        };
        let json = serde_json::to_value(&args).unwrap();
        assert_eq!(json["account_id"], "alice.near");
        assert_eq!(json["token_id"], "nep141:wbtc.omft.near");
    }

    #[test]
    fn mt_transfer_call_args_serialize() {
        let args = MtTransferCallArgs {
            receiver_id: "market.tmplr.near".parse().unwrap(),
            token_id: "nep141:wbtc.omft.near".to_string(),
            amount: "100000000".to_string(),
            msg: "\"Collateralize\"".to_string(),
            memo: None,
        };
        let json = serde_json::to_value(&args).unwrap();
        assert_eq!(json["receiver_id"], "market.tmplr.near");
        assert_eq!(json["token_id"], "nep141:wbtc.omft.near");
        assert_eq!(json["amount"], "100000000");
        assert_eq!(json["msg"], "\"Collateralize\"");
        assert!(json.get("memo").is_none());
    }

    #[test]
    fn mt_transfer_call_args_with_memo_serialize() {
        let args = MtTransferCallArgs {
            receiver_id: "market.tmplr.near".parse().unwrap(),
            token_id: "nep141:wbtc.omft.near".to_string(),
            amount: "100000000".to_string(),
            msg: "\"Supply\"".to_string(),
            memo: Some("templar-cli supply".to_string()),
        };
        let json = serde_json::to_value(&args).unwrap();
        assert_eq!(json["memo"], "templar-cli supply");
    }

    #[test]
    fn mt_metadata_args_serialize() {
        let args = MtMetadataArgs {
            token_id: "nep141:usdc.omft.near".to_string(),
        };
        let json = serde_json::to_value(&args).unwrap();
        assert_eq!(json["token_id"], "nep141:usdc.omft.near");
    }

    #[test]
    fn mt_token_metadata_deserialize_full() {
        let json = serde_json::json!({
            "name": "Wrapped BTC",
            "symbol": "WBTC",
            "decimals": 8,
            "icon": "data:image/svg+xml,...",
            "reference": "https://example.com",
            "reference_hash": "abc123",
            "spec": "mt-1.0.0"
        });
        let meta: MtTokenMetadata = serde_json::from_value(json).unwrap();
        assert_eq!(meta.name, "Wrapped BTC");
        assert_eq!(meta.symbol, "WBTC");
        assert_eq!(meta.decimals, Some(8));
        assert!(meta.icon.is_some());
    }

    #[test]
    fn mt_token_metadata_deserialize_minimal() {
        let json = serde_json::json!({
            "name": "Test Token",
            "symbol": "TEST"
        });
        let meta: MtTokenMetadata = serde_json::from_value(json).unwrap();
        assert_eq!(meta.name, "Test Token");
        assert_eq!(meta.symbol, "TEST");
        assert!(meta.decimals.is_none());
        assert!(meta.icon.is_none());
        assert!(meta.spec.is_none());
    }
}
