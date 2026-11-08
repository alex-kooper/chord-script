//! Bar rows: the grid of bars a chart's chords are written in.
//!
//! The model holds what the author wrote, whether or not it makes musical
//! sense: a bar may be empty or start with a hold, `%` may stand in the first
//! bar of a chart, a row may be nothing but spans, and any bar line may stand
//! anywhere in a row: `||:` at its end, `|||` at its start, even `:||` at its
//! start, which a chart can't write since its rows start with `|`.

use super::{BarLine, Chord};
use derive_more::{Display, From};
use nutype::nutype;
use std::num::NonZeroUsize;
use std::{fmt, iter};
use vec1::Vec1;

/// One `| … |` line: its cells left to right, bars and spans standing in for
/// bars that are not drawn, and the bar lines around them.
///
/// The row is drawn as cells one bar wide, a span as many as it stands for.
/// It displays as written, with single spaces: `||: {2} | Am _ G | % :||`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BarRow {
    /// The bar line before the first cell
    pub start: BarLine,
    pub cells: Vec1<Entry>,
}

impl BarRow {
    /// A row of `cells` between plain bar lines.
    pub fn new(cells: Vec1<impl Into<Cell>>) -> Self {
        Self {
            start: BarLine::Single,
            cells: cells.mapped(|cell| Entry {
                cell: cell.into(),
                end: BarLine::Single,
            }),
        }
    }

    /// How many bars wide the row is, counting the bars its spans stand for.
    pub fn width(&self) -> NonZeroUsize {
        self.cells
            .iter()
            .map(|entry| entry.cell.width())
            .reduce(|total, width| total.saturating_add(width.get()))
            .expect("a row has at least one cell")
    }

    /// Each bar line of the row, left to right: the one it starts with, then
    /// the one closing each cell.
    pub fn lines(&self) -> impl Iterator<Item = BarLine> + '_ {
        iter::once(self.start).chain(self.cells.iter().map(|entry| entry.end))
    }

    /// Whether a bar stands on either side of the bar line at `index` in
    /// [`BarRow::lines`]. One that doesn't stands only next to spans.
    pub fn is_beside_bar(&self, index: usize) -> bool {
        let is_bar =
            |entry: Option<&Entry>| entry.is_some_and(|entry| matches!(entry.cell, Cell::Bar(_)));
        let before = index.checked_sub(1).and_then(|i| self.cells.get(i));
        is_bar(before) || is_bar(self.cells.get(index))
    }
}

impl fmt::Display for BarRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.start)?;
        for Entry { cell, end } in &self.cells {
            match cell {
                Cell::Bar(Bar::Slots(slots)) if slots.is_empty() => write!(f, " {end}")?,
                cell => write!(f, " {cell} {end}")?,
            }
        }
        Ok(())
    }
}

/// A cell of a row and the bar line that closes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub cell: Cell,
    pub end: BarLine,
}

/// What stands between two `|` of a row. It displays as written.
#[derive(Debug, Clone, PartialEq, Eq, From, Display)]
pub enum Cell {
    Bar(Bar),
    Span(Span),
}

impl Cell {
    /// How many bars wide the cell is.
    pub fn width(&self) -> NonZeroUsize {
        match self {
            Cell::Bar(_) => NonZeroUsize::MIN,
            Cell::Span(span) => span.width(),
        }
    }
}

/// `{2}`: room for bars that are not drawn, so a short row lines up with the
/// rows around it. At most [`Span::MAX_WIDTH`] bars wide. It displays as
/// written.
#[nutype(
    validate(predicate = |width| width.get() <= Span::MAX_WIDTH),
    derive(Debug, Clone, Copy, PartialEq, Eq)
)]
pub struct Span(NonZeroUsize);

impl Span {
    /// Wider than any row lined up with it is likely to be, and well within
    /// what row widths can add up to.
    pub const MAX_WIDTH: usize = 16;

    /// How many bars wide the span is.
    pub fn width(self) -> NonZeroUsize {
        self.into_inner()
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{{}}}", self.width())
    }
}

/// What a bar holds. It displays as written, with single spaces, and an
/// empty bar as nothing.
#[derive(Debug, Clone, PartialEq, Eq, From)]
pub enum Bar {
    /// Slots dividing the bar evenly: `| Am _ _ A7 |` is four slots with `A7`
    /// in the last. Empty for an empty bar, `| |`.
    ///
    /// Slots are positions, not beats: `| Am G |` is two halves of the bar,
    /// whatever its meter.
    Slots(Vec<Slot>),
    /// `%`: the previous bar in the chart again, skipping any spans, text lines
    /// and page breaks in between
    #[from(skip)]
    Repeat,
}

impl fmt::Display for Bar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Bar::Repeat => write!(f, "%"),
            Bar::Slots(slots) => {
                for (index, slot) in slots.iter().enumerate() {
                    let separator = if index == 0 { "" } else { " " };
                    write!(f, "{separator}{slot}")?;
                }
                Ok(())
            }
        }
    }
}

/// One position in a bar. It displays as written.
#[derive(Debug, Clone, PartialEq, Eq, From, Display)]
pub enum Slot {
    /// A chord starting at this position
    Chord(Chord),
    /// `_`: the chord before continues; drawn as nothing
    #[from(skip)]
    #[display("_")]
    Hold,
    /// `/`: the chord before continues on this beat; drawn as a beat slash
    #[from(skip)]
    #[display("/")]
    Slash,
}

#[cfg(test)]
mod tests;
