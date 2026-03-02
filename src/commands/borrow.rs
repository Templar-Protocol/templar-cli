//! Borrow operations — collateralize, take, repay, withdraw collateral.

use super::GlobalOpts;
use crate::error::CliError;
use clap::Subcommand;

/// Borrow subcommands.
#[derive(Subcommand, Debug)]
pub enum BorrowCommand {
    /// Deposit collateral into a market.
    Collateralize {
        /// Market contract account ID.
        market_id: String,
        /// Amount of collateral to deposit.
        amount: String,
        /// Signer account ID.
        #[arg(long)]
        signer: String,
    },
    /// Take a borrow from a market.
    Take {
        /// Market contract account ID.
        market_id: String,
        /// Amount to borrow.
        amount: String,
        /// Signer account ID.
        #[arg(long)]
        signer: String,
    },
    /// Repay a borrow.
    Repay {
        /// Market contract account ID.
        market_id: String,
        /// Amount to repay.
        amount: String,
        /// Signer account ID.
        #[arg(long)]
        signer: String,
    },
    /// Withdraw collateral from a market.
    WithdrawCollateral {
        /// Market contract account ID.
        market_id: String,
        /// Amount of collateral to withdraw.
        amount: String,
        /// Signer account ID.
        #[arg(long)]
        signer: String,
    },
    /// Apply pending interest to a borrow position.
    ApplyInterest {
        /// Market contract account ID.
        market_id: String,
        /// Account to apply interest for (defaults to signer).
        #[arg(long)]
        account: Option<String>,
        /// Signer account ID.
        #[arg(long)]
        signer: String,
    },
}

impl BorrowCommand {
    /// Execute the borrow command.
    pub async fn run(&self, _opts: &GlobalOpts) -> Result<(), CliError> {
        Err(CliError::Other(
            "borrow write operations are not yet implemented — \
             signer integration is required (Phase 2)"
                .into(),
        ))
    }
}
