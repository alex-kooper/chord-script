//! Drawing a bar row as SVG. Where everything goes is decided by
//! [`crate::render::bar_row`]; this module only turns its marks into elements.

use super::line::{render_piece, rounded};
use crate::model::{BarRow, TextStyle};
use crate::render::RenderConfig;
use crate::render::bar_row::{self, Mark, Point, REPEAT_SIGN};
use crate::render::chord::Piece;
use crate::render::fonts;
use svg::Node;
use svg::node::Blob;
use svg::node::element::{Group, Line as SvgLine, Text as SvgText};

/// The vertical space a bar row occupies.
pub(super) fn height_of(config: &RenderConfig) -> f64 {
    bar_row::height(&config.bar_row)
}

/// Render a bar row whose box has its bottom edge at `bottom`.
pub(super) fn render(row: &BarRow, config: &RenderConfig, bottom: f64) -> Vec<Box<dyn Node>> {
    let page = &config.layout;
    let left = page.margin_horizontal;
    let right = page.width - page.margin_horizontal;
    bar_row::layout(row, &config.bar_row, left, right, bottom)
        .iter()
        .map(|mark| render_mark(mark, &config.font_family))
        .collect()
}

fn render_mark(mark: &Mark, font_family: &str) -> Box<dyn Node> {
    match mark {
        Mark::Stroke { from, to, width } => Box::new(stroke(*from, *to, *width)),
        Mark::Chord {
            at,
            size,
            pieces,
            squeeze,
        } => chord(*at, *size, pieces, *squeeze, font_family),
        Mark::Repeat { at, size } => Box::new(repeat_sign(*at, *size)),
    }
}

fn stroke(from: Point, to: Point, width: f64) -> SvgLine {
    SvgLine::new()
        .set("x1", rounded(from.x))
        .set("y1", rounded(from.y))
        .set("x2", rounded(to.x))
        .set("y2", rounded(to.y))
        .set("stroke", "black")
        .set("stroke-width", rounded(width))
}

/// A chord as one text element; a squeezed one is drawn at the origin of a
/// group that moves it into place and narrows it.
fn chord(at: Point, size: f64, pieces: &[Piece], squeeze: f64, font_family: &str) -> Box<dyn Node> {
    let tspans: String = pieces
        .iter()
        .map(|piece| render_piece(piece, TextStyle::Normal, size).to_string())
        .collect();
    let text = SvgText::new("")
        .set("font-family", font_family)
        .set("font-size", rounded(size))
        .add(Blob::new(tspans));
    if squeeze >= 1.0 {
        return Box::new(text.set("x", rounded(at.x)).set("y", rounded(at.y)));
    }
    let transform = format!(
        "translate({} {}) scale({} 1)",
        rounded(at.x),
        rounded(at.y),
        rounded(squeeze)
    );
    Box::new(
        Group::new()
            .set("transform", transform)
            .add(text.set("x", 0).set("y", 0)),
    )
}

fn repeat_sign(at: Point, size: f64) -> SvgText {
    SvgText::new(REPEAT_SIGN.to_string())
        .set("x", rounded(at.x))
        .set("y", rounded(at.y))
        .set("font-family", fonts::MUSIC_FAMILY)
        .set("font-size", rounded(size))
        .set("text-anchor", "middle")
}
