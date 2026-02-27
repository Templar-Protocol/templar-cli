//! Unified error types for all CLI operations.
//!
//! [`CliError`] is the top-level error type returned by all fallible
//! functions in the crate. Each variant maps to a distinct failure domain
//! with user-friendly display messages.

/// Top-level error type for the Templar CLI.
///
/// Each variant represents a distinct failure domain. The [`Display`]
/// implementation produces human-readable messages suitable for terminal output.
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    /// Configuration file errors (missing, invalid TOML, permission issues).
    #[error("Configuration error: {0}")]
    Config(String),

    /// NEAR RPC call failures.
    #[error("RPC error: {0}")]
    Rpc(String),

    /// HTTP request failures (backend, relayer, bridge APIs).
    #[error("HTTP error: {0}")]
    Http(String),

    /// Cross-chain bridge operation failures.
    #[error("Bridge error: {0}")]
    Bridge(String),

    /// Transaction signing failures.
    #[error("Signing error: {0}")]
    Signing(String),

    /// Invalid user input (bad arguments, malformed addresses).
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// File system I/O errors.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/deserialization errors.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// User interrupted the operation.
    #[error("Operation interrupted")]
    Interrupted,

    /// Catch-all for unexpected errors.
    #[error("{0}")]
    Other(String),
}

impl From<serde_json::Error> for CliError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialization(e.to_string())
    }
}

impl From<toml::de::Error> for CliError {
    fn from(e: toml::de::Error) -> Self {
        Self::Config(format!("invalid TOML: {e}"))
    }
}

impl From<toml::ser::Error> for CliError {
    fn from(e: toml::ser::Error) -> Self {
        Self::Serialization(format!("TOML serialization: {e}"))
    }
}

impl From<reqwest::Error> for CliError {
    fn from(e: reqwest::Error) -> Self {
        Self::Http(e.to_string())
    }
}

impl From<url::ParseError> for CliError {
    fn from(e: url::ParseError) -> Self {
        Self::InvalidInput(format!("invalid URL: {e}"))
    }
}

impl CliError {
    /// Returns the exit code to use for this error.
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::InvalidInput(_) => 2,
            Self::Config(_) => 3,
            Self::Interrupted => 130,
            _ => 1,
        }
    }

    /// Returns `true` if this error is likely transient and retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Rpc(_) | Self::Http(_))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_formats_correctly() {
        let err = CliError::Config("missing file".into());
        assert!(err.to_string().contains("Configuration error"));
        assert!(err.to_string().contains("missing file"));
    }

    #[test]
    fn display_all_variants() {
        let variants: Vec<CliError> = vec![
            CliError::Config("test".into()),
            CliError::Rpc("test".into()),
            CliError::Http("test".into()),
            CliError::Bridge("test".into()),
            CliError::Signing("test".into()),
            CliError::InvalidInput("test".into()),
            CliError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "test")),
            CliError::Serialization("test".into()),
            CliError::Interrupted,
            CliError::Other("test".into()),
        ];
        for err in &variants {
            let msg = err.to_string();
            assert!(!msg.is_empty(), "empty display for {err:?}");
        }
    }

    #[test]
    fn exit_codes() {
        assert_eq!(CliError::InvalidInput("x".into()).exit_code(), 2);
        assert_eq!(CliError::Config("x".into()).exit_code(), 3);
        assert_eq!(CliError::Interrupted.exit_code(), 130);
        assert_eq!(CliError::Rpc("x".into()).exit_code(), 1);
    }

    #[test]
    fn retryable() {
        assert!(CliError::Rpc("timeout".into()).is_retryable());
        assert!(CliError::Http("502".into()).is_retryable());
        assert!(!CliError::Config("bad".into()).is_retryable());
        assert!(!CliError::InvalidInput("bad".into()).is_retryable());
    }

    #[test]
    fn from_serde_json_error() {
        let e: Result<serde_json::Value, _> = serde_json::from_str("invalid");
        let cli_err: CliError = e.unwrap_err().into();
        assert!(matches!(cli_err, CliError::Serialization(_)));
    }

    #[test]
    fn from_toml_error() {
        let e: Result<toml::Value, _> = toml::from_str("= invalid");
        let cli_err: CliError = e.unwrap_err().into();
        assert!(matches!(cli_err, CliError::Config(_)));
    }

    #[test]
    fn from_io_error() {
        let e = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let cli_err: CliError = e.into();
        assert!(matches!(cli_err, CliError::Io(_)));
    }

    #[test]
    fn from_url_error() {
        let e: Result<url::Url, _> = url::Url::parse("not a url");
        let cli_err: CliError = e.unwrap_err().into();
        assert!(matches!(cli_err, CliError::InvalidInput(_)));
    }
}
