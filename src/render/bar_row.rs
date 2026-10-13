//! How a bar row is laid out, whatever the output format.
//!
//! The row spans the page between its margins, split evenly into its bars,
//! and each bar evenly into its slots. A chord or beat slash starts a little
//! after the start of its slot, so a beat always lands at the same place in a
//! bar; `%` is centered in its bar. A chord may reach over the empty slots
//! after it, and one too wide for the room before the next thing drawn in its
//! bar is squeezed horizontally to fit, keeping its height.
//!
//! The row's box has the gap at its top, then the chords, then the bass
//! hanging below them. Bar lines run from the top of the chords to about the
//! middle of a hanging bass, as in iReal Pro.

use super::BarRowStyle;
use super::chord::{self, BassPlacement, Piece};
use super::fonts::Font;
use crate::model::{Bar, BarRow, Slot};

/// Fractions of the chord size, set by eye after iReal Pro.
///
/// How far the row reaches above the chords' baseline: past a raised root
/// accidental. A ♯ reaches a little further and peeks into the gap.
const ASCENT: f64 = 1.0;
/// How far the row reaches below the baseline: past a hanging bass and its ♯.
const DESCENT: f64 = 0.9;
/// Where bar lines end below the baseline: about the middle of a hanging bass.
const BAR_LINE_DROP: f64 = 0.43;
const BAR_LINE_WIDTH: f64 = 0.04;
/// The space between the start of a slot and what is drawn in it.
const INSET: f64 = 0.2;
/// The least space left between a chord and what follows it in its bar.
const CLEARANCE: f64 = 0.15;
/// The height of Noto Sans capitals: a beat slash is as tall as the root.
const CAP_HEIGHT: f64 = 0.714;
/// How far right a beat slash leans over its height.
const SLASH_LEAN: f64 = 0.3;
/// Between the stems of the root and of the smaller quality: a long diagonal
/// looks heavier than a stem as thick.
const SLASH_WIDTH: f64 = 0.08;
/// The repeat sign is a slash as tall as the root between two dots; its
/// proportions follow Noto Music's one-bar repeat sign. How far right its
/// slash leans over its height:
const REPEAT_LEAN: f64 = 0.8;
/// Heavier than a beat slash: the sign stands for a bar, like a chord, where a
/// beat slash is only a mark.
const REPEAT_WIDTH: f64 = 0.11;
/// How far each dot sits from the sign's middle, and how big it is, as
/// fractions of the sign's height:
const REPEAT_DOT_ACROSS: f64 = 0.35;
const REPEAT_DOT_UP: f64 = 0.225;
const REPEAT_DOT_RADIUS: f64 = 0.11;

/// A position on the page, in points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Point {
    pub x: f64,
    pub y: f64,
}

/// Something drawn in a bar row, positioned on the page.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum Mark {
    /// A straight line: a bar line, a beat slash, or the slash of a repeat sign
    Stroke { from: Point, to: Point, width: f64 },
    /// A filled circle: a dot of a repeat sign
    Dot { at: Point, radius: f64 },
    /// A chord whose baseline starts at `at`, drawn at `size` and squeezed
    /// horizontally by `squeeze`: 1 when it fits as it is
    Chord {
        at: Point,
        size: f64,
        pieces: Vec<Piece>,
        squeeze: f64,
    },
}

/// The vertical space a bar row takes: the gap above it, its chords, and the
/// bass hanging below them.
pub(crate) fn height(style: &BarRowStyle) -> f64 {
    style.gap + (ASCENT + DESCENT) * style.chord_size
}

/// The marks of `row`, drawn between `left` and `right` in a row box whose
/// bottom is at `bottom`.
pub(super) fn layout(
    row: &BarRow,
    style: &BarRowStyle,
    left: f64,
    right: f64,
    bottom: f64,
) -> Vec<Mark> {
    let size = style.chord_size;
    let baseline = bottom - DESCENT * size;
    let bar_width = (right - left) / row.bars.len() as f64;
    let edges = (0..=row.bars.len()).map(|index| left + index as f64 * bar_width);
    let bar_lines = edges.map(|x| Mark::Stroke {
        from: Point {
            x,
            y: baseline - ASCENT * size,
        },
        to: Point {
            x,
            y: baseline + BAR_LINE_DROP * size,
        },
        width: BAR_LINE_WIDTH * size,
    });
    let contents = row.bars.iter().enumerate().flat_map(|(index, bar)| {
        let start = left + index as f64 * bar_width;
        bar_marks(bar, start, start + bar_width, baseline, size)
    });
    bar_lines.chain(contents).collect()
}

/// The marks inside the bar from `start` to `end`.
fn bar_marks(bar: &Bar, start: f64, end: f64, baseline: f64, size: f64) -> Vec<Mark> {
    let slots = match bar {
        Bar::Repeat => return repeat_sign(start, end, baseline, size),
        Bar::Slots(slots) => slots,
    };
    let slot_width = (end - start) / slots.len() as f64;
    let drawn: Vec<(f64, &Slot)> = slots
        .iter()
        .enumerate()
        .filter(|(_, slot)| !matches!(slot, Slot::Hold))
        .map(|(index, slot)| (start + index as f64 * slot_width + INSET * size, slot))
        .collect();

    let mut marks = Vec::new();
    for (index, &(x, slot)) in drawn.iter().enumerate() {
        let next = drawn.get(index + 1).map_or(end, |&(next, _)| next);
        let at = Point { x, y: baseline };
        marks.push(match slot {
            Slot::Chord(symbol) => {
                let pieces = chord::layout(symbol, Font::Chord, size, BassPlacement::Below);
                let room = next - x - CLEARANCE * size;
                let squeeze = (room.max(0.0) / chord::width(&pieces)).min(1.0);
                Mark::Chord {
                    at,
                    size,
                    pieces,
                    squeeze,
                }
            }
            Slot::Slash => beat_slash(at, size),
            Slot::Hold => unreachable!("holds are filtered out above"),
        });
    }
    marks
}

/// A beat slash rising from `at` on the baseline to the height of the root.
fn beat_slash(at: Point, size: f64) -> Mark {
    Mark::Stroke {
        from: at,
        to: Point {
            x: at.x + SLASH_LEAN * size,
            y: at.y - CAP_HEIGHT * size,
        },
        width: SLASH_WIDTH * size,
    }
}

/// The bar repeat sign, centered in the bar from `start` to `end` and spanning
/// the height of the root: a slash leaning right, a dot above it on the left,
/// and one below it on the right.
fn repeat_sign(start: f64, end: f64, baseline: f64, size: f64) -> Vec<Mark> {
    let height = CAP_HEIGHT * size;
    let middle = Point {
        x: (start + end) / 2.0,
        y: baseline - height / 2.0,
    };
    let from_middle = |across: f64, up: f64| Point {
        x: middle.x + across * height,
        y: middle.y - up * height,
    };
    let half_lean = REPEAT_LEAN / 2.0;
    let dot = |at: Point| Mark::Dot {
        at,
        radius: REPEAT_DOT_RADIUS * height,
    };
    vec![
        Mark::Stroke {
            from: from_middle(-half_lean, -0.5),
            to: from_middle(half_lean, 0.5),
            width: REPEAT_WIDTH * size,
        },
        dot(from_middle(-REPEAT_DOT_ACROSS, REPEAT_DOT_UP)),
        dot(from_middle(REPEAT_DOT_ACROSS, -REPEAT_DOT_UP)),
    ]
}

#[cfg(test)]
mod tests;
