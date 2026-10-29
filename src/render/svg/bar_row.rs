//! Drawing a bar row as SVG. Where everything goes is decided by
//! [`crate::render::bar_row`]; this module only turns its marks into elements.

use super::line::{render_piece, rounded};
use crate::model::{BarRow, TextStyle};
use crate::render::RenderConfig;
use crate::render::bar_row::{self, Mark, Point};
use crate::render::chord::{Offset, Piece, Slash};
use svg::Node;
use svg::node::Blob;
use svg::node::element::{Circle, Group, Polygon, Text as SvgText};

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

/// A stroke `width` thick across from `from` to `to`, its ends cut level: a
/// parallelogram, or a rectangle when upright. Its ends are wider than
/// `width` the more it leans.
fn stroke(from: Point, to: Point, width: f64) -> Polygon {
    let (across, up) = (to.x - from.x, to.y - from.y);
    // A chord size too small to tell its ends apart on the page leaves a
    // stroke with no height, which has no lean to widen its ends for.
    let half_end = if up == 0.0 {
        width / 2.0
    } else {
        width / 2.0 * across.hypot(up) / up.abs()
    };
    let corners = [
        (from.x - half_end, from.y),
        (from.x + half_end, from.y),
        (to.x + half_end, to.y),
        (to.x - half_end, to.y),
    ];
    let points: Vec<String> = corners
        .iter()
        .map(|&(x, y)| format!("{},{}", rounded(x), rounded(y)))
        .collect();
    Polygon::new().set("points", points.join(" "))
}

fn dot(at: Point, radius: f64) -> Circle {
    Circle::new()
        .set("cx", rounded(at.x))
        .set("cy", rounded(at.y))
        .set("r", rounded(radius))
}

/// A chord as one text element, with the slash of a hanging bass as a stroke
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

/// `slash` as a stroke, for a chord whose baseline starts at `origin`.
fn slash_line(slash: &Slash, origin: Point) -> Polygon {
    let point = |offset: Offset| Point {
        x: origin.x + offset.x,
        y: origin.y - offset.rise,
    };
    stroke(point(slash.from), point(slash.to), slash.width)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::svg::tests::polygons;

    fn corners(polygon: Polygon) -> Vec<(f64, f64)> {
        let mut drawn = polygons(&polygon.to_string());
        assert_eq!(drawn.len(), 1, "one polygon");
        drawn.remove(0)
    }

    #[test]
    fn a_leaning_stroke_has_level_ends_and_keeps_its_thickness_across() {
        // Across 12 and up 16 make it 20 long, so its ends are 20/16 as wide.
        let from = Point { x: 10.0, y: 50.0 };
        let to = Point { x: 22.0, y: 34.0 };
        assert_eq!(
            corners(stroke(from, to, 2.0)),
            [(8.75, 50.0), (11.25, 50.0), (23.25, 34.0), (20.75, 34.0)]
        );
    }

    #[test]
    fn a_stroke_with_no_height_has_ends_as_wide_as_it_is_thick() {
        let at = Point { x: 5.0, y: 70.0 };
        assert_eq!(
            corners(stroke(at, at, 2.0)),
            [(4.0, 70.0), (6.0, 70.0), (6.0, 70.0), (4.0, 70.0)]
        );
    }

    #[test]
    fn an_upright_stroke_is_a_rectangle() {
        let from = Point { x: 5.0, y: 0.0 };
        let to = Point { x: 5.0, y: 10.0 };
        assert_eq!(
            corners(stroke(from, to, 2.0)),
            [(4.0, 0.0), (6.0, 0.0), (6.0, 10.0), (4.0, 10.0)]
        );
    }
}
