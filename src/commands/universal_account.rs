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
        match self {
            Self::Whoami => {
                println!("  Universal Account lookup requires key configuration.");
                println!("  Run 'templar config import-key' first.");
                Ok(())
            }
            Self::ListKeys => {
                println!("  Key listing requires Phase 3 implementation.");
                Ok(())
            }
            Self::AddKey { r#type, source } => {
                println!("  Adding {type} key from {source}...");
                println!("  Key management requires Phase 3 implementation.");
                Ok(())
            }
            Self::RemoveKey { alias } => {
                println!("  Removing key '{alias}'...");
                println!("  Key management requires Phase 3 implementation.");
                Ok(())
            }
        }
    }
}
