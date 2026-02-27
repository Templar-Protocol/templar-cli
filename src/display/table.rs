//! Themed table rendering for terminal output.
//!
//! Provides a simple column-based table renderer that respects the active
//! [`Theme`] and supports both heavy and light [`FrameStyle`] borders.
//!
//! Tables are built row-by-row and rendered as a single `String` ready for
//! printing to stderr or stdout.

use std::fmt::Write as FmtWrite;

use unicode_width::UnicodeWidthStr;

use super::frame::FrameStyle;
use super::theme::Theme;

// ---------------------------------------------------------------------------
// Column alignment
// ---------------------------------------------------------------------------

/// Horizontal alignment for a table column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    /// Left-aligned (default).
    Left,
    /// Right-aligned (e.g., numeric columns).
    Right,
    /// Centre-aligned (e.g., status indicators).
    Center,
}

impl Default for Align {
    fn default() -> Self {
        Self::Left
    }
}

// ---------------------------------------------------------------------------
// Column definition
// ---------------------------------------------------------------------------

/// Definition of a single table column.
#[derive(Debug, Clone)]
pub struct Column {
    /// Column header text.
    pub header: String,
    /// Horizontal alignment of both header and data cells.
    pub align: Align,
    /// Optional minimum width override (0 = auto-size).
    pub min_width: usize,
}

impl Column {
    /// Create a new left-aligned column with the given header.
    pub fn new(header: &str) -> Self {
        Self {
            header: header.to_string(),
            align: Align::Left,
            min_width: 0,
        }
    }

    /// Set the alignment (builder pattern).
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Set the minimum width (builder pattern).
    pub fn min_width(mut self, w: usize) -> Self {
        self.min_width = w;
        self
    }
}

// ---------------------------------------------------------------------------
// Table
// ---------------------------------------------------------------------------

/// A themed table ready for rendering.
///
/// # Example
///
/// ```rust,no_run
/// use templar_cli::display::table::{Table, Column, Align};
/// use templar_cli::display::frame::FrameStyle;
/// use templar_cli::display::theme::Theme;
///
/// let theme = Theme::resolve(Some("never"), None);
/// let mut table = Table::new(
///     &theme,
///     FrameStyle::Light,
///     vec![
///         Column::new("Asset"),
///         Column::new("Balance").align(Align::Right),
///     ],
/// );
/// table.add_row(vec!["NEAR".into(), "12.50".into()]);
/// table.add_row(vec!["USDC".into(), "1,250.00".into()]);
/// let output = table.render();
/// println!("{output}");
/// ```
pub struct Table<'t> {
    /// Reference to the active theme.
    theme: &'t Theme,
    /// Frame weight.
    style: FrameStyle,
    /// Column definitions.
    columns: Vec<Column>,
    /// Row data (each row is a `Vec<String>` matching `columns` length).
    rows: Vec<Vec<String>>,
}

impl<'t> Table<'t> {
    /// Create a new table with the given columns.
    pub fn new(theme: &'t Theme, style: FrameStyle, columns: Vec<Column>) -> Self {
        Self {
            theme,
            style,
            columns,
            rows: Vec::new(),
        }
    }

    /// Add a data row. The vector length should match the number of columns.
    ///
    /// Extra cells are silently ignored; missing cells are treated as empty.
    pub fn add_row(&mut self, row: Vec<String>) {
        self.rows.push(row);
    }

    /// Render the table to a `String`.
    pub fn render(&self) -> String {
        let col_widths = self.compute_widths();
        let fc = self.style.chars();
        let total_inner = self.total_inner_width(&col_widths);

        let mut out = String::new();

        // Top border
        let _ = writeln!(
            out,
            "{}",
            self.theme.gold.apply_to(format!(
                "{}{}{}",
                fc.tl, repeat_char(fc.h, total_inner), fc.tr,
            ))
        );

        // Header row
        let _ = writeln!(out, "{}", self.render_row_cells(&col_widths, &self.header_strings(), true));

        // Separator
        let _ = writeln!(
            out,
            "{}",
            self.theme.gold.apply_to(format!(
                "{}{}{}",
                fc.tl, repeat_char(fc.h, total_inner), fc.tr,
            ))
        );

        // Data rows
        for row in &self.rows {
            let _ = writeln!(out, "{}", self.render_row_cells(&col_widths, row, false));
        }

        // Bottom border
        let _ = write!(
            out,
            "{}",
            self.theme.gold.apply_to(format!(
                "{}{}{}",
                fc.bl, repeat_char(fc.h, total_inner), fc.br,
            ))
        );

        out
    }

    /// Compute the display width for each column.
    fn compute_widths(&self) -> Vec<usize> {
        self.columns
            .iter()
            .enumerate()
            .map(|(i, col)| {
                let header_w = UnicodeWidthStr::width(col.header.as_str());
                let data_max = self
                    .rows
                    .iter()
                    .map(|row| {
                        row.get(i)
                            .map(|c| UnicodeWidthStr::width(c.as_str()))
                            .unwrap_or(0)
                    })
                    .max()
                    .unwrap_or(0);
                header_w.max(data_max).max(col.min_width)
            })
            .collect()
    }

    /// Total inner width = sum of column widths + padding + separators.
    fn total_inner_width(&self, widths: &[usize]) -> usize {
        if widths.is_empty() {
            return 0;
        }
        // Each column contributes (w + 2) for padding, plus (n-1) vertical separators
        let n = widths.len();
        widths.iter().sum::<usize>() + 2 * n + n.saturating_sub(1)
    }

    /// Extract header strings.
    fn header_strings(&self) -> Vec<String> {
        self.columns.iter().map(|c| c.header.clone()).collect()
    }

    /// Render a single row of cells.
    fn render_row_cells(&self, widths: &[usize], cells: &[String], is_header: bool) -> String {
        let fc = self.style.chars();
        let mut parts: Vec<String> = Vec::new();

        for (i, col) in self.columns.iter().enumerate() {
            let cell_text = cells.get(i).map(String::as_str).unwrap_or("");
            let w = widths[i];
            let padded = align_text(cell_text, w, col.align);
            parts.push(format!(" {} ", padded));
        }

        let content = parts.join(&fc.v.to_string());
        let v = fc.v;

        if is_header {
            format!(
                "{}{}{}",
                self.theme.gold.apply_to(v.to_string()),
                self.theme.heading.apply_to(&content),
                self.theme.gold.apply_to(v.to_string()),
            )
        } else {
            format!(
                "{}{}{}",
                self.theme.gold.apply_to(v.to_string()),
                self.theme.ivory.apply_to(&content),
                self.theme.gold.apply_to(v.to_string()),
            )
        }
    }
}

// ---------------------------------------------------------------------------
// Text alignment helper
// ---------------------------------------------------------------------------

/// Pad `text` to `width` display columns using the given alignment.
fn align_text(text: &str, width: usize, align: Align) -> String {
    let display_w = UnicodeWidthStr::width(text);
    if display_w >= width {
        return text.to_string();
    }
    let pad = width - display_w;
    match align {
        Align::Left => format!("{}{}", text, " ".repeat(pad)),
        Align::Right => format!("{}{}", " ".repeat(pad), text),
        Align::Center => {
            let left = pad / 2;
            let right = pad - left;
            format!("{}{}{}", " ".repeat(left), text, " ".repeat(right))
        }
    }
}

/// Repeat a character `n` times.
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

    // -- Align --------------------------------------------------------------

    #[test]
    fn align_default_is_left() {
        assert_eq!(Align::default(), Align::Left);
    }

    // -- Column -------------------------------------------------------------

    #[test]
    fn column_new() {
        let col = Column::new("Name");
        assert_eq!(col.header, "Name");
        assert_eq!(col.align, Align::Left);
        assert_eq!(col.min_width, 0);
    }

    #[test]
    fn column_builder() {
        let col = Column::new("Amount").align(Align::Right).min_width(12);
        assert_eq!(col.align, Align::Right);
        assert_eq!(col.min_width, 12);
    }

    // -- Table rendering ----------------------------------------------------

    #[test]
    fn table_renders_header_and_rows() {
        let theme = plain_theme();
        let mut table = Table::new(
            &theme,
            FrameStyle::Light,
            vec![Column::new("Name"), Column::new("Value")],
        );
        table.add_row(vec!["foo".into(), "123".into()]);
        table.add_row(vec!["bar".into(), "456".into()]);

        let output = table.render();
        assert!(output.contains("Name"));
        assert!(output.contains("Value"));
        assert!(output.contains("foo"));
        assert!(output.contains("123"));
        assert!(output.contains("bar"));
        assert!(output.contains("456"));
    }

    #[test]
    fn table_empty_rows() {
        let theme = plain_theme();
        let table = Table::new(
            &theme,
            FrameStyle::Heavy,
            vec![Column::new("Col1")],
        );
        let output = table.render();
        assert!(output.contains("Col1"));
    }

    #[test]
    fn table_missing_cells_treated_as_empty() {
        let theme = plain_theme();
        let mut table = Table::new(
            &theme,
            FrameStyle::Light,
            vec![Column::new("A"), Column::new("B"), Column::new("C")],
        );
        table.add_row(vec!["only-one".into()]);
        let output = table.render();
        assert!(output.contains("only-one"));
    }

    #[test]
    fn table_right_aligned_column() {
        let theme = plain_theme();
        let mut table = Table::new(
            &theme,
            FrameStyle::Light,
            vec![
                Column::new("Label"),
                Column::new("Num").align(Align::Right),
            ],
        );
        table.add_row(vec!["x".into(), "9".into()]);
        let output = table.render();
        assert!(output.contains("x"));
        assert!(output.contains("9"));
    }

    #[test]
    fn table_center_aligned_column() {
        let theme = plain_theme();
        let mut table = Table::new(
            &theme,
            FrameStyle::Light,
            vec![Column::new("Status").align(Align::Center)],
        );
        table.add_row(vec!["OK".into()]);
        let output = table.render();
        assert!(output.contains("OK"));
    }

    // -- align_text ---------------------------------------------------------

    #[test]
    fn align_text_left() {
        assert_eq!(align_text("hi", 5, Align::Left), "hi   ");
    }

    #[test]
    fn align_text_right() {
        assert_eq!(align_text("hi", 5, Align::Right), "   hi");
    }

    #[test]
    fn align_text_center() {
        assert_eq!(align_text("hi", 5, Align::Center), " hi  ");
    }

    #[test]
    fn align_text_exact_width() {
        assert_eq!(align_text("hello", 5, Align::Left), "hello");
    }

    #[test]
    fn align_text_overflow() {
        assert_eq!(align_text("toolong", 3, Align::Left), "toolong");
    }

    // -- FrameStyle chars via table -----------------------------------------

    #[test]
    fn frame_chars_heavy_accessible() {
        let fc = FrameStyle::Heavy.chars();
        assert_eq!(fc.tl, '\u{250F}');
        assert_eq!(fc.tr, '\u{2513}');
        assert_eq!(fc.bl, '\u{2517}');
        assert_eq!(fc.br, '\u{251B}');
        assert_eq!(fc.h, '\u{2501}');
        assert_eq!(fc.v, '\u{2503}');
    }

    #[test]
    fn frame_chars_light_accessible() {
        let fc = FrameStyle::Light.chars();
        assert_eq!(fc.tl, '\u{250C}');
        assert_eq!(fc.tr, '\u{2510}');
        assert_eq!(fc.bl, '\u{2514}');
        assert_eq!(fc.br, '\u{2518}');
        assert_eq!(fc.h, '\u{2500}');
        assert_eq!(fc.v, '\u{2502}');
    }
}
