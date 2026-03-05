//! Relayer API client (stub for Phase 4).
//!
//! The relayer handles meta-transaction relay for Universal Account
//! operations across multiple chains.

use serde::{Deserialize, Serialize};

use crate::error::CliError;

/// Relayer configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayerConfig {
    /// V0 relayer URL (passkey, solana).
    pub v0_url: String,
    /// V1 relayer URL (stellar, evm).
    pub v1_url: String,
    /// Request timeout in seconds.
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_timeout() -> u64 {
    30
}

/// Relayer client (stub — full implementation in Phase 4).
pub struct RelayerClient {
    _config: RelayerConfig,
}

impl RelayerClient {
    /// Create a new relayer client.
    pub fn new(config: RelayerConfig) -> Result<Self, CliError> {
        Ok(Self { _config: config })
    }
}
