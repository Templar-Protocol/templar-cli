//! Cross-chain bridge operations.

use super::GlobalOpts;
use crate::error::CliError;
use clap::Subcommand;

/// Bridge subcommands.
#[derive(Subcommand, Debug)]
pub enum BridgeCommand {
    /// Initiate a cross-chain deposit.
    Deposit {
        /// Asset to deposit (e.g., BTC, ETH, SOL).
        asset: String,
        /// Amount to deposit.
        amount: String,
    },
    /// Initiate a cross-chain withdrawal.
    Withdraw {
        /// Asset to withdraw.
        asset: String,
        /// Amount to withdraw.
        amount: String,
        /// Destination address on the target chain.
        #[arg(long)]
        destination: String,
        /// Signer account ID.
        #[arg(long)]
        signer: String,
    },
    /// Track a pending bridge operation.
    Track {
        /// Deposit or withdrawal ID to track.
        id: String,
    },
    /// List supported bridge assets.
    SupportedAssets,
}

impl BridgeCommand {
    /// Execute the bridge command.
    pub async fn run(&self, _opts: &GlobalOpts) -> Result<(), CliError> {
        match self {
            Self::Deposit { .. } | Self::Withdraw { .. } | Self::Track { .. } => {
                Err(CliError::Other("bridge operations are not yet implemented (Phase 4)".into()))
            }
            Self::SupportedAssets => {
                println!("  Supported bridge assets:");
                println!("    BTC  — Bitcoin (via OMFT → intents.near)");
                println!("    ETH  — Ethereum (via OMFT → intents.near)");
                println!("    SOL  — Solana (via OMFT → intents.near)");
                println!("    XRP  — XRP Ledger (via OMFT → intents.near)");
                println!("    XLM  — Stellar (via Hot Bridge → intents.near)");
                println!("    ADA  — Cardano (via OMFT → intents.near)");
                println!("    LTC  — Litecoin (via OMFT → intents.near)");
                println!("    ZEC  — Zcash (via OMFT → intents.near)");
                println!("    DOGE — Dogecoin (via OMFT → intents.near)");
                Ok(())
            }
        }
    }
}
