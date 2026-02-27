//! Themed terminal output for the Templar CLI.
//!
//! This module provides all visual presentation logic: banners, spinners,
//! box-drawing frames, tables, and JSON output formatting. Every visual
//! element respects the active [`theme::Theme`] so that colour, voice, and
//! animation preferences are applied consistently.
//!
//! # Sub-modules
//!
//! | Module      | Purpose |
//! |-------------|---------|
//! | [`theme`]   | Colour palette, terminal capability detection, `NO_COLOR` support |
//! | [`logo`]    | ASCII art constants (full mark and inline glyph) |
//! | [`banner`]  | Startup banner with first-run detection |
//! | [`spinner`] | Binary spinner, text scramble reveal, progress bars |
//! | [`frame`]   | Box-drawing panel / table frames (heavy and light) |
//! | [`table`]   | Column-based table renderer |
//! | [`json`]    | JSON output envelope for machine-readable mode |
//!
//! # Output format dispatch
//!
//! Commands call [`OutputFormat::from_opts`] to determine the active output
//! mode and then branch accordingly:
//!
//! ```rust,no_run
//! use templar_cli::display::{OutputFormat, dispatch_output};
//! use templar_cli::display::theme::Theme;
//! use serde_json::json;
//!
//! let format = OutputFormat::from_flag(Some("json"));
//! dispatch_output(format, "position list", &json!({"count": 3}), |data| {
//!     println!("Human-readable: {data}");
//!     Ok(())
//! }).unwrap();
//! ```

pub mod banner;
pub mod frame;
pub mod json;
pub mod logo;
pub mod spinner;
pub mod table;
pub mod theme;

use serde::Serialize;

use crate::error::CliError;

// ---------------------------------------------------------------------------
// OutputFormat
// ---------------------------------------------------------------------------

/// The output format for a CLI command.
///
/// Determined by the `--output` flag or config defaults. Commands use this
/// to decide whether to render human-readable themed output or structured
/// JSON.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-readable, themed terminal output (the default).
    Text,
    /// Machine-readable JSON output (for scripting / piping).
    Json,
}

impl OutputFormat {
    /// Resolve the output format from the `--output` flag value.
    ///
    /// Recognised values (case-insensitive):
    /// - `"json"` -> [`OutputFormat::Json`]
    /// - anything else or `None` -> [`OutputFormat::Text`]
    pub fn from_flag(flag: Option<&str>) -> Self {
        match flag.map(|s| s.to_ascii_lowercase()).as_deref() {
            Some("json") => Self::Json,
            _ => Self::Text,
        }
    }

    /// Returns `true` if this is the JSON output mode.
    pub fn is_json(self) -> bool {
        self == Self::Json
    }

    /// Returns `true` if this is the human-readable text mode.
    pub fn is_text(self) -> bool {
        self == Self::Text
    }
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Text
    }
}

// ---------------------------------------------------------------------------
// Format dispatch
// ---------------------------------------------------------------------------

/// Dispatch command output to the appropriate renderer.
///
/// In [`OutputFormat::Json`] mode the `data` is serialised and wrapped in a
/// [`json::JsonEnvelope`]. In [`OutputFormat::Text`] mode the caller-supplied
/// `render_text` closure is invoked to produce human-readable output.
///
/// This keeps command implementations concise — they prepare the data once and
/// let the display layer handle formatting.
pub fn dispatch_output<T, F>(
    format: OutputFormat,
    command: &str,
    data: &T,
    render_text: F,
) -> Result<(), CliError>
where
    T: Serialize,
    F: FnOnce(&T) -> Result<(), CliError>,
{
    match format {
        OutputFormat::Json => json::print_json(command, data),
        OutputFormat::Text => render_text(data),
    }
}

/// Dispatch an error to the appropriate renderer.
///
/// In JSON mode the error is wrapped in a [`json::JsonEnvelope`] and printed
/// to stdout. In text mode it is formatted via the theme and printed to
/// stderr.
pub fn dispatch_error(
    format: OutputFormat,
    command: &str,
    err: &CliError,
    theme: &theme::Theme,
) -> Result<(), CliError> {
    match format {
        OutputFormat::Json => json::print_json_error(command, err),
        OutputFormat::Text => {
            eprintln!("{}", theme.fmt_error(&err.to_string()));
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// Re-exports for convenience
// ---------------------------------------------------------------------------

pub use banner::BannerOpts;
pub use frame::FrameStyle;
pub use theme::{Theme, Voice};

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // -- OutputFormat -------------------------------------------------------

    #[test]
    fn output_format_from_flag_json() {
        assert_eq!(OutputFormat::from_flag(Some("json")), OutputFormat::Json);
        assert_eq!(OutputFormat::from_flag(Some("JSON")), OutputFormat::Json);
        assert_eq!(OutputFormat::from_flag(Some("Json")), OutputFormat::Json);
    }

    #[test]
    fn output_format_from_flag_text() {
        assert_eq!(OutputFormat::from_flag(Some("text")), OutputFormat::Text);
        assert_eq!(OutputFormat::from_flag(Some("table")), OutputFormat::Text);
        assert_eq!(OutputFormat::from_flag(None), OutputFormat::Text);
    }

    #[test]
    fn output_format_default() {
        assert_eq!(OutputFormat::default(), OutputFormat::Text);
    }

    #[test]
    fn output_format_is_json() {
        assert!(OutputFormat::Json.is_json());
        assert!(!OutputFormat::Text.is_json());
    }

    #[test]
    fn output_format_is_text() {
        assert!(OutputFormat::Text.is_text());
        assert!(!OutputFormat::Json.is_text());
    }

    // -- dispatch_output ----------------------------------------------------

    #[test]
    fn dispatch_output_text_calls_closure() {
        let mut called = false;
        let data = json!({"hello": "world"});
        let result = dispatch_output(OutputFormat::Text, "test", &data, |_d| {
            called = true;
            Ok(())
        });
        assert!(result.is_ok());
        assert!(called);
    }

    #[test]
    fn dispatch_output_json_does_not_call_closure() {
        let data = json!({"hello": "world"});
        // In JSON mode the closure should NOT be called
        let result = dispatch_output(OutputFormat::Json, "test", &data, |_d| {
            panic!("should not be called in JSON mode");
        });
        assert!(result.is_ok());
    }

    // -- dispatch_error -----------------------------------------------------

    #[test]
    fn dispatch_error_text_mode() {
        let theme = Theme::resolve(Some("never"), None);
        let err = CliError::Config("bad config".into());
        let result = dispatch_error(OutputFormat::Text, "init", &err, &theme);
        assert!(result.is_ok());
    }

    #[test]
    fn dispatch_error_json_mode() {
        let theme = Theme::resolve(Some("never"), None);
        let err = CliError::Rpc("timeout".into());
        let result = dispatch_error(OutputFormat::Json, "rpc", &err, &theme);
        assert!(result.is_ok());
    }

    // -- Re-exports ---------------------------------------------------------

    #[test]
    fn re_exports_are_accessible() {
        // Just verify the re-exports compile
        let _theme = Theme::resolve(Some("never"), None);
        let _voice = Voice::Cypherpunk;
        let _style = FrameStyle::Heavy;
        let _opts = BannerOpts {
            quiet: false,
            color: None,
            output: None,
            no_banner: false,
            no_animation: false,
        };
    }
}
