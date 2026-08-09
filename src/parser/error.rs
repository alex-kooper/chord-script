//! Structured parse diagnostics and the error they compose into.
//!
//! Nothing here refers to the parsing library: diagnostics are owned data, so a
//! `ParseError` outlives the borrowed input it was produced from.

use ariadne::{Color, Label, Report, ReportKind, Source};
use std::ops::Range;
use thiserror::Error;

/// Result type alias for parser operations
pub type Result<T> = std::result::Result<T, ParseError>;

/// A single diagnostic produced while parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Diagnostic {
    message: String,
    span: Range<usize>,
    contexts: Vec<(String, Range<usize>)>,
}

impl Diagnostic {
    pub(super) fn new(
        message: String,
        span: Range<usize>,
        contexts: Vec<(String, Range<usize>)>,
    ) -> Self {
        Self {
            message,
            span,
            contexts,
        }
    }
}

/// Error returned when a chart fails to parse.
///
/// Carries the original source plus every diagnostic the grammar reported, so a
/// caller can render a full report rather than just the first failure.
#[derive(Debug, Clone, Error)]
#[error("failed to parse chart: {} error(s)", diagnostics.len())]
pub struct ParseError {
    src: String,
    diagnostics: Vec<Diagnostic>,
}

impl ParseError {
    pub(super) fn new(src: &str, diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            src: src.to_string(),
            diagnostics,
        }
    }

    /// Number of diagnostics collected for this failure.
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    /// Whether there are no diagnostics (should never happen on a real failure).
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Render all diagnostics as a human-readable report using `ariadne`.
    ///
    /// `name` labels the source in the report (e.g. the input file name).
    pub fn report(&self, name: &str) -> String {
        let mut buf = Vec::new();
        for diagnostic in &self.diagnostics {
            let mut builder = Report::build(ReportKind::Error, (name, diagnostic.span.clone()))
                .with_message(&diagnostic.message)
                .with_label(
                    Label::new((name, diagnostic.span.clone()))
                        .with_message(&diagnostic.message)
                        .with_color(Color::Red),
                );

            for (label, span) in &diagnostic.contexts {
                builder = builder.with_label(
                    Label::new((name, span.clone()))
                        .with_message(format!("while parsing this {label}"))
                        .with_color(Color::Yellow),
                );
            }

            // A fresh cache per diagnostic keeps the borrow simple; rendering
            // errors are themselves unexpected, so surface them as a defect.
            builder
                .finish()
                .write((name, Source::from(self.src.as_str())), &mut buf)
                .expect("writing an ariadne report to an in-memory buffer cannot fail");
        }
        String::from_utf8_lossy(&buf).into_owned()
    }
}
