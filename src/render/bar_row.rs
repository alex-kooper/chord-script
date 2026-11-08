//! How a bar row is laid out, whatever the output format.
//!
//! The row spans the page between its margins, split evenly into its bars and
//! the bars its spans stand for. Only bars are drawn, with a bar line on each
//! side; a double or final line or a repeat sign takes some of a bar's room,
//! and the rest is split evenly into its slots. A chord or beat slash starts a
//! little after the start of its slot, so a beat lands at the same place in
//! bars with as many slots, unless a sign beside one takes some of its room;
//! `%` is centered in its bar. A chord may reach over the empty slots after
//! it, and one too wide for the room before the next thing drawn in its bar
//! is squeezed horizontally to fit, keeping its height. A row too cramped for
//! that is reported instead.
//!
//! The row's box holds the chords, then the bass hanging below them. Bar
//! lines run from the top of the box to about the middle of a hanging bass,
//! as in iReal Pro.

use super::BarRowStyle;
use super::chord::{self, BassPlacement, ChordLayout, Piece, Slash};
use super::fonts::{self, Font};
use crate::model::{Bar, BarRow, Cell, Chord, Slot};
use bar_line::{Place, Sign};
use std::iter;

mod bar_line;

/// Fractions of the chord size, set by eye after iReal Pro.
///
/// How far the row reaches above the chords' baseline: past a raised root
/// accidental. A ♯ reaches a little further, into the space above the row.
const ASCENT: f64 = 1.08;
/// How far the row reaches below the baseline: past a hanging bass and its ♯.
const DESCENT: f64 = 0.7;
/// Where bar lines end below the baseline: about the middle of a hanging bass.
const BAR_LINE_DROP: f64 = 0.43;
const BAR_LINE_WIDTH: f64 = 0.04;
/// The space between the start of a slot and what is drawn in it.
const INSET: f64 = 0.2;
/// The least space left between a chord and what follows it in its bar.
const CLEARANCE: f64 = 0.15;
/// The narrowest a chord may be squeezed, as a fraction of its width: below
/// that it can no longer be read, and the row is reported as too cramped.
const MIN_SQUEEZE: f64 = 0.1;
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
    /// A straight line: a bar line, a beat slash, or the slash of a repeat
    /// sign. Its ends are cut level, like a glyph standing on the baseline,
    /// and `width` is its thickness across.
    Stroke { from: Point, to: Point, width: f64 },
    /// A filled circle: a dot of a repeat sign
    Dot { at: Point, radius: f64 },
    /// A chord whose baseline starts at `at`, drawn at `size` and squeezed
    /// horizontally by `squeeze`: 1 when it fits as it is. The slash of its
    /// hanging bass is drawn and squeezed with it.
    Chord {
        at: Point,
        size: f64,
        pieces: Vec<Piece>,
        slash: Option<Slash>,
        squeeze: f64,
    },
}

/// The height of a bar row's box: its chords and the bass hanging below them.
pub(crate) fn height(style: &BarRowStyle) -> f64 {
    (ASCENT + DESCENT) * style.chord_size
}

/// The room at the bottom of a bar row, below its bar lines, kept for a
/// hanging bass.
pub(crate) fn room_below_bar_lines(style: &BarRowStyle) -> f64 {
    (DESCENT - BAR_LINE_DROP) * style.chord_size
}

/// What in a bar row has too little room to be drawn between its margins.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Cramped<'a> {
    /// The bar lines on either side of a bar run into each other
    BarLines,
    /// A chord would be squeezed narrower than [`MIN_SQUEEZE`]
    Chord(&'a Chord),
}

/// The marks of `row`, drawn between `left` and `right` in a row box whose
/// bottom is at `bottom`, or what has too little room there.
pub(super) fn layout<'a>(
    row: &'a BarRow,
    style: &BarRowStyle,
    left: f64,
    right: f64,
    bottom: f64,
) -> Result<Vec<Mark>, Cramped<'a>> {
    let size = style.chord_size;
    let baseline = bottom - DESCENT * size;
    let signs = signs(row, left, right, baseline, size);
    let room = |index: usize| {
        signs[index]
            .as_ref()
            .expect("a bar has a bar line on each side")
    };
    let contents = row
        .cells
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| match &entry.cell {
            Cell::Bar(bar) => Some((index, bar)),
            Cell::Span(_) => None,
        })
        .map(|(index, bar)| bar_marks(bar, room(index).right, room(index + 1).left, baseline, size))
        .collect::<Result<Vec<_>, _>>()?;

    let lines = signs.iter().flatten().flat_map(|sign| sign.marks.clone());
    Ok(lines.chain(contents.into_iter().flatten()).collect())
}

/// Each bar line of `row` as drawn between `left` and `right`, in the order of
/// [`BarRow::lines`]: `None` for one with no bar beside it, which is not drawn.
fn signs(row: &BarRow, left: f64, right: f64, baseline: f64, size: f64) -> Vec<Option<Sign>> {
    let bar_width = (right - left) / row.width().get() as f64;
    let edges = iter::once(0).chain(row.cells.iter().scan(0, |at, entry| {
        *at += entry.cell.width().get();
        Some(*at)
    }));
    let last = row.cells.len();
    row.lines()
        .zip(edges)
        .enumerate()
        .map(|(index, (line, at))| {
            let place = match index {
                0 => Place::Start,
                index if index == last => Place::End,
                _ => Place::Inside,
            };
            let x = left + at as f64 * bar_width;
            row.is_beside_bar(index)
                .then(|| bar_line::sign(line, x, place, baseline, size))
        })
        .collect()
}

/// The marks inside the bar from `start` to `end`, the room its bar lines
/// leave it.
fn bar_marks(
    bar: &Bar,
    start: f64,
    end: f64,
    baseline: f64,
    size: f64,
) -> Result<Vec<Mark>, Cramped<'_>> {
    if end <= start {
        return Err(Cramped::BarLines);
    }
    let slots = match bar {
        Bar::Repeat => return Ok(repeat_sign(start, end, baseline, size)),
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
                let ChordLayout { pieces, slash } =
                    chord::layout(symbol, Font::Chord, size, BassPlacement::Below);
                let room = next - x - CLEARANCE * size;
                let squeeze = (room / chord::width(&pieces)).min(1.0);
                if squeeze.is_nan() || squeeze < MIN_SQUEEZE {
                    return Err(Cramped::Chord(symbol));
                }
                Mark::Chord {
                    at,
                    size,
                    pieces,
                    slash,
                    squeeze,
                }
            }
            Slot::Slash => beat_slash(at, size),
            Slot::Hold => unreachable!("holds are filtered out above"),
        });
    }
    Ok(marks)
}

/// A beat slash rising from `at` on the baseline to the height of the root.
fn beat_slash(at: Point, size: f64) -> Mark {
    Mark::Stroke {
        from: at,
        to: Point {
            x: at.x + SLASH_LEAN * size,
            y: at.y - fonts::cap_height(Font::Chord, size),
        },
        width: SLASH_WIDTH * size,
    }
}

/// The bar repeat sign, centered in the bar from `start` to `end` and spanning
/// the height of the root: a slash leaning right, a dot above it on the left,
/// and one below it on the right.
fn repeat_sign(start: f64, end: f64, baseline: f64, size: f64) -> Vec<Mark> {
    let height = fonts::cap_height(Font::Chord, size);
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
