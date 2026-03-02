//! Oracle price query commands.

use super::GlobalOpts;
use crate::error::CliError;
use clap::Args;

/// Arguments for the prices command.
#[derive(Args, Debug)]
pub struct PricesArgs {
    /// Asset identifiers to query prices for.
    #[arg(required = true)]
    pub asset_ids: Vec<String>,
}

impl PricesArgs {
    /// Execute the prices command.
    pub async fn run(&self, opts: &GlobalOpts) -> Result<(), CliError> {
        let config = crate::config::Config::load()?;
        let profile = if let Some(ref name) = opts.profile {
            config
                .profiles
                .get(name)
                .ok_or_else(|| CliError::Config(format!("profile '{name}' not found")))?
        } else {
            config.active_profile()?
        };
        let client = crate::client::pyth::PythClient::new(&profile.hermes_url)?;
        let prices = client.get_latest_prices(&self.asset_ids).await?;

        if opts.json_output() {
            println!("{}", serde_json::to_string_pretty(&prices)?);
        } else {
            for (id, price) in &prices {
                match price {
                    Some(p) => {
                        println!("  {id}: ${} (conf: {}, expo: {})", p.price, p.conf, p.expo);
                    }
                    None => println!("  {id}: no price available"),
                }
            }
        }
        Ok(())
    }
}
