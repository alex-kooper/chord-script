//! Bar lines: what stands at each end of a row and between its cells.

use nutype::nutype;
use std::fmt;

/// A bar line. It displays as written.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BarLine {
    /// `|`
    #[default]
    Single,
    /// `||`: a section starts or ends here
    Double,
    /// `|||`: the piece ends here
    Final,
    /// `||:`: a repeat starts here
    RepeatStart,
    /// `:||`: a repeat ends here, `:||x3` with how many times it is played
    RepeatEnd(Option<RepeatCount>),
    /// `:||:`: one repeat ends and the next starts here, `:||:x3` with how
    /// many times the one ending is played
    RepeatBoth(Option<RepeatCount>),
}

impl BarLine {
    /// How many times the repeat ending here is played, if written.
    pub fn count(self) -> Option<RepeatCount> {
        match self {
            BarLine::RepeatEnd(count) | BarLine::RepeatBoth(count) => count,
            _ => None,
        }
    }
}

impl fmt::Display for BarLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = match self {
            BarLine::Single => "|",
            BarLine::Double => "||",
            BarLine::Final => "|||",
            BarLine::RepeatStart => "||:",
            BarLine::RepeatEnd(_) => ":||",
            BarLine::RepeatBoth(_) => ":||:",
        };
        write!(f, "{sign}")?;
        match self.count() {
            Some(count) => write!(f, "{count}"),
            None => Ok(()),
        }
    }
}

/// `x3`: how many times a repeat is played, from [`RepeatCount::MIN`] to
/// [`RepeatCount::MAX`]. Once is no repeat, and twice is what a repeat sign
/// means without a count. It displays as written.
#[nutype(
    validate(predicate = |count| (RepeatCount::MIN..=RepeatCount::MAX).contains(count)),
    derive(Debug, Clone, Copy, PartialEq, Eq)
)]
pub struct RepeatCount(u8);

impl RepeatCount {
    pub const MIN: u8 = 2;
    /// More than any chart is likely to need: a larger count is a typo.
    pub const MAX: u8 = 99;
}

impl fmt::Display for RepeatCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "x{}", self.into_inner())
    }
}
