//! CLI command definitions and dispatch.
//!
//! All commands are defined using `clap` derive mode. The top-level [`Cli`]
//! struct contains global options and the [`Commands`] enum dispatches to
//! individual command handlers.

pub mod account;
pub mod borrow;
pub mod bridge_cmd;
pub mod config_cmd;
pub mod markets;
pub mod prices;
pub mod registry;
pub mod supply;
pub mod tx;
pub mod universal_account;
pub mod vault;

use clap::{Parser, Subcommand};

use crate::error::CliError;

/// Templar Protocol CLI — Cypher Lending, Be Your Own Bank.
///
/// Interact with Templar Protocol contracts and services across
/// multiple blockchains from your sovereign terminal.
#[derive(Parser, Debug)]
#[command(
    name = "templar",
    version,
    about = "Templar Protocol CLI — Cypher Lending, Be Your Own Bank",
    long_about = "The Templar Protocol command-line interface.\n\n\
                  Interact with lending markets, vaults, and cross-chain bridge \
                  operations. Your keys. Your protocol. Your bank."
)]
pub struct Cli {
    /// Global options shared across all commands.
    #[command(flatten)]
    pub global_opts: GlobalOpts,

    /// The subcommand to execute.
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Global options available on every command invocation.
#[derive(Parser, Debug, Clone)]
pub struct GlobalOpts {
    /// Suppress banner and non-essential output.
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Color mode: auto, always, or never.
    #[arg(long, global = true, value_name = "MODE")]
    pub color: Option<String>,

    /// Output format: table or json.
    #[arg(short, long, global = true, value_name = "FORMAT")]
    pub output: Option<String>,

    /// Suppress the startup banner.
    #[arg(long, global = true)]
    pub no_banner: bool,

    /// Disable animations (spinners, text scramble).
    #[arg(long, global = true)]
    pub no_animation: bool,

    /// Configuration profile to use.
    #[arg(long, global = true, value_name = "PROFILE")]
    pub profile: Option<String>,

    /// NEAR network override.
    #[arg(long, global = true, value_name = "NETWORK")]
    pub network: Option<String>,

    /// NEAR RPC URL override.
    #[arg(long, global = true, value_name = "URL")]
    pub rpc_url: Option<String>,
}

impl GlobalOpts {
    /// Returns true if JSON output mode is requested.
    pub fn json_output(&self) -> bool {
        self.output.as_deref().is_some_and(|v| v.eq_ignore_ascii_case("json"))
    }

    /// Returns true if color is explicitly disabled.
    pub fn no_color(&self) -> bool {
        self.color.as_deref() == Some("never") || std::env::var("NO_COLOR").is_ok()
    }

    /// Returns the effective NEAR RPC URL, preferring the `--rpc-url` CLI
    /// override over the profile value.
    pub fn effective_rpc_url<'a>(
        &'a self,
        profile: &'a crate::config::profile::Profile,
    ) -> &'a str {
        self.rpc_url.as_deref().unwrap_or(&profile.near_rpc_url)
    }

    /// Returns the effective NEAR network ID, preferring the `--network` CLI
    /// override over the profile value.
    pub fn effective_network<'a>(
        &'a self,
        profile: &'a crate::config::profile::Profile,
    ) -> &'a str {
        self.network.as_deref().unwrap_or(&profile.near_network_id)
    }
}

/// All available subcommands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Configuration management — init, show, set.
    #[command(subcommand)]
    Config(config_cmd::ConfigCommand),

    /// Lending market queries — list, show, metrics.
    #[command(subcommand)]
    Markets(markets::MarketsCommand),

    /// Account position queries — positions, balances, health.
    #[command(subcommand)]
    Account(account::AccountCommand),

    /// Supply operations — deposit, withdraw, harvest yield.
    #[command(subcommand)]
    Supply(supply::SupplyCommand),

    /// Borrow operations — collateralize, take, repay.
    #[command(subcommand)]
    Borrow(borrow::BorrowCommand),

    /// Vault operations — deposit, withdraw, redeem, info.
    #[command(subcommand)]
    Vault(vault::VaultCommand),

    /// Registry queries — list deployments, show versions.
    #[command(subcommand)]
    Registry(registry::RegistryCommand),

    /// Oracle price queries.
    Prices(prices::PricesArgs),

    /// Universal Account operations — whoami, keys.
    #[command(name = "ua", subcommand)]
    UniversalAccount(universal_account::UaCommand),

    /// Cross-chain bridge operations — deposit, withdraw, track.
    #[command(subcommand)]
    Bridge(bridge_cmd::BridgeCommand),

    /// Transaction inspection — status, history.
    #[command(subcommand)]
    Tx(tx::TxCommand),

    /// Check system health.
    Health,
}

impl Cli {
    /// Returns `true` if the banner should be displayed.
    pub fn should_show_banner(&self) -> bool {
        if self.global_opts.quiet || self.global_opts.no_banner {
            return false;
        }
        // Show banner only for root command (no subcommand) or specific commands
        self.command.is_none()
    }

    /// Execute the CLI command.
    pub async fn run(&self) -> Result<(), CliError> {
        match &self.command {
            None => {
                // No subcommand — banner already shown, display help hint
                println!("Run 'templar --help' for available commands.");
                Ok(())
            }
            Some(cmd) => match cmd {
                Commands::Config(c) => c.run(&self.global_opts).await,
                Commands::Markets(c) => c.run(&self.global_opts).await,
                Commands::Account(c) => c.run(&self.global_opts).await,
                Commands::Supply(c) => c.run(&self.global_opts).await,
                Commands::Borrow(c) => c.run(&self.global_opts).await,
                Commands::Vault(c) => c.run(&self.global_opts).await,
                Commands::Registry(c) => c.run(&self.global_opts).await,
                Commands::Prices(c) => c.run(&self.global_opts).await,
                Commands::UniversalAccount(c) => c.run(&self.global_opts).await,
                Commands::Bridge(c) => c.run(&self.global_opts).await,
                Commands::Tx(c) => c.run(&self.global_opts).await,
                Commands::Health => run_health(&self.global_opts).await,
            },
        }
    }
}

async fn run_health(opts: &GlobalOpts) -> Result<(), CliError> {
    let config = crate::config::Config::load()?;
    let profile = if let Some(ref name) = opts.profile {
        config
            .profiles
            .get(name)
            .ok_or_else(|| CliError::Config(format!("profile '{name}' not found")))?
    } else {
        config.active_profile()?
    };

    let client = reqwest::Client::new();
    let url = format!("{}/v1/health", profile.backend_url);
    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            println!("Backend: healthy ({})", profile.backend_url);
        }
        Ok(resp) => {
            println!("Backend: unhealthy — HTTP {} ({})", resp.status(), profile.backend_url);
        }
        Err(e) => {
            println!("Backend: unreachable — {} ({})", e, profile.backend_url);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_parses_no_args() {
        // Running with no args should succeed (shows banner)
        let cli = Cli::try_parse_from(["templar"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert!(cli.command.is_none());
        assert!(cli.should_show_banner());
    }

    #[test]
    fn cli_parses_help() {
        let result = Cli::try_parse_from(["templar", "--help"]);
        // --help causes early exit, so parse returns error
        assert!(result.is_err());
    }

    #[test]
    fn cli_parses_version() {
        let result = Cli::try_parse_from(["templar", "--version"]);
        assert!(result.is_err()); // --version causes early exit
    }

    #[test]
    fn quiet_suppresses_banner() {
        let cli = Cli::try_parse_from(["templar", "--quiet"]).unwrap();
        assert!(!cli.should_show_banner());
    }

    #[test]
    fn no_banner_flag() {
        let cli = Cli::try_parse_from(["templar", "--no-banner"]).unwrap();
        assert!(!cli.should_show_banner());
    }

    #[test]
    fn global_opts_json_output() {
        let cli = Cli::try_parse_from(["templar", "--output", "json"]).unwrap();
        assert!(cli.global_opts.json_output());
    }

    #[test]
    fn global_opts_no_color() {
        let cli = Cli::try_parse_from(["templar", "--color", "never"]).unwrap();
        assert!(cli.global_opts.no_color());
    }

    #[test]
    fn subcommand_suppresses_banner() {
        let cli = Cli::try_parse_from(["templar", "health"]).unwrap();
        assert!(!cli.should_show_banner());
    }

    #[test]
    fn cli_parses_markets_list() {
        let cli = Cli::try_parse_from(["templar", "markets", "list"]);
        assert!(cli.is_ok());
    }

    #[test]
    fn cli_parses_config_show() {
        let cli = Cli::try_parse_from(["templar", "config", "show"]);
        assert!(cli.is_ok());
    }

    #[test]
    fn effective_rpc_url_prefers_override() {
        let cli = Cli::try_parse_from(["templar", "--rpc-url", "https://custom-rpc.example.com"])
            .unwrap();
        let profile = crate::config::profile::Profile::mainnet();
        assert_eq!(cli.global_opts.effective_rpc_url(&profile), "https://custom-rpc.example.com");
    }

    #[test]
    fn effective_rpc_url_falls_back_to_profile() {
        let cli = Cli::try_parse_from(["templar"]).unwrap();
        let profile = crate::config::profile::Profile::mainnet();
        assert_eq!(cli.global_opts.effective_rpc_url(&profile), profile.near_rpc_url);
    }

    #[test]
    fn effective_network_prefers_override() {
        let cli = Cli::try_parse_from(["templar", "--network", "testnet"]).unwrap();
        let profile = crate::config::profile::Profile::mainnet();
        assert_eq!(cli.global_opts.effective_network(&profile), "testnet");
    }

    #[test]
    fn effective_network_falls_back_to_profile() {
        let cli = Cli::try_parse_from(["templar"]).unwrap();
        let profile = crate::config::profile::Profile::mainnet();
        assert_eq!(cli.global_opts.effective_network(&profile), profile.near_network_id);
    }
}
