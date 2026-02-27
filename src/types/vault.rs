//! Vault configuration types.
//!
//! A Templar vault is a managed fund that allocates capital across one or more
//! lending markets. These types mirror the vault contract's on-chain
//! configuration as returned by `get_configuration()`.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::number::{Decimal, U64};

// ---------------------------------------------------------------------------
// VaultConfiguration
// ---------------------------------------------------------------------------

/// Full configuration of a Templar vault contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VaultConfiguration {
    /// The vault owner account (full admin).
    pub owner: String,

    /// The curator account (manages allocations).
    pub curator: String,

    /// The guardian account (can pause).
    pub guardian: String,

    /// The sentinel account (automated health checks).
    pub sentinel: String,

    /// The underlying token contract ID.
    pub underlying_token: String,

    /// Initial timelock duration in nanoseconds for queued operations.
    pub initial_timelock_ns: U64,

    /// Fee configuration.
    pub fees: Fees,

    /// Account that receives skimmed (excess) tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skim_recipient: Option<String>,

    /// Human-readable vault name.
    pub name: String,

    /// Vault share token symbol.
    pub symbol: String,

    /// Vault share token decimals.
    pub decimals: u8,

    /// Access restrictions.
    pub restrictions: Restrictions,

    /// Minimum cooldown between refresh operations (nanoseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_cooldown_ns: Option<U64>,

    /// Minimum cooldown between idle resync operations (nanoseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idle_resync_cooldown_ns: Option<U64>,
}

// ---------------------------------------------------------------------------
// Fees
// ---------------------------------------------------------------------------

/// Vault fee configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fees {
    /// Performance fee applied to yield (basis points or decimal).
    pub performance: Fee,

    /// Management fee applied to total assets per period.
    pub management: Fee,

    /// Maximum allowed growth rate of total assets (safety cap).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_total_assets_growth_rate: Option<Decimal>,
}

/// A single fee parameter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fee {
    /// The fee rate (decimal string, e.g., "0.1" for 10%).
    pub rate: Decimal,

    /// The account that receives this fee.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipient: Option<String>,
}

// ---------------------------------------------------------------------------
// Restrictions
// ---------------------------------------------------------------------------

/// Access restriction mode for a vault.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
#[derive(Default)]
pub enum Restrictions {
    /// Vault is paused; no deposits or withdrawals allowed.
    Paused,

    /// Only accounts NOT in the set may interact with the vault.
    BlackList(BTreeSet<String>),

    /// Only accounts in the set may interact with the vault.
    WhiteList(BTreeSet<String>),

    /// No restrictions; anyone may interact.
    #[default]
    None,
}

#[cfg(test)]
mod tests {
    use super::super::number::U64;
    use super::*;

    fn sample_vault_config() -> VaultConfiguration {
        VaultConfiguration {
            owner: "owner.near".into(),
            curator: "curator.near".into(),
            guardian: "guardian.near".into(),
            sentinel: "sentinel.near".into(),
            underlying_token: "usdc.near".into(),
            initial_timelock_ns: U64(3_600_000_000_000),
            fees: Fees {
                performance: Fee {
                    rate: Decimal::new("0.1"),
                    recipient: Some("fee-collector.near".into()),
                },
                management: Fee { rate: Decimal::new("0.02"), recipient: None },
                max_total_assets_growth_rate: Some(Decimal::new("1.5")),
            },
            skim_recipient: Some("treasury.near".into()),
            name: "Templar USDC Vault".into(),
            symbol: "tUSDC".into(),
            decimals: 6,
            restrictions: Restrictions::None,
            refresh_cooldown_ns: Some(U64(60_000_000_000)),
            idle_resync_cooldown_ns: Some(U64(300_000_000_000)),
        }
    }

    #[test]
    fn vault_config_round_trip() {
        let original = sample_vault_config();
        let json = serde_json::to_string(&original).unwrap();
        let parsed: VaultConfiguration = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn vault_config_json_format() {
        let config = sample_vault_config();
        let value = serde_json::to_value(&config).unwrap();
        assert_eq!(value["owner"], "owner.near");
        assert_eq!(value["name"], "Templar USDC Vault");
        assert_eq!(value["symbol"], "tUSDC");
        assert_eq!(value["decimals"], 6);
        assert_eq!(value["initial_timelock_ns"], "3600000000000");
    }

    #[test]
    fn fees_round_trip() {
        let fees = Fees {
            performance: Fee { rate: Decimal::new("0.1"), recipient: Some("perf.near".into()) },
            management: Fee { rate: Decimal::new("0.02"), recipient: None },
            max_total_assets_growth_rate: None,
        };
        let json = serde_json::to_string(&fees).unwrap();
        let parsed: Fees = serde_json::from_str(&json).unwrap();
        assert_eq!(fees, parsed);
    }

    #[test]
    fn restrictions_paused_round_trip() {
        let r = Restrictions::Paused;
        let json = serde_json::to_string(&r).unwrap();
        let parsed: Restrictions = serde_json::from_str(&json).unwrap();
        assert_eq!(r, parsed);
    }

    #[test]
    fn restrictions_blacklist_round_trip() {
        let mut set = BTreeSet::new();
        set.insert("bad-actor.near".into());
        set.insert("blocked.near".into());
        let r = Restrictions::BlackList(set);
        let json = serde_json::to_string(&r).unwrap();
        let parsed: Restrictions = serde_json::from_str(&json).unwrap();
        assert_eq!(r, parsed);
    }

    #[test]
    fn restrictions_whitelist_round_trip() {
        let mut set = BTreeSet::new();
        set.insert("alice.near".into());
        set.insert("bob.near".into());
        let r = Restrictions::WhiteList(set);
        let json = serde_json::to_string(&r).unwrap();
        let parsed: Restrictions = serde_json::from_str(&json).unwrap();
        assert_eq!(r, parsed);
    }

    #[test]
    fn restrictions_none_round_trip() {
        let r = Restrictions::None;
        let json = serde_json::to_string(&r).unwrap();
        let parsed: Restrictions = serde_json::from_str(&json).unwrap();
        assert_eq!(r, parsed);
    }

    #[test]
    fn restrictions_json_format() {
        let r = Restrictions::Paused;
        let value = serde_json::to_value(&r).unwrap();
        assert_eq!(value["type"], "Paused");

        let mut set = BTreeSet::new();
        set.insert("alice.near".to_string());
        let r = Restrictions::WhiteList(set);
        let value = serde_json::to_value(&r).unwrap();
        assert_eq!(value["type"], "WhiteList");
        assert!(value["value"].is_array());
    }

    #[test]
    fn vault_config_minimal_restrictions() {
        let mut config = sample_vault_config();
        config.restrictions = Restrictions::Paused;
        config.skim_recipient = None;
        config.refresh_cooldown_ns = None;
        config.idle_resync_cooldown_ns = None;
        let json = serde_json::to_string(&config).unwrap();
        let parsed: VaultConfiguration = serde_json::from_str(&json).unwrap();
        assert_eq!(config, parsed);
    }

    #[test]
    fn fee_with_no_recipient() {
        let fee = Fee { rate: Decimal::new("0.05"), recipient: None };
        let json = serde_json::to_string(&fee).unwrap();
        // recipient should be omitted
        assert!(!json.contains("recipient"));
        let parsed: Fee = serde_json::from_str(&json).unwrap();
        assert_eq!(fee, parsed);
    }
}
