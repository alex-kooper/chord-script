//! A bass hanging below the chord: a slash drawn as a line under the root,
//! flatter than the font's `/`, and the note tucked under its upper end.

use super::{Offset, Piece, Slash, Stack, accidental_piece};
use crate::model::{Accidental, Note};
use crate::render::fonts::Font;

/// Sizes and baseline rises as fractions of the chord's font size, set by eye.
const BASS_SIZE: f64 = 0.6;
/// Low enough for the bass to clear the quality, parentheses included
const BASS_RISE: f64 = -0.62;
/// Relative to the bass: no taller than its letter, so it stays clear of the
/// quality above
const ACCIDENTAL_SIZE: f64 = 1.0;
/// Relative to the bass accidental: a ♯ reaches below the baseline, so it is
/// raised to sit about on it.
const SHARP_EXTRA_RISE: f64 = 0.15;
/// Where the slash starts, as a fraction of the root letter's width
const SLASH_INDENT: f64 = 0.4;
/// Where the slash's lower end sits, below the chord's baseline
const SLASH_RISE: f64 = -0.44;
/// The rest of the slash, as fractions of the bass size; clear of the quality
const SLASH_HEIGHT: f64 = 0.62;
/// How far right it leans over its height: about 50° from the horizontal,
/// where the font's `/` stands at 70°
const SLASH_LEAN: f64 = 0.85;
/// Light, like the beat slashes: a long diagonal looks heavier than a stem
const SLASH_WIDTH: f64 = 0.08;
/// How far left of the slash's upper end the note starts, tucked under it
const SLASH_TUCK: f64 = 0.1;

/// `bass` hanging under a root `root_width` wide in a chord at `size`, its
/// letter in `letters`, and the slash leading to it.
pub(super) fn bass(bass: Note, letters: Font, size: f64, root_width: f64) -> (Stack, Slash) {
    let (slash, note_start) = slash(root_width, size);
    let note_size = size * BASS_SIZE;
    let rise = size * BASS_RISE;
    let note = Piece::new(bass.letter.to_string(), letters, note_size, rise);
    let accidental = bass.accidental.map(|accidental| {
        let size = note_size * ACCIDENTAL_SIZE;
        let extra = match accidental {
            Accidental::Flat => 0.0,
            Accidental::Sharp => size * SHARP_EXTRA_RISE,
        };
        accidental_piece(accidental, size, rise + extra)
    });
    let stack = Stack {
        x: note_start,
        pieces: std::iter::once(note).chain(accidental).collect(),
    };
    (stack, slash)
}

/// The slash under a root `root_width` wide, and where the note after it
/// starts.
fn slash(root_width: f64, size: f64) -> (Slash, f64) {
    let bass_size = size * BASS_SIZE;
    let from = Offset {
        x: root_width * SLASH_INDENT,
        rise: size * SLASH_RISE,
    };
    let height = bass_size * SLASH_HEIGHT;
    let to = Offset {
        x: from.x + height * SLASH_LEAN,
        rise: from.rise + height,
    };
    let slash = Slash {
        from,
        to,
        width: bass_size * SLASH_WIDTH,
    };
    (slash, to.x - bass_size * SLASH_TUCK)
}
