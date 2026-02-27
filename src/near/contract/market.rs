//! Templar lending market contract wrappers.
//!
//! Provides typed view-call and function-call methods for all Templar market
//! contract interactions including supply, borrow, collateral, and liquidation
//! operations.
//!
//! ## Token operations via `ft_transfer_call`
//!
//! Several market operations are initiated by calling `ft_transfer_call` on the
//! token contract (NEP-141) or `mt_transfer_call` on the multi-token contract
//! (NEP-245). The `msg` parameter encodes the market action:
//!
//! - Supply: `msg = "\"Supply\""`
//! - Collateralize: `msg = "\"Collateralize\""`
//! - Repay: `msg = "\"Repay\""`
//! - Liquidate: `msg = "{\"Liquidate\":{\"account_id\":\"<target>\"}}"`

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
// View-call argument & response types
// ---------------------------------------------------------------------------

/// Arguments for position lookups (account-scoped).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountArgs {
    /// The account to query.
    pub account_id: AccountId,
}

/// Arguments for paginated list queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationArgs {
    /// Starting offset.
    pub from_index: u64,
    /// Maximum number of items to return.
    pub limit: u64,
}

/// Arguments for borrow/supply operations that include an amount.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmountArgs {
    /// The amount in the token's base units.
    pub amount: String,
}

/// Arguments for collateral withdrawal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawCollateralArgs {
    /// The amount of collateral to withdraw.
    pub amount: String,
}

/// Token-action message formats for `ft_transfer_call` / `mt_transfer_call`.
#[derive(Debug, Clone)]
pub enum MarketTokenAction {
    /// Supply tokens to the market.
    Supply,
    /// Collateralize tokens in the market.
    Collateralize,
    /// Repay a borrow position.
    Repay,
    /// Liquidate another account's position.
    Liquidate {
        /// The account to liquidate.
        account_id: AccountId,
    },
}

impl MarketTokenAction {
    /// Encode the action as the JSON `msg` string for `ft_transfer_call`.
    pub fn to_msg(&self) -> String {
        match self {
            Self::Supply => "\"Supply\"".to_string(),
            Self::Collateralize => "\"Collateralize\"".to_string(),
            Self::Repay => "\"Repay\"".to_string(),
            Self::Liquidate { account_id } => {
                format!("{{\"Liquidate\":{{\"account_id\":\"{account_id}\"}}}}")
            }
        }
    }
}

// ---------------------------------------------------------------------------
// MarketClient
// ---------------------------------------------------------------------------

/// Typed client for a Templar lending market contract.
///
/// Wraps [`ContractClient`] and provides methods for all market contract
/// view calls and function calls.
pub struct MarketClient {
    inner: ContractClient,
}

impl MarketClient {
    /// Create a new market client.
    ///
    /// If `signer` is `None`, only view calls will be available.
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

    /// Get the market configuration.
    pub async fn get_configuration(&self) -> Result<serde_json::Value, CliError> {
        self.inner.view("get_configuration", &serde_json::json!({})).await
    }

    /// Get the current (non-finalized) snapshot.
    pub async fn get_current_snapshot(&self) -> Result<serde_json::Value, CliError> {
        self.inner.view("get_current_snapshot", &serde_json::json!({})).await
    }

    /// Get the number of finalized snapshots.
    pub async fn get_finalized_snapshots_len(&self) -> Result<u64, CliError> {
        self.inner.view("get_finalized_snapshots_len", &serde_json::json!({})).await
    }

    /// List finalized snapshots with pagination.
    pub async fn list_finalized_snapshots(
        &self,
        from_index: u64,
        limit: u64,
    ) -> Result<Vec<serde_json::Value>, CliError> {
        self.inner.view("list_finalized_snapshots", &PaginationArgs { from_index, limit }).await
    }

    /// Get borrow asset metrics.
    pub async fn get_borrow_asset_metrics(&self) -> Result<serde_json::Value, CliError> {
        self.inner.view("get_borrow_asset_metrics", &serde_json::json!({})).await
    }

    /// Get a specific account's borrow position.
    pub async fn get_borrow_position(
        &self,
        account_id: &AccountId,
    ) -> Result<serde_json::Value, CliError> {
        self.inner
            .view("get_borrow_position", &AccountArgs { account_id: account_id.clone() })
            .await
    }

    /// List all borrow positions with pagination.
    pub async fn list_borrow_positions(
        &self,
        from_index: u64,
        limit: u64,
    ) -> Result<Vec<serde_json::Value>, CliError> {
        self.inner.view("list_borrow_positions", &PaginationArgs { from_index, limit }).await
    }

    /// Get pending interest for a borrow position.
    pub async fn get_borrow_position_pending_interest(
        &self,
        account_id: &AccountId,
    ) -> Result<serde_json::Value, CliError> {
        self.inner
            .view(
                "get_borrow_position_pending_interest",
                &AccountArgs { account_id: account_id.clone() },
            )
            .await
    }

    /// Get borrow status for an account.
    pub async fn get_borrow_status(
        &self,
        account_id: &AccountId,
    ) -> Result<serde_json::Value, CliError> {
        self.inner.view("get_borrow_status", &AccountArgs { account_id: account_id.clone() }).await
    }

    /// Get a specific account's supply position.
    pub async fn get_supply_position(
        &self,
        account_id: &AccountId,
    ) -> Result<serde_json::Value, CliError> {
        self.inner
            .view("get_supply_position", &AccountArgs { account_id: account_id.clone() })
            .await
    }

    /// List all supply positions with pagination.
    pub async fn list_supply_positions(
        &self,
        from_index: u64,
        limit: u64,
    ) -> Result<Vec<serde_json::Value>, CliError> {
        self.inner.view("list_supply_positions", &PaginationArgs { from_index, limit }).await
    }

    /// Get pending yield for a supply position.
    pub async fn get_supply_position_pending_yield(
        &self,
        account_id: &AccountId,
    ) -> Result<serde_json::Value, CliError> {
        self.inner
            .view(
                "get_supply_position_pending_yield",
                &AccountArgs { account_id: account_id.clone() },
            )
            .await
    }

    /// Get the status of a supply withdrawal request.
    pub async fn get_supply_withdrawal_request_status(
        &self,
        account_id: &AccountId,
    ) -> Result<serde_json::Value, CliError> {
        self.inner
            .view(
                "get_supply_withdrawal_request_status",
                &AccountArgs { account_id: account_id.clone() },
            )
            .await
    }

    /// Get the overall supply withdrawal queue status.
    pub async fn get_supply_withdrawal_queue_status(&self) -> Result<serde_json::Value, CliError> {
        self.inner.view("get_supply_withdrawal_queue_status", &serde_json::json!({})).await
    }

    /// Get the last yield rate.
    pub async fn get_last_yield_rate(&self) -> Result<serde_json::Value, CliError> {
        self.inner.view("get_last_yield_rate", &serde_json::json!({})).await
    }

    /// Get the static yield configuration.
    pub async fn get_static_yield(&self) -> Result<serde_json::Value, CliError> {
        self.inner.view("get_static_yield", &serde_json::json!({})).await
    }

    // -----------------------------------------------------------------------
    // Write calls
    // -----------------------------------------------------------------------

    /// Borrow from the market.
    pub async fn borrow(&self, amount: &str) -> Result<FinalExecutionOutcomeView, CliError> {
        self.inner
            .call("borrow", &AmountArgs { amount: amount.to_string() }, DEFAULT_GAS, ONE_YOCTO)
            .await
    }

    /// Withdraw collateral from the market.
    pub async fn withdraw_collateral(
        &self,
        amount: &str,
    ) -> Result<FinalExecutionOutcomeView, CliError> {
        self.inner
            .call(
                "withdraw_collateral",
                &WithdrawCollateralArgs { amount: amount.to_string() },
                DEFAULT_GAS,
                ONE_YOCTO,
            )
            .await
    }

    /// Create a supply withdrawal request.
    pub async fn create_supply_withdrawal_request(
        &self,
        amount: &str,
    ) -> Result<FinalExecutionOutcomeView, CliError> {
        self.inner
            .call(
                "create_supply_withdrawal_request",
                &AmountArgs { amount: amount.to_string() },
                DEFAULT_GAS,
                ONE_YOCTO,
            )
            .await
    }

    /// Cancel a supply withdrawal request.
    pub async fn cancel_supply_withdrawal_request(
        &self,
    ) -> Result<FinalExecutionOutcomeView, CliError> {
        self.inner.call_one_yocto("cancel_supply_withdrawal_request", &serde_json::json!({})).await
    }

    /// Execute the next supply withdrawal request in the queue.
    pub async fn execute_next_supply_withdrawal_request(
        &self,
    ) -> Result<FinalExecutionOutcomeView, CliError> {
        self.inner
            .call_no_deposit("execute_next_supply_withdrawal_request", &serde_json::json!({}))
            .await
    }

    /// Harvest accrued yield from a supply position.
    pub async fn harvest_yield(&self) -> Result<FinalExecutionOutcomeView, CliError> {
        self.inner.call_one_yocto("harvest_yield", &serde_json::json!({})).await
    }

    /// Apply interest to a borrow position.
    pub async fn apply_interest(&self) -> Result<FinalExecutionOutcomeView, CliError> {
        self.inner.call_no_deposit("apply_interest", &serde_json::json!({})).await
    }

    /// Accumulate static yield.
    pub async fn accumulate_static_yield(&self) -> Result<FinalExecutionOutcomeView, CliError> {
        self.inner.call_no_deposit("accumulate_static_yield", &serde_json::json!({})).await
    }

    /// Withdraw accumulated static yield.
    pub async fn withdraw_static_yield(&self) -> Result<FinalExecutionOutcomeView, CliError> {
        self.inner.call_one_yocto("withdraw_static_yield", &serde_json::json!({})).await
    }
}

impl std::fmt::Debug for MarketClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MarketClient").field("contract_id", self.inner.contract_id()).finish()
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
                block_height: 100,
                block_hash: CryptoHash::default(),
            })
        });
        Arc::new(mock)
    }

    fn market_client(rpc: Arc<MockNearRpcClient>) -> MarketClient {
        let contract_id: AccountId = "market.tmplr.near".parse().unwrap();
        MarketClient::new(rpc, contract_id, None)
    }

    #[test]
    fn market_token_action_supply_msg() {
        assert_eq!(MarketTokenAction::Supply.to_msg(), "\"Supply\"");
    }

    #[test]
    fn market_token_action_collateralize_msg() {
        assert_eq!(MarketTokenAction::Collateralize.to_msg(), "\"Collateralize\"");
    }

    #[test]
    fn market_token_action_repay_msg() {
        assert_eq!(MarketTokenAction::Repay.to_msg(), "\"Repay\"");
    }

    #[test]
    fn market_token_action_liquidate_msg() {
        let action = MarketTokenAction::Liquidate { account_id: "victim.near".parse().unwrap() };
        let msg = action.to_msg();
        assert_eq!(msg, "{\"Liquidate\":{\"account_id\":\"victim.near\"}}");
    }

    #[tokio::test]
    async fn get_configuration() {
        let rpc = mock_view_rpc(serde_json::json!({
            "borrow_token": "usdc.near",
            "supply_token": "wbtc.near"
        }));
        let client = market_client(rpc);
        let config = client.get_configuration().await.unwrap();
        assert_eq!(config["borrow_token"], "usdc.near");
    }

    #[tokio::test]
    async fn get_current_snapshot() {
        let rpc = mock_view_rpc(serde_json::json!({
            "total_supply": "1000000",
            "total_borrow": "500000"
        }));
        let client = market_client(rpc);
        let snapshot = client.get_current_snapshot().await.unwrap();
        assert_eq!(snapshot["total_supply"], "1000000");
    }

    #[tokio::test]
    async fn get_finalized_snapshots_len() {
        let rpc = mock_view_rpc(serde_json::json!(42));
        let client = market_client(rpc);
        let len = client.get_finalized_snapshots_len().await.unwrap();
        assert_eq!(len, 42);
    }

    #[tokio::test]
    async fn list_finalized_snapshots() {
        let rpc = mock_view_rpc(serde_json::json!([
            {"epoch": 1},
            {"epoch": 2}
        ]));
        let client = market_client(rpc);
        let snapshots = client.list_finalized_snapshots(0, 10).await.unwrap();
        assert_eq!(snapshots.len(), 2);
    }

    #[tokio::test]
    async fn get_borrow_position() {
        let rpc = mock_view_rpc(serde_json::json!({
            "principal": "100000",
            "interest_accrued": "500"
        }));
        let client = market_client(rpc);
        let account_id: AccountId = "alice.near".parse().unwrap();
        let position = client.get_borrow_position(&account_id).await.unwrap();
        assert_eq!(position["principal"], "100000");
    }

    #[tokio::test]
    async fn get_supply_position() {
        let rpc = mock_view_rpc(serde_json::json!({
            "shares": "50000",
            "yield_earned": "250"
        }));
        let client = market_client(rpc);
        let account_id: AccountId = "alice.near".parse().unwrap();
        let position = client.get_supply_position(&account_id).await.unwrap();
        assert_eq!(position["shares"], "50000");
    }

    #[tokio::test]
    async fn get_borrow_asset_metrics() {
        let rpc = mock_view_rpc(serde_json::json!({
            "utilization_rate": "0.75",
            "borrow_rate": "0.08"
        }));
        let client = market_client(rpc);
        let metrics = client.get_borrow_asset_metrics().await.unwrap();
        assert_eq!(metrics["utilization_rate"], "0.75");
    }

    #[tokio::test]
    async fn list_borrow_positions() {
        let rpc = mock_view_rpc(serde_json::json!([]));
        let client = market_client(rpc);
        let positions = client.list_borrow_positions(0, 50).await.unwrap();
        assert!(positions.is_empty());
    }

    #[tokio::test]
    async fn list_supply_positions() {
        let rpc = mock_view_rpc(serde_json::json!([{"account_id": "alice.near"}]));
        let client = market_client(rpc);
        let positions = client.list_supply_positions(0, 50).await.unwrap();
        assert_eq!(positions.len(), 1);
    }

    #[tokio::test]
    async fn get_borrow_position_pending_interest() {
        let rpc = mock_view_rpc(serde_json::json!({"pending_interest": "100"}));
        let client = market_client(rpc);
        let account_id: AccountId = "alice.near".parse().unwrap();
        let result = client.get_borrow_position_pending_interest(&account_id).await.unwrap();
        assert_eq!(result["pending_interest"], "100");
    }

    #[tokio::test]
    async fn get_borrow_status() {
        let rpc = mock_view_rpc(serde_json::json!({"status": "healthy"}));
        let client = market_client(rpc);
        let account_id: AccountId = "alice.near".parse().unwrap();
        let result = client.get_borrow_status(&account_id).await.unwrap();
        assert_eq!(result["status"], "healthy");
    }

    #[tokio::test]
    async fn get_supply_position_pending_yield() {
        let rpc = mock_view_rpc(serde_json::json!({"pending_yield": "50"}));
        let client = market_client(rpc);
        let account_id: AccountId = "alice.near".parse().unwrap();
        let result = client.get_supply_position_pending_yield(&account_id).await.unwrap();
        assert_eq!(result["pending_yield"], "50");
    }

    #[tokio::test]
    async fn get_supply_withdrawal_request_status() {
        let rpc = mock_view_rpc(serde_json::json!({"position": 3}));
        let client = market_client(rpc);
        let account_id: AccountId = "alice.near".parse().unwrap();
        let result = client.get_supply_withdrawal_request_status(&account_id).await.unwrap();
        assert_eq!(result["position"], 3);
    }

    #[tokio::test]
    async fn get_supply_withdrawal_queue_status() {
        let rpc = mock_view_rpc(serde_json::json!({"queue_length": 5}));
        let client = market_client(rpc);
        let result = client.get_supply_withdrawal_queue_status().await.unwrap();
        assert_eq!(result["queue_length"], 5);
    }

    #[tokio::test]
    async fn get_last_yield_rate() {
        let rpc = mock_view_rpc(serde_json::json!({"rate": "0.05"}));
        let client = market_client(rpc);
        let result = client.get_last_yield_rate().await.unwrap();
        assert_eq!(result["rate"], "0.05");
    }

    #[tokio::test]
    async fn get_static_yield() {
        let rpc = mock_view_rpc(serde_json::json!({"accumulated": "1000"}));
        let client = market_client(rpc);
        let result = client.get_static_yield().await.unwrap();
        assert_eq!(result["accumulated"], "1000");
    }

    #[tokio::test]
    async fn write_call_without_signer_errors() {
        let rpc: Arc<MockNearRpcClient> = Arc::new(MockNearRpcClient::new());
        let client = market_client(rpc);
        let result = client.borrow("1000").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CliError::Signing(_)));
    }

    #[test]
    fn market_client_contract_id() {
        let rpc: Arc<MockNearRpcClient> = Arc::new(MockNearRpcClient::new());
        let contract_id: AccountId = "market.tmplr.near".parse().unwrap();
        let client = MarketClient::new(rpc, contract_id.clone(), None);
        assert_eq!(client.contract_id(), &contract_id);
    }

    #[test]
    fn market_client_debug() {
        let rpc: Arc<MockNearRpcClient> = Arc::new(MockNearRpcClient::new());
        let contract_id: AccountId = "market.tmplr.near".parse().unwrap();
        let client = MarketClient::new(rpc, contract_id, None);
        let debug = format!("{client:?}");
        assert!(debug.contains("market.tmplr.near"));
    }

    #[test]
    fn pagination_args_serialize() {
        let args = PaginationArgs { from_index: 0, limit: 50 };
        let json = serde_json::to_value(&args).unwrap();
        assert_eq!(json["from_index"], 0);
        assert_eq!(json["limit"], 50);
    }

    #[test]
    fn account_args_serialize() {
        let args = AccountArgs { account_id: "alice.near".parse().unwrap() };
        let json = serde_json::to_value(&args).unwrap();
        assert_eq!(json["account_id"], "alice.near");
    }

    #[test]
    fn amount_args_serialize() {
        let args = AmountArgs { amount: "1000000".to_string() };
        let json = serde_json::to_value(&args).unwrap();
        assert_eq!(json["amount"], "1000000");
    }
}
