//! Contract client wrappers for Templar Protocol contracts.
//!
//! Each sub-module provides a typed client for a specific contract kind:
//!
//! - [`market`] — Templar lending market contracts
//! - [`vault`] — Templar vault (ERC-4626-style) contracts
//! - [`registry`] — Templar deployment registry
//! - [`token`] — NEP-141 fungible token standard
//! - [`multi_token`] — NEP-245 multi-token standard
//!
//! All clients share a common pattern:
//!
//! 1. **View calls** serialize arguments to JSON, call `rpc.view_function()`,
//!    and deserialize the result.
//! 2. **Function calls** build a transaction via [`TransactionBuilder`],
//!    sign it via [`NearSigner`], and broadcast via `rpc.send_transaction()`.

pub mod market;
pub mod multi_token;
pub mod registry;
pub mod token;
pub mod vault;

use std::sync::Arc;

use near_primitives::types::AccountId;
use near_primitives::views::FinalExecutionOutcomeView;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::CliError;
use crate::near::rpc::NearRpcClient;
use crate::near::signer::NearSigner;
use crate::near::tx_builder::{self, TransactionBuilder};

// ---------------------------------------------------------------------------
// ContractClient — shared base
// ---------------------------------------------------------------------------

/// Base contract client that holds references to RPC, signer, and the target
/// contract account.
///
/// This is used internally by each specific contract client. It provides the
/// common `view` and `call` helper methods.
#[derive(Clone)]
pub struct ContractClient {
    /// The NEAR RPC client.
    rpc: Arc<dyn NearRpcClient>,

    /// The contract account ID.
    contract_id: AccountId,

    /// Optional signer for write operations.
    signer: Option<NearSigner>,
}

impl ContractClient {
    /// Create a new contract client.
    ///
    /// If `signer` is `None`, only view calls will be available.
    pub fn new(
        rpc: Arc<dyn NearRpcClient>,
        contract_id: AccountId,
        signer: Option<NearSigner>,
    ) -> Self {
        Self {
            rpc,
            contract_id,
            signer,
        }
    }

    /// The contract account ID this client targets.
    pub fn contract_id(&self) -> &AccountId {
        &self.contract_id
    }

    /// Execute a view call on the contract, deserializing the JSON result.
    ///
    /// # Arguments
    ///
    /// * `method_name` — the contract method to call.
    /// * `args` — serializable arguments (will be JSON-encoded).
    pub async fn view<A, R>(&self, method_name: &str, args: &A) -> Result<R, CliError>
    where
        A: Serialize + Send + Sync,
        R: DeserializeOwned,
    {
        let args_bytes = tx_builder::json_args(args)?;
        let result = self
            .rpc
            .view_function(&self.contract_id, method_name, args_bytes)
            .await?;
        result.json()
    }

    /// Execute a view call with raw bytes arguments.
    pub async fn view_raw<R>(&self, method_name: &str, args: Vec<u8>) -> Result<R, CliError>
    where
        R: DeserializeOwned,
    {
        let result = self
            .rpc
            .view_function(&self.contract_id, method_name, args)
            .await?;
        result.json()
    }

    /// Execute a signed function call on the contract.
    ///
    /// # Arguments
    ///
    /// * `method_name` — the contract method to call.
    /// * `args` — serializable arguments (will be JSON-encoded).
    /// * `gas` — gas to attach in gas units.
    /// * `deposit` — deposit in yoctoNEAR.
    pub async fn call<A>(
        &self,
        method_name: &str,
        args: &A,
        gas: u64,
        deposit: u128,
    ) -> Result<FinalExecutionOutcomeView, CliError>
    where
        A: Serialize + Send + Sync,
    {
        let signer = self.require_signer()?;
        let args_bytes = tx_builder::json_args(args)?;

        let ak = self
            .rpc
            .access_key(signer.account_id(), &signer.public_key())
            .await?;

        let tx = TransactionBuilder::new(signer.account_id().clone(), self.contract_id.clone())
            .function_call(method_name, args_bytes, gas, deposit)
            .build(ak.nonce, signer.public_key(), ak.block_hash)?;

        let signed_tx = signer.sign_transaction(tx);
        self.rpc.send_transaction(signed_tx).await
    }

    /// Execute a signed function call with default gas and 1 yoctoNEAR deposit.
    pub async fn call_one_yocto<A>(
        &self,
        method_name: &str,
        args: &A,
    ) -> Result<FinalExecutionOutcomeView, CliError>
    where
        A: Serialize + Send + Sync,
    {
        self.call(method_name, args, tx_builder::DEFAULT_GAS, tx_builder::ONE_YOCTO)
            .await
    }

    /// Execute a signed function call with default gas and zero deposit.
    pub async fn call_no_deposit<A>(
        &self,
        method_name: &str,
        args: &A,
    ) -> Result<FinalExecutionOutcomeView, CliError>
    where
        A: Serialize + Send + Sync,
    {
        self.call(method_name, args, tx_builder::DEFAULT_GAS, tx_builder::ZERO_DEPOSIT)
            .await
    }

    /// Get the signer or return an error if none is configured.
    fn require_signer(&self) -> Result<&NearSigner, CliError> {
        self.signer.as_ref().ok_or_else(|| {
            CliError::Signing(
                "no signer configured — import a key with 'templar config import-key'".into(),
            )
        })
    }

    /// Returns a reference to the underlying RPC client.
    pub fn rpc(&self) -> &dyn NearRpcClient {
        self.rpc.as_ref()
    }

    /// Returns a reference to the signer, if configured.
    pub fn signer(&self) -> Option<&NearSigner> {
        self.signer.as_ref()
    }
}

impl std::fmt::Debug for ContractClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContractClient")
            .field("contract_id", &self.contract_id)
            .field("has_signer", &self.signer.is_some())
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

    fn mock_rpc_with_view(
        expected_method: &'static str,
        response_json: serde_json::Value,
    ) -> Arc<MockNearRpcClient> {
        let mut mock = MockNearRpcClient::new();
        let response_bytes = serde_json::to_vec(&response_json).unwrap();

        mock.expect_view_function()
            .withf(move |_contract_id, method_name, _args| method_name == expected_method)
            .returning(move |_, _, _| {
                Ok(ViewCallResult {
                    result: response_bytes.clone(),
                    logs: vec![],
                    block_height: 100,
                    block_hash: CryptoHash::default(),
                })
            });

        Arc::new(mock)
    }

    #[tokio::test]
    async fn contract_client_view() {
        let rpc = mock_rpc_with_view("get_data", serde_json::json!({"value": 42}));
        let contract_id: AccountId = "contract.near".parse().unwrap();
        let client = ContractClient::new(rpc, contract_id, None);

        #[derive(serde::Serialize)]
        struct Args {}
        let result: serde_json::Value = client.view("get_data", &Args {}).await.unwrap();
        assert_eq!(result["value"], 42);
    }

    #[tokio::test]
    async fn contract_client_view_raw() {
        let rpc = mock_rpc_with_view("get_info", serde_json::json!({"status": "ok"}));
        let contract_id: AccountId = "contract.near".parse().unwrap();
        let client = ContractClient::new(rpc, contract_id, None);

        let result: serde_json::Value =
            client.view_raw("get_info", b"{}".to_vec()).await.unwrap();
        assert_eq!(result["status"], "ok");
    }

    #[tokio::test]
    async fn contract_client_call_without_signer_error() {
        let rpc = Arc::new(MockNearRpcClient::new());
        let contract_id: AccountId = "contract.near".parse().unwrap();
        let client = ContractClient::new(rpc, contract_id, None);

        #[derive(serde::Serialize)]
        struct Args {}
        let result = client
            .call("do_thing", &Args {}, tx_builder::DEFAULT_GAS, 0)
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, CliError::Signing(_)));
        assert!(err.to_string().contains("no signer"));
    }

    #[test]
    fn contract_client_debug() {
        let rpc = Arc::new(MockNearRpcClient::new());
        let contract_id: AccountId = "market.near".parse().unwrap();
        let client = ContractClient::new(rpc, contract_id, None);

        let debug = format!("{client:?}");
        assert!(debug.contains("market.near"));
        assert!(debug.contains("has_signer: false"));
    }

    #[test]
    fn contract_client_accessors() {
        let rpc = Arc::new(MockNearRpcClient::new());
        let contract_id: AccountId = "market.near".parse().unwrap();
        let client = ContractClient::new(rpc, contract_id.clone(), None);

        assert_eq!(client.contract_id(), &contract_id);
        assert!(client.signer().is_none());
    }
}
