// Model module for chord-script domain types

/// Represents a complete music chart
#[derive(Debug, Clone, PartialEq)]
pub struct Chart {
    /// The lines that make up the chart content
    pub lines: Vec<Line>,
}

impl Chart {
    /// Creates a new chart with the given lines
    pub fn new(lines: Vec<Line>) -> Self {
        Self { lines }
    }
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

/// A styled span of text
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextSpan {
    pub text: String,
    pub style: TextStyle,
}

impl TextSpan {
    pub fn new(text: impl Into<String>, style: TextStyle) -> Self {
        Self {
            text: text.into(),
            style,
        }
    }

    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: TextStyle::Normal,
        }
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