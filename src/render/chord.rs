//! How a chord symbol is laid out, whatever the output format.
//!
//! The root is drawn at full size. Its accidental is small and raised at the
//! root's upper right; the quality is smaller and lowered, right after the
//! root, under the accidental. The bass, a slash and a note, either follows
//! on the root's baseline or hangs below it; see [`BassPlacement`]. A hanging
//! bass's slash is drawn as a line, flatter than the font's `/`; see
//! [`ChordLayout::slash`].

mod hanging;
mod symbols;

use super::fonts::{self, Font};
use crate::model::{Accidental, Chord, Note};
use symbols::{RunKind, accidental_sign, quality_runs};

/// Sizes and baseline rises as fractions of the chord's font size, set by eye.
const ROOT_ACCIDENTAL_SIZE: f64 = 0.95;
/// High enough to clear digits in the quality, which stand taller than `m`
const ROOT_ACCIDENTAL_RISE: f64 = 0.5;
/// Relative to the root accidental: a ♯ reaches lower than a ♭, so it is
/// raised more to clear the quality.
const ROOT_SHARP_EXTRA_RISE: f64 = 0.19;
const QUALITY_SIZE: f64 = 0.6;
const QUALITY_RISE: f64 = -0.04;
/// Relative to the quality: a ♭ or ♯ inside it, such as the ♭ of `7♭9`
const QUALITY_ACCIDENTAL_SIZE: f64 = 1.5;
const QUALITY_ACCIDENTAL_RISE: f64 = 0.0;
const BASS_AFTER_SIZE: f64 = 0.8;
/// Relative to the bass note
const BASS_ACCIDENTAL_SIZE: f64 = 1.5;

/// One run of a chord symbol in a single font and size.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Piece {
    pub text: String,
    pub font: Font,
    pub size: f64,
    /// The horizontal move from where the previous piece ended to where this
    /// one starts; negative moves back under what is already drawn.
    pub dx: f64,
    /// How far the baseline is raised; negative lowers it.
    pub rise: f64,
}

impl Piece {
    fn new(text: impl Into<String>, font: Font, size: f64, rise: f64) -> Self {
        Self {
            text: text.into(),
            font,
            size,
            dx: 0.0,
            rise,
        }
    }

    fn width(&self) -> f64 {
        fonts::advance(&self.text, self.font, self.size)
    }
}

/// A point of a chord, `x` after the start of the root's baseline and `rise`
/// above it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Offset {
    pub x: f64,
    pub rise: f64,
}

/// A straight line drawn with a chord's pieces, as wide as `width`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Slash {
    pub from: Offset,
    pub to: Offset,
    pub width: f64,
}

/// Pieces drawn one after another, starting `x` after the root's start.
struct Stack {
    x: f64,
    pieces: Vec<Piece>,
}

impl Stack {
    fn right(&self) -> f64 {
        self.x + self.pieces.iter().map(Piece::width).sum::<f64>()
    }
}

/// Where a chord's bass goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BassPlacement {
    /// Hanging below the baseline, the slash tucked under the root, as iReal
    /// Pro draws it; this needs room below the line. The pieces leave a gap
    /// for the slash, which comes with them in [`ChordLayout::slash`].
    Below,
    /// After the quality, a little smaller than the root and on its baseline:
    /// the chord fits in a line of text, and the bass stands apart from the
    /// quality.
    After,
}

/// A chord laid out for drawing.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct ChordLayout {
    /// In drawing order: each piece moves on from where the previous one
    /// ended, so the pieces are ordered by their right edges. Once all are
    /// drawn, the pen stands at the chord's right edge and following text
    /// continues from there.
    pub pieces: Vec<Piece>,
    /// Drawn with the pieces when the bass hangs below.
    pub slash: Option<Slash>,
}

/// `chord` laid out at `size`, its letters in `letters`.
pub(super) fn layout(
    chord: &Chord,
    letters: Font,
    size: f64,
    placement: BassPlacement,
) -> ChordLayout {
    let root = Piece::new(chord.root.letter.to_string(), letters, size, 0.0);
    let root_width = root.width();

    let mut stacks = Vec::new();
    if let Some(accidental) = chord.root.accidental {
        let sign = root_accidental_piece(accidental, size);
        stacks.push(Stack {
            x: root_width,
            pieces: vec![sign],
        });
    }
    if let Some(quality) = &chord.quality {
        stacks.push(Stack {
            x: root_width,
            pieces: quality_pieces(quality.as_ref(), letters, size),
        });
    }
    let mut slash = None;
    if let Some(bass) = chord.bass {
        stacks.push(match placement {
            BassPlacement::Below => {
                let (stack, line) = hanging::bass(bass, letters, size, root_width);
                slash = Some(line);
                stack
            }
            BassPlacement::After => {
                let after = stacks.iter().map(Stack::right).fold(root_width, f64::max);
                bass_after(bass, letters, size, after)
            }
        });
    }
    ChordLayout {
        pieces: in_drawing_order(root, stacks),
        slash,
    }
}

/// `root` followed by `stacks`, ordered by their right edges.
fn in_drawing_order(root: Piece, mut stacks: Vec<Stack>) -> Vec<Piece> {
    let root_width = root.width();
    stacks.sort_by(|a, b| a.right().total_cmp(&b.right()));
    let mut pieces = vec![root];
    let mut pen = root_width;
    for mut stack in stacks {
        stack.pieces[0].dx = stack.x - pen;
        pen = stack.right();
        pieces.append(&mut stack.pieces);
    }
    pieces
}

fn root_accidental_piece(accidental: Accidental, chord_size: f64) -> Piece {
    let size = chord_size * ROOT_ACCIDENTAL_SIZE;
    let extra = match accidental {
        Accidental::Flat => 0.0,
        Accidental::Sharp => size * ROOT_SHARP_EXTRA_RISE,
    };
    accidental_piece(accidental, size, chord_size * ROOT_ACCIDENTAL_RISE + extra)
}

fn accidental_piece(accidental: Accidental, size: f64, rise: f64) -> Piece {
    Piece::new(accidental_sign(accidental), Font::Music, size, rise)
}

fn quality_pieces(quality: &str, letters: Font, chord_size: f64) -> Vec<Piece> {
    let size = chord_size * QUALITY_SIZE;
    let rise = chord_size * QUALITY_RISE;
    quality_runs(quality)
        .into_iter()
        .map(|run| match run.kind {
            RunKind::Text => Piece::new(run.text, letters, size, rise),
            RunKind::Accidental => Piece::new(
                run.text,
                Font::Music,
                size * QUALITY_ACCIDENTAL_SIZE,
                rise + size * QUALITY_ACCIDENTAL_RISE,
            ),
        })
        .collect()
}

/// The bass of a chord at `size` with the font's slash, at `after`, where the
/// rest of the chord ends.
fn bass_after(bass: Note, letters: Font, size: f64, after: f64) -> Stack {
    let size = size * BASS_AFTER_SIZE;
    let slash = Piece::new("/", letters, size, 0.0);
    let note = Piece::new(bass.letter.to_string(), letters, size, 0.0);
    let accidental = bass
        .accidental
        .map(|accidental| accidental_piece(accidental, size * BASS_ACCIDENTAL_SIZE, 0.0));
    Stack {
        x: after,
        pieces: [slash, note].into_iter().chain(accidental).collect(),
    }
}

/// The horizontal moves of `pieces` added up: how much narrower the drawn
/// chord is than its pieces laid end to end.
pub(super) fn overlap(pieces: &[Piece]) -> f64 {
    -pieces.iter().map(|piece| piece.dx).sum::<f64>()
}

/// How wide the chord drawn from `pieces` is, from the root's start to the
/// rightmost piece's end.
pub(super) fn width(pieces: &[Piece]) -> f64 {
    pieces.iter().map(Piece::width).sum::<f64>() - overlap(pieces)
}

#[cfg(test)]
mod tests;
