//! Spinners, progress bars, and text-scramble reveal animations.
//!
//! Built on top of [`indicatif`] for cross-platform terminal animation support.
//! The binary spinner frames give the CLI a distinctive cypherpunk aesthetic
//! while the text scramble effect is used for dramatic reveal of important
//! values (transaction hashes, addresses, etc.).

use std::borrow::Cow;
use std::time::Duration;

use console::Style;
use indicatif::{ProgressBar, ProgressStyle};

use super::theme::{TemplarPalette, Theme};

// ---------------------------------------------------------------------------
// Binary spinner frames
// ---------------------------------------------------------------------------

/// Binary-themed spinner frames.
///
/// Each frame pairs a braille spinner character with a short binary string
/// to evoke the cypherpunk data-stream aesthetic.
pub const BINARY_SPINNER_FRAMES: &[&str] = &[
    "\u{280B} 01001",
    "\u{2819} 10110",
    "\u{2839} 01101",
    "\u{2838} 11001",
    "\u{283C} 00101",
    "\u{2834} 10010",
    "\u{2826} 01011",
    "\u{2827} 11010",
    "\u{2807} 00110",
    "\u{280F} 10101",
];

/// Interval between binary spinner frame updates.
pub const SPINNER_TICK_MS: u64 = 100;

// ---------------------------------------------------------------------------
// Spinner builder
// ---------------------------------------------------------------------------

/// Create a themed [`ProgressBar`] spinner with the binary frame set.
///
/// The returned spinner is already ticking. Call
/// [`ProgressBar::finish_with_message`] or [`ProgressBar::finish_and_clear`]
/// when the operation completes.
///
/// If `no_animation` is `true` a hidden (no-draw) progress bar is returned
/// so callers do not need to branch.
pub fn binary_spinner(theme: &Theme, message: &str, no_animation: bool) -> ProgressBar {
    if no_animation {
        let pb = ProgressBar::hidden();
        pb.set_message(message.to_string());
        return pb;
    }

    let pb = ProgressBar::new_spinner();

    let tick_strings: Vec<Cow<'static, str>> = BINARY_SPINNER_FRAMES
        .iter()
        .map(|s| Cow::Borrowed(*s))
        .collect();

    let style = ProgressStyle::default_spinner()
        .tick_strings(&tick_strings.iter().map(|c| c.as_ref()).collect::<Vec<_>>())
        .template("{spinner} {msg}")
        .expect("valid spinner template");

    pb.set_style(style);
    pb.set_message(format!(
        "{}",
        if theme.color_enabled {
            Style::new()
                .color256(TemplarPalette::IVORY)
                .apply_to(message)
                .to_string()
        } else {
            message.to_string()
        }
    ));
    pb.enable_steady_tick(Duration::from_millis(SPINNER_TICK_MS));
    pb
}

/// Finish a spinner with a themed success message.
pub fn finish_spinner_success(pb: &ProgressBar, theme: &Theme, message: &str) {
    pb.finish_with_message(theme.fmt_success(message));
}

/// Finish a spinner with a themed error message.
pub fn finish_spinner_error(pb: &ProgressBar, theme: &Theme, message: &str) {
    pb.finish_with_message(theme.fmt_error(message));
}

// ---------------------------------------------------------------------------
// Text scramble reveal
// ---------------------------------------------------------------------------

/// Characters used during the scramble phase before revealing real text.
const SCRAMBLE_CHARS: &[u8] = b"0123456789abcdef!@#$%^&*(){}[]|/<>";

/// Perform a text-scramble reveal animation for the given `text`.
///
/// Each character position starts as a random scramble glyph and is
/// progressively replaced by the real character, left-to-right. The
/// animation is purely visual (printed to stderr) and blocks for the
/// total duration.
///
/// When `no_animation` is `true` the text is printed immediately without
/// any scramble effect.
pub fn scramble_reveal(
    theme: &Theme,
    text: &str,
    total_duration: Duration,
    no_animation: bool,
) {
    let term = console::Term::stderr();

    if no_animation || text.is_empty() {
        let styled = if theme.color_enabled {
            format!(
                "{}",
                Style::new()
                    .color256(TemplarPalette::GOLD)
                    .apply_to(text)
            )
        } else {
            text.to_string()
        };
        let _ = term.write_line(&styled);
        return;
    }

    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let step_duration = total_duration / (len as u32).max(1);

    // Use a simple pseudo-random based on position to avoid pulling in rand at
    // display time (rand is still in Cargo.toml but we keep display lightweight).
    let scramble_char = |pos: usize, tick: usize| -> char {
        let idx = (pos.wrapping_mul(7) + tick.wrapping_mul(13)) % SCRAMBLE_CHARS.len();
        SCRAMBLE_CHARS[idx] as char
    };

    for reveal_up_to in 0..=len {
        let mut line = String::with_capacity(len);
        for (i, &ch) in chars.iter().enumerate() {
            if i < reveal_up_to {
                line.push(ch);
            } else {
                line.push(scramble_char(i, reveal_up_to));
            }
        }

        let styled = if theme.color_enabled {
            format!(
                "{}",
                Style::new()
                    .color256(TemplarPalette::GOLD)
                    .apply_to(&line)
            )
        } else {
            line.clone()
        };

        let _ = term.clear_line();
        let _ = term.write_str(&format!("\r{styled}"));
        std::thread::sleep(step_duration);
    }

    let _ = term.write_line("");
}

// ---------------------------------------------------------------------------
// Progress bar helpers
// ---------------------------------------------------------------------------

/// Create a themed determinate progress bar with the given total.
///
/// Suitable for operations where the total number of steps is known
/// (e.g., downloading bytes, processing batches).
pub fn progress_bar(theme: &Theme, total: u64, message: &str, no_animation: bool) -> ProgressBar {
    if no_animation {
        let pb = ProgressBar::hidden();
        pb.set_length(total);
        pb.set_message(message.to_string());
        return pb;
    }

    let pb = ProgressBar::new(total);

    let template = if theme.color_enabled {
        "{msg} [{bar:40.178/144}] {pos}/{len} ({eta})"
    } else {
        "{msg} [{bar:40}] {pos}/{len} ({eta})"
    };

    let style = ProgressStyle::default_bar()
        .template(template)
        .expect("valid bar template")
        .progress_chars("\u{2593}\u{2592}\u{2591}"); // ▓▒░

    pb.set_style(style);
    pb.set_message(message.to_string());
    pb
}

/// Create a themed byte-download progress bar.
///
/// Shows bytes transferred and transfer rate.
pub fn download_bar(theme: &Theme, total_bytes: u64, no_animation: bool) -> ProgressBar {
    if no_animation {
        let pb = ProgressBar::hidden();
        pb.set_length(total_bytes);
        return pb;
    }

    let pb = ProgressBar::new(total_bytes);

    let template = if theme.color_enabled {
        "{msg} [{bar:40.178/144}] {bytes}/{total_bytes} ({bytes_per_sec})"
    } else {
        "{msg} [{bar:40}] {bytes}/{total_bytes} ({bytes_per_sec})"
    };

    let style = ProgressStyle::default_bar()
        .template(template)
        .expect("valid download bar template")
        .progress_chars("\u{2593}\u{2592}\u{2591}");

    pb.set_style(style);
    pb
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

    #[test]
    fn binary_spinner_frames_count() {
        assert_eq!(BINARY_SPINNER_FRAMES.len(), 10);
    }

    #[test]
    fn binary_spinner_frames_contain_braille() {
        for frame in BINARY_SPINNER_FRAMES {
            let first_char = frame.chars().next().unwrap();
            // Braille pattern range: U+2800..U+28FF
            assert!(
                ('\u{2800}'..='\u{28FF}').contains(&first_char),
                "frame does not start with braille: {frame}",
            );
        }
    }

    #[test]
    fn binary_spinner_frames_contain_binary_digits() {
        for frame in BINARY_SPINNER_FRAMES {
            // After the braille char and space, there should be 5 binary digits
            let binary_part: String = frame.chars().skip(2).collect();
            assert_eq!(binary_part.len(), 5, "unexpected binary length in: {frame}");
            assert!(
                binary_part.chars().all(|c| c == '0' || c == '1'),
                "non-binary char in: {frame}",
            );
        }
    }

    #[test]
    fn binary_spinner_hidden_when_no_animation() {
        let theme = plain_theme();
        let pb = binary_spinner(&theme, "test", true);
        // Hidden spinners are created but do not draw
        assert!(pb.is_hidden());
    }

    #[test]
    fn binary_spinner_created_when_animated() {
        let theme = plain_theme();
        let pb = binary_spinner(&theme, "working...", false);
        // In environments with a TTY the bar is visible;
        // in headless CI the bar may be hidden, so we only
        // verify creation does not panic.
        pb.finish_and_clear();
    }

    #[test]
    fn finish_spinner_success_does_not_panic() {
        let theme = plain_theme();
        let pb = binary_spinner(&theme, "test", true);
        finish_spinner_success(&pb, &theme, "done");
    }

    #[test]
    fn finish_spinner_error_does_not_panic() {
        let theme = plain_theme();
        let pb = binary_spinner(&theme, "test", true);
        finish_spinner_error(&pb, &theme, "failed");
    }

    #[test]
    fn scramble_reveal_no_animation() {
        let theme = plain_theme();
        // Should print immediately without blocking
        scramble_reveal(&theme, "hello", Duration::from_millis(10), true);
    }

    #[test]
    fn scramble_reveal_empty_string() {
        let theme = plain_theme();
        scramble_reveal(&theme, "", Duration::from_millis(10), true);
    }

    #[test]
    fn progress_bar_hidden_when_no_animation() {
        let theme = plain_theme();
        let pb = progress_bar(&theme, 100, "loading", true);
        assert!(pb.is_hidden());
    }

    #[test]
    fn progress_bar_created_when_animated() {
        let theme = plain_theme();
        let pb = progress_bar(&theme, 100, "loading", false);
        pb.finish_and_clear();
    }

    #[test]
    fn download_bar_hidden_when_no_animation() {
        let theme = plain_theme();
        let pb = download_bar(&theme, 1024, true);
        assert!(pb.is_hidden());
    }

    #[test]
    fn download_bar_created_when_animated() {
        let theme = plain_theme();
        let pb = download_bar(&theme, 1024, false);
        pb.finish_and_clear();
    }

    #[test]
    fn spinner_tick_ms_is_reasonable() {
        assert!(SPINNER_TICK_MS >= 50);
        assert!(SPINNER_TICK_MS <= 500);
    }

    #[test]
    fn scramble_chars_are_ascii() {
        for &ch in SCRAMBLE_CHARS {
            assert!(ch.is_ascii(), "non-ASCII scramble char: {ch}");
        }
    }
}
