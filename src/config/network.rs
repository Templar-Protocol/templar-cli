//! Network and chain definitions for NEAR and cross-chain operations.

use serde::{Deserialize, Serialize};

/// NEAR network identifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NearNetwork {
    /// NEAR mainnet.
    Mainnet,
    /// NEAR testnet.
    Testnet,
    /// Custom network with a user-provided ID.
    Custom(String),
}

impl NearNetwork {
    /// Returns the NEAR network ID string.
    pub fn network_id(&self) -> &str {
        match self {
            Self::Mainnet => "mainnet",
            Self::Testnet => "testnet",
            Self::Custom(id) => id,
        }
    }

    /// Returns the default RPC URL for this network.
    pub fn default_rpc_url(&self) -> &str {
        match self {
            Self::Mainnet => "https://rpc.mainnet.fastnear.com",
            Self::Testnet => "https://rpc.testnet.near.org",
            Self::Custom(_) => "",
        }
    }
}

impl Default for NearNetwork {
    fn default() -> Self {
        Self::Mainnet
    }
}

impl std::fmt::Display for NearNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.network_id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_ids() {
        assert_eq!(NearNetwork::Mainnet.network_id(), "mainnet");
        assert_eq!(NearNetwork::Testnet.network_id(), "testnet");
        assert_eq!(NearNetwork::Custom("localnet".into()).network_id(), "localnet");
    }

    #[test]
    fn default_rpc_urls() {
        assert!(NearNetwork::Mainnet.default_rpc_url().contains("mainnet"));
        assert!(NearNetwork::Testnet.default_rpc_url().contains("testnet"));
    }

    #[test]
    fn serde_round_trip() {
        let net = NearNetwork::Mainnet;
        let json = serde_json::to_string(&net).unwrap();
        let parsed: NearNetwork = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, net);
    }
}
