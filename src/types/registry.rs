//! Registry types for contract deployment tracking.
//!
//! The Templar registry contract keeps a record of all deployed market and
//! vault contracts, including their version keys and code hashes.

use serde::{Deserialize, Serialize};

use super::number::U64;

// ---------------------------------------------------------------------------
// Deployment
// ---------------------------------------------------------------------------

/// A record of a deployed contract stored in the registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deployment {
    /// Version key identifying the contract release (e.g., "v1.2.0").
    pub version_key: String,

    /// Base58-encoded SHA-256 hash of the deployed WASM code.
    pub code_hash: String,

    /// Block height at which the deployment was recorded.
    pub block_height: U64,
}

// ---------------------------------------------------------------------------
// DeployMode
// ---------------------------------------------------------------------------

/// Mode of contract deployment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DeployMode {
    /// Standard deployment: each contract gets its own code upload.
    #[default]
    Normal,

    /// Global-hash deployment: contracts share a single code hash.
    GlobalHash,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_deployment() -> Deployment {
        Deployment {
            version_key: "v1.2.0".into(),
            code_hash: "4uQeVj5tqViQh7yWWGStvkEG1Zmhx6uasJtWCJziofM".into(),
            block_height: U64(123_456_789),
        }
    }

    #[test]
    fn deployment_round_trip() {
        let original = sample_deployment();
        let json = serde_json::to_string(&original).unwrap();
        let parsed: Deployment = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn deployment_json_format() {
        let dep = sample_deployment();
        let value = serde_json::to_value(&dep).unwrap();
        assert_eq!(value["version_key"], "v1.2.0");
        assert_eq!(value["code_hash"], "4uQeVj5tqViQh7yWWGStvkEG1Zmhx6uasJtWCJziofM");
        // block_height is U64 so serialized as string
        assert_eq!(value["block_height"], "123456789");
    }

    #[test]
    fn deployment_from_json() {
        let json = r#"{
            "version_key": "v2.0.0-rc1",
            "code_hash": "ABCdef123456",
            "block_height": "999999999"
        }"#;
        let dep: Deployment = serde_json::from_str(json).unwrap();
        assert_eq!(dep.version_key, "v2.0.0-rc1");
        assert_eq!(dep.code_hash, "ABCdef123456");
        assert_eq!(dep.block_height.0, 999_999_999);
    }

    #[test]
    fn deploy_mode_round_trip() {
        for mode in [DeployMode::Normal, DeployMode::GlobalHash] {
            let json = serde_json::to_string(&mode).unwrap();
            let parsed: DeployMode = serde_json::from_str(&json).unwrap();
            assert_eq!(mode, parsed);
        }
    }

    #[test]
    fn deploy_mode_json_strings() {
        let normal = serde_json::to_string(&DeployMode::Normal).unwrap();
        assert_eq!(normal, "\"Normal\"");
        let global = serde_json::to_string(&DeployMode::GlobalHash).unwrap();
        assert_eq!(global, "\"GlobalHash\"");
    }

    #[test]
    fn deploy_mode_default() {
        assert_eq!(DeployMode::default(), DeployMode::Normal);
    }
}
