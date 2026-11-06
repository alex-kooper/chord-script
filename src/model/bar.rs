//! Bar rows: the grid of bars a chart's chords are written in.
//!
//! The model holds what the author wrote, whether or not it makes musical
//! sense: a bar may be empty or start with a hold, `%` may stand in the first
//! bar of a chart, and a row may be nothing but spans.

use super::Chord;
use derive_more::{Display, From};
use nutype::nutype;
use std::fmt;
use std::num::NonZeroUsize;
use vec1::Vec1;

/// The cells of one `| … |` line, left to right: bars, and spans standing in
/// for bars that are not drawn.
///
/// The row is drawn as cells one bar wide, a span as many as it stands for.
/// It displays as written, with single spaces: `| {2} | Am _ G | % |`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BarRow {
    pub cells: Vec1<Cell>,
}

impl BarRow {
    pub fn new(cells: Vec1<impl Into<Cell>>) -> Self {
        Self {
            cells: cells.mapped(Into::into),
        }
    }

    /// How many bars wide the row is, counting the bars its spans stand for.
    pub fn width(&self) -> NonZeroUsize {
        self.cells
            .iter()
            .map(Cell::width)
            .reduce(|total, width| total.saturating_add(width.get()))
            .expect("a row has at least one cell")
    }

    /// Each bar of the row, left to right, with how many bars wide the row is
    /// before it, spans included.
    pub fn bars(&self) -> impl Iterator<Item = (usize, &Bar)> {
        self.cells
            .iter()
            .scan(0, |next, cell| {
                let start = *next;
                *next += cell.width().get();
                Some((start, cell))
            })
            .filter_map(|(start, cell)| match cell {
                Cell::Bar(bar) => Some((start, bar)),
                Cell::Span(_) => None,
            })
    }
}

impl fmt::Display for BarRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for cell in &self.cells {
            match cell {
                Cell::Bar(Bar::Slots(slots)) if slots.is_empty() => write!(f, "| ")?,
                cell => write!(f, "| {cell} ")?,
            }
        }
        write!(f, "|")
    }
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
mod tests {
    use super::*;
    use crate::model::{Letter, Note};
    use vec1::vec1;

    fn span(width: usize) -> Cell {
        let width = NonZeroUsize::new(width).expect("a positive width");
        Span::try_new(width)
            .expect("a width within the limit")
            .into()
    }

    #[test]
    fn rows_display_as_written() {
        let chord = |letter| Slot::Chord(Chord::new(Note::natural(letter), None, None));
        let row = BarRow::new(vec1![
            span(2),
            Bar::Slots(vec![
                chord(Letter::A),
                Slot::Hold,
                chord(Letter::G),
                Slot::Slash
            ])
            .into(),
            Bar::Repeat.into(),
            Bar::Slots(Vec::new()).into(),
        ]);
        assert_eq!(row.to_string(), "| {2} | A _ G / | % | |");
    }

    #[test]
    fn a_row_is_as_wide_as_its_bars_and_the_bars_its_spans_stand_for() {
        let row = BarRow::new(vec1![span(2), Bar::Repeat.into(), span(3)]);
        assert_eq!(row.width().get(), 6);
        assert_eq!(BarRow::new(vec1![Bar::Repeat]).width().get(), 1);
    }

    #[test]
    fn each_bar_starts_after_the_cells_before_it() {
        let empty = || Cell::from(Bar::Slots(Vec::new()));
        let row = BarRow::new(vec1![
            span(2),
            Bar::Repeat.into(),
            empty(),
            span(3),
            empty()
        ]);
        let starts: Vec<(usize, &Bar)> = row.bars().collect();
        let empty_bar = Bar::Slots(Vec::new());
        assert_eq!(
            starts,
            [(2, &Bar::Repeat), (3, &empty_bar), (7, &empty_bar)]
        );
    }

    #[test]
    fn a_span_is_at_most_its_max_width() {
        let width = |bars| NonZeroUsize::new(bars).expect("a positive width");
        assert!(Span::try_new(width(Span::MAX_WIDTH)).is_ok());
        assert!(Span::try_new(width(Span::MAX_WIDTH + 1)).is_err());
    }
}
