//! Named configuration profiles for different networks and environments.

use serde::{Deserialize, Serialize};

/// A named configuration profile containing all network settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// NEAR RPC endpoint URL.
    pub near_rpc_url: String,

    /// Templar backend API URL.
    pub backend_url: String,

    /// Relayer V0 URL (Solana, Passkey).
    pub relayer_v0_url: String,

    /// Relayer V1 URL (Stellar, EVM).
    pub relayer_v1_url: String,

    /// NEAR network ID.
    pub near_network_id: String,

    /// NEAR chain ID (EIP-155 style).
    pub near_chain_id: u64,

    /// Registry contract account IDs.
    pub registry_contract_ids: Vec<String>,

    /// Pyth/Hermes oracle endpoint.
    pub hermes_url: String,

    /// Intents/Defuse bridge RPC URL.
    pub bridge_rpc_url: String,

    /// Solver relayer URL.
    pub solver_relayer_url: String,

    /// Hot Bridge contract ID.
    pub hot_bridge_contract: String,

    /// Intents verifier contract ID.
    pub intents_contract: String,

    /// Whether CLI analytics are enabled.
    #[serde(default = "default_true")]
    pub analytics_enabled: bool,

    /// Analytics collection endpoint.
    #[serde(default)]
    pub analytics_endpoint: String,
}

fn default_true() -> bool {
    true
}

impl Profile {
    /// Default mainnet profile with production endpoints.
    pub fn mainnet() -> Self {
        Self {
            near_rpc_url: "https://rpc.mainnet.fastnear.com".into(),
            backend_url: "https://api.templarfi.org".into(),
            relayer_v0_url: "https://relayer.templarfi.org".into(),
            relayer_v1_url: "https://relayer.templarfi.org:4001".into(),
            near_network_id: "mainnet".into(),
            near_chain_id: 397,
            registry_contract_ids: vec!["v1.tmplr.near".into()],
            hermes_url: "https://hermes.pyth.network".into(),
            bridge_rpc_url: "https://bridge.chaindefuser.com/rpc".into(),
            solver_relayer_url: "https://solver-relay.chaindefuser.com/rpc".into(),
            hot_bridge_contract: "v2_1.omni.hot.tg".into(),
            intents_contract: "intents.near".into(),
            analytics_enabled: true,
            analytics_endpoint: "https://analytics.templarfi.org/cli".into(),
        }
    }

    /// Default testnet profile with test endpoints.
    pub fn testnet() -> Self {
        Self {
            near_rpc_url: "https://rpc.testnet.near.org".into(),
            backend_url: "https://api-testnet.templarfi.org".into(),
            relayer_v0_url: "https://relayer-testnet.templarfi.org".into(),
            relayer_v1_url: "https://relayer-testnet.templarfi.org:4001".into(),
            near_network_id: "testnet".into(),
            near_chain_id: 398,
            registry_contract_ids: vec!["v1.tmplr.testnet".into()],
            hermes_url: "https://hermes.pyth.network".into(),
            bridge_rpc_url: "https://bridge-testnet.chaindefuser.com/rpc".into(),
            solver_relayer_url: "https://solver-relay-testnet.chaindefuser.com/rpc".into(),
            hot_bridge_contract: "v2_1.omni-testnet.hot.tg".into(),
            intents_contract: "intents.testnet".into(),
            analytics_enabled: false,
            analytics_endpoint: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mainnet_profile_has_valid_urls() {
        let p = Profile::mainnet();
        assert!(p.near_rpc_url.starts_with("https://"));
        assert!(p.backend_url.starts_with("https://"));
        assert_eq!(p.near_network_id, "mainnet");
        assert_eq!(p.near_chain_id, 397);
        assert!(!p.registry_contract_ids.is_empty());
    }

    #[test]
    fn testnet_profile_has_valid_urls() {
        let p = Profile::testnet();
        assert!(p.near_rpc_url.contains("testnet"));
        assert_eq!(p.near_network_id, "testnet");
        assert_eq!(p.near_chain_id, 398);
    }

    #[test]
    fn profile_round_trips_through_toml() {
        let p = Profile::mainnet();
        let toml_str = toml::to_string_pretty(&p).unwrap();
        let parsed: Profile = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.near_rpc_url, p.near_rpc_url);
        assert_eq!(parsed.near_chain_id, p.near_chain_id);
    }
}
