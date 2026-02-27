//! Startup banner for the Templar CLI.
//!
//! The [`print_banner`] function renders either the full Templar mark ASCII art
//! or a compact single-line fallback depending on terminal width, and performs
//! first-run detection to show a welcome message on initial invocation.

use console::Term;

use super::logo::{COMPACT_BANNER, FULL_MARK_MIN_WIDTH, TEMPLAR_MARK};
use super::theme::Theme;

/// Marker file written to `~/.templar/` after the first run.
const FIRST_RUN_MARKER: &str = ".first_run_done";

/// Global CLI options that influence banner display.
///
/// This mirrors the fields from the top-level `GlobalOpts` struct so that
/// the display module does not depend on the commands module.
pub struct BannerOpts {
    /// Suppress all non-essential output.
    pub quiet: bool,
    /// Colour mode override (`always`, `never`, `auto`, or `None`).
    pub color: Option<String>,
    /// Output format override (e.g., `json`).
    pub output: Option<String>,
    /// Explicitly disable the banner.
    pub no_banner: bool,
    /// Disable animations (scramble reveal, spinner).
    pub no_animation: bool,
}

/// Print the startup banner to stderr.
///
/// The banner is printed to **stderr** so that it does not interfere with
/// piped / machine-readable output on stdout.
///
/// # Behaviour
///
/// | Condition | Result |
/// |-----------|--------|
/// | `quiet` or `no_banner` is true | no output |
/// | `output` is `Some("json")` | no output |
/// | Terminal width >= [`FULL_MARK_MIN_WIDTH`] | full Templar mark |
/// | Terminal width < [`FULL_MARK_MIN_WIDTH`] | compact one-liner |
/// | First run detected | additional welcome message |
pub fn print_banner(opts: &BannerOpts) {
    // Skip banner entirely for quiet / json / explicit opt-out
    if opts.quiet || opts.no_banner {
        return;
    }
    if let Some(ref fmt) = opts.output {
        if fmt.eq_ignore_ascii_case("json") {
            return;
        }
    }

    let theme = Theme::resolve(opts.color.as_deref(), None);
    let term = Term::stderr();

    let width = terminal_width(&term);

    if width >= FULL_MARK_MIN_WIDTH {
        print_full_banner(&theme, &term);
    } else {
        print_compact_banner(&theme, &term);
    }

    if is_first_run() {
        print_welcome(&theme, &term);
        mark_first_run_done();
    }
}

/// Print the full-size ASCII art banner.
fn print_full_banner(theme: &Theme, term: &Term) {
    for line in TEMPLAR_MARK.lines() {
        let _ = term.write_line(&format!("{}", theme.gold.apply_to(line)));
    }
    let _ = term.write_line("");
}

/// Print the compact single-line banner.
fn print_compact_banner(theme: &Theme, term: &Term) {
    let _ = term.write_line(&format!("{}", theme.gold.apply_to(COMPACT_BANNER)));
    let _ = term.write_line("");
}

/// Print a first-run welcome message.
fn print_welcome(theme: &Theme, term: &Term) {
    let _ = term.write_line(&format!(
        "{}",
        theme.ivory.apply_to("  Welcome to Templar CLI. Run `templar init` to get started.")
    ));
    let _ = term.write_line(&format!(
        "{}",
        theme
            .warm_grey
            .apply_to("  Documentation: https://templar.finance/docs")
    ));
    let _ = term.write_line("");
}

/// Determine the current terminal width, falling back to 80.
fn terminal_width(term: &Term) -> u16 {
    term.size_checked()
        .map(|(_, w)| w)
        .unwrap_or(80)
}

/// Returns `true` if this appears to be the first invocation (no marker file).
fn is_first_run() -> bool {
    marker_path().map_or(true, |p| !p.exists())
}

/// Record that the first run has completed.
fn mark_first_run_done() {
    if let Some(path) = marker_path() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, "1");
    }
}

/// Resolve the path to the first-run marker file.
fn marker_path() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|h| h.join(".templar").join(FIRST_RUN_MARKER))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_opts() -> BannerOpts {
        BannerOpts {
            quiet: false,
            color: Some("never".into()),
            output: None,
            no_banner: false,
            no_animation: true,
        }
    }

    #[test]
    fn quiet_suppresses_banner() {
        let mut opts = test_opts();
        opts.quiet = true;
        // Should return without output (no panic)
        print_banner(&opts);
    }

    #[test]
    fn no_banner_suppresses_banner() {
        let mut opts = test_opts();
        opts.no_banner = true;
        print_banner(&opts);
    }

    #[test]
    fn json_output_suppresses_banner() {
        let mut opts = test_opts();
        opts.output = Some("json".into());
        print_banner(&opts);
    }

    #[test]
    fn json_output_case_insensitive() {
        let mut opts = test_opts();
        opts.output = Some("JSON".into());
        print_banner(&opts);
    }

    #[test]
    fn print_full_banner_does_not_panic() {
        let theme = Theme::resolve(Some("never"), None);
        let term = Term::stderr();
        print_full_banner(&theme, &term);
    }

    #[test]
    fn print_compact_banner_does_not_panic() {
        let theme = Theme::resolve(Some("never"), None);
        let term = Term::stderr();
        print_compact_banner(&theme, &term);
    }

    #[test]
    fn terminal_width_returns_nonzero() {
        let term = Term::stderr();
        let w = terminal_width(&term);
        assert!(w > 0);
    }

    #[test]
    fn marker_path_is_under_templar_dir() {
        if let Some(path) = marker_path() {
            let path_str = path.to_string_lossy();
            assert!(path_str.contains(".templar"));
            assert!(path_str.contains(FIRST_RUN_MARKER));
        }
    }

    #[test]
    fn banner_opts_fields() {
        let opts = test_opts();
        assert!(!opts.quiet);
        assert!(!opts.no_banner);
        assert!(opts.no_animation);
        assert!(opts.color.is_some());
        assert!(opts.output.is_none());
    }
}
