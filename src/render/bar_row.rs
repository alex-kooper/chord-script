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
use crate::model::{Bar, BarRow, Slot, TextStyle};

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
const SLASH_WIDTH: f64 = 0.12;
const REPEAT_SIZE: f64 = 1.2;
/// The middle of the repeat sign above its baseline, as a fraction of its size.
const REPEAT_SIGN_MIDDLE: f64 = 0.5;

/// The one-bar repeat sign drawn for `%`, a slash between two dots, in the
/// music font.
pub(super) const REPEAT_SIGN: char = '\u{1D10E}';

/// A position on the page, in points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Point {
    pub x: f64,
    pub y: f64,
}

/// Something drawn in a bar row, positioned on the page.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum Mark {
    /// A straight line: a bar line or a beat slash
    Stroke { from: Point, to: Point, width: f64 },
    /// A chord whose baseline starts at `at`, drawn at `size` and squeezed
    /// horizontally by `squeeze`: 1 when it fits as it is
    Chord {
        at: Point,
        size: f64,
        pieces: Vec<Piece>,
        squeeze: f64,
    },
    /// The bar repeat sign at `size`, centered on `at.x` with its baseline at
    /// `at.y`
    Repeat { at: Point, size: f64 },
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
        Bar::Repeat => return vec![repeat_sign(start, end, baseline, size)],
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
                let pieces = chord::layout(symbol, TextStyle::Normal, size, BassPlacement::Below);
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

/// The bar repeat sign, centered in the bar from `start` to `end` and on the
/// height of the root.
fn repeat_sign(start: f64, end: f64, baseline: f64, size: f64) -> Mark {
    let sign_size = REPEAT_SIZE * size;
    let drop = REPEAT_SIGN_MIDDLE * sign_size - CAP_HEIGHT * size / 2.0;
    Mark::Repeat {
        at: Point {
            x: (start + end) / 2.0,
            y: baseline + drop,
        },
        size: sign_size,
    }
}

#[cfg(test)]
mod tests;
