//! Theme engine for Templar CLI terminal output.
//!
//! Provides the [`TemplarPalette`] brand-colour constants, the [`Theme`] struct
//! that resolves colours to [`console::Style`] instances, and the [`Voice`] enum
//! that switches between cypherpunk and standard copy.
//!
//! Terminal capability detection honours:
//! - The `NO_COLOR` environment variable (<https://no-color.org/>)
//! - The `--color` CLI flag (`always`, `never`, `auto`)
//! - Whether stdout is a real TTY

use console::Style;
use std::env;
use std::io::IsTerminal;

// ---------------------------------------------------------------------------
// Brand palette (ANSI-256 approximations)
// ---------------------------------------------------------------------------

/// Templar Protocol brand colour palette.
///
/// Every constant is an ANSI-256 colour index that closely matches the brand
/// hex value listed in the doc comment.
pub struct TemplarPalette;

impl TemplarPalette {
    /// Gold `#D5AA51` -- primary accent.
    pub const GOLD: u8 = 178;
    /// Antique Gold `#AE8227` -- secondary accent.
    pub const ANTIQUE_GOLD: u8 = 136;
    /// Ivory `#E8E1D3` -- primary text on dark backgrounds.
    pub const IVORY: u8 = 253;
    /// Warm Grey `#A59B89` -- muted / secondary text.
    pub const WARM_GREY: u8 = 144;
    /// Cipher Purple `#963CDC` -- highlight / link colour.
    pub const CIPHER_PURPLE: u8 = 134;
    /// Success Green `#50C759` -- success indicators.
    pub const SUCCESS_GREEN: u8 = 77;
    /// Danger Red `#E03535` -- error / danger indicators.
    pub const DANGER_RED: u8 = 160;
    /// Info Teal `#1499B6` -- informational highlights.
    pub const INFO_TEAL: u8 = 37;
}

// ---------------------------------------------------------------------------
// Voice
// ---------------------------------------------------------------------------

/// Controls the copy style used in user-facing messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Voice {
    /// Themed cypherpunk copy -- evocative, branded language.
    Cypherpunk,
    /// Neutral, plain-English copy.
    Standard,
}

impl Voice {
    /// Parse from a string value (case-insensitive).
    ///
    /// Unrecognised values default to [`Voice::Cypherpunk`].
    pub fn from_str_loose(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "standard" | "plain" | "neutral" => Self::Standard,
            _ => Self::Cypherpunk,
        }
    }
}

impl Default for Voice {
    fn default() -> Self {
        Self::Cypherpunk
    }
}

// ---------------------------------------------------------------------------
// Colour mode resolution
// ---------------------------------------------------------------------------

/// Resolved colour mode after considering env, flags, and TTY state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    /// Force colour output even if not a TTY.
    Always,
    /// Never emit ANSI escape codes.
    Never,
    /// Emit colour only when stdout is a TTY and `NO_COLOR` is unset.
    Auto,
}

impl ColorMode {
    /// Resolve from the `--color` flag value. `None` means `auto`.
    pub fn from_flag(flag: Option<&str>) -> Self {
        match flag.map(|s| s.to_ascii_lowercase()).as_deref() {
            Some("always") | Some("yes") | Some("true") => Self::Always,
            Some("never") | Some("no") | Some("false") => Self::Never,
            _ => Self::Auto,
        }
    }

    /// Returns `true` if colour output should be produced right now.
    pub fn should_color(self) -> bool {
        match self {
            Self::Always => true,
            Self::Never => false,
            Self::Auto => !no_color_set() && std::io::stdout().is_terminal(),
        }
    }
}

/// Returns `true` when the `NO_COLOR` environment variable is set and
/// non-empty, per <https://no-color.org/>.
pub fn no_color_set() -> bool {
    env::var("NO_COLOR").map_or(false, |v| !v.is_empty())
}

/// Returns `true` when stdout is connected to an interactive terminal.
pub fn is_tty() -> bool {
    std::io::stdout().is_terminal()
}

// ---------------------------------------------------------------------------
// Theme
// ---------------------------------------------------------------------------

/// Resolved set of [`console::Style`] instances for the current terminal
/// session.
///
/// When colour is disabled every style is an unstyled default so callers can
/// always call `.apply_to(text)` without branching.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Whether colour is active for this session.
    pub color_enabled: bool,
    /// The active voice mode.
    pub voice: Voice,
    /// Primary brand accent (Gold).
    pub gold: Style,
    /// Secondary brand accent (Antique Gold).
    pub antique_gold: Style,
    /// Primary body text (Ivory).
    pub ivory: Style,
    /// Muted / secondary text (Warm Grey).
    pub warm_grey: Style,
    /// Highlight / link colour (Cipher Purple).
    pub cipher_purple: Style,
    /// Success indicator (Green).
    pub success: Style,
    /// Danger / error indicator (Red).
    pub danger: Style,
    /// Informational highlight (Teal).
    pub info: Style,
    /// Bold variant of Gold for headings.
    pub heading: Style,
    /// Dim style for de-emphasised content.
    pub dim: Style,
    /// Bold style for emphasis.
    pub bold: Style,
}

impl Theme {
    /// Build a [`Theme`] by resolving the colour flag and voice preference.
    ///
    /// `color_flag` is the raw value of the `--color` CLI option (if any).
    /// `voice_pref` is the raw value from config or CLI (if any).
    pub fn resolve(color_flag: Option<&str>, voice_pref: Option<&str>) -> Self {
        let mode = ColorMode::from_flag(color_flag);
        let color_enabled = mode.should_color();
        let voice = voice_pref.map_or(Voice::default(), Voice::from_str_loose);

        if color_enabled {
            Self::colored(voice)
        } else {
            Self::plain(voice)
        }
    }

    /// Build a fully-coloured theme.
    fn colored(voice: Voice) -> Self {
        Self {
            color_enabled: true,
            voice,
            gold: Style::new().color256(TemplarPalette::GOLD),
            antique_gold: Style::new().color256(TemplarPalette::ANTIQUE_GOLD),
            ivory: Style::new().color256(TemplarPalette::IVORY),
            warm_grey: Style::new().color256(TemplarPalette::WARM_GREY),
            cipher_purple: Style::new().color256(TemplarPalette::CIPHER_PURPLE),
            success: Style::new().color256(TemplarPalette::SUCCESS_GREEN),
            danger: Style::new().color256(TemplarPalette::DANGER_RED),
            info: Style::new().color256(TemplarPalette::INFO_TEAL),
            heading: Style::new().color256(TemplarPalette::GOLD).bold(),
            dim: Style::new().dim(),
            bold: Style::new().bold(),
        }
    }

    /// Build a theme where every style is plain (no escape sequences).
    fn plain(voice: Voice) -> Self {
        let s = Style::new();
        Self {
            color_enabled: false,
            voice,
            gold: s.clone(),
            antique_gold: s.clone(),
            ivory: s.clone(),
            warm_grey: s.clone(),
            cipher_purple: s.clone(),
            success: s.clone(),
            danger: s.clone(),
            info: s.clone(),
            heading: s.clone(),
            dim: s.clone(),
            bold: s,
        }
    }

    /// The Templar cross glyph used as the primary prompt prefix.
    pub const CROSS: &'static str = "\u{2720}"; // ✠

    /// The temple glyph used as a complementary decoration.
    pub const TEMPLE: &'static str = "\u{2564}\u{2551}\u{2564}"; // ╤║╤

    /// Format the prompt prefix in the current theme.
    pub fn prompt_prefix(&self) -> String {
        format!("{} ", self.gold.apply_to(Self::CROSS))
    }

    /// Format a success message.
    pub fn fmt_success(&self, msg: &str) -> String {
        format!("{} {}", self.success.apply_to("\u{2714}"), msg)
    }

    /// Format an error message.
    pub fn fmt_error(&self, msg: &str) -> String {
        format!("{} {}", self.danger.apply_to("\u{2718}"), msg)
    }

    /// Format an info message.
    pub fn fmt_info(&self, msg: &str) -> String {
        format!("{} {}", self.info.apply_to("\u{2139}"), msg)
    }

    /// Format a warning message.
    pub fn fmt_warn(&self, msg: &str) -> String {
        format!(
            "{} {}",
            self.antique_gold.apply_to("\u{26A0}"),
            msg
        )
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::resolve(None, None)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voice_from_str_loose_cypherpunk() {
        assert_eq!(Voice::from_str_loose("cypherpunk"), Voice::Cypherpunk);
        assert_eq!(Voice::from_str_loose("CYPHERPUNK"), Voice::Cypherpunk);
        assert_eq!(Voice::from_str_loose("anything"), Voice::Cypherpunk);
    }

    #[test]
    fn voice_from_str_loose_standard() {
        assert_eq!(Voice::from_str_loose("standard"), Voice::Standard);
        assert_eq!(Voice::from_str_loose("Standard"), Voice::Standard);
        assert_eq!(Voice::from_str_loose("plain"), Voice::Standard);
        assert_eq!(Voice::from_str_loose("neutral"), Voice::Standard);
    }

    #[test]
    fn voice_default_is_cypherpunk() {
        assert_eq!(Voice::default(), Voice::Cypherpunk);
    }

    #[test]
    fn color_mode_from_flag() {
        assert_eq!(ColorMode::from_flag(Some("always")), ColorMode::Always);
        assert_eq!(ColorMode::from_flag(Some("ALWAYS")), ColorMode::Always);
        assert_eq!(ColorMode::from_flag(Some("yes")), ColorMode::Always);
        assert_eq!(ColorMode::from_flag(Some("true")), ColorMode::Always);
        assert_eq!(ColorMode::from_flag(Some("never")), ColorMode::Never);
        assert_eq!(ColorMode::from_flag(Some("no")), ColorMode::Never);
        assert_eq!(ColorMode::from_flag(Some("false")), ColorMode::Never);
        assert_eq!(ColorMode::from_flag(Some("auto")), ColorMode::Auto);
        assert_eq!(ColorMode::from_flag(None), ColorMode::Auto);
    }

    #[test]
    fn color_mode_always_should_color() {
        assert!(ColorMode::Always.should_color());
    }

    #[test]
    fn color_mode_never_should_not_color() {
        assert!(!ColorMode::Never.should_color());
    }

    #[test]
    fn theme_resolve_never_disables_color() {
        let theme = Theme::resolve(Some("never"), None);
        assert!(!theme.color_enabled);
    }

    #[test]
    fn theme_resolve_always_enables_color() {
        let theme = Theme::resolve(Some("always"), None);
        assert!(theme.color_enabled);
    }

    #[test]
    fn theme_resolve_voice() {
        let theme = Theme::resolve(None, Some("standard"));
        assert_eq!(theme.voice, Voice::Standard);
    }

    #[test]
    fn theme_cross_constant() {
        assert_eq!(Theme::CROSS, "\u{2720}");
    }

    #[test]
    fn theme_temple_constant() {
        assert_eq!(Theme::TEMPLE, "\u{2564}\u{2551}\u{2564}");
    }

    #[test]
    fn prompt_prefix_contains_cross() {
        let theme = Theme::resolve(Some("never"), None);
        let prefix = theme.prompt_prefix();
        assert!(prefix.contains('\u{2720}'));
    }

    #[test]
    fn fmt_success_contains_checkmark() {
        let theme = Theme::resolve(Some("never"), None);
        let msg = theme.fmt_success("done");
        assert!(msg.contains('\u{2714}'));
        assert!(msg.contains("done"));
    }

    #[test]
    fn fmt_error_contains_cross_mark() {
        let theme = Theme::resolve(Some("never"), None);
        let msg = theme.fmt_error("failed");
        assert!(msg.contains('\u{2718}'));
        assert!(msg.contains("failed"));
    }

    #[test]
    fn fmt_info_contains_info_symbol() {
        let theme = Theme::resolve(Some("never"), None);
        let msg = theme.fmt_info("notice");
        assert!(msg.contains('\u{2139}'));
        assert!(msg.contains("notice"));
    }

    #[test]
    fn fmt_warn_contains_warning_symbol() {
        let theme = Theme::resolve(Some("never"), None);
        let msg = theme.fmt_warn("careful");
        assert!(msg.contains('\u{26A0}'));
        assert!(msg.contains("careful"));
    }

    #[test]
    fn palette_constants_are_valid_ansi256() {
        // ANSI-256 range is 0..=255
        let vals = [
            TemplarPalette::GOLD,
            TemplarPalette::ANTIQUE_GOLD,
            TemplarPalette::IVORY,
            TemplarPalette::WARM_GREY,
            TemplarPalette::CIPHER_PURPLE,
            TemplarPalette::SUCCESS_GREEN,
            TemplarPalette::DANGER_RED,
            TemplarPalette::INFO_TEAL,
        ];
        for v in vals {
            assert!(v <= 255);
        }
    }

    #[test]
    fn theme_default_creates_a_theme() {
        let theme = Theme::default();
        // Just verify it does not panic and has the right voice default
        assert_eq!(theme.voice, Voice::Cypherpunk);
    }
}
