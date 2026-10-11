//! Bar rows: the grid of bars a chart's chords are written in.
//!
//! The model holds what the author wrote, whether or not it makes musical
//! sense: a bar may be empty or start with a hold, and `%` may stand in the
//! first bar of a chart.

use super::Chord;
use derive_more::{Display, From};
use std::fmt;
use vec1::Vec1;

/// The bars of one `| … |` line, left to right.
///
/// The row is drawn as bars of equal width, as many as it has. It displays as
/// written, with single spaces: `| Am _ G | % |`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BarRow {
    pub bars: Vec1<Bar>,
}

impl BarRow {
    pub fn new(bars: Vec1<Bar>) -> Self {
        Self { bars }
    }
}

impl fmt::Display for BarRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for bar in &self.bars {
            match bar {
                Bar::Slots(slots) if slots.is_empty() => write!(f, "| ")?,
                bar => write!(f, "| {bar} ")?,
            }
        }
        write!(f, "|")
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
    /// `%`: the previous bar in the chart again, skipping any text lines and
    /// page breaks in between
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

    #[test]
    fn rows_display_as_written() {
        let chord = |letter| Slot::Chord(Chord::new(Note::natural(letter), None, None));
        let row = BarRow::new(vec1![
            Bar::Slots(vec![
                chord(Letter::A),
                Slot::Hold,
                chord(Letter::G),
                Slot::Slash
            ]),
            Bar::Repeat,
            Bar::Slots(Vec::new()),
        ]);
        assert_eq!(row.to_string(), "| A _ G / | % | |");
    }
}
