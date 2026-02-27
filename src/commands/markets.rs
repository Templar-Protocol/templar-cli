//! Lending market query commands.

use std::sync::Arc;

use super::GlobalOpts;
use crate::error::CliError;
use crate::near::rpc::NearRpcClient;
use clap::Subcommand;

/// Market subcommands.
#[derive(Subcommand, Debug)]
pub enum MarketsCommand {
    /// List all markets from the registry.
    List,
    /// Show detailed market configuration and metrics.
    Show {
        /// Market contract account ID.
        market_id: String,
    },
    /// Display the current market snapshot.
    Snapshot {
        /// Market contract account ID.
        market_id: String,
    },
    /// Display historical snapshots.
    Snapshots {
        /// Market contract account ID.
        market_id: String,
        /// Number of snapshots to retrieve.
        #[arg(long, default_value = "10")]
        count: u64,
    },
    /// Display borrow asset metrics.
    Metrics {
        /// Market contract account ID.
        market_id: String,
    },
    /// Display the current yield rate.
    YieldRate {
        /// Market contract account ID.
        market_id: String,
    },
}

fn make_rpc(
    profile: &crate::config::profile::Profile,
    opts: &GlobalOpts,
) -> Arc<dyn NearRpcClient> {
    Arc::new(crate::near::rpc::RpcClient::new(opts.effective_rpc_url(profile)))
}

impl MarketsCommand {
    /// Execute the markets command.
    pub async fn run(&self, opts: &GlobalOpts) -> Result<(), CliError> {
        let config = crate::config::Config::load()?;
        let profile = resolve_profile(&config, opts)?;

        match self {
            Self::List => {
                println!("Querying markets from registry...");
                let rpc = make_rpc(profile, opts);
                for registry_id in &profile.registry_contract_ids {
                    let reg_account = registry_id
                        .parse()
                        .map_err(|e| CliError::InvalidInput(format!("invalid registry ID: {e}")))?;
                    let client = crate::near::contract::registry::RegistryClient::new(
                        rpc.clone(),
                        reg_account,
                        None,
                    );
                    let deployments = client.list_deployments(0, 100).await?;
                    for dep in &deployments {
                        let acct = dep["contract_id"].as_str().unwrap_or("unknown");
                        let version = dep["version"].as_str().unwrap_or("?");
                        println!("  {acct} (v{version})");
                    }
                }
                Ok(())
            }
            Self::Show { market_id } => {
                let rpc = make_rpc(profile, opts);
                let market_account = market_id
                    .parse()
                    .map_err(|e| CliError::InvalidInput(format!("invalid market ID: {e}")))?;
                let client =
                    crate::near::contract::market::MarketClient::new(rpc, market_account, None);
                let market_config = client.get_configuration().await?;
                if opts.json_output() {
                    println!("{}", serde_json::to_string_pretty(&market_config)?);
                } else {
                    println!("  Market: {market_id}");
                    println!("  Borrow asset: {}", market_config["borrow_asset"]);
                    println!("  Collateral asset: {}", market_config["collateral_asset"]);
                }
                Ok(())
            }
            Self::Snapshot { market_id } => {
                let rpc = make_rpc(profile, opts);
                let market_account = market_id
                    .parse()
                    .map_err(|e| CliError::InvalidInput(format!("invalid market ID: {e}")))?;
                let client =
                    crate::near::contract::market::MarketClient::new(rpc, market_account, None);
                let snapshot = client.get_current_snapshot().await?;
                if opts.json_output() {
                    println!("{}", serde_json::to_string_pretty(&snapshot)?);
                } else {
                    println!("  Snapshot for: {market_id}");
                    let rate = snapshot["interest_rate"].as_str().unwrap_or("N/A");
                    println!("  Interest rate: {rate}");
                }
                Ok(())
            }
            Self::Snapshots { market_id, count } => {
                let rpc = make_rpc(profile, opts);
                let market_account = market_id
                    .parse()
                    .map_err(|e| CliError::InvalidInput(format!("invalid market ID: {e}")))?;
                let client =
                    crate::near::contract::market::MarketClient::new(rpc, market_account, None);
                let snapshots = client.list_finalized_snapshots(0, *count).await?;
                if opts.json_output() {
                    println!("{}", serde_json::to_string_pretty(&snapshots)?);
                } else {
                    for snap in &snapshots {
                        let rate = snap["interest_rate"].as_str().unwrap_or("N/A");
                        let chunk = snap["time_chunk"].as_u64().unwrap_or(0);
                        println!("  Chunk {chunk}: rate={rate}");
                    }
                }
                Ok(())
            }
            Self::Metrics { market_id } => {
                let rpc = make_rpc(profile, opts);
                let market_account = market_id
                    .parse()
                    .map_err(|e| CliError::InvalidInput(format!("invalid market ID: {e}")))?;
                let client =
                    crate::near::contract::market::MarketClient::new(rpc, market_account, None);
                let metrics = client.get_borrow_asset_metrics().await?;
                if opts.json_output() {
                    println!("{}", serde_json::to_string_pretty(&metrics)?);
                } else {
                    println!("  Deposited: {}", metrics["deposited_active"]);
                    println!("  Borrowed:  {}", metrics["borrowed"]);
                }
                Ok(())
            }
            Self::YieldRate { market_id } => {
                let rpc = make_rpc(profile, opts);
                let market_account = market_id
                    .parse()
                    .map_err(|e| CliError::InvalidInput(format!("invalid market ID: {e}")))?;
                let client =
                    crate::near::contract::market::MarketClient::new(rpc, market_account, None);
                let rate = client.get_last_yield_rate().await?;
                println!("  Yield rate: {rate}");
                Ok(())
            }
        }
    }
}

/// Resolve the profile from config and global opts. Public for reuse.
pub fn resolve_profile_pub<'a>(
    config: &'a crate::config::Config,
    opts: &GlobalOpts,
) -> Result<&'a crate::config::profile::Profile, CliError> {
    resolve_profile(config, opts)
}

fn resolve_profile<'a>(
    config: &'a crate::config::Config,
    opts: &GlobalOpts,
) -> Result<&'a crate::config::profile::Profile, CliError> {
    if let Some(ref name) = opts.profile {
        config
            .profiles
            .get(name)
            .ok_or_else(|| CliError::Config(format!("profile '{name}' not found")))
    } else {
        config.active_profile()
    }
}
