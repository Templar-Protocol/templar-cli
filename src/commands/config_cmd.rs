//! Configuration management commands.

use clap::Subcommand;

use crate::config::Config;
use crate::error::CliError;
use super::GlobalOpts;

/// Configuration management subcommands.
#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// Initialize a new configuration file interactively.
    Init,
    /// Display the active configuration.
    Show,
    /// Set a configuration value.
    Set {
        /// The config key to set (e.g., `near_rpc_url`).
        key: String,
        /// The value to set.
        value: String,
    },
}

impl ConfigCommand {
    /// Execute the config command.
    pub async fn run(&self, opts: &GlobalOpts) -> Result<(), CliError> {
        match self {
            Self::Init => run_init(opts).await,
            Self::Show => run_show(opts).await,
            Self::Set { key, value } => run_set(opts, key, value).await,
        }
    }
}

async fn run_init(_opts: &GlobalOpts) -> Result<(), CliError> {
    if Config::exists()? {
        println!("Configuration already exists. Use 'templar config show' to view.");
        return Ok(());
    }

    let config = Config::default();
    config.save()?;
    let path = Config::config_path()?;
    println!("  \u{2720} Configuration forged at {}", path.display());
    println!("    Your keys. Your protocol. Your bank.");
    Ok(())
}

async fn run_show(opts: &GlobalOpts) -> Result<(), CliError> {
    let config = Config::load()?;
    if opts.json_output() {
        println!("{}", serde_json::to_string_pretty(&config).map_err(|e| CliError::Serialization(e.to_string()))?);
    } else {
        println!("  Active profile: {}", config.active_profile);
        let profile = config.active_profile()?;
        println!("  NEAR RPC:       {}", profile.near_rpc_url);
        println!("  Network:        {}", profile.near_network_id);
        println!("  Backend:        {}", profile.backend_url);
        println!("  Registry:       {:?}", profile.registry_contract_ids);
    }
    Ok(())
}

async fn run_set(_opts: &GlobalOpts, key: &str, value: &str) -> Result<(), CliError> {
    let mut config = Config::load()?;
    let profile_name = config.active_profile.clone();
    let profile = config
        .profiles
        .get_mut(&profile_name)
        .ok_or_else(|| CliError::Config(format!("profile '{profile_name}' not found")))?;

    match key {
        "near_rpc_url" => profile.near_rpc_url = value.to_string(),
        "backend_url" => profile.backend_url = value.to_string(),
        "near_network_id" => profile.near_network_id = value.to_string(),
        _ => return Err(CliError::InvalidInput(format!("unknown config key: {key}"))),
    }

    config.save()?;
    println!("  Set {key} = {value}");
    Ok(())
}
