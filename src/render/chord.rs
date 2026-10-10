//! How a chord symbol is laid out, whatever the output format.
//!
//! The root is drawn at full size. Its accidental is small and raised at the
//! root's upper right; the quality is smaller and lowered, right after the
//! root, under the accidental. The bass, a slash and a note, either follows
//! on the root's baseline or hangs below it; see [`BassPlacement`].

mod symbols;

use super::fonts::{self, Font};
use crate::model::{Accidental, Chord, Note, TextStyle};
use symbols::{RunKind, accidental_sign, quality_runs};

/// Sizes and baseline rises as fractions of the chord's font size, set by eye.
const ROOT_ACCIDENTAL_SIZE: f64 = 0.95;
const ROOT_ACCIDENTAL_RISE: f64 = 0.36;
/// Relative to the root accidental: a ♯ reaches lower than a ♭, so it is
/// raised more to clear the quality.
const ROOT_SHARP_EXTRA_RISE: f64 = 0.19;
const QUALITY_SIZE: f64 = 0.6;
const QUALITY_RISE: f64 = -0.12;
/// Relative to the quality: a ♭ or ♯ inside it, such as the ♭ of `7♭9`
const QUALITY_ACCIDENTAL_SIZE: f64 = 1.5;
const QUALITY_ACCIDENTAL_RISE: f64 = 0.0;
const BASS_BELOW_SIZE: f64 = 0.6;
const BASS_BELOW_RISE: f64 = -0.55;
/// Where the slash starts, as a fraction of the root letter's width
const BASS_BELOW_INDENT: f64 = 0.5;
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
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "chord lines will hang the bass below")
)]
pub(super) enum BassPlacement {
    /// Hanging below the baseline, the slash tucked under the root, as iReal
    /// Pro draws it; this needs room below the line.
    Below,
    /// After the quality, a little smaller than the root and on its baseline:
    /// the chord fits in a line of text, and the bass stands apart from the
    /// quality.
    After,
}

/// The pieces of `chord` at `size` in `style`, in drawing order.
///
/// Each piece moves on from where the previous one ended, so the pieces are
/// ordered by their right edges: once all are drawn, the pen stands at the
/// chord's right edge and following text continues from there.
pub(super) fn layout(
    chord: &Chord,
    style: TextStyle,
    size: f64,
    placement: BassPlacement,
) -> Vec<Piece> {
    let root = Piece::new(chord.root.letter.to_string(), Font::Text(style), size, 0.0);
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
            pieces: quality_pieces(quality.as_ref(), style, size),
        });
    }
    if let Some(bass) = chord.bass {
        let stack = match placement {
            BassPlacement::Below => Stack {
                x: root_width * BASS_BELOW_INDENT,
                pieces: bass_pieces(bass, style, size * BASS_BELOW_SIZE, size * BASS_BELOW_RISE),
            },
            BassPlacement::After => Stack {
                x: stacks.iter().map(Stack::right).fold(root_width, f64::max),
                pieces: bass_pieces(bass, style, size * BASS_AFTER_SIZE, 0.0),
            },
        };
        stacks.push(stack);
    }
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

fn quality_pieces(quality: &str, style: TextStyle, chord_size: f64) -> Vec<Piece> {
    let size = chord_size * QUALITY_SIZE;
    let rise = chord_size * QUALITY_RISE;
    quality_runs(quality)
        .into_iter()
        .map(|run| match run.kind {
            RunKind::Text => Piece::new(run.text, Font::Text(style), size, rise),
            RunKind::Accidental => Piece::new(
                run.text,
                Font::Music,
                size * QUALITY_ACCIDENTAL_SIZE,
                rise + size * QUALITY_ACCIDENTAL_RISE,
            ),
        })
        .collect()
}

fn bass_pieces(bass: Note, style: TextStyle, size: f64, rise: f64) -> Vec<Piece> {
    let note = Piece::new(format!("/{}", bass.letter), Font::Text(style), size, rise);
    let accidental = bass
        .accidental
        .map(|accidental| accidental_piece(accidental, size * BASS_ACCIDENTAL_SIZE, rise));
    std::iter::once(note).chain(accidental).collect()
}

/// The horizontal moves of `pieces` added up: how much narrower the drawn
/// chord is than its pieces laid end to end.
pub(super) fn overlap(pieces: &[Piece]) -> f64 {
    -pieces.iter().map(|piece| piece.dx).sum::<f64>()
}

#[cfg(test)]
mod tests;
