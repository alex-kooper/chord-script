//! Per-line rendering: everything specific to drawing a single line.
//!
//! This module owns *how* a line looks — its height and its SVG output — while
//! the vertical placement (which `y` a line sits at) is decided by [`super::layout`].
//! Keeping both behaviors here means adding a new line kind later touches one
//! place: its height and its drawing sit side by side.

use super::config::SvgConfig;
use crate::model::{Line, LineLevel, TextSpan, TextStyle};
use svg::node::element::{TSpan, Text as SvgText};

/// The vertical space a line occupies.
///
/// Today this is just the per-level line height; when richer line kinds arrive
/// (e.g. chord lines with a bar row plus a chord row) this becomes kind-specific.
pub(super) fn height_of(line: &Line, config: &SvgConfig) -> f64 {
    config.line_height_for_level(line.level)
}

/// Render a line at the given baseline `y`, producing one text element per
/// non-empty column (left / center / right).
pub(super) fn render(line: &Line, config: &SvgConfig, y: f64) -> Vec<SvgText> {
    let page = &config.layout;

    // The three columns differ only in their spans, x-position, and anchor.
    // Left has no anchor: SVG's default `start` is exactly left alignment.
    let columns = [
        (&line.left, page.margin_horizontal, None),
        (&line.center, page.width / 2.0, Some("middle")),
        (
            &line.right,
            page.width - page.margin_horizontal,
            Some("end"),
        ),
    ];

    columns
        .into_iter()
        .filter(|(spans, _, _)| !spans.is_empty())
        .map(|(spans, x, anchor)| {
            let element = render_spans(spans, x, y, line.level, config);
            match anchor {
                Some(anchor) => element.set("text-anchor", anchor),
                None => element,
            }
        })
        .collect()
}

/// Render a sequence of styled text spans as a single SVG text element with tspans.
fn render_spans(
    spans: &[TextSpan],
    x: f64,
    y: f64,
    level: LineLevel,
    config: &SvgConfig,
) -> SvgText {
    let style = config.font_style_for_level(level);

    let mut text_el = SvgText::new("")
        .set("x", x)
        .set("y", y)
        .set("font-family", config.font_family.as_str())
        .set("font-size", style.size)
        .set("font-weight", style.weight.as_str());

    for span in spans {
        let text: &str = span.text.as_ref();
        let mut tspan = TSpan::new(text);

        tspan = match span.style {
            TextStyle::Normal => tspan,
            TextStyle::Bold => tspan.set("font-weight", "bold"),
            TextStyle::Italic => tspan.set("font-style", "italic"),
            TextStyle::BoldItalic => tspan.set("font-weight", "bold").set("font-style", "italic"),
        };

        text_el = text_el.add(tspan);
    }

    text_el
}
