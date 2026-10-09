// Model module for chord-script domain types

use derive_more::From;

mod chord;
mod inline;
mod line;
mod note;

pub use chord::{Chord, ChordQuality, ChordQualityError, is_quality_char};
pub use inline::{Content, Inline, InlineText, InlineTextError, TextStyle, is_displayable};
pub use line::{Line, LineLevel};
pub use note::{Accidental, Letter, Note, Semitones, SemitonesError, Spelling, Transposition};

/// Represents a complete music chart
#[derive(Debug, Clone, PartialEq)]
pub struct Chart {
    /// The chart content, top to bottom
    pub blocks: Vec<Block>,
}

impl Chart {
    /// Creates a new chart with the given blocks
    pub fn new(blocks: Vec<Block>) -> Self {
        Self { blocks }
    }
}

/// One top-level element of a chart, in source order.
///
/// Blocks record what the author wrote, not how it lands on paper: physical
/// pages are decided by the renderer, which honors [`Block::PageBreak`] and
/// also breaks wherever content no longer fits.
#[derive(Debug, Clone, PartialEq, Eq, From)]
pub enum Block {
    /// A line of text
    Text(Line),
    /// An explicit request for a new page (`#page_break`)
    #[from(skip)]
    PageBreak,
}
