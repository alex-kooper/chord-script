//! Bar rows: the grid of bars a chart's chords are written in.
//!
//! The model holds what the author wrote, whether or not it makes musical
//! sense: a bar may be empty or start with a hold, and `%` may stand in the
//! first bar of a chart.

use super::Chord;
use derive_more::From;
use vec1::Vec1;

/// The bars of one `| … |` line, left to right.
///
/// The row is drawn as bars of equal width, as many as it has.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BarRow {
    pub bars: Vec1<Bar>,
}

impl BarRow {
    pub fn new(bars: Vec1<Bar>) -> Self {
        Self { bars }
    }
}

/// What a bar holds.
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

/// One position in a bar.
#[derive(Debug, Clone, PartialEq, Eq, From)]
pub enum Slot {
    /// A chord starting at this position
    Chord(Chord),
    /// `_`: the chord before continues; drawn as nothing
    #[from(skip)]
    Hold,
    /// `/`: the chord before continues on this beat; drawn as a beat slash
    #[from(skip)]
    Slash,
}
