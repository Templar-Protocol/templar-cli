//! Universal account key types.
//!
//! Templar uses a universal account abstraction that supports multiple key
//! types (passkeys, raw Ed25519, EIP-712 signatures, etc.). These types
//! represent key identifiers and their parameters as returned by the
//! account contracts.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// KeyId
// ---------------------------------------------------------------------------

/// A key identifier for a universal account.
///
/// Each variant corresponds to a different signing scheme. The string payload
/// is the public key or identifier in the scheme's native encoding.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyId {
    /// A WebAuthn / FIDO2 passkey.
    Passkey(String),

    /// A raw Ed25519 public key (base58 or hex encoded).
    ///
    /// The contract serializes this variant as `"Ed25519RawKey"`.
    #[serde(rename = "Ed25519RawKey")]
    Ed25519Raw(String),

    /// An EIP-712 typed-data signing key (Ethereum-style).
    Eip712(String),

    /// A SEP-53 Stellar key.
    Sep53(String),

    /// An EIP-191 personal-sign key (Ethereum-style).
    Eip191(String),
}

impl KeyId {
    /// Returns the inner key string regardless of variant.
    pub fn key_str(&self) -> &str {
        match self {
            Self::Passkey(s)
            | Self::Ed25519Raw(s)
            | Self::Eip712(s)
            | Self::Sep53(s)
            | Self::Eip191(s) => s,
        }
    }

    /// Returns the key type name as it appears in JSON.
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Passkey(_) => "Passkey",
            Self::Ed25519Raw(_) => "Ed25519RawKey",
            Self::Eip712(_) => "Eip712",
            Self::Sep53(_) => "Sep53",
            Self::Eip191(_) => "Eip191",
        }
    }
}

impl std::fmt::Display for KeyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.type_name(), self.key_str())
    }
}

// ---------------------------------------------------------------------------
// KeyParameters
// ---------------------------------------------------------------------------

/// Parameters associated with a registered key in a universal account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyParameters {
    /// The key identifier.
    pub key_id: KeyId,

    /// Whether this key is currently active (can sign transactions).
    #[serde(default = "default_true")]
    pub is_active: bool,

    /// Human-readable label for the key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    /// Timestamp (milliseconds) when the key was registered.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registered_at_ms: Option<super::number::U64>,
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_id_passkey_round_trip() {
        let key = KeyId::Passkey("cred-id-abc123".into());
        let json = serde_json::to_string(&key).unwrap();
        let parsed: KeyId = serde_json::from_str(&json).unwrap();
        assert_eq!(key, parsed);
    }

    #[test]
    fn key_id_ed25519raw_round_trip() {
        let key = KeyId::Ed25519Raw("ed25519:abc123".into());
        let json = serde_json::to_string(&key).unwrap();
        let parsed: KeyId = serde_json::from_str(&json).unwrap();
        assert_eq!(key, parsed);
    }

    #[test]
    fn key_id_ed25519raw_serde_rename() {
        let key = KeyId::Ed25519Raw("ed25519:abc123".into());
        let json = serde_json::to_string(&key).unwrap();
        // Must serialize as "Ed25519RawKey", not "Ed25519Raw"
        assert!(json.contains("Ed25519RawKey"), "json was: {json}");
    }

    #[test]
    fn key_id_ed25519raw_deserialize_renamed() {
        let json = r#"{"Ed25519RawKey":"ed25519:abc123"}"#;
        let parsed: KeyId = serde_json::from_str(json).unwrap();
        assert_eq!(parsed, KeyId::Ed25519Raw("ed25519:abc123".into()));
    }

    #[test]
    fn key_id_eip712_round_trip() {
        let key = KeyId::Eip712("0xabcdef".into());
        let json = serde_json::to_string(&key).unwrap();
        let parsed: KeyId = serde_json::from_str(&json).unwrap();
        assert_eq!(key, parsed);
    }

    #[test]
    fn key_id_sep53_round_trip() {
        let key = KeyId::Sep53("GABCDEF".into());
        let json = serde_json::to_string(&key).unwrap();
        let parsed: KeyId = serde_json::from_str(&json).unwrap();
        assert_eq!(key, parsed);
    }

    #[test]
    fn key_id_eip191_round_trip() {
        let key = KeyId::Eip191("0x1234".into());
        let json = serde_json::to_string(&key).unwrap();
        let parsed: KeyId = serde_json::from_str(&json).unwrap();
        assert_eq!(key, parsed);
    }

    #[test]
    fn key_id_key_str() {
        let key = KeyId::Passkey("my-key".into());
        assert_eq!(key.key_str(), "my-key");
    }

    #[test]
    fn key_id_type_name() {
        assert_eq!(KeyId::Passkey("x".into()).type_name(), "Passkey");
        assert_eq!(KeyId::Ed25519Raw("x".into()).type_name(), "Ed25519RawKey");
        assert_eq!(KeyId::Eip712("x".into()).type_name(), "Eip712");
        assert_eq!(KeyId::Sep53("x".into()).type_name(), "Sep53");
        assert_eq!(KeyId::Eip191("x".into()).type_name(), "Eip191");
    }

    #[test]
    fn key_id_display() {
        let key = KeyId::Ed25519Raw("abc".into());
        assert_eq!(key.to_string(), "Ed25519RawKey(abc)");
    }

    #[test]
    fn key_parameters_round_trip() {
        let params = KeyParameters {
            key_id: KeyId::Passkey("cred-123".into()),
            is_active: true,
            label: Some("My Passkey".into()),
            registered_at_ms: Some(super::super::number::U64(1_700_000_000_000)),
        };
        let json = serde_json::to_string(&params).unwrap();
        let parsed: KeyParameters = serde_json::from_str(&json).unwrap();
        assert_eq!(params, parsed);
    }

    #[test]
    fn key_parameters_defaults() {
        // is_active defaults to true, label and registered_at_ms are optional
        let json = r#"{"key_id": {"Passkey": "cred-456"}}"#;
        let params: KeyParameters = serde_json::from_str(json).unwrap();
        assert!(params.is_active);
        assert!(params.label.is_none());
        assert!(params.registered_at_ms.is_none());
    }

    #[test]
    fn key_parameters_inactive() {
        let params = KeyParameters {
            key_id: KeyId::Eip712("0xabc".into()),
            is_active: false,
            label: None,
            registered_at_ms: None,
        };
        let json = serde_json::to_string(&params).unwrap();
        let parsed: KeyParameters = serde_json::from_str(&json).unwrap();
        assert!(!parsed.is_active);
    }

    #[test]
    fn all_key_variants_round_trip() {
        let variants = vec![
            KeyId::Passkey("p".into()),
            KeyId::Ed25519Raw("e".into()),
            KeyId::Eip712("712".into()),
            KeyId::Sep53("s".into()),
            KeyId::Eip191("191".into()),
        ];
        for key in variants {
            let json = serde_json::to_string(&key).unwrap();
            let parsed: KeyId = serde_json::from_str(&json).unwrap();
            assert_eq!(key, parsed, "failed for {json}");
        }
    }
}
