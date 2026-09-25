//! Premium box-drawn ad inventory: the multi-line banner placement.
//!
//! Turns a single ad line into above-the-fold, framed real estate in your log
//! stream. The frame assumes standard-width, single-cell Latin creative and
//! lays out the right border by character count. Emoji, CJK, or combining marks
//! render wider than one cell and can nudge the border off its column, a known
//! trade-off of premium placement. For hostile sinks, `ascii_only` renders the
//! portable `+`/`-`/`|` glyph set for 100% viewability.

use crate::advertisers::Box;

/// Body width, in columns, of a banner placement.
pub const WIDTH: usize = 60;

/// Glyph set for a frame: [top_left, top_right, bottom_left, bottom_right,
/// horizontal, vertical].
fn glyphs(tier: Box, ascii_only: bool) -> [&'static str; 6] {
    if ascii_only {
        return ["+", "+", "+", "+", "-", "|"];
    }
    match tier {
        Box::Light => ["┌", "┐", "└", "┘", "─", "│"],
        Box::Heavy => ["┏", "┓", "┗", "┛", "━", "┃"],
        Box::Double => ["╔", "╗", "╚", "╝", "═", "║"],
    }
}

/// Draw the frame for one creative. `ascii_only` overrides whatever impact tier
/// was purchased with the plain fallback set. The prefix is promoted into the
/// top border as a masthead; a blank prefix collapses to a solid rule. When
/// `gild` is set, the masthead prefix is wrapped in zero-width gold, computed
/// after the fill math so the border stays pixel-aligned.
pub fn render(text: &str, prefix: &str, tier: Box, ascii_only: bool, gild: bool) -> String {
    let [top, top_r, bot, bot_r, horiz, vert] = glyphs(tier, ascii_only);
    let span = WIDTH + 2;

    let mut out = String::new();
    out.push_str(&top_border(prefix, top, top_r, horiz, span, gild));
    for line in wrap_text(text, WIDTH) {
        out.push('\n');
        out.push_str(&format!("{vert} {:<width$} {vert}", line, width = WIDTH));
    }
    out.push('\n');
    out.push_str(&format!("{bot}{}{bot_r}", horiz.repeat(span)));
    out
}

/// Top border with the prefix embedded as `<h> [AD] <h-fill>`. A blank prefix
/// collapses to a solid rule with no gap and no tag. The fill count is computed
/// against the PLAIN prefix, then the gilded tag is swapped in, since ANSI
/// escapes are zero-width and must not shift the border count.
fn top_border(
    prefix: &str,
    corner: &str,
    corner_r: &str,
    horiz: &str,
    span: usize,
    gild: bool,
) -> String {
    if prefix.is_empty() {
        return format!("{corner}{}{corner_r}", horiz.repeat(span));
    }
    let tag_cols = prefix.chars().count() + 2; // one space either side
    let fill = span.saturating_sub(1 + tag_cols);
    let gilded = crate::color::colorize(prefix, gild);
    format!("{corner}{horiz} {gilded} {}{corner_r}", horiz.repeat(fill))
}

/// Word-wrap text to `width` columns, breaking a single word longer than the
/// width mid-word. Always returns at least one (possibly blank) line.
fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();

    for raw in text.split_whitespace() {
        let mut word = raw.to_string();

        // Break a word too long for the frame rather than overflow inventory.
        while word.chars().count() > width {
            if !current.is_empty() {
                lines.push(std::mem::take(&mut current));
            }
            let head: String = word.chars().take(width).collect();
            lines.push(head);
            word = word.chars().skip(width).collect();
        }

        let candidate = if current.is_empty() {
            word.clone()
        } else {
            format!("{current} {word}")
        };

        if candidate.chars().count() > width {
            lines.push(std::mem::take(&mut current));
            current = word;
        } else {
            current = candidate;
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_a_double_banner_with_masthead() {
        let out = render("Own the viewport.", "[AD]", Box::Double, false, false);
        let lines: Vec<&str> = out.lines().collect();
        assert!(
            lines[0].starts_with("╔═ [AD] "),
            "top border carries the tag"
        );
        assert!(lines[0].ends_with('╗'));
        assert!(lines.last().unwrap().starts_with('╚'));
        assert!(lines[1].starts_with('║') && lines[1].ends_with('║'));
    }

    #[test]
    fn every_line_is_the_same_width() {
        let out = render(
            "a somewhat longer creative that will wrap",
            "[AD]",
            Box::Light,
            false,
            false,
        );
        let widths: Vec<usize> = out.lines().map(|l| l.chars().count()).collect();
        assert!(
            widths.iter().all(|w| *w == widths[0]),
            "frame stays on-grid: {widths:?}"
        );
    }

    #[test]
    fn ascii_only_uses_portable_glyphs() {
        let out = render("legacy sink", "[AD]", Box::Double, true, false);
        assert!(out.contains('+') && out.contains('|') && out.contains('-'));
        assert!(!out.contains('╔'), "no box-drawing when ascii_only");
    }

    #[test]
    fn gilded_masthead_keeps_the_border_aligned() {
        // The gold escapes are zero-width, so the gilded top border must be the
        // same visible width as the plain one.
        let plain = render("aligned?", "[AD]", Box::Double, false, false);
        let gold = render("aligned?", "[AD]", Box::Double, false, true);
        assert!(gold.contains("\x1b[38;5;214m"), "masthead is gilded");

        let strip = |s: &str| s.replace("\x1b[38;5;214m", "").replace("\x1b[0m", "");
        assert_eq!(
            strip(&gold),
            plain,
            "stripped of escapes, the gilded frame is byte-identical"
        );
    }

    #[test]
    fn blank_prefix_collapses_to_a_solid_rule() {
        let out = render("no tag", "", Box::Light, false, false);
        let top = out.lines().next().unwrap();
        assert!(!top.contains("[AD]"));
        assert!(top.starts_with('┌') && top.ends_with('┐'));
    }

    #[test]
    fn wraps_a_word_longer_than_the_frame() {
        let long = "x".repeat(WIDTH + 10);
        let lines = wrap_text(&long, WIDTH);
        assert_eq!(lines[0].chars().count(), WIDTH);
        assert_eq!(lines[1].chars().count(), 10);
    }

    #[test]
    fn wrap_of_empty_text_is_one_blank_line() {
        assert_eq!(wrap_text("", WIDTH), vec![String::new()]);
    }
}
