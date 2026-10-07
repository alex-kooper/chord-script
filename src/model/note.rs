//! Notes as written on a chart, and the arithmetic that transposes them.
//!
//! Notes have no octave: they name a pitch up to octave equivalence, which is
//! all a chord root or bass needs.

use derive_more::Display;
use nutype::nutype;
use std::fmt;
use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum Letter {
    C,
    D,
    E,
    F,
    G,
    A,
    B,
}

impl Letter {
    fn semitones_above_c(self) -> Semitones {
        Semitones::wrapping(match self {
            Self::C => 0,
            Self::D => 2,
            Self::E => 4,
            Self::F => 5,
            Self::G => 7,
            Self::A => 9,
            Self::B => 11,
        })
    }
}

/// A single flat or sharp; charts do not use double accidentals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum Accidental {
    #[display("b")]
    Flat,
    #[display("#")]
    Sharp,
}

impl Accidental {
    /// How far the accidental moves its letter; a flat's step down is the
    /// rest of the octave up.
    fn shift(self) -> Semitones {
        Semitones::wrapping(match self {
            Self::Flat => -1,
            Self::Sharp => 1,
        })
    }
}

/// A note spelled as written: `Bb` and `A#` are different notes that sound
/// the same. Every letter takes either accidental, so `Cb` and `E#` are valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Note {
    pub letter: Letter,
    pub accidental: Option<Accidental>,
}

impl Note {
    pub const fn natural(letter: Letter) -> Self {
        Self {
            letter,
            accidental: None,
        }
    }

    pub const fn flat(letter: Letter) -> Self {
        Self {
            letter,
            accidental: Some(Accidental::Flat),
        }
    }

    pub const fn sharp(letter: Letter) -> Self {
        Self {
            letter,
            accidental: Some(Accidental::Sharp),
        }
    }

    /// Distance up from C, e.g. 10 for `Bb`, 11 for `Cb`, 0 for `B#`.
    pub fn semitones_above_c(self) -> Semitones {
        let letter = self.letter.semitones_above_c();
        self.accidental
            .map_or(letter, |accidental| letter + accidental.shift())
    }

    /// The note moved by `transposition`.
    ///
    /// A shifted note is respelled from its pitch alone: naturals where
    /// possible, otherwise the requested accidental. So `E` up one is `F`, and
    /// `Cb`, `Fb`, `E#`, `B#` never result from a shift.
    pub fn transpose(self, transposition: Transposition) -> Note {
        match transposition {
            Transposition::Unchanged => self,
            Transposition::Shift { up, spelling } => {
                spelling.note_at(self.semitones_above_c() + up)
            }
        }
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.letter)?;
        match self.accidental {
            Some(accidental) => write!(f, "{accidental}"),
            None => Ok(()),
        }
    }
}

/// How far `self` is above `other`: `D - C` is 2, `C - D` is 10.
impl Sub for Note {
    type Output = Semitones;

    fn sub(self, other: Note) -> Semitones {
        let above = i32::from(self.semitones_above_c().into_inner());
        let below = i32::from(other.semitones_above_c().into_inner());
        Semitones::wrapping(above - below)
    }
}

/// An upward distance between two notes, from 0 to 11 (an ordered
/// pitch-class interval). Going down is going up the rest of the octave.
#[nutype(
    validate(less_or_equal = 11),
    derive(
        Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Display, Into
    )
)]
pub struct Semitones(u8);

impl Semitones {
    /// Any number of semitones, ignoring whole octaves: -2 is 10.
    fn wrapping(semitones: i32) -> Self {
        let within_octave =
            u8::try_from(semitones.rem_euclid(12)).expect("a value modulo 12 is within 0..=11");
        Self::try_new(within_octave).expect("a value modulo 12 is within 0..=11")
    }
}

/// Adds modulo 12: 11 + 1 is 0.
impl Add for Semitones {
    type Output = Semitones;

    fn add(self, other: Semitones) -> Semitones {
        Semitones::wrapping(i32::from(self.into_inner()) + i32::from(other.into_inner()))
    }
}

/// Which accidental a transposed note uses when it needs one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Spelling {
    Sharps,
    Flats,
}

impl Spelling {
    fn note_at(self, above_c: Semitones) -> Note {
        let notes = match self {
            Self::Sharps => &SHARPS,
            Self::Flats => &FLATS,
        };
        notes[usize::from(above_c.into_inner())]
    }
}

const SHARPS: [Note; 12] = {
    use Letter::*;
    [
        Note::natural(C),
        Note::sharp(C),
        Note::natural(D),
        Note::sharp(D),
        Note::natural(E),
        Note::natural(F),
        Note::sharp(F),
        Note::natural(G),
        Note::sharp(G),
        Note::natural(A),
        Note::sharp(A),
        Note::natural(B),
    ]
};

const FLATS: [Note; 12] = {
    use Letter::*;
    [
        Note::natural(C),
        Note::flat(D),
        Note::natural(D),
        Note::flat(E),
        Note::natural(E),
        Note::natural(F),
        Note::flat(G),
        Note::natural(G),
        Note::flat(A),
        Note::natural(A),
        Note::flat(B),
        Note::natural(B),
    ]
};

/// A request to move notes, as in ChordPro: a number of semitones, with
/// sharps going up and flats going down unless a spelling is given.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Transposition {
    /// Notes stay exactly as written.
    Unchanged,
    /// Notes move up and are respelled; a shift of 0 only respells.
    Shift { up: Semitones, spelling: Spelling },
}

impl Transposition {
    /// `semitones` may be negative or span octaves: `-10` is up 2 using flats.
    /// Whole octaves (0, 12, -24, ...) without a spelling leave notes unchanged.
    pub fn new(semitones: i32, spelling: Option<Spelling>) -> Self {
        let up = Semitones::wrapping(semitones);
        let spelling = match spelling {
            Some(spelling) => spelling,
            None if up.into_inner() == 0 => return Self::Unchanged,
            None if semitones > 0 => Spelling::Sharps,
            None => Spelling::Flats,
        };
        Self::Shift { up, spelling }
    }
}

#[cfg(test)]
mod tests;
