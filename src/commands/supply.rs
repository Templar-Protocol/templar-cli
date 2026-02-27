//! Supply operations — deposit, withdraw, harvest yield.

use super::GlobalOpts;
use crate::error::CliError;
use clap::Subcommand;

/// Supply subcommands.
#[derive(Subcommand, Debug)]
pub enum SupplyCommand {
    /// Deposit borrow asset into a market.
    Deposit {
        /// Market contract account ID.
        market_id: String,
        /// Amount to deposit (in human-readable units).
        amount: String,
        /// Signer account ID.
        #[arg(long)]
        signer: String,
    },
    /// Request withdrawal from a market.
    Withdraw {
        /// Market contract account ID.
        market_id: String,
        /// Amount to withdraw.
        amount: String,
        /// Signer account ID.
        #[arg(long)]
        signer: String,
    },
    /// Cancel a pending withdrawal request.
    CancelWithdraw {
        /// Market contract account ID.
        market_id: String,
        /// Signer account ID.
        #[arg(long)]
        signer: String,
    },
    /// Execute the next withdrawal request in queue.
    ExecuteWithdraw {
        /// Market contract account ID.
        market_id: String,
        /// Signer account ID.
        #[arg(long)]
        signer: String,
    },
    /// Harvest accumulated yield.
    HarvestYield {
        /// Market contract account ID.
        market_id: String,
        /// Signer account ID.
        #[arg(long)]
        signer: String,
    },
    /// Claim static yield.
    ClaimStaticYield {
        /// Market contract account ID.
        market_id: String,
        /// Signer account ID.
        #[arg(long)]
        signer: String,
    },
}

impl SupplyCommand {
    /// Execute the supply command.
    pub async fn run(&self, _opts: &GlobalOpts) -> Result<(), CliError> {
        if let Self::Deposit { market_id, amount, signer } = self {
            println!("  \u{2720} Preparing supply deposit...");
            println!("    Market:  {market_id}");
            println!("    Amount:  {amount}");
            println!("    Signer:  {signer}");
            println!("  Supply deposit requires signer credentials.");
            println!("  Use `templar config import-key` to set up signing.");
        } else {
            println!("  Supply command: {self:?}");
            println!("  Write operations require signer setup.");
        }
        Ok(())
    }
}
