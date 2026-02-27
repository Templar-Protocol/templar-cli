//! NEAR credential loading and transaction signing.
//!
//! [`NearSigner`] loads ed25519 credentials from `~/.near-credentials/`
//! (the standard NEAR CLI credential directory) and signs transactions.
//!
//! ## Credential file format
//!
//! The standard JSON key file format used by `near-cli`:
//!
//! ```json
//! {
//!   "account_id": "alice.near",
//!   "public_key": "ed25519:...",
//!   "private_key": "ed25519:..."
//! }
//! ```

use std::path::{Path, PathBuf};

use near_crypto::{InMemorySigner, PublicKey, SecretKey};
use near_primitives::transaction::{SignedTransaction, Transaction};
use near_primitives::types::AccountId;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::error::CliError;

// ---------------------------------------------------------------------------
// Credential file structures
// ---------------------------------------------------------------------------

/// JSON key file as stored by `near-cli` in `~/.near-credentials/`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearKeyFile {
    /// The account ID this key belongs to.
    pub account_id: String,

    /// The ed25519 public key in NEAR format (e.g., "ed25519:...").
    pub public_key: String,

    /// The ed25519 private/secret key in NEAR format (e.g., "ed25519:...").
    pub private_key: String,
}

// ---------------------------------------------------------------------------
// NearSigner
// ---------------------------------------------------------------------------

/// Loads NEAR credentials and signs transactions.
///
/// The signer holds an in-memory copy of the secret key and the associated
/// account ID. It is typically constructed from a credential file on disk.
#[derive(Clone)]
pub struct NearSigner {
    /// The account ID associated with these credentials.
    account_id: AccountId,

    /// The underlying signer with the secret key in memory.
    signer: InMemorySigner,
}

impl NearSigner {
    /// Create a signer from an account ID and secret key.
    pub fn new(account_id: AccountId, secret_key: SecretKey) -> Self {
        let signer = InMemorySigner::from_secret_key(account_id.clone(), secret_key);
        Self { account_id, signer }
    }

    /// Load credentials from a specific JSON key file.
    pub fn from_key_file(path: &Path) -> Result<Self, CliError> {
        let contents = std::fs::read_to_string(path).map_err(|e| {
            CliError::Signing(format!("cannot read key file {}: {e}", path.display()))
        })?;

        let key_file: NearKeyFile = serde_json::from_str(&contents).map_err(|e| {
            CliError::Signing(format!("invalid key file format in {}: {e}", path.display()))
        })?;

        Self::from_key_file_data(&key_file)
    }

    /// Construct a signer from parsed key file data.
    pub fn from_key_file_data(key_file: &NearKeyFile) -> Result<Self, CliError> {
        let account_id: AccountId = key_file.account_id.parse().map_err(|e| {
            CliError::Signing(format!("invalid account_id '{}': {e}", key_file.account_id))
        })?;

        let secret_key: SecretKey = key_file
            .private_key
            .parse()
            .map_err(|e| CliError::Signing(format!("invalid private_key: {e}")))?;

        Ok(Self::new(account_id, secret_key))
    }

    /// Resolve the default credential directory: `~/.near-credentials/`.
    pub fn credentials_dir() -> Result<PathBuf, CliError> {
        let home = dirs::home_dir()
            .ok_or_else(|| CliError::Signing("cannot determine home directory".into()))?;
        Ok(home.join(".near-credentials"))
    }

    /// Resolve the path to a key file for a given network and account.
    ///
    /// Standard layout: `~/.near-credentials/{network_id}/{account_id}.json`
    pub fn key_file_path(network_id: &str, account_id: &AccountId) -> Result<PathBuf, CliError> {
        let creds_dir = Self::credentials_dir()?;
        Ok(creds_dir.join(network_id).join(format!("{account_id}.json")))
    }

    /// Load credentials from the standard location for a network/account.
    ///
    /// Looks up `~/.near-credentials/{network_id}/{account_id}.json`.
    pub fn from_credentials_dir(
        network_id: &str,
        account_id: &AccountId,
    ) -> Result<Self, CliError> {
        let path = Self::key_file_path(network_id, account_id)?;

        if !path.exists() {
            return Err(CliError::Signing(format!(
                "no credentials found for {account_id} on {network_id}. \
                 Expected key file at: {}",
                path.display()
            )));
        }

        debug!(
            account_id = %account_id,
            network = network_id,
            path = %path.display(),
            "loading credentials"
        );

        Self::from_key_file(&path)
    }

    /// Returns the account ID associated with this signer.
    pub fn account_id(&self) -> &AccountId {
        &self.account_id
    }

    /// Returns the public key of this signer.
    pub fn public_key(&self) -> PublicKey {
        self.signer.public_key()
    }

    /// Sign a transaction, producing a [`SignedTransaction`].
    ///
    /// The transaction must already have the correct `signer_id`, nonce,
    /// `block_hash`, and actions set (via [`crate::near::tx_builder::TransactionBuilder`]).
    pub fn sign_transaction(&self, tx: Transaction) -> SignedTransaction {
        let hash = tx.get_hash_and_size().0;
        let signature = self.signer.sign(hash.as_ref());
        SignedTransaction::new(signature, tx)
    }

    /// Sign arbitrary bytes and return the signature.
    pub fn sign_bytes(&self, data: &[u8]) -> near_crypto::Signature {
        self.signer.sign(data)
    }

    /// Verify that the signer's key matches an expected public key.
    pub fn verify_public_key(&self, expected: &PublicKey) -> Result<(), CliError> {
        let actual = self.public_key();
        if &actual != expected {
            return Err(CliError::Signing(format!(
                "public key mismatch: expected {expected}, got {actual}"
            )));
        }
        Ok(())
    }
}

impl std::fmt::Debug for NearSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NearSigner")
            .field("account_id", &self.account_id)
            .field("public_key", &self.public_key().to_string())
            .finish_non_exhaustive()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use near_primitives::hash::CryptoHash;
    use tempfile::TempDir;

    fn make_test_secret_key() -> SecretKey {
        SecretKey::from_random(near_crypto::KeyType::ED25519)
    }

    fn test_key_file_json() -> String {
        let sk = make_test_secret_key();
        let pk = sk.public_key();
        serde_json::json!({
            "account_id": "alice.testnet",
            "public_key": pk.to_string(),
            "private_key": sk.to_string()
        })
        .to_string()
    }

    fn make_test_signer() -> NearSigner {
        let account_id: AccountId = "alice.testnet".parse().unwrap();
        let sk = make_test_secret_key();
        NearSigner::new(account_id, sk)
    }

    #[test]
    fn parse_key_file_json() {
        let json = test_key_file_json();
        let key_file: NearKeyFile = serde_json::from_str(&json).unwrap();
        assert_eq!(key_file.account_id, "alice.testnet");
        assert!(key_file.public_key.starts_with("ed25519:"));
        assert!(key_file.private_key.starts_with("ed25519:"));
    }

    #[test]
    fn signer_from_key_file_data() {
        let signer = make_test_signer();
        assert_eq!(signer.account_id().as_str(), "alice.testnet");
        assert!(signer.public_key().to_string().starts_with("ed25519:"));
    }

    #[test]
    fn signer_from_key_file_on_disk() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("alice.testnet.json");
        std::fs::write(&path, test_key_file_json()).unwrap();

        let signer = NearSigner::from_key_file(&path).unwrap();
        assert_eq!(signer.account_id().as_str(), "alice.testnet");
    }

    #[test]
    fn signer_from_missing_file_error() {
        let path = Path::new("/nonexistent/path/key.json");
        let result = NearSigner::from_key_file(path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, CliError::Signing(_)));
    }

    #[test]
    fn signer_from_invalid_json_error() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("bad.json");
        std::fs::write(&path, "not json").unwrap();

        let result = NearSigner::from_key_file(&path);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CliError::Signing(_)));
    }

    #[test]
    fn signer_from_invalid_account_id_error() {
        let key_file = NearKeyFile {
            account_id: String::new(),
            public_key: "ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp".into(),
            private_key: "ed25519:3D4YudUahN1nawWogh8pAKSj92sUNMdbZGjn7PnUKtg85Y3Yoap7CiE9F8bBdt5vTz7UzLpNxqv5ux7TnUbkrLh".into(),
        };
        let result = NearSigner::from_key_file_data(&key_file);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CliError::Signing(_)));
    }

    #[test]
    fn signer_from_invalid_private_key_error() {
        let key_file = NearKeyFile {
            account_id: "alice.testnet".into(),
            public_key: "ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp".into(),
            private_key: "ed25519:INVALID_KEY".into(),
        };
        let result = NearSigner::from_key_file_data(&key_file);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CliError::Signing(_)));
    }

    #[test]
    fn sign_transaction_produces_valid_signature() {
        let signer = make_test_signer();

        let tx = Transaction::V0(near_primitives::transaction::TransactionV0 {
            signer_id: signer.account_id().clone(),
            public_key: signer.public_key(),
            nonce: 1,
            receiver_id: "contract.testnet".parse().unwrap(),
            block_hash: CryptoHash::default(),
            actions: vec![],
        });

        let signed_tx = signer.sign_transaction(tx);
        assert!(!signed_tx.get_hash().0.is_empty());

        // The signature should verify with the signer's public key.
        let hash = signed_tx.get_hash();
        let valid = signed_tx.signature.verify(hash.as_ref(), &signer.public_key());
        assert!(valid);
    }

    #[test]
    fn sign_bytes_produces_verifiable_signature() {
        let signer = make_test_signer();
        let data = b"hello templar";
        let signature = signer.sign_bytes(data);

        let valid = signature.verify(data, &signer.public_key());
        assert!(valid);
    }

    #[test]
    fn verify_public_key_success() {
        let signer = make_test_signer();
        let pk = signer.public_key();
        assert!(signer.verify_public_key(&pk).is_ok());
    }

    #[test]
    fn verify_public_key_mismatch() {
        let signer = make_test_signer();
        let other_pk: PublicKey =
            "ed25519:Hax8amLbTaTKEvjYBtcJyxcp7D8gKaKBz1XTkPHt4xwR".parse().unwrap();
        let result = signer.verify_public_key(&other_pk);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("mismatch"));
    }

    #[test]
    fn credentials_dir_resolves() {
        // This should not error in any environment with a HOME variable.
        let result = NearSigner::credentials_dir();
        if dirs::home_dir().is_some() {
            let dir = result.unwrap();
            assert!(dir.ends_with(".near-credentials"));
        }
    }

    #[test]
    fn key_file_path_format() {
        let account_id: AccountId = "alice.testnet".parse().unwrap();
        if let Ok(path) = NearSigner::key_file_path("testnet", &account_id) {
            assert!(path.to_string_lossy().contains("testnet/alice.testnet.json"));
        }
    }

    #[test]
    fn credentials_dir_not_found() {
        let account_id: AccountId = "nonexistent.testnet".parse().unwrap();
        let result = NearSigner::from_credentials_dir("testnet", &account_id);
        // In the test environment there is no ~/.near-credentials/ so this
        // should either fail with "no credentials found" or an I/O error.
        assert!(result.is_err());
    }

    #[test]
    fn debug_does_not_leak_secret_key() {
        let signer = make_test_signer();
        let debug_str = format!("{signer:?}");
        assert!(debug_str.contains("alice.testnet"));
        assert!(debug_str.contains("ed25519:"));
        // The debug output should NOT contain the full private key.
        assert!(!debug_str.contains("3D4YudUahN1nawWogh8pAKSj92sUNMdbZGjn7PnUKtg8"));
    }

    #[test]
    fn key_file_round_trip() {
        let key_file = NearKeyFile {
            account_id: "bob.near".into(),
            public_key: "ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp".into(),
            private_key: "ed25519:3D4YudUahN1nawWogh8pAKSj92sUNMdbZGjn7PnUKtg85Y3Yoap7CiE9F8bBdt5vTz7UzLpNxqv5ux7TnUbkrLh".into(),
        };

        let json = serde_json::to_string(&key_file).unwrap();
        let parsed: NearKeyFile = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.account_id, key_file.account_id);
        assert_eq!(parsed.public_key, key_file.public_key);
        assert_eq!(parsed.private_key, key_file.private_key);
    }

    #[test]
    fn signer_new_from_secret_key() {
        let secret_key: SecretKey =
            "ed25519:3D4YudUahN1nawWogh8pAKSj92sUNMdbZGjn7PnUKtg85Y3Yoap7CiE9F8bBdt5vTz7UzLpNxqv5ux7TnUbkrLh"
                .parse()
                .unwrap();
        let account_id: AccountId = "test.near".parse().unwrap();
        let signer = NearSigner::new(account_id.clone(), secret_key);

        assert_eq!(signer.account_id(), &account_id);
        assert!(signer.public_key().to_string().starts_with("ed25519:"));
    }
}
