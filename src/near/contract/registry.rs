//! Templar deployment registry contract wrappers.
//!
//! The registry contract tracks all deployed Templar protocol contracts
//! (markets, vaults) and their versions. This client provides view methods
//! for querying deployments and version metadata.

use std::sync::Arc;

use near_primitives::types::AccountId;
use serde::{Deserialize, Serialize};

use crate::error::CliError;
use crate::near::contract::ContractClient;
use crate::near::rpc::NearRpcClient;
use crate::near::signer::NearSigner;

// ---------------------------------------------------------------------------
// Argument types
// ---------------------------------------------------------------------------

/// Arguments for paginated list queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationArgs {
    /// Starting offset.
    pub from_index: u64,
    /// Maximum number of items to return.
    pub limit: u64,
}

/// Arguments for version lookup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionArgs {
    /// The version string to look up (e.g., "1.0.0").
    pub version: String,
}

/// Arguments for deployment lookup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentArgs {
    /// The contract account ID of the deployment.
    pub contract_id: AccountId,
}

// ---------------------------------------------------------------------------
// RegistryClient
// ---------------------------------------------------------------------------

/// Typed client for the Templar deployment registry contract.
pub struct RegistryClient {
    inner: ContractClient,
}

impl RegistryClient {
    /// Create a new registry client.
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

    /// List all known protocol versions with pagination.
    pub async fn list_versions(
        &self,
        from_index: u64,
        limit: u64,
    ) -> Result<Vec<serde_json::Value>, CliError> {
        self.inner
            .view("list_versions", &PaginationArgs { from_index, limit })
            .await
    }

    /// Get the code hash for a specific version.
    pub async fn get_version_code_hash(
        &self,
        version: &str,
    ) -> Result<serde_json::Value, CliError> {
        self.inner
            .view(
                "get_version_code_hash",
                &VersionArgs {
                    version: version.to_string(),
                },
            )
            .await
    }

    /// List all contract deployments with pagination.
    pub async fn list_deployments(
        &self,
        from_index: u64,
        limit: u64,
    ) -> Result<Vec<serde_json::Value>, CliError> {
        self.inner
            .view("list_deployments", &PaginationArgs { from_index, limit })
            .await
    }

    /// Get deployment metadata for a specific contract.
    pub async fn get_deployment(
        &self,
        contract_id: &AccountId,
    ) -> Result<serde_json::Value, CliError> {
        self.inner
            .view(
                "get_deployment",
                &DeploymentArgs {
                    contract_id: contract_id.clone(),
                },
            )
            .await
    }
}

impl std::fmt::Debug for RegistryClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegistryClient")
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
                    block_height: 300,
                    block_hash: CryptoHash::default(),
                })
            });
        Arc::new(mock)
    }

    fn registry_client(rpc: Arc<MockNearRpcClient>) -> RegistryClient {
        let contract_id: AccountId = "v1.tmplr.near".parse().unwrap();
        RegistryClient::new(rpc, contract_id, None)
    }

    #[tokio::test]
    async fn list_versions() {
        let rpc = mock_view_rpc(serde_json::json!([
            {"version": "1.0.0", "deployed_at": "2024-01-01"},
            {"version": "1.1.0", "deployed_at": "2024-06-01"}
        ]));
        let client = registry_client(rpc);
        let versions = client.list_versions(0, 50).await.unwrap();
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0]["version"], "1.0.0");
    }

    #[tokio::test]
    async fn get_version_code_hash() {
        let rpc = mock_view_rpc(serde_json::json!({
            "code_hash": "4uNMsUP8mzmLDbEMxBHL5dVC4QKWByokvpZjJSWag6zk"
        }));
        let client = registry_client(rpc);
        let result = client.get_version_code_hash("1.0.0").await.unwrap();
        assert!(result["code_hash"].is_string());
    }

    #[tokio::test]
    async fn list_deployments() {
        let rpc = mock_view_rpc(serde_json::json!([
            {
                "contract_id": "market-btc.tmplr.near",
                "version": "1.0.0",
                "kind": "market"
            }
        ]));
        let client = registry_client(rpc);
        let deployments = client.list_deployments(0, 100).await.unwrap();
        assert_eq!(deployments.len(), 1);
        assert_eq!(deployments[0]["kind"], "market");
    }

    #[tokio::test]
    async fn get_deployment() {
        let rpc = mock_view_rpc(serde_json::json!({
            "contract_id": "market-btc.tmplr.near",
            "version": "1.0.0",
            "kind": "market",
            "borrow_token": "usdc.near",
            "collateral_token": "wbtc.near"
        }));
        let client = registry_client(rpc);
        let contract_id: AccountId = "market-btc.tmplr.near".parse().unwrap();
        let deployment = client.get_deployment(&contract_id).await.unwrap();
        assert_eq!(deployment["kind"], "market");
        assert_eq!(deployment["borrow_token"], "usdc.near");
    }

    #[tokio::test]
    async fn list_versions_empty() {
        let rpc = mock_view_rpc(serde_json::json!([]));
        let client = registry_client(rpc);
        let versions = client.list_versions(0, 50).await.unwrap();
        assert!(versions.is_empty());
    }

    #[tokio::test]
    async fn list_deployments_empty() {
        let rpc = mock_view_rpc(serde_json::json!([]));
        let client = registry_client(rpc);
        let deployments = client.list_deployments(0, 100).await.unwrap();
        assert!(deployments.is_empty());
    }

    #[test]
    fn registry_client_contract_id() {
        let rpc: Arc<MockNearRpcClient> = Arc::new(MockNearRpcClient::new());
        let contract_id: AccountId = "v1.tmplr.near".parse().unwrap();
        let client = RegistryClient::new(rpc, contract_id.clone(), None);
        assert_eq!(client.contract_id(), &contract_id);
    }

    #[test]
    fn registry_client_debug() {
        let rpc: Arc<MockNearRpcClient> = Arc::new(MockNearRpcClient::new());
        let contract_id: AccountId = "v1.tmplr.near".parse().unwrap();
        let client = RegistryClient::new(rpc, contract_id, None);
        let debug = format!("{client:?}");
        assert!(debug.contains("v1.tmplr.near"));
    }

    #[test]
    fn pagination_args_serialize() {
        let args = PaginationArgs {
            from_index: 5,
            limit: 25,
        };
        let json = serde_json::to_value(&args).unwrap();
        assert_eq!(json["from_index"], 5);
        assert_eq!(json["limit"], 25);
    }

    #[test]
    fn version_args_serialize() {
        let args = VersionArgs {
            version: "1.0.0".to_string(),
        };
        let json = serde_json::to_value(&args).unwrap();
        assert_eq!(json["version"], "1.0.0");
    }

    #[test]
    fn deployment_args_serialize() {
        let args = DeploymentArgs {
            contract_id: "market.tmplr.near".parse().unwrap(),
        };
        let json = serde_json::to_value(&args).unwrap();
        assert_eq!(json["contract_id"], "market.tmplr.near");
    }
}
