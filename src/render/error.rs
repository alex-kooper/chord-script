//! Errors that stop a chart from being rendered, in any output format.

use super::BarRowStyle;
use super::inline::typed_text;
use crate::model::{BarRow, Line};
use thiserror::Error;

/// Result type alias for rendering operations
pub type Result<T> = std::result::Result<T, RenderError>;

/// Error returned when a chart cannot be rendered.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum RenderError {
    /// A single row, a line of text or a row of bars, needs more vertical
    /// space than an empty page offers, so no page can hold it.
    #[error("{row} is {height}pt tall, but a page has only {available}pt between its margins")]
    RowTallerThanPage {
        /// A short description of the row, for the message
        row: String,
        height: f64,
        available: f64,
    },

    /// The configured page cannot exist, e.g. it has zero or negative width.
    #[error("invalid page size {width}pt × {height}pt: both sides must be positive")]
    InvalidPageSize { width: f64, height: f64 },

    /// A bar row cannot be drawn: its chord size is not positive, or its gap
    /// is negative.
    #[error(
        "invalid bar row style: chord size {chord_size}pt must be a positive number and gap {gap}pt a non-negative one"
    )]
    InvalidBarRowStyle { chord_size: f64, gap: f64 },

    /// No family in the configured font family list is a bundled font, so a
    /// PDF would have no text.
    #[error("none of the fonts `{family}` is available; PDF output can only use `{bundled}`")]
    FontUnavailable {
        family: String,
        bundled: &'static str,
    },

    /// The text uses a character that no bundled font can draw, so a PDF
    /// would silently lose it.
    #[error(
        "{line} contains `{character}` ({}), which the bundled font cannot draw",
        code_point(.character)
    )]
    UnsupportedCharacter { line: String, character: char },

    /// The PDF writer rejected the document, e.g. a font could not be embedded.
    #[error("could not write PDF: {message}")]
    PdfExport { message: String },
}

impl RenderError {
    pub(super) fn invalid_bar_row_style(style: &BarRowStyle) -> Self {
        Self::InvalidBarRowStyle {
            chord_size: style.chord_size,
            gap: style.gap,
        }
    }

    pub(super) fn line_taller_than_page(line: &Line, height: f64, available: f64) -> Self {
        Self::RowTallerThanPage {
            row: describe(line),
            height,
            available,
        }
    }

    pub(super) fn bar_row_taller_than_page(row: &BarRow, height: f64, available: f64) -> Self {
        Self::RowTallerThanPage {
            row: format!("bar row \"{}\"", preview(&row.to_string())),
            height,
            available,
        }
    }

    pub(super) fn unsupported_character(line: &Line, character: char) -> Self {
        Self::UnsupportedCharacter {
            line: describe(line),
            character,
        }
    }
}

fn code_point(character: &char) -> String {
    format!("U+{:04X}", u32::from(*character))
}

/// Characters of a row's text shown in an error before it is cut off.
const PREVIEW_CHARS: usize = 40;

/// Describe a line by its text, since the model carries no source positions.
fn describe(line: &Line) -> String {
    let text: String = line.inlines().map(typed_text).collect();

    if text.is_empty() {
        return format!("an empty {:?} line", line.level);
    }
    format!("line \"{}\"", preview(&text))
}

/// The start of `text`, with an ellipsis if it is cut off.
fn preview(text: &str) -> String {
    let start: String = text.chars().take(PREVIEW_CHARS).collect();
    if text.chars().count() > PREVIEW_CHARS {
        start + "…"
    } else {
        start
    }
}
