//! Universal Account operations.

use super::GlobalOpts;
use crate::error::CliError;
use clap::Subcommand;

/// Universal Account subcommands.
#[derive(Subcommand, Debug)]
pub enum UaCommand {
    /// Show the current Universal Account ID.
    Whoami,
    /// List keys associated with the Universal Account.
    ListKeys,
    /// Add a key to the Universal Account.
    AddKey {
        /// Key type (near, solana, evm, stellar).
        #[arg(long)]
        r#type: String,
        /// Path to the key file.
        #[arg(long)]
        source: String,
    },
    /// Remove a key from the Universal Account.
    RemoveKey {
        /// Key alias to remove.
        alias: String,
    },
}

impl UaCommand {
    /// Execute the UA command.
    pub async fn run(&self, _opts: &GlobalOpts) -> Result<(), CliError> {
        Err(CliError::Other(
            "Universal Account operations are not yet implemented (Phase 3)".into(),
        ))
    }
}
