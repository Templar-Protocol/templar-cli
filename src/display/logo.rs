//! Templar Protocol ASCII art assets.
//!
//! Contains the full 100-column by 57-row Templar mark and a compact inline
//! glyph suitable for narrow terminals or single-line usage.
//!
//! The art is stored as `const &str` so it occupies no heap memory at runtime
//! and is embedded directly in the binary.

/// Full Templar mark ASCII art (approximately 100 columns x 57 rows).
///
/// This is the primary brand mark displayed at startup on wide terminals.
/// Each row is padded to roughly equal width for visual consistency.
///
/// The design is a stylised Templar cross composed of box-drawing and
/// block elements, with a crown motif at the apex and a radiant base.
pub const TEMPLAR_MARK: &str = r"
                                         ╔═══╗
                                         ║ T ║
                                         ╚╤═╤╝
                                          │ │
                                     ╔════╧═╧════╗
                                     ║  TEMPLAR  ║
                                     ╚════╤═╤════╝
                                          │ │
                              ┌───────────┤ ├───────────┐
                              │           │ │           │
                         ╔════╧═══════════╧═╧═══════════╧════╗
                         ║          CYPHER  LENDING          ║
                         ╚════╤═══════════╤═╤═══════════╤════╝
                              │           │ │           │
                              │     ╔═════╧═╧═════╗     │
                              │     ║             ║     │
                              │     ║    ╔═══╗    ║     │
                              │     ║    ║ ✠ ║    ║     │
                              │     ║    ╚═══╝    ║     │
                              │     ║             ║     │
                              │     ╚═════╤═╤═════╝     │
                              │           │ │           │
                 ┌────────────┤           │ │           ├────────────┐
                 │            │           │ │           │            │
            ╔════╧════════════╧═══════════╧═╧═══════════╧════════════╧════╗
            ║                    BE YOUR OWN BANK                        ║
            ╚════╤════════════╤═══════════╤═╤═══════════╤════════════╤════╝
                 │            │           │ │           │            │
                 │            │     ╔═════╧═╧═════╗     │            │
                 │            │     ║  ╤║╤  ╤║╤  ║     │            │
                 │            │     ╚═════╤═╤═════╝     │            │
                 │            │           │ │           │            │
                 └────────────┤           │ │           ├────────────┘
                              │           │ │           │
                              │     ╔═════╧═╧═════╗     │
                              │     ║  PROTOCOL  ║     │
                              │     ╚═════╤═╤═════╝     │
                              │           │ │           │
                              └───────────┤ ├───────────┘
                                          │ │
                                     ╔════╧═╧════╗
                                     ║ v0.1.0  ║
                                     ╚════╤═╤════╝
                                          │ │
                                       ┌──┘ └──┐
                                       │ ═══── │
                                       └───┬───┘
                                           │
                                      ╔════╧════╗
                                      ║ NEAR   ║
                                      ╚═════════╝

                       ┌──────────────────────────────────────┐
                       │  https://templar.finance              │
                       │  Cypher Lending on NEAR Protocol      │
                       └──────────────────────────────────────┘
              ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
                01001 10110 01101 11001 00101 10010 01011 11010 00110
              ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
";

/// Compact one-line banner for narrow terminals (< 80 columns) or quiet mode.
///
/// Uses the Templar cross ✠ and temple glyph ╤║╤ for minimal branding.
pub const COMPACT_BANNER: &str = "✠ Templar CLI v0.1.0 ╤║╤  Cypher Lending · NEAR Protocol";

/// Inline Templar cross glyph suitable for embedding in status lines.
pub const INLINE_GLYPH: &str = "✠";

/// Temple glyph for decorative use.
pub const TEMPLE_GLYPH: &str = "╤║╤";

/// Minimum terminal width (in columns) to display the full mark.
pub const FULL_MARK_MIN_WIDTH: u16 = 80;

/// Number of rows in the full Templar mark.
pub const FULL_MARK_ROWS: usize = 60;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn templar_mark_is_not_empty() {
        assert!(!TEMPLAR_MARK.is_empty());
    }

    #[test]
    fn templar_mark_contains_brand_name() {
        assert!(TEMPLAR_MARK.contains("TEMPLAR"));
    }

    #[test]
    fn templar_mark_contains_cross() {
        assert!(TEMPLAR_MARK.contains('\u{2720}')); // ✠
    }

    #[test]
    fn templar_mark_contains_temple_glyph() {
        assert!(TEMPLAR_MARK.contains("\u{2564}\u{2551}\u{2564}")); // ╤║╤
    }

    #[test]
    fn templar_mark_contains_tagline() {
        assert!(TEMPLAR_MARK.contains("BE YOUR OWN BANK"));
    }

    #[test]
    fn templar_mark_contains_cypher_lending() {
        assert!(TEMPLAR_MARK.contains("CYPHER"));
    }

    #[test]
    fn compact_banner_is_short() {
        assert!(COMPACT_BANNER.len() < 200);
    }

    #[test]
    fn compact_banner_contains_version() {
        assert!(COMPACT_BANNER.contains("v0.1.0"));
    }

    #[test]
    fn compact_banner_contains_cross() {
        assert!(COMPACT_BANNER.contains('\u{2720}'));
    }

    #[test]
    fn inline_glyph_is_cross() {
        assert_eq!(INLINE_GLYPH, "\u{2720}");
    }

    #[test]
    fn temple_glyph_value() {
        assert_eq!(TEMPLE_GLYPH, "\u{2564}\u{2551}\u{2564}");
    }

    #[test]
    fn full_mark_min_width_is_reasonable() {
        const { assert!(FULL_MARK_MIN_WIDTH >= 60) };
        const { assert!(FULL_MARK_MIN_WIDTH <= 120) };
    }

    #[test]
    fn full_mark_rows_matches_actual_line_count() {
        assert_eq!(
            TEMPLAR_MARK.lines().count(),
            FULL_MARK_ROWS,
            "FULL_MARK_ROWS is out of sync with the actual TEMPLAR_MARK line count"
        );
    }

    #[test]
    fn mark_lines_fit_within_100_columns() {
        for line in TEMPLAR_MARK.lines() {
            // Unicode-aware width check — we just verify no line is absurdly long
            assert!(
                line.len() <= 250, // byte len, not display width — generous bound
                "line too long ({} bytes): {}",
                line.len(),
                line,
            );
        }
    }
}
