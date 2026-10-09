//! Text lines: a level and three columns of inline text and chords.

use super::Inline;

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
    pub left: Vec<Inline>,
    pub center: Vec<Inline>,
    pub right: Vec<Inline>,
}

impl Line {
    /// Create a new line with explicit columns and level
    pub fn new(
        level: LineLevel,
        left: Vec<Inline>,
        center: Vec<Inline>,
        right: Vec<Inline>,
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

    /// Every inline of the line: left column, then center, then right.
    pub fn inlines(&self) -> impl Iterator<Item = &Inline> {
        [&self.left, &self.center, &self.right]
            .into_iter()
            .flatten()
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
            left: vec![Inline::plain(left)],
            center: vec![Inline::plain(center)],
            right: vec![Inline::plain(right)],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inlines_run_left_to_right_across_columns() {
        let line = Line::plain_text(LineLevel::Text, "l", "c", "r");
        let inlines: Vec<&Inline> = line.inlines().collect();
        assert_eq!(
            inlines,
            [
                &Inline::plain("l"),
                &Inline::plain("c"),
                &Inline::plain("r")
            ]
        );
    }
}
