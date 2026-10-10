//! Per-line rendering: everything specific to drawing a single line.
//!
//! This module owns *how* a line looks — its height and its SVG output — while
//! the vertical placement (which `y` a line sits at) is decided by [`super::layout`].
//! Keeping both behaviors here means adding a new line kind later touches one
//! place: its height and its drawing sit side by side.

use crate::model::{Content, Inline, Line, LineLevel, TextStyle};
use crate::render::RenderConfig;
use crate::render::chord::{self, BassPlacement, Piece};
use crate::render::fonts::{self, Font};
use svg::node::Blob;
use svg::node::element::{TSpan, Text as SvgText};

/// The vertical space a line occupies.
///
/// Today this is just the per-level line height; when richer line kinds arrive
/// (e.g. chord lines with a bar row plus a chord row) this becomes kind-specific.
pub(super) fn height_of(line: &Line, config: &RenderConfig) -> f64 {
    config.line_height_for_level(line.level)
}

/// Where a column's `x` is: at its start, middle, or end.
#[derive(Debug, Clone, Copy)]
enum Anchor {
    Start,
    Middle,
    End,
}

/// Render a line at the given baseline `y`, producing one text element per
/// non-empty column (left / center / right).
pub(super) fn render(line: &Line, config: &RenderConfig, y: f64) -> Vec<SvgText> {
    let page = &config.layout;
    let columns = [
        (&line.left, page.margin_horizontal, Anchor::Start),
        (&line.center, page.width / 2.0, Anchor::Middle),
        (
            &line.right,
            page.width - page.margin_horizontal,
            Anchor::End,
        ),
    ];

    columns
        .into_iter()
        .filter(|(inlines, _, _)| !inlines.is_empty())
        .map(|(inlines, x, anchor)| render_column(inlines, x, anchor, y, line.level, config))
        .collect()
}

/// Render a column as a single SVG text element with tspans for its inlines.
///
/// The tspans are joined into one blob: the svg crate puts a newline before
/// every child element, which SVG would render as a space inside words
/// (`un_believ_able`).
fn render_column(
    inlines: &[Inline],
    x: f64,
    anchor: Anchor,
    y: f64,
    level: LineLevel,
    config: &RenderConfig,
) -> SvgText {
    let style = config.font_style_for_level(level);
    let level_style = TextStyle::from_flags(is_bold_weight(&style.weight), false);
    let mut tspans = String::new();
    let mut overlap = 0.0;
    for inline in inlines {
        match &inline.content {
            Content::Text(text) => {
                tspans += &styled(TSpan::new(text.as_ref()), inline.style).to_string();
            }
            Content::Chord(symbol) => {
                let measured_style = inline.style.combine(level_style);
                let pieces =
                    chord::layout(symbol, measured_style, style.size, BassPlacement::After);
                overlap += chord::overlap(&pieces);
                for piece in &pieces {
                    tspans += &render_piece(piece, inline.style, style.size).to_string();
                }
            }
        }
    }

    let element = SvgText::new("")
        .set("x", x + anchor_correction(anchor, overlap))
        .set("y", y)
        .set("font-family", config.font_family.as_str())
        .set("font-size", style.size)
        .set("font-weight", style.weight.as_str())
        .add(Blob::new(tspans));
    match anchor {
        Anchor::Start => element,
        Anchor::Middle => element.set("text-anchor", "middle"),
        Anchor::End => element.set("text-anchor", "end"),
    }
}

/// How far to move a column's `x` so its anchor holds despite chord parts
/// drawn under one another.
///
/// usvg, which draws the PDF, aligns a column by its glyph advances alone,
/// ignoring `dx`; the column would land off by the chords' `overlap`, or half
/// of it when centered. Viewers that count `dx` shift such columns instead.
fn anchor_correction(anchor: Anchor, overlap: f64) -> f64 {
    match anchor {
        Anchor::Start => 0.0,
        Anchor::Middle => overlap / 2.0,
        Anchor::End => overlap,
    }
}

/// Whether a CSS font weight draws in the bundled bold face: weights from
/// 600 up snap to bold.
fn is_bold_weight(weight: &str) -> bool {
    match weight.trim() {
        "bold" | "bolder" => true,
        weight => weight.parse::<u16>().is_ok_and(|weight| weight >= 600),
    }
}

fn render_piece(piece: &Piece, style: TextStyle, column_size: f64) -> TSpan {
    let mut tspan = styled(TSpan::new(piece.text.as_str()), style);
    if piece.font == Font::Music {
        tspan = tspan.set("font-family", fonts::MUSIC_FAMILY);
    }
    if piece.size != column_size {
        tspan = tspan.set("font-size", rounded(piece.size));
    }
    if piece.dx != 0.0 {
        tspan = tspan.set("dx", rounded(piece.dx));
    }
    if piece.rise != 0.0 {
        tspan = tspan.set("baseline-shift", rounded(piece.rise));
    }
    tspan
}

fn styled(tspan: TSpan, style: TextStyle) -> TSpan {
    match style {
        TextStyle::Normal => tspan,
        TextStyle::Bold => tspan.set("font-weight", "bold"),
        TextStyle::Italic => tspan.set("font-style", "italic"),
        TextStyle::BoldItalic => tspan.set("font-weight", "bold").set("font-style", "italic"),
    }
}

/// `value` to a thousandth of a point, which keeps the SVG readable.
fn rounded(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weights_from_600_up_are_bold() {
        for weight in ["bold", "bolder", "600", "700", "900", " bold "] {
            assert!(is_bold_weight(weight), "{weight:?}");
        }
        for weight in ["normal", "lighter", "500", "599", "100", "", "heavy"] {
            assert!(!is_bold_weight(weight), "{weight:?}");
        }
    }
}
