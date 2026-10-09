//! The pieces of a line's columns: styled text and inline chords.

use super::Chord;
use derive_more::From;
use nutype::nutype;

/// One piece of a column, in reading order: its content and the style it is
/// drawn in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inline {
    pub content: Content,
    pub style: TextStyle,
}

/// What an [`Inline`] holds.
#[derive(Debug, Clone, PartialEq, Eq, From)]
pub enum Content {
    /// Literal text
    Text(InlineText),
    /// A chord written inline, as in `Key of [Bb] minor`. A bare note is a
    /// chord with no quality and no bass.
    Chord(Chord),
}

impl Inline {
    pub fn new(content: impl Into<Content>, style: TextStyle) -> Self {
        Self {
            content: content.into(),
            style,
        }
    }

    /// Styled text. Panics if `text` is empty or not displayable; see
    /// [`InlineText`].
    pub fn text(text: impl Into<String>, style: TextStyle) -> Self {
        Self::try_text(text, style).expect("inline text must be non-empty and displayable")
    }

    /// Unstyled text. Panics if `text` is empty or not displayable; see
    /// [`InlineText`].
    pub fn plain(text: impl Into<String>) -> Self {
        Self::text(text, TextStyle::Normal)
    }

    /// Styled text, or `None` if `text` is empty or not displayable.
    pub fn try_text(text: impl Into<String>, style: TextStyle) -> Option<Self> {
        InlineText::try_new(text.into())
            .ok()
            .map(|text| Self::new(text, style))
    }
}

/// Non-empty text content of an inline, made only of characters for which
/// [`is_displayable`] holds.
///
/// Whitespace is kept as-is: the spaces in `a *b* c` live at the edges of the
/// plain text, and trimming them would glue the words together.
#[nutype(
    validate(not_empty, predicate = |text: &str| text.chars().all(is_displayable)),
    derive(Debug, Clone, PartialEq, Eq, AsRef)
)]
pub struct InlineText(String);

/// Whether `c` may appear in chart text.
///
/// Control characters other than tab, and the noncharacters U+FFFE and
/// U+FFFF, are excluded: they have no visible form, and text formats such as
/// XML cannot carry them.
pub fn is_displayable(c: char) -> bool {
    c == '\t' || !(c.is_control() || matches!(c, '\u{FFFE}' | '\u{FFFF}'))
}

/// The style an inline is drawn in
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_accepts_any_script_and_tabs() {
        for text in ["Am7", "Приспів: Ґ Ї", "a\tb", "😀"] {
            assert!(
                Inline::try_text(text, TextStyle::Normal).is_some(),
                "{text:?}"
            );
        }
    }

    #[test]
    fn text_rejects_undisplayable_characters() {
        for text in [
            "",
            "a\u{0}b",
            "a\u{1}",
            "\u{1B}[0m",
            "a\nb",
            "\u{7F}",
            "\u{85}",
            "\u{FFFF}",
        ] {
            assert!(
                Inline::try_text(text, TextStyle::Normal).is_none(),
                "{text:?}"
            );
        }
    }
}
