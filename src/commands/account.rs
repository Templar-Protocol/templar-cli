//! Account position query commands.

use std::sync::Arc;

use super::GlobalOpts;
use crate::error::CliError;
use crate::near::rpc::NearRpcClient;
use clap::Subcommand;

/// Account subcommands.
#[derive(Subcommand, Debug)]
pub enum AccountCommand {
    /// Show all supply and borrow positions for an account.
    Positions {
        /// NEAR account ID.
        account_id: String,
    },
    /// Show supply position details.
    Supply {
        /// Market contract account ID.
        market_id: String,
        /// NEAR account ID.
        account_id: String,
    },
    /// Show borrow position details.
    Borrow {
        /// Market contract account ID.
        market_id: String,
        /// NEAR account ID.
        account_id: String,
    },
    /// Show borrow health / MCR status.
    Health {
        /// Market contract account ID.
        market_id: String,
        /// NEAR account ID.
        account_id: String,
    },
    /// Show pending interest for a borrow position.
    PendingInterest {
        /// Market contract account ID.
        market_id: String,
        /// NEAR account ID.
        account_id: String,
    },
    /// Show pending yield for a supply position.
    PendingYield {
        /// Market contract account ID.
        market_id: String,
        /// NEAR account ID.
        account_id: String,
    },
    /// Show withdrawal queue position.
    WithdrawalStatus {
        /// Market contract account ID.
        market_id: String,
        /// NEAR account ID.
        account_id: String,
    },
    /// Show NEP-141 token balance.
    Balance {
        /// Token contract account ID.
        token_id: String,
        /// NEAR account ID.
        account_id: String,
    },
    /// Show NEP-245 multi-token balance.
    MtBalance {
        /// Multi-token contract account ID.
        contract_id: String,
        /// Token ID within the multi-token contract.
        token_id: String,
        /// NEAR account ID.
        account_id: String,
    },
}

impl AccountCommand {
    /// Execute the account command.
    pub async fn run(&self, opts: &GlobalOpts) -> Result<(), CliError> {
        let config = crate::config::Config::load()?;
        let profile = super::markets::resolve_profile_pub(&config, opts)?;
        let rpc: Arc<dyn NearRpcClient> =
            Arc::new(crate::near::rpc::RpcClient::new(opts.effective_rpc_url(profile)));

        match self {
            Self::Supply { market_id, account_id } => {
                let market_account = market_id
                    .parse()
                    .map_err(|e| CliError::InvalidInput(format!("invalid market ID: {e}")))?;
                let near_account = account_id
                    .parse()
                    .map_err(|e| CliError::InvalidInput(format!("invalid account ID: {e}")))?;
                let client =
                    crate::near::contract::market::MarketClient::new(rpc, market_account, None);
                let position = client.get_supply_position(&near_account).await?;
                if opts.json_output() {
                    println!("{}", serde_json::to_string_pretty(&position)?);
                } else if position.is_null() {
                    println!("  No supply position found for {account_id} in {market_id}");
                } else {
                    println!("  Supply position for {account_id} in {market_id}:");
                    println!("  {}", serde_json::to_string_pretty(&position)?);
                }
                Ok(())
            }
            Self::Borrow { market_id, account_id } => {
                let market_account = market_id
                    .parse()
                    .map_err(|e| CliError::InvalidInput(format!("invalid market ID: {e}")))?;
                let near_account = account_id
                    .parse()
                    .map_err(|e| CliError::InvalidInput(format!("invalid account ID: {e}")))?;
                let client =
                    crate::near::contract::market::MarketClient::new(rpc, market_account, None);
                let position = client.get_borrow_position(&near_account).await?;
                if opts.json_output() {
                    println!("{}", serde_json::to_string_pretty(&position)?);
                } else if position.is_null() {
                    println!("  No borrow position found for {account_id} in {market_id}");
                } else {
                    println!("  Borrow position for {account_id} in {market_id}:");
                    println!("  {}", serde_json::to_string_pretty(&position)?);
                }
                Ok(())
            }
            Self::Health { market_id, account_id } => {
                let market_account = market_id
                    .parse()
                    .map_err(|e| CliError::InvalidInput(format!("invalid market ID: {e}")))?;
                let near_account = account_id
                    .parse()
                    .map_err(|e| CliError::InvalidInput(format!("invalid account ID: {e}")))?;
                let client =
                    crate::near::contract::market::MarketClient::new(rpc, market_account, None);
                let status = client.get_borrow_status(&near_account).await?;
                if opts.json_output() {
                    println!("{}", serde_json::to_string_pretty(&status)?);
                } else {
                    println!("  Borrow health for {account_id} in {market_id}:");
                    println!("  {}", serde_json::to_string_pretty(&status)?);
                }
                Ok(())
            }
            Self::PendingInterest { market_id, account_id } => {
                let market_account = market_id
                    .parse()
                    .map_err(|e| CliError::InvalidInput(format!("invalid market ID: {e}")))?;
                let near_account = account_id
                    .parse()
                    .map_err(|e| CliError::InvalidInput(format!("invalid account ID: {e}")))?;
                let client =
                    crate::near::contract::market::MarketClient::new(rpc, market_account, None);
                let interest = client.get_borrow_position_pending_interest(&near_account).await?;
                if opts.json_output() {
                    println!("{}", serde_json::to_string_pretty(&interest)?);
                } else {
                    println!("  Pending interest for {account_id} in {market_id}:");
                    println!("  {}", serde_json::to_string_pretty(&interest)?);
                }
                Ok(())
            }
            Self::PendingYield { market_id, account_id } => {
                let market_account = market_id
                    .parse()
                    .map_err(|e| CliError::InvalidInput(format!("invalid market ID: {e}")))?;
                let near_account = account_id
                    .parse()
                    .map_err(|e| CliError::InvalidInput(format!("invalid account ID: {e}")))?;
                let client =
                    crate::near::contract::market::MarketClient::new(rpc, market_account, None);
                let yield_info = client.get_supply_position_pending_yield(&near_account).await?;
                if opts.json_output() {
                    println!("{}", serde_json::to_string_pretty(&yield_info)?);
                } else {
                    println!("  Pending yield for {account_id} in {market_id}:");
                    println!("  {}", serde_json::to_string_pretty(&yield_info)?);
                }
                Ok(())
            }
            Self::WithdrawalStatus { market_id, account_id } => {
                let market_account = market_id
                    .parse()
                    .map_err(|e| CliError::InvalidInput(format!("invalid market ID: {e}")))?;
                let near_account = account_id
                    .parse()
                    .map_err(|e| CliError::InvalidInput(format!("invalid account ID: {e}")))?;
                let client =
                    crate::near::contract::market::MarketClient::new(rpc, market_account, None);
                let status = client.get_supply_withdrawal_request_status(&near_account).await?;
                if opts.json_output() {
                    println!("{}", serde_json::to_string_pretty(&status)?);
                } else {
                    println!("  Withdrawal status for {account_id} in {market_id}:");
                    println!("  {}", serde_json::to_string_pretty(&status)?);
                }
                Ok(())
            }
            Self::Balance { token_id, account_id } => {
                let token_account: near_primitives::types::AccountId = token_id
                    .parse()
                    .map_err(|e| CliError::InvalidInput(format!("invalid token ID: {e}")))?;
                let near_account: near_primitives::types::AccountId = account_id
                    .parse()
                    .map_err(|e| CliError::InvalidInput(format!("invalid account ID: {e}")))?;
                let client =
                    crate::near::contract::token::TokenClient::new(rpc, token_account, None);
                let balance = client.ft_balance_of(&near_account).await?;
                println!("  Balance: {balance}");
                Ok(())
            }
            Self::MtBalance { contract_id, token_id, account_id } => {
                let contract_account: near_primitives::types::AccountId = contract_id
                    .parse()
                    .map_err(|e| CliError::InvalidInput(format!("invalid contract ID: {e}")))?;
                let near_account: near_primitives::types::AccountId = account_id
                    .parse()
                    .map_err(|e| CliError::InvalidInput(format!("invalid account ID: {e}")))?;
                let client = crate::near::contract::multi_token::MultiTokenClient::new(
                    rpc,
                    contract_account,
                    None,
                );
                let balance = client.mt_balance_of(&near_account, token_id).await?;
                println!("  Balance: {balance}");
                Ok(())
            }
            Self::Positions { .. } => Err(CliError::Other(
                "aggregate position listing is not yet implemented — \
                 use 'templar account supply' or 'templar account borrow' for specific positions"
                    .into(),
            )),
        }
    }
}
