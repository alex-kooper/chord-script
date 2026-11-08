//! Bar lines: plain, double and final lines, and repeat signs with their dots.
//!
//! A sign stands on its place in the row by its heaviest stroke, so it reaches
//! into the bar it opens or closes: a repeat start into the bar after it, a
//! repeat end or final line into the bar before. A double line reaches back,
//! like a final one. At either end of the row a sign is pushed inward, so its
//! outer stroke stands on the row's edge like a plain line.

use super::{ASCENT, BAR_LINE_DROP, BAR_LINE_WIDTH, Mark, Point};
use crate::model::BarLine;
use crate::render::fonts::{self, Font};

/// Fractions of the chord size, set by eye after iReal Pro.
const THICK_WIDTH: f64 = 0.08;
/// Between two strokes of a sign, edge to edge.
const STROKE_GAP: f64 = 0.07;
/// Between a stroke and the repeat dots beside it, edge to edge.
const DOT_GAP: f64 = 0.08;
const DOT_RADIUS: f64 = 0.058;
/// How far each repeat dot sits above or below the middle of a root.
const DOT_SPREAD: f64 = 0.2;

/// Where a bar line stands in its row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Place {
    Start,
    Inside,
    End,
}

/// A bar line as drawn, and the room it takes from the bars beside it: what is
/// drawn in a bar stays between `right` of the sign before it and `left` of the
/// one after. A plain line takes none.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Sign {
    pub marks: Vec<Mark>,
    pub left: f64,
    pub right: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Piece {
    Thin,
    Thick,
    /// Two repeat dots, one above the other
    Dots,
}

impl Piece {
    fn half_width(self) -> f64 {
        match self {
            Piece::Thin => BAR_LINE_WIDTH / 2.0,
            Piece::Thick => THICK_WIDTH / 2.0,
            Piece::Dots => DOT_RADIUS,
        }
    }

    /// How far past its middle the piece may not reach out of the row: a
    /// stroke stands on the row's edge like a plain line, dots stay inside.
    fn overhang(self) -> f64 {
        if self == Piece::Dots { DOT_RADIUS } else { 0.0 }
    }
}

/// The pieces of `line`, left to right, and the one standing on its place.
fn pieces(line: BarLine) -> (&'static [Piece], usize) {
    use Piece::{Dots, Thick, Thin};
    match line {
        BarLine::Single => (&[Thin], 0),
        BarLine::Double => (&[Thin, Thin], 1),
        BarLine::Final => (&[Thin, Thick], 1),
        BarLine::RepeatStart => (&[Thick, Thin, Dots], 0),
        BarLine::RepeatEnd => (&[Dots, Thin, Thick], 2),
        BarLine::RepeatBoth => (&[Dots, Thin, Thick, Thin, Dots], 2),
    }
}

/// `line` drawn at `x`, in a row whose chords stand on `baseline` at `size`.
pub(super) fn sign(line: BarLine, x: f64, place: Place, baseline: f64, size: f64) -> Sign {
    let (pieces, anchor) = pieces(line);
    let middles = middles(pieces, size);
    let (first, last) = (0, pieces.len() - 1);
    let mut offset = x - middles[anchor];
    let outer_left = offset + middles[first] - pieces[first].overhang() * size;
    let outer_right = offset + middles[last] + pieces[last].overhang() * size;
    match place {
        Place::Start if outer_left < x => offset += x - outer_left,
        Place::End if outer_right > x => offset -= outer_right - x,
        _ => {}
    }

    let marks = pieces
        .iter()
        .zip(&middles)
        .flat_map(|(&piece, &middle)| piece_marks(piece, offset + middle, baseline, size))
        .collect();
    let (left, right) = if line == BarLine::Single {
        (x, x)
    } else {
        (
            offset + middles[first] - pieces[first].half_width() * size,
            offset + middles[last] + pieces[last].half_width() * size,
        )
    };
    Sign { marks, left, right }
}

/// Where the middle of each of `pieces` is, from the middle of the first.
fn middles(pieces: &[Piece], size: f64) -> Vec<f64> {
    let mut at = 0.0;
    let mut middles = Vec::with_capacity(pieces.len());
    for (index, &piece) in pieces.iter().enumerate() {
        if let Some(&before) = index.checked_sub(1).and_then(|i| pieces.get(i)) {
            let gap = if before == Piece::Dots || piece == Piece::Dots {
                DOT_GAP
            } else {
                STROKE_GAP
            };
            at += (before.half_width() + gap + piece.half_width()) * size;
        }
        middles.push(at);
    }
    middles
}

fn piece_marks(piece: Piece, x: f64, baseline: f64, size: f64) -> Vec<Mark> {
    match piece {
        Piece::Thin => vec![stroke(x, BAR_LINE_WIDTH, baseline, size)],
        Piece::Thick => vec![stroke(x, THICK_WIDTH, baseline, size)],
        Piece::Dots => {
            let middle = baseline - fonts::cap_height(Font::Chord, size) / 2.0;
            [-DOT_SPREAD, DOT_SPREAD]
                .map(|spread| Mark::Dot {
                    at: Point {
                        x,
                        y: middle + spread * size,
                    },
                    radius: DOT_RADIUS * size,
                })
                .into()
        }
    }
}

#[cfg(test)]
mod tests;

/// A stroke `width` thick at `x`, from the top of the row's box into the bass.
fn stroke(x: f64, width: f64, baseline: f64, size: f64) -> Mark {
    Mark::Stroke {
        from: Point {
            x,
            y: baseline - ASCENT * size,
        },
        to: Point {
            x,
            y: baseline + BAR_LINE_DROP * size,
        },
        width: width * size,
    }
}
