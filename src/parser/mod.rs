use crate::model::{Chart, Line, LineLevel, TextSpan, TextStyle};
use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::extra;
use chumsky::prelude::*;
use std::ops::Range;
use thiserror::Error;

/// Result type alias for parser operations
pub type Result<T> = std::result::Result<T, ParseError>;

/// A single diagnostic produced while parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Diagnostic {
    message: String,
    span: Range<usize>,
    contexts: Vec<(String, Range<usize>)>,
}

/// Error returned when a chart fails to parse.
///
/// Carries the original source plus every diagnostic chumsky reported, so a
/// caller can render a full report rather than just the first failure.
#[derive(Debug, Clone, Error)]
#[error("failed to parse chart: {} error(s)", diagnostics.len())]
pub struct ParseError {
    src: String,
    diagnostics: Vec<Diagnostic>,
}

impl ParseError {
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

/// Parse a complete chart from input text.
pub fn parse_chart(input: &str) -> Result<Chart> {
    match chart_parser().parse(input).into_result() {
        Ok(lines) => Ok(Chart::new(lines)),
        Err(errors) => Err(ParseError {
            src: input.to_string(),
            diagnostics: errors.into_iter().map(diagnostic_from_rich).collect(),
        }),
    }
}

fn diagnostic_from_rich(error: Rich<'_, char>) -> Diagnostic {
    let span = error.span();
    Diagnostic {
        message: error.to_string(),
        span: span.start..span.end,
        contexts: error
            .contexts()
            .map(|(label, span)| (label.to_string(), span.start..span.end))
            .collect(),
    }
}

fn chart_parser<'a>() -> impl Parser<'a, &'a str, Vec<Line>, extra::Err<Rich<'a, char>>> {
    line_parser()
        .padded()
        .repeated()
        .collect()
        .then_ignore(end())
}

fn line_parser<'a>() -> impl Parser<'a, &'a str, Line, extra::Err<Rich<'a, char>>> {
    let header1 = just("===").ignored().to(LineLevel::Header1);
    let header2 = just("==").ignored().to(LineLevel::Header2);
    let header3 = just("=").ignored().to(LineLevel::Header3);
    let text_level = just("-").ignored().to(LineLevel::Text);

    let level = header1
        .or(header2)
        .or(header3)
        .or(text_level)
        .labelled("line level (===, ==, =, or -)");

    (level.padded())
        .then(columns_parser())
        .map(|(level, (left, center, right))| Line {
            level,
            left,
            center,
            right,
        })
}

fn columns_parser<'a>(
) -> impl Parser<'a, &'a str, (Vec<TextSpan>, Vec<TextSpan>, Vec<TextSpan>), extra::Err<Rich<'a, char>>>
{
    // Center marker first: `<>` starts with `<`, so it must beat the `<` branch.
    let with_center = just("<>")
        .ignore_then(spans())
        .then(just(">").ignore_then(spans()).or_not())
        .map(|(center, right)| (Vec::new(), center, right.unwrap_or_default()));

    let with_left = just("<")
        .ignore_then(spans())
        .then(just("<>").ignore_then(spans()).or_not())
        .then(just(">").ignore_then(spans()).or_not())
        .map(|((left, center), right)| {
            (left, center.unwrap_or_default(), right.unwrap_or_default())
        });

    let with_right = just(">")
        .ignore_then(spans())
        .map(|right| (Vec::new(), Vec::new(), right));

    let no_markers = spans().map(|left| (left, Vec::new(), Vec::new()));

    with_center.or(with_left).or(with_right).or(no_markers)
}

/// Parse a run of styled spans, dropping any that are empty after trimming.
///
/// Whitespace-only spans (e.g. `<>   `) carry no content; they are silently
/// discarded rather than treated as an error.
fn spans<'a>() -> impl Parser<'a, &'a str, Vec<TextSpan>, extra::Err<Rich<'a, char>>> {
    styled_text_parser()
        .repeated()
        .collect::<Vec<Option<TextSpan>>>()
        .map(|spans| spans.into_iter().flatten().collect())
}

fn styled_text_parser<'a>() -> impl Parser<'a, &'a str, Option<TextSpan>, extra::Err<Rich<'a, char>>>
{
    let bold_italic = just("***")
        .ignore_then(none_of("*").repeated().at_least(1).collect::<String>())
        .then_ignore(just("***"))
        .map(|text| TextSpan::try_new(text, TextStyle::BoldItalic));

    let bold = just("**")
        .ignore_then(none_of("*").repeated().at_least(1).collect::<String>())
        .then_ignore(just("**"))
        .map(|text| TextSpan::try_new(text, TextStyle::Bold));

    let italic = just("*")
        .ignore_then(none_of("*<>\n").repeated().at_least(1).collect::<String>())
        .then_ignore(just("*"))
        .map(|text| TextSpan::try_new(text, TextStyle::Italic));

    let plain = none_of("<>*\n")
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(|text| TextSpan::try_new(text, TextStyle::Normal));

    bold_italic.or(bold).or(italic).or(plain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let result = parse_chart("");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().lines.len(), 0);
    }

    #[test]
    fn test_parse_header1() {
        let result = parse_chart("=== Left");
        assert!(result.is_ok());
        let chart = result.unwrap();
        assert_eq!(chart.lines.len(), 1);
        assert_eq!(chart.lines[0].level, LineLevel::Header1);
    }

    #[test]
    fn test_parse_alignment() {
        let result = parse_chart("=== <Left <>Center >Right");
        assert!(result.is_ok());
        let chart = result.unwrap();
        assert_eq!(chart.lines.len(), 1);
        assert_eq!(chart.lines[0].level, LineLevel::Header1);

        // Check left column
        assert_eq!(chart.lines[0].left.len(), 1);
        assert_eq!(chart.lines[0].left[0].text.as_ref(), "Left");
        assert_eq!(chart.lines[0].left[0].style, TextStyle::Normal);

        // Check center column
        assert_eq!(chart.lines[0].center.len(), 1);
        assert_eq!(chart.lines[0].center[0].text.as_ref(), "Center");
        assert_eq!(chart.lines[0].center[0].style, TextStyle::Normal);

        // Check right column
        assert_eq!(chart.lines[0].right.len(), 1);
        assert_eq!(chart.lines[0].right[0].text.as_ref(), "Right");
        assert_eq!(chart.lines[0].right[0].style, TextStyle::Normal);
    }

    #[test]
    fn test_parse_multiline() {
        let input = r#"=== <Song Title <>Composer >2024
== <Verse 1
= Intro
- Piano only"#;

        let result = parse_chart(input);
        assert!(result.is_ok());
        let chart = result.unwrap();
        assert_eq!(chart.lines.len(), 4);

        // Header1
        assert_eq!(chart.lines[0].level, LineLevel::Header1);
        assert_eq!(chart.lines[0].left[0].text.as_ref(), "Song Title");
        assert_eq!(chart.lines[0].center[0].text.as_ref(), "Composer");
        assert_eq!(chart.lines[0].right[0].text.as_ref(), "2024");

        // Header2
        assert_eq!(chart.lines[1].level, LineLevel::Header2);
        assert_eq!(chart.lines[1].left[0].text.as_ref(), "Verse 1");

        // Header3
        assert_eq!(chart.lines[2].level, LineLevel::Header3);
        assert_eq!(chart.lines[2].left[0].text.as_ref(), "Intro");

        // Text
        assert_eq!(chart.lines[3].level, LineLevel::Text);
        assert_eq!(chart.lines[3].left[0].text.as_ref(), "Piano only");
    }

    #[test]
    fn test_parse_invalid_input_returns_error() {
        // Mismatched italic markers
        let result = parse_chart("=== *Unclosed italic marker");
        assert!(result.is_err(), "unclosed italic marker should be an error");

        let error = result.unwrap_err();
        assert!(!error.is_empty(), "error should carry at least one diagnostic");
        assert!(
            !error.report("test").is_empty(),
            "report should produce output"
        );
    }

    #[test]
    fn test_whitespace_only_span_is_not_an_error() {
        // A center marker followed by only spaces must not panic and must not
        // produce a span; it parses to a line with empty columns.
        let result = parse_chart("= <>   ");
        assert!(result.is_ok(), "whitespace-only span should parse cleanly");
        let chart = result.unwrap();
        assert_eq!(chart.lines.len(), 1);
        assert!(chart.lines[0].center.is_empty());
    }
}
