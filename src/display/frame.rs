//! Box-drawing frame helpers for panels and tables.
//!
//! Two visual weights are provided:
//!
//! - **Heavy** (`┏━┓ ┗━┛`) — used for primary panels, headings, call-outs.
//! - **Light** (`┌─┐ └─┘`) — used for secondary information, nested panels.
//!
//! All frame functions accept a [`Theme`] so that borders and content can be
//! coloured consistently with the rest of the CLI output.

use std::fmt::Write as FmtWrite;

use unicode_width::UnicodeWidthStr;

use super::theme::Theme;

// ---------------------------------------------------------------------------
// Frame style
// ---------------------------------------------------------------------------

/// Visual weight of box-drawing characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameStyle {
    /// Heavy lines: `┏━┓ ┃ ┗━┛`
    Heavy,
    /// Light lines: `┌─┐ │ └─┘`
    Light,
}

/// Box-drawing character set for a specific frame style.
#[derive(Debug, Clone, Copy)]
pub(super) struct FrameChars {
    /// Top-left corner.
    pub(super) tl: char,
    /// Top-right corner.
    pub(super) tr: char,
    /// Bottom-left corner.
    pub(super) bl: char,
    /// Bottom-right corner.
    pub(super) br: char,
    /// Horizontal line.
    pub(super) h: char,
    /// Vertical line.
    pub(super) v: char,
}

impl FrameStyle {
    /// Return the character set for this frame style.
    pub(super) const fn chars(self) -> FrameChars {
        match self {
            Self::Heavy => FrameChars {
                tl: '\u{250F}', // ┏
                tr: '\u{2513}', // ┓
                bl: '\u{2517}', // ┗
                br: '\u{251B}', // ┛
                h: '\u{2501}',  // ━
                v: '\u{2503}',  // ┃
            },
            Self::Light => FrameChars {
                tl: '\u{250C}', // ┌
                tr: '\u{2510}', // ┐
                bl: '\u{2514}', // └
                br: '\u{2518}', // ┘
                h: '\u{2500}',  // ─
                v: '\u{2502}',  // │
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Panel
// ---------------------------------------------------------------------------

/// Render a panel (box) around one or more lines of text.
///
/// The panel auto-sizes to the widest content line. Each line is
/// left-aligned within the frame with one space of padding on each side.
///
/// # Example output (heavy)
///
/// ```text
/// ┏━━━━━━━━━━━━━━━━━┓
/// ┃ TEMPLAR v0.1.0  ┃
/// ┃ Be Your Own Bank┃
/// ┗━━━━━━━━━━━━━━━━━┛
/// ```
pub fn panel(theme: &Theme, style: FrameStyle, lines: &[&str]) -> String {
    let fc = style.chars();
    let max_width = lines
        .iter()
        .map(|l| UnicodeWidthStr::width(*l))
        .max()
        .unwrap_or(0);

    // Inner width = content + 2 spaces of padding
    let inner = max_width + 2;

    let mut out = String::new();

    // Top border
    let _ = writeln!(
        out,
        "{}",
        theme.gold.apply_to(format!(
            "{}{}{}",
            fc.tl,
            repeat_char(fc.h, inner),
            fc.tr,
        ))
    );

    // Content rows
    for line in lines {
        let display_w = UnicodeWidthStr::width(*line);
        let pad = max_width - display_w;
        let _ = writeln!(
            out,
            "{}",
            theme.gold.apply_to(format!(
                "{} {}{} {}",
                fc.v,
                line,
                " ".repeat(pad),
                fc.v,
            ))
        );
    }

    // Bottom border
    let _ = write!(
        out,
        "{}",
        theme.gold.apply_to(format!(
            "{}{}{}",
            fc.bl,
            repeat_char(fc.h, inner),
            fc.br,
        ))
    );

    out
}

// ---------------------------------------------------------------------------
// Horizontal rule
// ---------------------------------------------------------------------------

/// Render a horizontal rule of the given width using the specified frame style.
pub fn horizontal_rule(theme: &Theme, style: FrameStyle, width: usize) -> String {
    let fc = style.chars();
    format!("{}", theme.warm_grey.apply_to(repeat_char(fc.h, width)))
}

// ---------------------------------------------------------------------------
// Header bar
// ---------------------------------------------------------------------------

/// Render a single-line header bar with centred text.
///
/// # Example output (heavy)
///
/// ```text
/// ┏━━━━━━━━ POSITIONS ━━━━━━━━┓
/// ```
pub fn header_bar(theme: &Theme, style: FrameStyle, title: &str, width: usize) -> String {
    let fc = style.chars();
    let title_display_w = UnicodeWidthStr::width(title) + 2; // +2 for surrounding spaces
    let remaining = width.saturating_sub(title_display_w + 2); // -2 for corners
    let left = remaining / 2;
    let right = remaining - left;

    format!(
        "{}",
        theme.heading.apply_to(format!(
            "{}{} {} {}{}",
            fc.tl,
            repeat_char(fc.h, left),
            title,
            repeat_char(fc.h, right),
            fc.tr,
        ))
    )
}

// ---------------------------------------------------------------------------
// Key-value pair panel
// ---------------------------------------------------------------------------

/// Render a panel containing key-value pairs.
///
/// Keys are right-aligned and coloured with the warm grey style; values are
/// left-aligned with ivory.
///
/// # Example output
///
/// ```text
/// ┌──────────────────────────┐
/// │     Account: alice.near  │
/// │     Balance: 5.23 NEAR   │
/// │      Status: Active      │
/// └──────────────────────────┘
/// ```
pub fn kv_panel(
    theme: &Theme,
    style: FrameStyle,
    pairs: &[(&str, &str)],
) -> String {
    let fc = style.chars();

    let max_key = pairs.iter().map(|(k, _)| UnicodeWidthStr::width(*k)).max().unwrap_or(0);
    let max_val = pairs.iter().map(|(_, v)| UnicodeWidthStr::width(*v)).max().unwrap_or(0);

    // key_col + ": " + val_col + 2 padding spaces
    let inner = max_key + 2 + max_val + 2;

    let mut out = String::new();

    // Top border
    let _ = writeln!(
        out,
        "{}",
        theme.gold.apply_to(format!("{}{}{}", fc.tl, repeat_char(fc.h, inner), fc.tr))
    );

    // Rows
    for (key, val) in pairs {
        let key_w = UnicodeWidthStr::width(*key);
        let val_w = UnicodeWidthStr::width(*val);
        let key_pad = max_key - key_w;
        let val_pad = max_val - val_w;

        let _ = writeln!(
            out,
            "{} {}{}{} {}{} {}",
            theme.gold.apply_to(fc.v.to_string()),
            " ".repeat(key_pad),
            theme.warm_grey.apply_to(*key),
            theme.warm_grey.apply_to(": "),
            theme.ivory.apply_to(*val),
            " ".repeat(val_pad),
            theme.gold.apply_to(fc.v.to_string()),
        );
    }

    // Bottom border
    let _ = write!(
        out,
        "{}",
        theme.gold.apply_to(format!("{}{}{}", fc.bl, repeat_char(fc.h, inner), fc.br))
    );

    out
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Repeat a character `n` times into a `String`.
fn repeat_char(ch: char, n: usize) -> String {
    std::iter::repeat(ch).take(n).collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn plain_theme() -> Theme {
        Theme::resolve(Some("never"), None)
    }

    // -- FrameStyle ---------------------------------------------------------

    #[test]
    fn heavy_chars() {
        let fc = FrameStyle::Heavy.chars();
        assert_eq!(fc.tl, '\u{250F}');
        assert_eq!(fc.tr, '\u{2513}');
        assert_eq!(fc.bl, '\u{2517}');
        assert_eq!(fc.br, '\u{251B}');
        assert_eq!(fc.h, '\u{2501}');
        assert_eq!(fc.v, '\u{2503}');
    }

    #[test]
    fn light_chars() {
        let fc = FrameStyle::Light.chars();
        assert_eq!(fc.tl, '\u{250C}');
        assert_eq!(fc.tr, '\u{2510}');
        assert_eq!(fc.bl, '\u{2514}');
        assert_eq!(fc.br, '\u{2518}');
        assert_eq!(fc.h, '\u{2500}');
        assert_eq!(fc.v, '\u{2502}');
    }

    // -- Panel --------------------------------------------------------------

    #[test]
    fn panel_heavy_single_line() {
        let theme = plain_theme();
        let output = panel(&theme, FrameStyle::Heavy, &["Hello"]);
        assert!(output.contains('\u{250F}')); // ┏
        assert!(output.contains('\u{2513}')); // ┓
        assert!(output.contains('\u{2517}')); // ┗
        assert!(output.contains('\u{251B}')); // ┛
        assert!(output.contains("Hello"));
    }

    #[test]
    fn panel_light_single_line() {
        let theme = plain_theme();
        let output = panel(&theme, FrameStyle::Light, &["World"]);
        assert!(output.contains('\u{250C}')); // ┌
        assert!(output.contains('\u{2510}')); // ┐
        assert!(output.contains('\u{2514}')); // └
        assert!(output.contains('\u{2518}')); // ┘
        assert!(output.contains("World"));
    }

    #[test]
    fn panel_multi_line_aligns() {
        let theme = plain_theme();
        let output = panel(&theme, FrameStyle::Light, &["Short", "A longer line"]);
        // Both lines should be present
        assert!(output.contains("Short"));
        assert!(output.contains("A longer line"));
    }

    #[test]
    fn panel_empty_lines() {
        let theme = plain_theme();
        let output = panel(&theme, FrameStyle::Light, &[]);
        // Should still produce valid frame
        assert!(output.contains('\u{250C}'));
        assert!(output.contains('\u{2518}'));
    }

    // -- Horizontal rule ----------------------------------------------------

    #[test]
    fn horizontal_rule_heavy() {
        let theme = plain_theme();
        let hr = horizontal_rule(&theme, FrameStyle::Heavy, 10);
        assert_eq!(hr.chars().filter(|&c| c == '\u{2501}').count(), 10);
    }

    #[test]
    fn horizontal_rule_light() {
        let theme = plain_theme();
        let hr = horizontal_rule(&theme, FrameStyle::Light, 5);
        assert_eq!(hr.chars().filter(|&c| c == '\u{2500}').count(), 5);
    }

    #[test]
    fn horizontal_rule_zero_width() {
        let theme = plain_theme();
        let hr = horizontal_rule(&theme, FrameStyle::Light, 0);
        assert!(hr.is_empty() || hr.chars().all(|c| c == '\u{2500}'));
    }

    // -- Header bar ---------------------------------------------------------

    #[test]
    fn header_bar_contains_title() {
        let theme = plain_theme();
        let bar = header_bar(&theme, FrameStyle::Heavy, "POSITIONS", 40);
        assert!(bar.contains("POSITIONS"));
        assert!(bar.contains('\u{250F}')); // ┏
        assert!(bar.contains('\u{2513}')); // ┓
    }

    #[test]
    fn header_bar_light() {
        let theme = plain_theme();
        let bar = header_bar(&theme, FrameStyle::Light, "INFO", 30);
        assert!(bar.contains("INFO"));
    }

    // -- Key-value panel ----------------------------------------------------

    #[test]
    fn kv_panel_renders_pairs() {
        let theme = plain_theme();
        let pairs = vec![("Account", "alice.near"), ("Balance", "5.23 NEAR")];
        let output = kv_panel(&theme, FrameStyle::Light, &pairs);
        assert!(output.contains("Account"));
        assert!(output.contains("alice.near"));
        assert!(output.contains("Balance"));
        assert!(output.contains("5.23 NEAR"));
    }

    #[test]
    fn kv_panel_empty() {
        let theme = plain_theme();
        let output = kv_panel(&theme, FrameStyle::Light, &[]);
        // Should produce a valid (empty) frame
        assert!(output.contains('\u{250C}'));
    }

    // -- Helpers ------------------------------------------------------------

    #[test]
    fn repeat_char_works() {
        assert_eq!(repeat_char('x', 0), "");
        assert_eq!(repeat_char('x', 3), "xxx");
        assert_eq!(repeat_char('\u{2501}', 2), "\u{2501}\u{2501}");
    }
}
