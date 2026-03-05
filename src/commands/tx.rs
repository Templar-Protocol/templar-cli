//! Transaction inspection commands.

use super::GlobalOpts;
use crate::error::CliError;
use crate::near::rpc::NearRpcClient;
use clap::Subcommand;

/// Transaction subcommands.
#[derive(Subcommand, Debug)]
pub enum TxCommand {
    /// Check the status of a transaction.
    Status {
        /// Transaction hash.
        tx_hash: String,
        /// Sender account ID.
        #[arg(long)]
        signer: String,
    },
}

impl TxCommand {
    /// Execute the tx command.
    pub async fn run(&self, opts: &GlobalOpts) -> Result<(), CliError> {
        match self {
            Self::Status { tx_hash, signer } => {
                let config = crate::config::Config::load()?;
                let profile = super::markets::resolve_profile_pub(&config, opts)?;
                let rpc = crate::near::rpc::RpcClient::new(opts.effective_rpc_url(profile));
                let hash: near_primitives::hash::CryptoHash = tx_hash.parse().map_err(|_| {
                    CliError::InvalidInput(format!("invalid transaction hash: {tx_hash}"))
                })?;
                let sender: near_primitives::types::AccountId = signer
                    .parse()
                    .map_err(|e| CliError::InvalidInput(format!("invalid signer ID: {e}")))?;
                let outcome = rpc.tx_status(hash, &sender).await?;
                if opts.json_output() {
                    println!("{}", serde_json::to_string_pretty(&outcome)?);
                } else {
                    println!("  Transaction: {tx_hash}");
                    println!("  Status: {:?}", outcome.status);
                }
                Ok(())
            }
        }
    }
}
