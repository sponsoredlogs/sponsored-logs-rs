//! Brand-safe gilding: the gold `[AD]` standard.
//!
//! Gold is the color of money, and money is the color of your log stream. When
//! an impression lands in a live terminal, the `[AD]` tag is gilded in premium
//! 256-color gold, turning a plain tag into a high-visibility trust signal at
//! the moment of peak incident attention. The escape codes are zero-width, so
//! the gilding costs your layout nothing: banner borders stay pixel-aligned.
//!
//! Gilding is brand-safe by default: the gold only ships to a real interactive
//! terminal (a TTY) with color enabled, never to files, pipes, or non-interactive
//! sinks, which continue to receive the byte-identical plain line. We also honor
//! the [`NO_COLOR`](https://no-color.org) convention. Consent is our moat.

use std::io::IsTerminal;

/// SGR 256-color gold (xterm 214). The `[AD]` tag is always the money color.
const GOLD: &str = "\x1b[38;5;214m";
/// The universal SGR reset.
const RESET: &str = "\x1b[0m";

/// When the exchange gilds the `[AD]` tag.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Color {
    /// Gild only on a real TTY when `NO_COLOR` is unset. The safe default.
    #[default]
    Auto,
    /// Force gold on every surface, overriding `NO_COLOR`. Maximum salience.
    Always,
    /// Never gild. Plain tag everywhere, even on a premium terminal.
    Never,
}

impl Color {
    /// Parse a raw env string. Unrecognized values settle to `Auto`, because an
    /// invalid setting must never force color onto a sink that cannot render it.
    pub fn from_env_str(value: &str) -> Color {
        match value.trim().to_ascii_lowercase().as_str() {
            "always" => Color::Always,
            "never" => Color::Never,
            _ => Color::Auto,
        }
    }
}

/// Gild `text` in gold when `enabled`, otherwise hand it back untouched so the
/// plain path stays byte-identical to the classic line.
pub fn colorize(text: &str, enabled: bool) -> String {
    if enabled {
        format!("{GOLD}{text}{RESET}")
    } else {
        text.to_string()
    }
}

/// Decide whether an emission should be gilded, reading the real environment and
/// stdout. `Auto` gilds only when `NO_COLOR` is unset and stdout is a TTY, our
/// best-effort proxy since most `fmt` subscribers write to stdout.
pub fn gild(mode: Color) -> bool {
    gild_with(
        mode,
        no_color_unset(|k| std::env::var_os(k)),
        std::io::stdout().is_terminal(),
    )
}

/// [`gild`], but against caller-supplied signals. Testable without touching the
/// real environment or terminal.
pub fn gild_with(mode: Color, no_color_unset: bool, is_tty: bool) -> bool {
    match mode {
        Color::Never => false,
        Color::Always => true,
        Color::Auto => no_color_unset && is_tty,
    }
}

/// `NO_COLOR` convention: any non-empty value disables color. Unset or empty
/// leaves auto-gilding available. Reads through the supplied getter for testing.
fn no_color_unset(get: impl Fn(&str) -> Option<std::ffi::OsString>) -> bool {
    match get("NO_COLOR") {
        Some(v) => v.is_empty(),
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    #[test]
    fn colorize_wraps_in_gold_when_enabled() {
        let out = colorize("[AD]", true);
        assert!(out.starts_with(GOLD));
        assert!(out.ends_with(RESET));
        assert!(out.contains("[AD]"));
    }

    #[test]
    fn colorize_is_identity_when_disabled() {
        assert_eq!(colorize("[AD]", false), "[AD]");
    }

    #[test]
    fn never_never_gilds() {
        assert!(!gild_with(Color::Never, true, true));
    }

    #[test]
    fn always_always_gilds_even_off_tty_or_with_no_color() {
        assert!(gild_with(Color::Always, false, false));
    }

    #[test]
    fn auto_gilds_only_on_tty_with_no_color_unset() {
        assert!(gild_with(Color::Auto, true, true));
        assert!(!gild_with(Color::Auto, true, false), "not a tty");
        assert!(!gild_with(Color::Auto, false, true), "NO_COLOR set");
    }

    #[test]
    fn no_color_unset_honors_the_convention() {
        assert!(no_color_unset(|_| None), "unset leaves auto available");
        assert!(
            no_color_unset(|_| Some(OsString::from(""))),
            "empty is treated as unset"
        );
        assert!(
            !no_color_unset(|_| Some(OsString::from("1"))),
            "any non-empty value disables color"
        );
    }

    #[test]
    fn from_env_str_parses_modes() {
        assert_eq!(Color::from_env_str("always"), Color::Always);
        assert_eq!(Color::from_env_str(" NEVER "), Color::Never);
        assert_eq!(Color::from_env_str("auto"), Color::Auto);
        assert_eq!(
            Color::from_env_str("banana"),
            Color::Auto,
            "invalid settles to auto"
        );
    }
}
