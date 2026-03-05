//! Registry query commands.

use std::sync::Arc;

use super::GlobalOpts;
use crate::error::CliError;
use crate::near::rpc::NearRpcClient;
use clap::Subcommand;

/// Registry subcommands.
#[derive(Subcommand, Debug)]
pub enum RegistryCommand {
    /// List registered deployments.
    List {
        /// Number of results to return.
        #[arg(long, default_value = "50")]
        count: u64,
        /// Offset for pagination.
        #[arg(long, default_value = "0")]
        offset: u64,
    },
    /// Show deployment details for a specific account.
    Show {
        /// Contract account ID.
        account_id: String,
    },
    /// List available versions.
    Versions {
        /// Number of results to return.
        #[arg(long, default_value = "50")]
        count: u64,
    },
}

impl RegistryCommand {
    /// Execute the registry command.
    pub async fn run(&self, opts: &GlobalOpts) -> Result<(), CliError> {
        let config = crate::config::Config::load()?;
        let profile = super::markets::resolve_profile_pub(&config, opts)?;
        let rpc: Arc<dyn NearRpcClient> =
            Arc::new(crate::near::rpc::RpcClient::new(opts.effective_rpc_url(profile)));

        match self {
            Self::List { count, offset } => {
                for registry_id in &profile.registry_contract_ids {
                    let reg_account = registry_id
                        .parse()
                        .map_err(|e| CliError::InvalidInput(format!("invalid registry ID: {e}")))?;
                    let client = crate::near::contract::registry::RegistryClient::new(
                        rpc.clone(),
                        reg_account,
                        None,
                    );
                    let deployments = client.list_deployments(*offset, *count).await?;
                    if opts.json_output() {
                        println!("{}", serde_json::to_string_pretty(&deployments)?);
                    } else {
                        println!("  Registry: {registry_id}");
                        for dep in &deployments {
                            let acct = dep["contract_id"].as_str().unwrap_or("unknown");
                            let version = dep["version"].as_str().unwrap_or("?");
                            let kind = dep["kind"].as_str().unwrap_or("?");
                            println!("    {acct} — v{version} ({kind})");
                        }
                    }
                }
                Ok(())
            }
            Self::Show { account_id } => {
                let near_account = account_id
                    .parse()
                    .map_err(|e| CliError::InvalidInput(format!("invalid account ID: {e}")))?;
                for registry_id in &profile.registry_contract_ids {
                    let reg_account = registry_id
                        .parse()
                        .map_err(|e| CliError::InvalidInput(format!("invalid registry ID: {e}")))?;
                    let client = crate::near::contract::registry::RegistryClient::new(
                        rpc.clone(),
                        reg_account,
                        None,
                    );
                    let deployment = client.get_deployment(&near_account).await?;
                    if !deployment.is_null() {
                        if opts.json_output() {
                            println!("{}", serde_json::to_string_pretty(&deployment)?);
                        } else {
                            println!("  Deployment: {account_id}");
                            println!("  Version: {}", deployment["version"]);
                            println!("  Code hash: {}", deployment["code_hash"]);
                        }
                        return Ok(());
                    }
                }
                if opts.json_output() {
                    println!(
                        "{}",
                        serde_json::json!({
                            "error": "no_deployment_found",
                            "account_id": account_id
                        })
                    );
                } else {
                    println!("  No deployment found for {account_id}");
                }
                Ok(())
            }
            Self::Versions { count } => {
                for registry_id in &profile.registry_contract_ids {
                    let reg_account = registry_id
                        .parse()
                        .map_err(|e| CliError::InvalidInput(format!("invalid registry ID: {e}")))?;
                    let client = crate::near::contract::registry::RegistryClient::new(
                        rpc.clone(),
                        reg_account,
                        None,
                    );
                    let versions = client.list_versions(0, *count).await?;
                    if opts.json_output() {
                        println!("{}", serde_json::to_string_pretty(&versions)?);
                    } else {
                        println!("  Available versions:");
                        for v in &versions {
                            println!("    {v}");
                        }
                    }
                }
                Ok(())
            }
        }
    }
}
