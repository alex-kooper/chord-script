//! Chords as written: a root, an optional quality kept as text, and an
//! optional bass.
//!
//! The quality is not interpreted: `maj7` and `^7` are different text for the
//! same chord, and both are kept as the author wrote them.

use super::note::{Note, Transposition};
use nutype::nutype;
use std::fmt;

/// Everything between the root and the bass, e.g. `m7b5`, `^7`, `o7`, `h7`,
/// `7(b9#13)`.
///
/// Made only of characters for which [`is_quality_char`] holds, and never
/// starting with `b` or `#`: right after the root those read as the root's
/// accidental, so a flat fifth is written `C(b5)` (or `C7b5` for the seventh
/// chord), not `Cb5`. Parentheses come in pairs, each with something inside
/// and none nested: `7(b9)(#11)`, not `7(` or `()`.
#[nutype(
    validate(not_empty, predicate = |quality: &str| is_valid_quality(quality)),
    derive(Debug, Clone, PartialEq, Eq, Hash, AsRef, Display)
)]
pub struct ChordQuality(String);

/// Whether `c` may appear in a chord quality: ASCII letters and digits, and
/// `#`, `^`, `-`, `+`, `(`, `)`.
pub fn is_quality_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '#' | '^' | '-' | '+' | '(' | ')')
}

fn is_valid_quality(quality: &str) -> bool {
    quality.chars().all(is_quality_char)
        && !quality.starts_with(['b', '#'])
        && has_paired_parentheses(quality)
}

fn has_paired_parentheses(quality: &str) -> bool {
    let mut open = false;
    let mut empty = true;
    for c in quality.chars() {
        match c {
            '(' if open => return false,
            '(' => {
                open = true;
                empty = true;
            }
            ')' if !open || empty => return false,
            ')' => open = false,
            _ => empty = false,
        }
    }
    !open
}

/// A chord symbol such as `Bbh7/E`: root `Bb`, quality `h7`, bass `E`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Chord {
    pub root: Note,
    /// `None` is a major triad, e.g. `C`.
    pub quality: Option<ChordQuality>,
    /// The note after the slash, e.g. the `E` of `C/E`.
    pub bass: Option<Note>,
}

impl Chord {
    pub fn new(root: Note, quality: Option<ChordQuality>, bass: Option<Note>) -> Self {
        Self {
            root,
            quality,
            bass,
        }
    }

    /// The chord with its root and bass moved; the quality is unchanged.
    pub fn transpose(self, transposition: Transposition) -> Self {
        Self {
            root: self.root.transpose(transposition),
            quality: self.quality,
            bass: self.bass.map(|bass| bass.transpose(transposition)),
        }
    }
}

/// The chord in ASCII, as it would be typed: `Bbh7/E`.
impl fmt::Display for Chord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.root)?;
        if let Some(quality) = &self.quality {
            write!(f, "{quality}")?;
        }
        if let Some(bass) = self.bass {
            write!(f, "/{bass}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Letter, Spelling};

    fn quality(text: &str) -> ChordQuality {
        ChordQuality::try_new(text).unwrap_or_else(|e| panic!("{text:?} should be valid: {e}"))
    }

    fn chord(root: Note, text: Option<&str>, bass: Option<Note>) -> Chord {
        Chord::new(root, text.map(quality), bass)
    }

    #[test]
    fn quality_accepts_common_chord_symbols() {
        for text in [
            "m", "-7", "m7b5", "^7", "maj7", "7#9", "+", "sus4", "add9", "o7", "h7", "13", "7(b9)",
            "7(b9#13)", "5",
        ] {
            assert!(ChordQuality::try_new(text).is_ok(), "{text:?}");
        }
    }

    #[test]
    fn quality_rejects_reserved_and_non_ascii_characters() {
        for text in [
            "", "7 9", "7,9", "7_9", "m7/5", "7]", "7|", "7*", "Δ7", "7♭9",
        ] {
            assert!(ChordQuality::try_new(text).is_err(), "{text:?}");
        }
    }

    #[test]
    fn quality_cannot_start_with_an_accidental() {
        for text in ["b5", "#9", "bb"] {
            assert!(ChordQuality::try_new(text).is_err(), "{text:?}");
        }
        assert!(ChordQuality::try_new("(b5)").is_ok());
    }

    #[test]
    fn parentheses_are_paired_non_empty_and_flat() {
        for text in ["(b5)", "7(b9)", "7(b9#13)", "7(b9)(#11)"] {
            assert!(ChordQuality::try_new(text).is_ok(), "{text:?}");
        }
        for text in [")", "(", "7(", "7)", "()", "7)(b9", "7((b9))", "7(b9"] {
            assert!(ChordQuality::try_new(text).is_err(), "{text:?}");
        }
    }

    #[test]
    fn uppercase_qualities_are_allowed() {
        for text in ["M7", "M", "E"] {
            assert!(ChordQuality::try_new(text).is_ok(), "{text:?}");
        }
    }

    #[test]
    fn display_is_the_typed_symbol() {
        let b_flat = Note::flat(Letter::B);
        let e = Note::natural(Letter::E);
        assert_eq!(chord(Note::natural(Letter::C), None, None).to_string(), "C");
        assert_eq!(chord(b_flat, Some("h7"), Some(e)).to_string(), "Bbh7/E");
        assert_eq!(
            chord(Note::natural(Letter::C), None, Some(e)).to_string(),
            "C/E"
        );
        let c = Note::natural(Letter::C);
        assert_eq!(chord(c, Some("(b5)"), None).to_string(), "C(b5)");
        assert_eq!(chord(c, Some("7(b9)"), Some(e)).to_string(), "C7(b9)/E");
    }

    #[test]
    fn transpose_moves_root_and_bass_but_not_the_quality() {
        let original = chord(
            Note::flat(Letter::B),
            Some("m7b5"),
            Some(Note::natural(Letter::E)),
        );
        let up_two = original.transpose(Transposition::new(2, None));
        assert_eq!(up_two.to_string(), "Cm7b5/F#");

        let original = chord(Note::natural(Letter::A), Some("7#9"), None);
        let down_one = original.transpose(Transposition::new(-1, None));
        assert_eq!(down_one.to_string(), "Ab7#9");
    }

    #[test]
    fn root_and_bass_share_the_spelling() {
        let original = chord(
            Note::natural(Letter::C),
            None,
            Some(Note::natural(Letter::G)),
        );
        let up_one_in_flats = original.transpose(Transposition::new(1, Some(Spelling::Flats)));
        assert_eq!(up_one_in_flats.to_string(), "Db/Ab");
    }

    #[test]
    fn unchanged_keeps_the_chord_as_written() {
        let original = chord(
            Note::sharp(Letter::E),
            Some("^7"),
            Some(Note::flat(Letter::C)),
        );
        assert_eq!(
            original.clone().transpose(Transposition::Unchanged),
            original
        );
    }
}
