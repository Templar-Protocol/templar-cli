//! JSON output mode for machine-readable CLI output.
//!
//! When the user passes `--output json`, all command output should be emitted
//! as structured JSON to stdout. This module provides helpers to format
//! command results, errors, and status messages as JSON objects with a
//! consistent envelope structure.

use serde::Serialize;
use serde_json::Value;

use crate::error::CliError;

// ---------------------------------------------------------------------------
// JSON envelope
// ---------------------------------------------------------------------------

/// Standard JSON envelope wrapping every CLI output in JSON mode.
///
/// The envelope provides a uniform structure so that scripts and tools can
/// reliably parse output regardless of the specific command.
///
/// # Fields
///
/// | Field     | Description |
/// |-----------|-------------|
/// | `ok`      | `true` for success, `false` for error |
/// | `command` | The CLI command that produced this output (e.g., `"position list"`) |
/// | `data`    | The command-specific payload (absent on error) |
/// | `error`   | Error message string (absent on success) |
#[derive(Debug, Clone, Serialize)]
pub struct JsonEnvelope {
    /// Whether the operation succeeded.
    pub ok: bool,
    /// The command path that produced this output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// The command-specific payload (present on success).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    /// Error message (present on failure).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl JsonEnvelope {
    /// Create a success envelope containing arbitrary serialisable `data`.
    pub fn success<T: Serialize>(command: &str, data: &T) -> Result<Self, CliError> {
        let value = serde_json::to_value(data)?;
        Ok(Self { ok: true, command: Some(command.to_string()), data: Some(value), error: None })
    }

    /// Create a success envelope from a pre-built [`serde_json::Value`].
    pub fn success_value(command: &str, value: Value) -> Self {
        Self { ok: true, command: Some(command.to_string()), data: Some(value), error: None }
    }

    /// Create an error envelope.
    pub fn error(command: &str, err: &CliError) -> Self {
        Self {
            ok: false,
            command: Some(command.to_string()),
            data: None,
            error: Some(err.to_string()),
        }
    }

    /// Create an error envelope from a plain message string.
    pub fn error_msg(command: &str, msg: &str) -> Self {
        Self {
            ok: false,
            command: Some(command.to_string()),
            data: None,
            error: Some(msg.to_string()),
        }
    }

    /// Serialise the envelope to a pretty-printed JSON string.
    pub fn to_json_pretty(&self) -> Result<String, CliError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Serialise the envelope to a compact (single-line) JSON string.
    pub fn to_json(&self) -> Result<String, CliError> {
        Ok(serde_json::to_string(self)?)
    }
}

// ---------------------------------------------------------------------------
// Convenience functions
// ---------------------------------------------------------------------------

/// Print a success JSON envelope to stdout (pretty-printed).
///
/// This is the primary entry point for commands running in JSON output mode.
pub fn print_json<T: Serialize>(command: &str, data: &T) -> Result<(), CliError> {
    let envelope = JsonEnvelope::success(command, data)?;
    let json = envelope.to_json_pretty()?;
    println!("{json}");
    Ok(())
}

/// Print a success JSON envelope from a raw [`Value`] to stdout.
pub fn print_json_value(command: &str, value: Value) -> Result<(), CliError> {
    let envelope = JsonEnvelope::success_value(command, value);
    let json = envelope.to_json_pretty()?;
    println!("{json}");
    Ok(())
}

/// Print an error JSON envelope to stdout.
///
/// Errors in JSON mode are also written to **stdout** (not stderr) so that
/// the consuming process receives a single consistent stream.
pub fn print_json_error(command: &str, err: &CliError) -> Result<(), CliError> {
    let envelope = JsonEnvelope::error(command, err);
    let json = envelope.to_json_pretty()?;
    println!("{json}");
    Ok(())
}

/// Print an error JSON envelope from a plain message to stdout.
pub fn print_json_error_msg(command: &str, msg: &str) -> Result<(), CliError> {
    let envelope = JsonEnvelope::error_msg(command, msg);
    let json = envelope.to_json_pretty()?;
    println!("{json}");
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn success_envelope_structure() {
        let env = JsonEnvelope::success("test cmd", &json!({"key": "value"})).unwrap();
        assert!(env.ok);
        assert_eq!(env.command.as_deref(), Some("test cmd"));
        assert!(env.data.is_some());
        assert!(env.error.is_none());
    }

    #[test]
    fn success_value_envelope() {
        let env = JsonEnvelope::success_value("cmd", json!(42));
        assert!(env.ok);
        assert_eq!(env.data, Some(json!(42)));
    }

    #[test]
    fn error_envelope_structure() {
        let err = CliError::Rpc("timeout".into());
        let env = JsonEnvelope::error("rpc call", &err);
        assert!(!env.ok);
        assert_eq!(env.command.as_deref(), Some("rpc call"));
        assert!(env.data.is_none());
        assert!(env.error.is_some());
        assert!(env.error.as_ref().unwrap().contains("timeout"));
    }

    #[test]
    fn error_msg_envelope() {
        let env = JsonEnvelope::error_msg("cmd", "something broke");
        assert!(!env.ok);
        assert_eq!(env.error.as_deref(), Some("something broke"));
    }

    #[test]
    fn to_json_pretty_is_multiline() {
        let env = JsonEnvelope::success("cmd", &json!({"a": 1})).unwrap();
        let json_str = env.to_json_pretty().unwrap();
        assert!(json_str.contains('\n'), "pretty JSON should contain newlines");
    }

    #[test]
    fn to_json_compact_is_single_line() {
        let env = JsonEnvelope::success("cmd", &json!({"a": 1})).unwrap();
        let json_str = env.to_json().unwrap();
        assert!(!json_str.contains('\n'), "compact JSON should be single-line");
    }

    #[test]
    fn success_envelope_serialises_string_data() {
        let env = JsonEnvelope::success("cmd", &"hello").unwrap();
        assert_eq!(env.data, Some(json!("hello")));
    }

    #[test]
    fn success_envelope_serialises_vec_data() {
        let data = vec![1u32, 2, 3];
        let env = JsonEnvelope::success("cmd", &data).unwrap();
        assert_eq!(env.data, Some(json!([1, 2, 3])));
    }

    #[test]
    fn envelope_omits_none_fields() {
        let env = JsonEnvelope::success_value("cmd", json!(null));
        let json_str = env.to_json().unwrap();
        // "error" field should be absent (skip_serializing_if = None)
        assert!(!json_str.contains("\"error\""));
    }

    #[test]
    fn error_envelope_omits_data() {
        let env = JsonEnvelope::error_msg("cmd", "oops");
        let json_str = env.to_json().unwrap();
        assert!(!json_str.contains("\"data\""));
    }

    #[test]
    fn round_trip_through_serde() {
        let env = JsonEnvelope::success("cmd", &json!({"x": [1, 2]})).unwrap();
        let json_str = env.to_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed["ok"], json!(true));
        assert_eq!(parsed["command"], json!("cmd"));
        assert_eq!(parsed["data"]["x"], json!([1, 2]));
    }

    #[test]
    fn cli_error_variants_in_envelope() {
        let variants: Vec<CliError> = vec![
            CliError::Config("bad".into()),
            CliError::Rpc("timeout".into()),
            CliError::InvalidInput("nope".into()),
        ];
        for err in &variants {
            let env = JsonEnvelope::error("test", err);
            assert!(!env.ok);
            let msg = env.error.as_ref().unwrap();
            assert!(!msg.is_empty());
        }
    }
}
