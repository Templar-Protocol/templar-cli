//! Cross-chain bridge types.
//!
//! These types support Templar's cross-chain bridge operations via the
//! Intents/Defuse system. They describe chain identifiers, token information,
//! and bridge intent parameters.

use serde::{Deserialize, Serialize};

use super::number::U128;

// ---------------------------------------------------------------------------
// ChainId
// ---------------------------------------------------------------------------

/// A chain identifier string used by the bridge system.
///
/// Examples: `"near"`, `"eth"`, `"sol"`, `"btc"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ChainId(pub String);

impl ChainId {
    /// Create a new chain ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for ChainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<IntentsChain> for ChainId {
    fn from(chain: IntentsChain) -> Self {
        Self(chain.as_str().to_string())
    }
}

// ---------------------------------------------------------------------------
// IntentsChain
// ---------------------------------------------------------------------------

/// Enumeration of blockchain networks supported by the Intents bridge system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IntentsChain {
    /// NEAR Protocol.
    Near,
    /// Ethereum.
    Eth,
    /// Bitcoin.
    Btc,
    /// Solana.
    Sol,
    /// Stellar.
    Xlm,
    /// Zcash.
    Zec,
    /// Dogecoin.
    Doge,
    /// Cardano.
    Ada,
    /// Litecoin.
    Ltc,
    /// XRP Ledger.
    Xrp,
}

impl IntentsChain {
    /// Returns the lowercase string identifier for this chain.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Near => "near",
            Self::Eth => "eth",
            Self::Btc => "btc",
            Self::Sol => "sol",
            Self::Xlm => "xlm",
            Self::Zec => "zec",
            Self::Doge => "doge",
            Self::Ada => "ada",
            Self::Ltc => "ltc",
            Self::Xrp => "xrp",
        }
    }

    /// All supported chains.
    pub fn all() -> &'static [IntentsChain] {
        &[
            Self::Near,
            Self::Eth,
            Self::Btc,
            Self::Sol,
            Self::Xlm,
            Self::Zec,
            Self::Doge,
            Self::Ada,
            Self::Ltc,
            Self::Xrp,
        ]
    }
}

impl std::fmt::Display for IntentsChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for IntentsChain {
    type Err = crate::error::CliError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "near" => Ok(Self::Near),
            "eth" | "ethereum" => Ok(Self::Eth),
            "btc" | "bitcoin" => Ok(Self::Btc),
            "sol" | "solana" => Ok(Self::Sol),
            "xlm" | "stellar" => Ok(Self::Xlm),
            "zec" | "zcash" => Ok(Self::Zec),
            "doge" | "dogecoin" => Ok(Self::Doge),
            "ada" | "cardano" => Ok(Self::Ada),
            "ltc" | "litecoin" => Ok(Self::Ltc),
            "xrp" | "ripple" => Ok(Self::Xrp),
            other => Err(crate::error::CliError::InvalidInput(format!(
                "unknown chain: '{other}'"
            ))),
        }
    }
}

// ---------------------------------------------------------------------------
// TokenInfo
// ---------------------------------------------------------------------------

/// Information about a token on a specific chain in the bridge system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenInfo {
    /// The chain this token lives on.
    pub chain: ChainId,

    /// The token address or account ID on the source chain.
    pub address: String,

    /// Human-readable token symbol (e.g., "USDC").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,

    /// Token decimals.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decimals: Option<u8>,
}

// ---------------------------------------------------------------------------
// BridgeIntent
// ---------------------------------------------------------------------------

/// A cross-chain bridge intent specifying a token transfer between chains.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BridgeIntent {
    /// Source chain.
    pub src_chain: ChainId,

    /// Destination chain.
    pub dst_chain: ChainId,

    /// Source token address or account ID.
    pub src_token: String,

    /// Destination token address or account ID.
    pub dst_token: String,

    /// Amount to bridge (source token units).
    pub amount: U128,

    /// Sender address on the source chain.
    pub sender: String,

    /// Recipient address on the destination chain.
    pub recipient: String,

    /// Optional deadline timestamp in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deadline_ms: Option<super::number::U64>,
}

// ---------------------------------------------------------------------------
// BridgeStatus
// ---------------------------------------------------------------------------

/// Status of a cross-chain bridge operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum BridgeStatus {
    /// Intent has been submitted and is awaiting solver pickup.
    Pending {
        /// The intent ID.
        intent_id: String,
    },

    /// A solver has picked up the intent and is processing.
    Processing {
        /// The intent ID.
        intent_id: String,
        /// The solver that picked up the intent.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        solver: Option<String>,
    },

    /// The bridge transfer is complete.
    Completed {
        /// The intent ID.
        intent_id: String,
        /// Transaction hash on the destination chain.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        dst_tx_hash: Option<String>,
    },

    /// The bridge transfer failed.
    Failed {
        /// The intent ID.
        intent_id: String,
        /// Error description.
        error: String,
    },

    /// The bridge transfer expired without completion.
    Expired {
        /// The intent ID.
        intent_id: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- ChainId tests --

    #[test]
    fn chain_id_round_trip() {
        let id = ChainId::new("near");
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"near\"");
        let parsed: ChainId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn chain_id_display() {
        assert_eq!(ChainId::new("eth").to_string(), "eth");
    }

    #[test]
    fn chain_id_from_intents_chain() {
        let id: ChainId = IntentsChain::Eth.into();
        assert_eq!(id.0, "eth");
    }

    // -- IntentsChain tests --

    #[test]
    fn intents_chain_round_trip_all() {
        for chain in IntentsChain::all() {
            let json = serde_json::to_string(chain).unwrap();
            let parsed: IntentsChain = serde_json::from_str(&json).unwrap();
            assert_eq!(*chain, parsed);
        }
    }

    #[test]
    fn intents_chain_as_str() {
        assert_eq!(IntentsChain::Near.as_str(), "near");
        assert_eq!(IntentsChain::Eth.as_str(), "eth");
        assert_eq!(IntentsChain::Btc.as_str(), "btc");
        assert_eq!(IntentsChain::Sol.as_str(), "sol");
        assert_eq!(IntentsChain::Xlm.as_str(), "xlm");
        assert_eq!(IntentsChain::Zec.as_str(), "zec");
        assert_eq!(IntentsChain::Doge.as_str(), "doge");
        assert_eq!(IntentsChain::Ada.as_str(), "ada");
        assert_eq!(IntentsChain::Ltc.as_str(), "ltc");
        assert_eq!(IntentsChain::Xrp.as_str(), "xrp");
    }

    #[test]
    fn intents_chain_from_str() {
        assert_eq!("near".parse::<IntentsChain>().unwrap(), IntentsChain::Near);
        assert_eq!("eth".parse::<IntentsChain>().unwrap(), IntentsChain::Eth);
        assert_eq!(
            "ethereum".parse::<IntentsChain>().unwrap(),
            IntentsChain::Eth
        );
        assert_eq!(
            "bitcoin".parse::<IntentsChain>().unwrap(),
            IntentsChain::Btc
        );
        assert_eq!(
            "solana".parse::<IntentsChain>().unwrap(),
            IntentsChain::Sol
        );
        assert_eq!(
            "stellar".parse::<IntentsChain>().unwrap(),
            IntentsChain::Xlm
        );
        assert_eq!("zcash".parse::<IntentsChain>().unwrap(), IntentsChain::Zec);
        assert_eq!(
            "dogecoin".parse::<IntentsChain>().unwrap(),
            IntentsChain::Doge
        );
        assert_eq!(
            "cardano".parse::<IntentsChain>().unwrap(),
            IntentsChain::Ada
        );
        assert_eq!(
            "litecoin".parse::<IntentsChain>().unwrap(),
            IntentsChain::Ltc
        );
        assert_eq!(
            "ripple".parse::<IntentsChain>().unwrap(),
            IntentsChain::Xrp
        );
    }

    #[test]
    fn intents_chain_from_str_case_insensitive() {
        assert_eq!("NEAR".parse::<IntentsChain>().unwrap(), IntentsChain::Near);
        assert_eq!("Eth".parse::<IntentsChain>().unwrap(), IntentsChain::Eth);
    }

    #[test]
    fn intents_chain_from_str_unknown() {
        let result = "polkadot".parse::<IntentsChain>();
        assert!(result.is_err());
    }

    #[test]
    fn intents_chain_display() {
        assert_eq!(IntentsChain::Near.to_string(), "near");
        assert_eq!(IntentsChain::Doge.to_string(), "doge");
    }

    #[test]
    fn intents_chain_all_count() {
        assert_eq!(IntentsChain::all().len(), 10);
    }

    // -- TokenInfo tests --

    #[test]
    fn token_info_round_trip() {
        let info = TokenInfo {
            chain: ChainId::new("near"),
            address: "usdc.near".into(),
            symbol: Some("USDC".into()),
            decimals: Some(6),
        };
        let json = serde_json::to_string(&info).unwrap();
        let parsed: TokenInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info, parsed);
    }

    #[test]
    fn token_info_minimal() {
        let json = r#"{"chain": "eth", "address": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"}"#;
        let info: TokenInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.chain.0, "eth");
        assert!(info.symbol.is_none());
        assert!(info.decimals.is_none());
    }

    #[test]
    fn token_info_json_format() {
        let info = TokenInfo {
            chain: ChainId::new("near"),
            address: "usdc.near".into(),
            symbol: Some("USDC".into()),
            decimals: Some(6),
        };
        let value = serde_json::to_value(&info).unwrap();
        assert_eq!(value["chain"], "near");
        assert_eq!(value["address"], "usdc.near");
        assert_eq!(value["symbol"], "USDC");
        assert_eq!(value["decimals"], 6);
    }

    // -- BridgeIntent tests --

    #[test]
    fn bridge_intent_round_trip() {
        let intent = BridgeIntent {
            src_chain: ChainId::new("eth"),
            dst_chain: ChainId::new("near"),
            src_token: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".into(),
            dst_token: "usdc.near".into(),
            amount: U128(1_000_000),
            sender: "0x1234567890abcdef1234567890abcdef12345678".into(),
            recipient: "alice.near".into(),
            deadline_ms: Some(super::super::number::U64(1_700_001_000_000)),
        };
        let json = serde_json::to_string(&intent).unwrap();
        let parsed: BridgeIntent = serde_json::from_str(&json).unwrap();
        assert_eq!(intent, parsed);
    }

    #[test]
    fn bridge_intent_no_deadline() {
        let intent = BridgeIntent {
            src_chain: ChainId::new("near"),
            dst_chain: ChainId::new("sol"),
            src_token: "usdc.near".into(),
            dst_token: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".into(),
            amount: U128(5_000_000),
            sender: "alice.near".into(),
            recipient: "SoLAnaAddress123".into(),
            deadline_ms: None,
        };
        let json = serde_json::to_string(&intent).unwrap();
        assert!(!json.contains("deadline_ms"));
        let parsed: BridgeIntent = serde_json::from_str(&json).unwrap();
        assert_eq!(intent, parsed);
    }

    // -- BridgeStatus tests --

    #[test]
    fn bridge_status_pending_round_trip() {
        let status = BridgeStatus::Pending {
            intent_id: "intent-123".into(),
        };
        let json = serde_json::to_string(&status).unwrap();
        let parsed: BridgeStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, parsed);
    }

    #[test]
    fn bridge_status_processing_round_trip() {
        let status = BridgeStatus::Processing {
            intent_id: "intent-123".into(),
            solver: Some("solver.near".into()),
        };
        let json = serde_json::to_string(&status).unwrap();
        let parsed: BridgeStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, parsed);
    }

    #[test]
    fn bridge_status_completed_round_trip() {
        let status = BridgeStatus::Completed {
            intent_id: "intent-123".into(),
            dst_tx_hash: Some("0xabc123".into()),
        };
        let json = serde_json::to_string(&status).unwrap();
        let parsed: BridgeStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, parsed);
    }

    #[test]
    fn bridge_status_failed_round_trip() {
        let status = BridgeStatus::Failed {
            intent_id: "intent-123".into(),
            error: "insufficient liquidity".into(),
        };
        let json = serde_json::to_string(&status).unwrap();
        let parsed: BridgeStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, parsed);
    }

    #[test]
    fn bridge_status_expired_round_trip() {
        let status = BridgeStatus::Expired {
            intent_id: "intent-123".into(),
        };
        let json = serde_json::to_string(&status).unwrap();
        let parsed: BridgeStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, parsed);
    }

    #[test]
    fn bridge_status_json_format() {
        let status = BridgeStatus::Pending {
            intent_id: "abc".into(),
        };
        let value = serde_json::to_value(&status).unwrap();
        assert_eq!(value["status"], "Pending");
        assert_eq!(value["intent_id"], "abc");
    }
}
