//! Vault operations — deposit, withdraw, redeem, info.

use std::sync::Arc;

use super::GlobalOpts;
use crate::error::CliError;
use crate::near::rpc::NearRpcClient;
use clap::Subcommand;

/// Vault subcommands.
#[derive(Subcommand, Debug)]
pub enum VaultCommand {
    /// Show vault details.
    Info {
        /// Vault contract account ID.
        vault_id: String,
    },
    /// Deposit into a vault.
    Deposit {
        /// Vault contract account ID.
        vault_id: String,
        /// Amount to deposit.
        amount: String,
        /// Signer account ID.
        #[arg(long)]
        signer: String,
    },
    /// Withdraw from a vault.
    Withdraw {
        /// Vault contract account ID.
        vault_id: String,
        /// Amount to withdraw.
        amount: String,
        /// Signer account ID.
        #[arg(long)]
        signer: String,
    },
    /// Redeem vault shares.
    Redeem {
        /// Vault contract account ID.
        vault_id: String,
        /// Number of shares to redeem.
        shares: String,
        /// Signer account ID.
        #[arg(long)]
        signer: String,
    },
    /// Preview a deposit operation.
    PreviewDeposit {
        /// Vault contract account ID.
        vault_id: String,
        /// Amount to preview.
        amount: String,
    },
    /// Preview a redeem operation.
    PreviewRedeem {
        /// Vault contract account ID.
        vault_id: String,
        /// Shares to preview.
        shares: String,
    },
}

impl VaultCommand {
    /// Execute the vault command.
    pub async fn run(&self, opts: &GlobalOpts) -> Result<(), CliError> {
        let config = crate::config::Config::load()?;
        let profile = super::markets::resolve_profile_pub(&config, opts)?;
        let rpc: Arc<dyn NearRpcClient> =
            Arc::new(crate::near::rpc::RpcClient::new(opts.effective_rpc_url(profile)));

        match self {
            Self::Info { vault_id } => {
                let vault_account = vault_id
                    .parse()
                    .map_err(|e| CliError::InvalidInput(format!("invalid vault ID: {e}")))?;
                let client =
                    crate::near::contract::vault::VaultClient::new(rpc, vault_account, None);
                let vault_config = client.get_configuration().await?;
                if opts.json_output() {
                    println!("{}", serde_json::to_string_pretty(&vault_config)?);
                } else {
                    println!("  Vault: {vault_id}");
                    println!("  Owner: {}", vault_config["owner"]);
                    println!("  Underlying token: {}", vault_config["underlying_token"]);
                }
                Ok(())
            }
            Self::PreviewDeposit { vault_id, amount } => {
                let vault_account = vault_id
                    .parse()
                    .map_err(|e| CliError::InvalidInput(format!("invalid vault ID: {e}")))?;
                let client =
                    crate::near::contract::vault::VaultClient::new(rpc, vault_account, None);
                let shares = client.preview_deposit(amount).await?;
                println!("  Depositing {amount} would mint {shares} shares");
                Ok(())
            }
            Self::PreviewRedeem { vault_id, shares } => {
                let vault_account = vault_id
                    .parse()
                    .map_err(|e| CliError::InvalidInput(format!("invalid vault ID: {e}")))?;
                let client =
                    crate::near::contract::vault::VaultClient::new(rpc, vault_account, None);
                let assets = client.preview_redeem(shares).await?;
                println!("  Redeeming {shares} shares would yield {assets} assets");
                Ok(())
            }
            _ => Err(CliError::Other(
                "vault write operations are not yet implemented — \
                 signer integration is required (Phase 2)"
                    .into(),
            )),
        }
    }
}
