//! Drawing a bar row as SVG. Where everything goes is decided by
//! [`crate::render::bar_row`]; this module only turns its marks into elements.

use super::line::{render_piece, rounded};
use crate::model::{BarRow, TextStyle};
use crate::render::RenderConfig;
use crate::render::bar_row::{self, Mark, Point};
use crate::render::chord::{Offset, Piece, Slash};
use svg::Node;
use svg::node::Blob;
use svg::node::element::{Circle, Group, Line as SvgLine, Text as SvgText};

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
        Mark::Dot { at, radius } => Box::new(dot(*at, *radius)),
        Mark::Chord {
            at,
            size,
            pieces,
            slash,
            squeeze,
        } => chord(*at, *size, pieces, slash.as_ref(), *squeeze, font_family),
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

fn dot(at: Point, radius: f64) -> Circle {
    Circle::new()
        .set("cx", rounded(at.x))
        .set("cy", rounded(at.y))
        .set("r", rounded(radius))
}

/// A chord as one text element, with the slash of a hanging bass as a line
/// beside it. A squeezed chord is drawn at the origin of a group that moves it
/// into place and narrows it, slash included.
fn chord(
    at: Point,
    size: f64,
    pieces: &[Piece],
    slash: Option<&Slash>,
    squeeze: f64,
    font_family: &str,
) -> Box<dyn Node> {
    let tspans: String = pieces
        .iter()
        .map(|piece| render_piece(piece, TextStyle::Normal, size).to_string())
        .collect();
    let text = SvgText::new("")
        .set("font-family", font_family)
        .set("font-size", rounded(size))
        .add(Blob::new(tspans));
    if squeeze >= 1.0 {
        let text = text.set("x", rounded(at.x)).set("y", rounded(at.y));
        return match slash {
            None => Box::new(text),
            Some(slash) => Box::new(Group::new().add(text).add(slash_line(slash, at))),
        };
    }
    let transform = format!(
        "translate({} {}) scale({} 1)",
        rounded(at.x),
        rounded(at.y),
        rounded(squeeze)
    );
    let mut group = Group::new()
        .set("transform", transform)
        .add(text.set("x", 0).set("y", 0));
    if let Some(slash) = slash {
        group = group.add(slash_line(slash, Point { x: 0.0, y: 0.0 }));
    }
    Box::new(group)
}

/// `slash` as a line, for a chord whose baseline starts at `origin`.
fn slash_line(slash: &Slash, origin: Point) -> SvgLine {
    let point = |offset: Offset| Point {
        x: origin.x + offset.x,
        y: origin.y - offset.rise,
    };
    stroke(point(slash.from), point(slash.to), slash.width)
}
