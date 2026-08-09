//! The chumsky grammar for chord-script source text.
//!
//! Every chumsky type is confined to this module; [`parse`] hands back owned
//! model values or owned diagnostics, keeping the parsing library an
//! implementation detail of `parser`.

use super::error::Diagnostic;
use crate::model::{Line, LineLevel, TextSpan, TextStyle};
use chumsky::extra;
use chumsky::prelude::*;

/// Parse source text into lines, collecting every diagnostic on failure.
pub(super) fn parse(input: &str) -> Result<Vec<Line>, Vec<Diagnostic>> {
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

type Columns = (Vec<TextSpan>, Vec<TextSpan>, Vec<TextSpan>);

fn columns_parser<'a>() -> impl Parser<'a, &'a str, Columns, extra::Err<Rich<'a, char>>> {
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
