//! Templar vault contract wrappers (ERC-4626-style).
//!
//! Provides typed view-call and function-call methods for Templar vault
//! contracts. Vaults follow the ERC-4626 tokenized vault pattern adapted
//! for NEAR:
//!
//! - Deposits via `ft_transfer_call` on the underlying asset token
//! - Share-based accounting: `convert_to_shares` / `convert_to_assets`
//! - Withdrawals and redemptions directly on the vault contract

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

/// Arguments for amount-based vault operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultAmountArgs {
    /// Amount in the asset's base units (string-encoded u128).
    pub amount: String,
}

/// Arguments for withdrawal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawArgs {
    /// Amount of assets to withdraw.
    pub assets: String,
    /// Receiver of the withdrawn assets.
    pub receiver_id: AccountId,
    /// Owner of the shares being burned.
    pub owner_id: AccountId,
}

/// Arguments for redemption.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedeemArgs {
    /// Amount of vault shares to redeem.
    pub shares: String,
    /// Receiver of the redeemed assets.
    pub receiver_id: AccountId,
    /// Owner of the shares being burned.
    pub owner_id: AccountId,
}

/// Arguments for conversion queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertArgs {
    /// The amount to convert.
    pub amount: String,
}

// ---------------------------------------------------------------------------
// VaultClient
// ---------------------------------------------------------------------------

/// Typed client for a Templar vault contract.
pub struct VaultClient {
    inner: ContractClient,
}

impl VaultClient {
    /// Create a new vault client.
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

    /// Get the vault configuration.
    pub async fn get_configuration(&self) -> Result<serde_json::Value, CliError> {
        self.inner.view("get_configuration", &serde_json::json!({})).await
    }

    /// Get total assets held by the vault.
    pub async fn get_total_assets(&self) -> Result<String, CliError> {
        self.inner.view("get_total_assets", &serde_json::json!({})).await
    }

    /// Get the last recorded total assets (before latest accrual).
    pub async fn get_last_total_assets(&self) -> Result<String, CliError> {
        self.inner.view("get_last_total_assets", &serde_json::json!({})).await
    }

    /// Get total supply of vault shares.
    pub async fn get_total_supply(&self) -> Result<String, CliError> {
        self.inner.view("get_total_supply", &serde_json::json!({})).await
    }

    /// Get the maximum deposit amount.
    pub async fn get_max_deposit(&self) -> Result<String, CliError> {
        self.inner.view("get_max_deposit", &serde_json::json!({})).await
    }

    /// Convert an asset amount to the equivalent vault shares.
    pub async fn convert_to_shares(&self, amount: &str) -> Result<String, CliError> {
        self.inner.view("convert_to_shares", &ConvertArgs { amount: amount.to_string() }).await
    }

    /// Convert a vault share amount to the equivalent assets.
    pub async fn convert_to_assets(&self, amount: &str) -> Result<String, CliError> {
        self.inner.view("convert_to_assets", &ConvertArgs { amount: amount.to_string() }).await
    }

    /// Preview the shares received for a deposit of the given asset amount.
    pub async fn preview_deposit(&self, amount: &str) -> Result<String, CliError> {
        self.inner.view("preview_deposit", &ConvertArgs { amount: amount.to_string() }).await
    }

    /// Preview the assets needed for minting a given number of shares.
    pub async fn preview_mint(&self, amount: &str) -> Result<String, CliError> {
        self.inner.view("preview_mint", &ConvertArgs { amount: amount.to_string() }).await
    }

    /// Preview the shares burned for a withdrawal of the given asset amount.
    pub async fn preview_withdraw(&self, amount: &str) -> Result<String, CliError> {
        self.inner.view("preview_withdraw", &ConvertArgs { amount: amount.to_string() }).await
    }

    /// Preview the assets returned for redeeming a given number of shares.
    pub async fn preview_redeem(&self, amount: &str) -> Result<String, CliError> {
        self.inner.view("preview_redeem", &ConvertArgs { amount: amount.to_string() }).await
    }

    /// Get the vault's cap groups configuration.
    pub async fn get_cap_groups(&self) -> Result<serde_json::Value, CliError> {
        self.inner.view("get_cap_groups", &serde_json::json!({})).await
    }

    /// Get the vault's fee configuration.
    pub async fn get_fees(&self) -> Result<serde_json::Value, CliError> {
        self.inner.view("get_fees", &serde_json::json!({})).await
    }

    /// Get the vault's access restrictions.
    pub async fn get_restrictions(&self) -> Result<serde_json::Value, CliError> {
        self.inner.view("get_restrictions", &serde_json::json!({})).await
    }

    // -----------------------------------------------------------------------
    // Write calls
    // -----------------------------------------------------------------------

    /// Withdraw assets from the vault (burns shares).
    pub async fn withdraw(
        &self,
        assets: &str,
        receiver_id: &AccountId,
        owner_id: &AccountId,
    ) -> Result<FinalExecutionOutcomeView, CliError> {
        self.inner
            .call(
                "withdraw",
                &WithdrawArgs {
                    assets: assets.to_string(),
                    receiver_id: receiver_id.clone(),
                    owner_id: owner_id.clone(),
                },
                DEFAULT_GAS,
                ONE_YOCTO,
            )
            .await
    }

    /// Redeem vault shares for assets.
    pub async fn redeem(
        &self,
        shares: &str,
        receiver_id: &AccountId,
        owner_id: &AccountId,
    ) -> Result<FinalExecutionOutcomeView, CliError> {
        self.inner
            .call(
                "redeem",
                &RedeemArgs {
                    shares: shares.to_string(),
                    receiver_id: receiver_id.clone(),
                    owner_id: owner_id.clone(),
                },
                DEFAULT_GAS,
                ONE_YOCTO,
            )
            .await
    }
}

impl std::fmt::Debug for VaultClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VaultClient").field("contract_id", self.inner.contract_id()).finish()
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
                block_height: 200,
                block_hash: CryptoHash::default(),
            })
        });
        Arc::new(mock)
    }

    fn vault_client(rpc: Arc<MockNearRpcClient>) -> VaultClient {
        let contract_id: AccountId = "vault.tmplr.near".parse().unwrap();
        VaultClient::new(rpc, contract_id, None)
    }

    #[tokio::test]
    async fn get_configuration() {
        let rpc = mock_view_rpc(serde_json::json!({
            "asset": "usdc.near",
            "decimals": 6
        }));
        let client = vault_client(rpc);
        let config = client.get_configuration().await.unwrap();
        assert_eq!(config["asset"], "usdc.near");
    }

    #[tokio::test]
    async fn get_total_assets() {
        let rpc = mock_view_rpc(serde_json::json!("5000000000"));
        let client = vault_client(rpc);
        let total = client.get_total_assets().await.unwrap();
        assert_eq!(total, "5000000000");
    }

    #[tokio::test]
    async fn get_last_total_assets() {
        let rpc = mock_view_rpc(serde_json::json!("4900000000"));
        let client = vault_client(rpc);
        let total = client.get_last_total_assets().await.unwrap();
        assert_eq!(total, "4900000000");
    }

    #[tokio::test]
    async fn get_total_supply() {
        let rpc = mock_view_rpc(serde_json::json!("4500000000"));
        let client = vault_client(rpc);
        let total = client.get_total_supply().await.unwrap();
        assert_eq!(total, "4500000000");
    }

    #[tokio::test]
    async fn get_max_deposit() {
        let rpc = mock_view_rpc(serde_json::json!("100000000000"));
        let client = vault_client(rpc);
        let max = client.get_max_deposit().await.unwrap();
        assert_eq!(max, "100000000000");
    }

    #[tokio::test]
    async fn convert_to_shares() {
        let rpc = mock_view_rpc(serde_json::json!("900"));
        let client = vault_client(rpc);
        let shares = client.convert_to_shares("1000").await.unwrap();
        assert_eq!(shares, "900");
    }

    #[tokio::test]
    async fn convert_to_assets() {
        let rpc = mock_view_rpc(serde_json::json!("1111"));
        let client = vault_client(rpc);
        let assets = client.convert_to_assets("1000").await.unwrap();
        assert_eq!(assets, "1111");
    }

    #[tokio::test]
    async fn preview_deposit() {
        let rpc = mock_view_rpc(serde_json::json!("950"));
        let client = vault_client(rpc);
        let shares = client.preview_deposit("1000").await.unwrap();
        assert_eq!(shares, "950");
    }

    #[tokio::test]
    async fn preview_mint() {
        let rpc = mock_view_rpc(serde_json::json!("1050"));
        let client = vault_client(rpc);
        let assets = client.preview_mint("1000").await.unwrap();
        assert_eq!(assets, "1050");
    }

    #[tokio::test]
    async fn preview_withdraw() {
        let rpc = mock_view_rpc(serde_json::json!("1100"));
        let client = vault_client(rpc);
        let shares = client.preview_withdraw("1000").await.unwrap();
        assert_eq!(shares, "1100");
    }

    #[tokio::test]
    async fn preview_redeem() {
        let rpc = mock_view_rpc(serde_json::json!("980"));
        let client = vault_client(rpc);
        let assets = client.preview_redeem("1000").await.unwrap();
        assert_eq!(assets, "980");
    }

    #[tokio::test]
    async fn get_cap_groups() {
        let rpc = mock_view_rpc(serde_json::json!({"groups": []}));
        let client = vault_client(rpc);
        let caps = client.get_cap_groups().await.unwrap();
        assert!(caps["groups"].is_array());
    }

    #[tokio::test]
    async fn get_fees() {
        let rpc = mock_view_rpc(serde_json::json!({"management_fee": "0.02"}));
        let client = vault_client(rpc);
        let fees = client.get_fees().await.unwrap();
        assert_eq!(fees["management_fee"], "0.02");
    }

    #[tokio::test]
    async fn get_restrictions() {
        let rpc = mock_view_rpc(serde_json::json!({"whitelist_only": false}));
        let client = vault_client(rpc);
        let restrictions = client.get_restrictions().await.unwrap();
        assert_eq!(restrictions["whitelist_only"], false);
    }

    #[tokio::test]
    async fn withdraw_without_signer_errors() {
        let rpc: Arc<MockNearRpcClient> = Arc::new(MockNearRpcClient::new());
        let client = vault_client(rpc);
        let receiver: AccountId = "alice.near".parse().unwrap();
        let owner: AccountId = "alice.near".parse().unwrap();
        let result = client.withdraw("1000", &receiver, &owner).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CliError::Signing(_)));
    }

    #[tokio::test]
    async fn redeem_without_signer_errors() {
        let rpc: Arc<MockNearRpcClient> = Arc::new(MockNearRpcClient::new());
        let client = vault_client(rpc);
        let receiver: AccountId = "alice.near".parse().unwrap();
        let owner: AccountId = "alice.near".parse().unwrap();
        let result = client.redeem("500", &receiver, &owner).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CliError::Signing(_)));
    }

    #[test]
    fn vault_client_contract_id() {
        let rpc: Arc<MockNearRpcClient> = Arc::new(MockNearRpcClient::new());
        let contract_id: AccountId = "vault.tmplr.near".parse().unwrap();
        let client = VaultClient::new(rpc, contract_id.clone(), None);
        assert_eq!(client.contract_id(), &contract_id);
    }

    #[test]
    fn vault_client_debug() {
        let rpc: Arc<MockNearRpcClient> = Arc::new(MockNearRpcClient::new());
        let contract_id: AccountId = "vault.tmplr.near".parse().unwrap();
        let client = VaultClient::new(rpc, contract_id, None);
        let debug = format!("{client:?}");
        assert!(debug.contains("vault.tmplr.near"));
    }

    #[test]
    fn withdraw_args_serialize() {
        let args = WithdrawArgs {
            assets: "1000".to_string(),
            receiver_id: "alice.near".parse().unwrap(),
            owner_id: "alice.near".parse().unwrap(),
        };
        let json = serde_json::to_value(&args).unwrap();
        assert_eq!(json["assets"], "1000");
        assert_eq!(json["receiver_id"], "alice.near");
    }

    #[test]
    fn redeem_args_serialize() {
        let args = RedeemArgs {
            shares: "500".to_string(),
            receiver_id: "bob.near".parse().unwrap(),
            owner_id: "bob.near".parse().unwrap(),
        };
        let json = serde_json::to_value(&args).unwrap();
        assert_eq!(json["shares"], "500");
        assert_eq!(json["receiver_id"], "bob.near");
    }

    #[test]
    fn convert_args_serialize() {
        let args = ConvertArgs { amount: "1000000".to_string() };
        let json = serde_json::to_value(&args).unwrap();
        assert_eq!(json["amount"], "1000000");
    }
}
