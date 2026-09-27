//! Errors that stop a chart from being rendered.

use crate::model::Line;
use thiserror::Error;

/// Result type alias for rendering operations
pub type Result<T> = std::result::Result<T, RenderError>;

/// Error returned when a chart cannot be rendered.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum RenderError {
    /// A single line needs more vertical space than an empty page offers, so
    /// no page can hold it.
    #[error("{line} is {height}pt tall, but a page has only {available}pt between its margins")]
    LineTallerThanPage {
        /// A short description of the line, for the message
        line: String,
        height: f64,
        available: f64,
    },
}

impl RenderError {
    pub(super) fn line_taller_than_page(line: &Line, height: f64, available: f64) -> Self {
        Self::LineTallerThanPage {
            line: describe(line),
            height,
            available,
        }
    }
}

/// Characters of line text shown in an error before it is cut off.
const PREVIEW_CHARS: usize = 40;

/// Describe a line by its text, since the model carries no source positions.
fn describe(line: &Line) -> String {
    let text: String = [&line.left, &line.center, &line.right]
        .into_iter()
        .flatten()
        .map(|span| -> &str { span.text.as_ref() })
        .collect();

    if text.is_empty() {
        return format!("an empty {:?} line", line.level);
    }
    let preview: String = text.chars().take(PREVIEW_CHARS).collect();
    let ellipsis = if text.chars().count() > PREVIEW_CHARS {
        "…"
    } else {
        ""
    };
    format!("line \"{preview}{ellipsis}\"")
}
