//! The chumsky grammar for chord-script source text.
//!
//! Every chumsky type is confined to this module and its submodules; [`parse`]
//! hands back owned model values or owned diagnostics, keeping the parsing
//! library an implementation detail of `parser`.

#[cfg_attr(
    not(test),
    expect(dead_code, reason = "used once text lines accept `[chord]`")
)]
mod chord;
mod directive;
mod spans;

use super::error::Diagnostic;
use crate::model::{Block, Line, LineLevel, TextSpan};
use chumsky::extra;
use chumsky::prelude::*;
use chumsky::text::{inline_whitespace, newline};
use spans::spans;

/// Parse source text into blocks, collecting every diagnostic on failure.
pub(super) fn parse(input: &str) -> Result<Vec<Block>, Vec<Diagnostic>> {
    chart_parser()
        .parse(input)
        .into_result()
        .map_err(|errors| errors.into_iter().map(diagnostic_from_rich).collect())
}

fn diagnostic_from_rich(error: Rich<'_, char>) -> Diagnostic {
    let span = error.span();
    Diagnostic::new(
        error.to_string(),
        span.start..span.end,
        error
            .contexts()
            .map(|(label, span)| (label.to_string(), span.start..span.end))
            .collect(),
    )
}

/// A chart is a sequence of source lines separated by newlines. Nothing in the
/// grammar crosses a newline, so every block ends where its source line ends.
fn chart_parser<'a>() -> impl Parser<'a, &'a str, Vec<Block>, extra::Err<Rich<'a, char>>> {
    source_line()
        .separated_by(newline().labelled("end of line"))
        .collect::<Vec<Option<Block>>>()
        .map(|blocks| blocks.into_iter().flatten().collect())
        .then_ignore(end())
}

/// One source line: a block, or `None` for a line that produces nothing
/// (blank lines, comments, and invalid directives, which are reported).
///
/// A line's prefix must sit in column 0; an indented one is reported but still
/// parsed, so the rest of the chart is checked too.
fn source_line<'a>() -> impl Parser<'a, &'a str, Option<Block>, extra::Err<Rich<'a, char>>> {
    let indent = inline_whitespace().at_least(1).validate(|(), e, emitter| {
        emitter.emit(Rich::custom(
            e.span(),
            "line markers must start at the beginning of the line",
        ))
    });

    line_content()
        .or(indent.ignore_then(line_content()))
        .or(inline_whitespace().to(None))
        .labelled("line start (===, ==, =, -, #, or //)")
}

/// The part of a line after any indentation, dispatched on its prefix.
fn line_content<'a>() -> impl Parser<'a, &'a str, Option<Block>, extra::Err<Rich<'a, char>>> {
    line_parser()
        .map(|line| Some(Block::Text(line)))
        .or(directive::directive())
        .or(directive::comment().to(None))
}

/// A line marker followed by its columns: `=== Left <Center> Right`.
///
/// The marker must be followed by a space, or end the line on its own. A bare
/// marker (`===`) is an empty line of that level's height.
fn line_parser<'a>() -> impl Parser<'a, &'a str, Line, extra::Err<Rich<'a, char>>> {
    let header1 = just("===").to(LineLevel::Header1);
    let header2 = just("==").to(LineLevel::Header2);
    let header3 = just("=").to(LineLevel::Header3);
    let text_level = just("-").to(LineLevel::Text);

    let level = header1
        .or(header2)
        .or(header3)
        .or(text_level)
        .labelled("line level (===, ==, =, or -)");

    let columns = inline_whitespace()
        .at_least(1)
        .labelled("space after the line marker")
        .ignore_then(columns_parser());
    let end_of_line = newline().labelled("end of line").or(end());
    let bare = end_of_line.rewind().to(Columns::default());

    level
        .then(columns.or(bare))
        .map(|(level, (left, center, right))| Line {
            level,
            left,
            center,
            right,
        })
}

type Columns = (Vec<TextSpan>, Vec<TextSpan>, Vec<TextSpan>);

/// Split a line into `LEFT <CENTER> RIGHT`.
///
/// The `<…>` center is optional and appears at most once. Text before it is
/// left-aligned, text after it is right-aligned, and `<>` is an empty center
/// that still separates left from right.
fn columns_parser<'a>() -> impl Parser<'a, &'a str, Columns, extra::Err<Rich<'a, char>>> {
    let center = spans()
        .delimited_by(just('<'), just('>'))
        .labelled("centered text in < >");

    spans()
        .then(center.then(spans()).or_not())
        .map(|(left, rest)| match rest {
            Some((center, right)) => (left, center, right),
            None => (left, Vec::new(), Vec::new()),
        })
}
