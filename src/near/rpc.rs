//! NEAR JSON-RPC client with retry/timeout logic.
//!
//! [`NearRpcClient`] defines the trait interface for all NEAR RPC interactions.
//! [`RpcClient`] is the concrete implementation using `near-jsonrpc-client`.
//!
//! ## Retry strategy
//!
//! - View calls retry up to [`MAX_RETRIES`] on transient failures (5xx, connection errors).
//! - Transaction sends do **not** retry blindly; on timeout they poll `tx_status`
//!   to avoid double-spending.
//! - Exponential backoff: starts at [`INITIAL_BACKOFF_MS`], multiplied by
//!   [`BACKOFF_MULTIPLIER`] each attempt, capped at [`MAX_BACKOFF_MS`].

use std::time::Duration;

use async_trait::async_trait;
use near_jsonrpc_client::methods;
use near_jsonrpc_client::JsonRpcClient;
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::hash::CryptoHash;
use near_primitives::transaction::SignedTransaction;
use near_primitives::types::{AccountId, BlockReference, Finality, FunctionArgs};
use near_primitives::views::{AccessKeyView, AccountView, FinalExecutionOutcomeView, QueryRequest};
use serde::de::DeserializeOwned;
use tracing::{debug, warn};

use crate::error::CliError;

// ---------------------------------------------------------------------------
// Retry / timeout constants
// ---------------------------------------------------------------------------

/// Maximum number of retry attempts for view calls.
pub const MAX_RETRIES: u32 = 3;

/// Initial backoff duration in milliseconds.
pub const INITIAL_BACKOFF_MS: u64 = 200;

/// Multiplier applied to the backoff after each retry.
pub const BACKOFF_MULTIPLIER: f64 = 2.0;

/// Maximum backoff duration in milliseconds.
pub const MAX_BACKOFF_MS: u64 = 5_000;

/// Total timeout for the entire operation in milliseconds.
pub const TOTAL_TIMEOUT_MS: u64 = 30_000;

/// Timeout for a single view call RPC request.
pub const VIEW_CALL_TIMEOUT_MS: u64 = 10_000;

/// Timeout for a single send-transaction RPC request.
pub const SEND_TX_TIMEOUT_MS: u64 = 30_000;

/// HTTP status codes that are considered retryable (transient server errors).
pub const RETRYABLE_STATUS_CODES: &[u16] = &[500, 502, 503, 504, 520, 521, 522, 523, 524];

// ---------------------------------------------------------------------------
// RPC error classification
// ---------------------------------------------------------------------------

/// Classification of RPC errors for retry decisions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RpcErrorKind {
    /// Client-side error (bad request, serialization failure).
    ClientError,
    /// Server-side transient error (5xx).
    ServerError,
    /// Request timed out.
    Timeout,
    /// Could not connect to the RPC endpoint.
    ConnectionRefused,
    /// The contract execution itself failed (e.g., panic, assertion).
    ContractError,
    /// The nonce used in the transaction was invalid/stale.
    InvalidNonce,
}

impl RpcErrorKind {
    /// Returns `true` if this error kind is safe to retry.
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::ServerError | Self::Timeout | Self::ConnectionRefused)
    }
}

/// Classify a raw error string into an [`RpcErrorKind`].
///
/// This is a heuristic based on common NEAR RPC error messages.
pub fn classify_rpc_error(err_msg: &str) -> RpcErrorKind {
    let lower = err_msg.to_lowercase();
    if lower.contains("timeout") || lower.contains("timed out") {
        RpcErrorKind::Timeout
    } else if lower.contains("connection refused") || lower.contains("connection reset") {
        RpcErrorKind::ConnectionRefused
    } else if lower.contains("invalid nonce") || lower.contains("invalidnonce") {
        RpcErrorKind::InvalidNonce
    } else if lower.contains("wasm execution failed")
        || lower.contains("smart contract panicked")
        || lower.contains("executionerror")
    {
        RpcErrorKind::ContractError
    } else if RETRYABLE_STATUS_CODES.iter().any(|code| lower.contains(&code.to_string()))
        || lower.contains("server error")
        || lower.contains("internal error")
    {
        RpcErrorKind::ServerError
    } else {
        RpcErrorKind::ClientError
    }
}

// ---------------------------------------------------------------------------
// Trait definition
// ---------------------------------------------------------------------------

/// Response from a view-function call.
#[derive(Debug, Clone)]
pub struct ViewCallResult {
    /// The raw bytes returned by the contract.
    pub result: Vec<u8>,
    /// Logs emitted during execution.
    pub logs: Vec<String>,
    /// Block height at which the query was executed.
    pub block_height: u64,
    /// Block hash at which the query was executed.
    pub block_hash: CryptoHash,
}

impl ViewCallResult {
    /// Deserialize the result bytes as JSON into `T`.
    pub fn json<T: DeserializeOwned>(&self) -> Result<T, CliError> {
        serde_json::from_slice(&self.result).map_err(|e| {
            CliError::Serialization(format!("failed to deserialize view call result: {e}"))
        })
    }
}

/// Response from an access-key query.
#[derive(Debug, Clone)]
pub struct AccessKeyResult {
    /// The nonce of the access key.
    pub nonce: u64,
    /// The block hash at which the query was executed.
    pub block_hash: CryptoHash,
    /// The full access key view.
    pub access_key: AccessKeyView,
}

/// Trait defining the NEAR RPC interface used throughout the CLI.
///
/// All methods are async and return `Result<_, CliError>`. The trait is
/// annotated with `#[cfg_attr(test, mockall::automock)]` so that tests
/// can substitute a mock implementation via `mockall`.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait NearRpcClient: Send + Sync {
    /// Call a view function on a contract.
    async fn view_function(
        &self,
        account_id: &AccountId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<ViewCallResult, CliError>;

    /// Query account information.
    async fn view_account(&self, account_id: &AccountId) -> Result<AccountView, CliError>;

    /// Broadcast a signed transaction and wait for the final execution outcome.
    async fn send_transaction(
        &self,
        tx: SignedTransaction,
    ) -> Result<FinalExecutionOutcomeView, CliError>;

    /// Poll the status of a previously sent transaction.
    async fn tx_status(
        &self,
        tx_hash: CryptoHash,
        sender_id: &AccountId,
    ) -> Result<FinalExecutionOutcomeView, CliError>;

    /// Query an access key for a given account and public key.
    async fn access_key(
        &self,
        account_id: &AccountId,
        public_key: &near_crypto::PublicKey,
    ) -> Result<AccessKeyResult, CliError>;
}

// ---------------------------------------------------------------------------
// Concrete implementation
// ---------------------------------------------------------------------------

/// Concrete NEAR JSON-RPC client.
///
/// Wraps [`JsonRpcClient`] with retry logic and timeout management.
pub struct RpcClient {
    client: JsonRpcClient,
}

/// Type alias for [`RpcClient`], used in CLI commands.
pub type LiveNearRpcClient = RpcClient;

impl RpcClient {
    /// Create a new RPC client pointing at the given endpoint URL.
    pub fn new(rpc_url: &str) -> Self {
        let client = JsonRpcClient::connect(rpc_url);
        Self { client }
    }

    /// Compute the backoff duration for a given retry attempt (0-indexed).
    fn backoff_duration(attempt: u32) -> Duration {
        let ms = INITIAL_BACKOFF_MS as f64 * BACKOFF_MULTIPLIER.powi(attempt as i32);
        let capped = ms.min(MAX_BACKOFF_MS as f64);
        Duration::from_millis(capped as u64)
    }

    /// Execute a view-function call with retry logic.
    async fn view_function_with_retry(
        &self,
        account_id: &AccountId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<ViewCallResult, CliError> {
        let mut last_err = None;

        for attempt in 0..=MAX_RETRIES {
            if attempt > 0 {
                let backoff = Self::backoff_duration(attempt - 1);
                debug!(attempt, backoff_ms = backoff.as_millis(), "retrying view call");
                tokio::time::sleep(backoff).await;
            }

            let request = methods::query::RpcQueryRequest {
                block_reference: BlockReference::Finality(Finality::Final),
                request: QueryRequest::CallFunction {
                    account_id: account_id.clone(),
                    method_name: method_name.to_string(),
                    args: FunctionArgs::from(args.clone()),
                },
            };

            let result = tokio::time::timeout(
                Duration::from_millis(VIEW_CALL_TIMEOUT_MS),
                self.client.call(request),
            )
            .await;

            match result {
                Ok(Ok(response)) => {
                    if let QueryResponseKind::CallResult(call_result) = response.kind {
                        return Ok(ViewCallResult {
                            result: call_result.result,
                            logs: call_result.logs,
                            block_height: response.block_height,
                            block_hash: response.block_hash,
                        });
                    }
                    return Err(CliError::Rpc("unexpected response kind for view call".into()));
                }
                Ok(Err(e)) => {
                    let err_msg = e.to_string();
                    let kind = classify_rpc_error(&err_msg);
                    if kind.is_retryable() && attempt < MAX_RETRIES {
                        warn!(
                            attempt,
                            error = %err_msg,
                            "transient RPC error, will retry"
                        );
                        last_err = Some(err_msg);
                        continue;
                    }
                    return Err(CliError::Rpc(err_msg));
                }
                Err(_elapsed) => {
                    let err_msg = format!(
                        "view call to {account_id}::{method_name} timed out after {VIEW_CALL_TIMEOUT_MS}ms"
                    );
                    if attempt < MAX_RETRIES {
                        warn!(attempt, "view call timed out, will retry");
                        last_err = Some(err_msg);
                        continue;
                    }
                    return Err(CliError::Rpc(err_msg));
                }
            }
        }

        Err(CliError::Rpc(last_err.unwrap_or_else(|| "view call failed after all retries".into())))
    }
}

#[async_trait]
impl NearRpcClient for RpcClient {
    async fn view_function(
        &self,
        account_id: &AccountId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<ViewCallResult, CliError> {
        self.view_function_with_retry(account_id, method_name, args).await
    }

    async fn view_account(&self, account_id: &AccountId) -> Result<AccountView, CliError> {
        let request = methods::query::RpcQueryRequest {
            block_reference: BlockReference::Finality(Finality::Final),
            request: QueryRequest::ViewAccount { account_id: account_id.clone() },
        };

        let result = tokio::time::timeout(
            Duration::from_millis(VIEW_CALL_TIMEOUT_MS),
            self.client.call(request),
        )
        .await;

        match result {
            Ok(Ok(response)) => {
                if let QueryResponseKind::ViewAccount(account_view) = response.kind {
                    Ok(account_view)
                } else {
                    Err(CliError::Rpc("unexpected response kind for view_account".into()))
                }
            }
            Ok(Err(e)) => Err(CliError::Rpc(e.to_string())),
            Err(_elapsed) => Err(CliError::Rpc(format!(
                "view_account for {account_id} timed out after {VIEW_CALL_TIMEOUT_MS}ms"
            ))),
        }
    }

    async fn send_transaction(
        &self,
        tx: SignedTransaction,
    ) -> Result<FinalExecutionOutcomeView, CliError> {
        let tx_hash = tx.get_hash();
        let sender_id = tx.transaction.signer_id().clone();

        let request =
            methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest { signed_transaction: tx };

        let result = tokio::time::timeout(
            Duration::from_millis(SEND_TX_TIMEOUT_MS),
            self.client.call(request),
        )
        .await;

        match result {
            Ok(Ok(outcome)) => Ok(outcome),
            Ok(Err(e)) => {
                let err_msg = e.to_string();
                let kind = classify_rpc_error(&err_msg);

                // On timeout or transient server errors during send, poll tx_status
                // instead of re-sending to avoid double-spend.
                if kind == RpcErrorKind::Timeout || kind == RpcErrorKind::ServerError {
                    warn!(
                        tx_hash = %tx_hash,
                        error = %err_msg,
                        "send_transaction failed with transient error, polling tx_status"
                    );
                    return self.tx_status(tx_hash, &sender_id).await;
                }

                Err(CliError::Rpc(err_msg))
            }
            Err(_elapsed) => {
                warn!(
                    tx_hash = %tx_hash,
                    "send_transaction timed out after {SEND_TX_TIMEOUT_MS}ms, polling tx_status"
                );
                // Transaction may have been received — poll status instead of failing.
                self.tx_status(tx_hash, &sender_id).await
            }
        }
    }

    async fn tx_status(
        &self,
        tx_hash: CryptoHash,
        sender_id: &AccountId,
    ) -> Result<FinalExecutionOutcomeView, CliError> {
        // Poll with retries and backoff.
        for attempt in 0..=MAX_RETRIES {
            if attempt > 0 {
                let backoff = Self::backoff_duration(attempt - 1);
                tokio::time::sleep(backoff).await;
            }

            let request = methods::tx::RpcTransactionStatusRequest {
                transaction_info: methods::tx::TransactionInfo::TransactionId {
                    tx_hash,
                    sender_account_id: sender_id.clone(),
                },
                wait_until: near_primitives::views::TxExecutionStatus::Final,
            };

            let result = tokio::time::timeout(
                Duration::from_millis(VIEW_CALL_TIMEOUT_MS),
                self.client.call(request),
            )
            .await;

            match result {
                Ok(Ok(response)) => {
                    if let Some(outcome) = response.final_execution_outcome {
                        return Ok(outcome.into_outcome());
                    }
                    // Not final yet — retry.
                    if attempt < MAX_RETRIES {
                        debug!(attempt, "tx not finalized yet, will poll again");
                        continue;
                    }
                    return Err(CliError::Rpc(format!(
                        "transaction {tx_hash} not finalized after polling"
                    )));
                }
                Ok(Err(e)) => {
                    let err_msg = e.to_string();
                    let kind = classify_rpc_error(&err_msg);
                    if kind.is_retryable() && attempt < MAX_RETRIES {
                        warn!(attempt, error = %err_msg, "tx_status transient error, retrying");
                        continue;
                    }
                    return Err(CliError::Rpc(err_msg));
                }
                Err(_elapsed) => {
                    if attempt < MAX_RETRIES {
                        warn!(attempt, "tx_status timed out, retrying");
                        continue;
                    }
                    return Err(CliError::Rpc(format!(
                        "tx_status for {tx_hash} timed out after all retries"
                    )));
                }
            }
        }

        Err(CliError::Rpc(format!("tx_status for {tx_hash} failed after all retries")))
    }

    async fn access_key(
        &self,
        account_id: &AccountId,
        public_key: &near_crypto::PublicKey,
    ) -> Result<AccessKeyResult, CliError> {
        let request = methods::query::RpcQueryRequest {
            block_reference: BlockReference::Finality(Finality::Final),
            request: QueryRequest::ViewAccessKey {
                account_id: account_id.clone(),
                public_key: public_key.clone(),
            },
        };

        let result = tokio::time::timeout(
            Duration::from_millis(VIEW_CALL_TIMEOUT_MS),
            self.client.call(request),
        )
        .await;

        match result {
            Ok(Ok(response)) => {
                if let QueryResponseKind::AccessKey(ak_view) = response.kind {
                    Ok(AccessKeyResult {
                        nonce: ak_view.nonce,
                        block_hash: response.block_hash,
                        access_key: ak_view,
                    })
                } else {
                    Err(CliError::Rpc("unexpected response kind for access_key query".into()))
                }
            }
            Ok(Err(e)) => Err(CliError::Rpc(e.to_string())),
            Err(_elapsed) => Err(CliError::Rpc(format!(
                "access_key query for {account_id} timed out after {VIEW_CALL_TIMEOUT_MS}ms"
            ))),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rpc_error_kind_retryable() {
        assert!(RpcErrorKind::ServerError.is_retryable());
        assert!(RpcErrorKind::Timeout.is_retryable());
        assert!(RpcErrorKind::ConnectionRefused.is_retryable());
        assert!(!RpcErrorKind::ClientError.is_retryable());
        assert!(!RpcErrorKind::ContractError.is_retryable());
        assert!(!RpcErrorKind::InvalidNonce.is_retryable());
    }

    #[test]
    fn classify_timeout_errors() {
        assert_eq!(classify_rpc_error("request timed out after 10s"), RpcErrorKind::Timeout);
        assert_eq!(classify_rpc_error("Connection timeout reached"), RpcErrorKind::Timeout);
    }

    #[test]
    fn classify_connection_errors() {
        assert_eq!(classify_rpc_error("connection refused"), RpcErrorKind::ConnectionRefused);
        assert_eq!(classify_rpc_error("Connection reset by peer"), RpcErrorKind::ConnectionRefused);
    }

    #[test]
    fn classify_nonce_errors() {
        assert_eq!(
            classify_rpc_error("InvalidNonce: expected 42, got 41"),
            RpcErrorKind::InvalidNonce
        );
        assert_eq!(classify_rpc_error("invalid nonce"), RpcErrorKind::InvalidNonce);
    }

    #[test]
    fn classify_contract_errors() {
        assert_eq!(
            classify_rpc_error("Smart contract panicked: insufficient balance"),
            RpcErrorKind::ContractError
        );
        assert_eq!(classify_rpc_error("Wasm execution failed"), RpcErrorKind::ContractError);
        assert_eq!(
            classify_rpc_error("ExecutionError: some contract error"),
            RpcErrorKind::ContractError
        );
    }

    #[test]
    fn classify_server_errors() {
        assert_eq!(classify_rpc_error("HTTP 502 Bad Gateway"), RpcErrorKind::ServerError);
        assert_eq!(classify_rpc_error("server error"), RpcErrorKind::ServerError);
        assert_eq!(classify_rpc_error("Internal error"), RpcErrorKind::ServerError);
        assert_eq!(classify_rpc_error("status code 503"), RpcErrorKind::ServerError);
    }

    #[test]
    fn classify_client_errors() {
        assert_eq!(classify_rpc_error("invalid method params"), RpcErrorKind::ClientError);
        assert_eq!(classify_rpc_error("unknown error occurred"), RpcErrorKind::ClientError);
    }

    #[test]
    fn backoff_duration_increases_exponentially() {
        let d0 = RpcClient::backoff_duration(0);
        let d1 = RpcClient::backoff_duration(1);
        let d2 = RpcClient::backoff_duration(2);

        assert_eq!(d0.as_millis(), 200);
        assert_eq!(d1.as_millis(), 400);
        assert_eq!(d2.as_millis(), 800);
    }

    #[test]
    fn backoff_duration_capped_at_max() {
        let d10 = RpcClient::backoff_duration(10);
        assert!(d10.as_millis() <= u128::from(MAX_BACKOFF_MS));
    }

    #[test]
    fn constants_are_sane() {
        const { assert!(MAX_RETRIES > 0) };
        const { assert!(INITIAL_BACKOFF_MS > 0) };
        const { assert!(BACKOFF_MULTIPLIER > 1.0) };
        const { assert!(MAX_BACKOFF_MS >= INITIAL_BACKOFF_MS) };
        const { assert!(TOTAL_TIMEOUT_MS > VIEW_CALL_TIMEOUT_MS) };
        const { assert!(SEND_TX_TIMEOUT_MS >= VIEW_CALL_TIMEOUT_MS) };
        const { assert!(!RETRYABLE_STATUS_CODES.is_empty()) };
    }

    #[test]
    fn retryable_status_codes_are_5xx_range() {
        for code in RETRYABLE_STATUS_CODES {
            assert!(*code >= 500, "expected 5xx status code, got {code}");
        }
    }

    #[test]
    fn view_call_result_json_success() {
        let result = ViewCallResult {
            result: br#"{"value":42}"#.to_vec(),
            logs: vec![],
            block_height: 100,
            block_hash: CryptoHash::default(),
        };

        let parsed: serde_json::Value = result.json().unwrap();
        assert_eq!(parsed["value"], 42);
    }

    #[test]
    fn view_call_result_json_error() {
        let result = ViewCallResult {
            result: b"not valid json".to_vec(),
            logs: vec![],
            block_height: 100,
            block_hash: CryptoHash::default(),
        };

        let parsed: Result<serde_json::Value, _> = result.json();
        assert!(parsed.is_err());
    }

    #[tokio::test]
    async fn mock_rpc_client_view_function() {
        let mut mock = MockNearRpcClient::new();
        let expected_result = ViewCallResult {
            result: br#"{"ok":true}"#.to_vec(),
            logs: vec!["log entry".to_string()],
            block_height: 12345,
            block_hash: CryptoHash::default(),
        };

        let expected_clone = expected_result.clone();
        mock.expect_view_function()
            .withf(|account_id, method_name, _args| {
                account_id.as_str() == "test.near" && method_name == "get_data"
            })
            .times(1)
            .returning(move |_, _, _| Ok(expected_clone.clone()));

        let account_id: AccountId = "test.near".parse().unwrap();
        let result = mock.view_function(&account_id, "get_data", b"{}".to_vec()).await.unwrap();

        assert_eq!(result.result, expected_result.result);
        assert_eq!(result.logs, expected_result.logs);
        assert_eq!(result.block_height, expected_result.block_height);
    }

    #[tokio::test]
    async fn mock_rpc_client_view_function_error() {
        let mut mock = MockNearRpcClient::new();

        mock.expect_view_function()
            .times(1)
            .returning(|_, _, _| Err(CliError::Rpc("contract not found".into())));

        let account_id: AccountId = "missing.near".parse().unwrap();
        let result = mock.view_function(&account_id, "get_data", b"{}".to_vec()).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, CliError::Rpc(_)));
    }

    #[tokio::test]
    async fn mock_rpc_client_view_account() {
        let mut mock = MockNearRpcClient::new();

        mock.expect_view_account()
            .withf(|account_id| account_id.as_str() == "alice.near")
            .times(1)
            .returning(|_| {
                Ok(AccountView {
                    amount: 1_000_000_000_000_000_000_000_000,
                    locked: 0,
                    code_hash: CryptoHash::default(),
                    storage_usage: 1000,
                    storage_paid_at: 0,
                })
            });

        let account_id: AccountId = "alice.near".parse().unwrap();
        let account = mock.view_account(&account_id).await.unwrap();
        assert_eq!(account.amount, 1_000_000_000_000_000_000_000_000);
        assert_eq!(account.storage_usage, 1000);
    }

    #[tokio::test]
    async fn mock_rpc_client_tx_status_error() {
        let mut mock = MockNearRpcClient::new();

        mock.expect_tx_status()
            .times(1)
            .returning(|_, _| Err(CliError::Rpc("transaction not found".into())));

        let tx_hash = CryptoHash::default();
        let sender: AccountId = "alice.near".parse().unwrap();
        let result = mock.tx_status(tx_hash, &sender).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn mock_rpc_client_access_key_error() {
        let mut mock = MockNearRpcClient::new();

        mock.expect_access_key()
            .times(1)
            .returning(|_, _| Err(CliError::Rpc("access key not found".into())));

        let account_id: AccountId = "alice.near".parse().unwrap();
        let public_key: near_crypto::PublicKey =
            "ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp".parse().unwrap();
        let result = mock.access_key(&account_id, &public_key).await;
        assert!(result.is_err());
    }
}
