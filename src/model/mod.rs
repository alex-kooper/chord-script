// Model module for chord-script domain types

use derive_more::From;
use nutype::nutype;

/// Non-empty text content for a styled span.
///
/// Whitespace is kept as-is: the spaces in `a *b* c` live at the edges of the
/// plain spans, and trimming them would glue the words together.
#[nutype(validate(not_empty), derive(Debug, Clone, PartialEq, Eq, AsRef))]
pub struct SpanText(String);

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

/// Text styling options for span of text
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextStyle {
    /// Normal, unstyled text
    Normal,
    /// Bold text
    Bold,
    /// Italic text
    Italic,
    /// Bold and italic text
    BoldItalic,
}

impl TextStyle {
    /// The style with both `self` and `other` applied, e.g. `Bold` + `Italic`
    /// is `BoldItalic`. Order does not matter.
    pub fn combine(self, other: TextStyle) -> TextStyle {
        Self::from_flags(
            self.is_bold() || other.is_bold(),
            self.is_italic() || other.is_italic(),
        )
    }

    fn is_bold(self) -> bool {
        matches!(self, Self::Bold | Self::BoldItalic)
    }

    fn is_italic(self) -> bool {
        matches!(self, Self::Italic | Self::BoldItalic)
    }

    fn from_flags(bold: bool, italic: bool) -> TextStyle {
        match (bold, italic) {
            (false, false) => Self::Normal,
            (true, false) => Self::Bold,
            (false, true) => Self::Italic,
            (true, true) => Self::BoldItalic,
        }
    }
}

/// A styled span of text
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextSpan {
    pub text: SpanText,
    pub style: TextStyle,
}

impl TextSpan {
    pub fn new(text: impl Into<String>, style: TextStyle) -> Self {
        Self {
            text: SpanText::try_new(text.into()).expect("TextSpan text must not be empty"),
            style,
        }
    }

    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: SpanText::try_new(text.into()).expect("TextSpan text must not be empty"),
            style: TextStyle::Normal,
        }
    }

    /// Fallible constructor — returns `None` if text is empty
    pub fn try_new(text: impl Into<String>, style: TextStyle) -> Option<Self> {
        SpanText::try_new(text.into())
            .ok()
            .map(|text| Self { text, style })
    }

    /// Fallible plain text constructor
    pub fn try_plain(text: impl Into<String>) -> Option<Self> {
        Self::try_new(text, TextStyle::Normal)
    }
}

/// Line level in the hierarchy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineLevel {
    /// Header level 1 (major section)
    Header1,
    /// Header level 2 (subsection)
    Header2,
    /// Header level 3 (detail)
    Header3,
    /// Text line (stage directions, comments)
    Text,
}

/// A line in a chart with three-column layout (left, center, right aligned)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub level: LineLevel,
    pub left: Vec<TextSpan>,
    pub center: Vec<TextSpan>,
    pub right: Vec<TextSpan>,
}
impl Line {
    /// Create a new line with explicit columns and level
    pub fn new(
        level: LineLevel,
        left: Vec<TextSpan>,
        center: Vec<TextSpan>,
        right: Vec<TextSpan>,
    ) -> Self {
        Self {
            level,
            left,
            center,
            right,
        }
    }

    /// Whether the line has no text in any column, e.g. a bare `===` spacer.
    pub fn is_empty(&self) -> bool {
        self.left.is_empty() && self.center.is_empty() && self.right.is_empty()
    }

    /// Create a line with plain text in each column (Normal style)
    pub fn plain_text(
        level: LineLevel,
        left: impl Into<String>,
        center: impl Into<String>,
        right: impl Into<String>,
    ) -> Self {
        Self {
            level,
            left: vec![TextSpan::plain(left)],
            center: vec![TextSpan::plain(center)],
            right: vec![TextSpan::plain(right)],
        }
    }
}
