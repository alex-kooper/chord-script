//! The chumsky grammar for chord-script source text.
//!
//! Every chumsky type is confined to this module; [`parse`] hands back owned
//! model values or owned diagnostics, keeping the parsing library an
//! implementation detail of `parser`.

use super::error::Diagnostic;
use crate::model::{Line, LineLevel, TextSpan, TextStyle};
use chumsky::extra;
use chumsky::prelude::*;
use chumsky::text::{Char, inline_whitespace, newline};

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

/// A chart is a sequence of source lines separated by newlines. Nothing in the
/// grammar crosses a newline, so every chart line ends where its source line ends.
fn chart_parser<'a>() -> impl Parser<'a, &'a str, Vec<Line>, extra::Err<Rich<'a, char>>> {
    source_line()
        .separated_by(newline().labelled("end of line"))
        .collect::<Vec<Option<Line>>>()
        .map(|lines| lines.into_iter().flatten().collect())
        .then_ignore(end())
}

/// One source line: a chart line, or `None` for a blank line.
///
/// Blank and whitespace-only lines are ignored, like whitespace in code. A
/// line marker must sit in column 0; an indented one is reported but still
/// parsed, so the rest of the chart is checked too.
fn source_line<'a>() -> impl Parser<'a, &'a str, Option<Line>, extra::Err<Rich<'a, char>>> {
    let indent = inline_whitespace().at_least(1).validate(|(), e, emitter| {
        emitter.emit(Rich::custom(
            e.span(),
            "line markers must start at the beginning of the line",
        ))
    });

    line_parser()
        .or(indent.ignore_then(line_parser()))
        .map(Some)
        .or(inline_whitespace().to(None))
        .labelled("line level (===, ==, =, or -)")
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

/// Parse one column: a run of plain and styled spans.
///
/// Whitespace at the column's edges (e.g. around `<` and `>`) is insignificant
/// and trimmed; whitespace between spans is kept. A whitespace-only column,
/// like the inside of `<   >`, becomes empty rather than an error.
fn spans<'a>() -> impl Parser<'a, &'a str, Vec<TextSpan>, extra::Err<Rich<'a, char>>> {
    span_group()
        .repeated()
        .collect::<Vec<Vec<TextSpan>>>()
        .map(|groups| trim_column(groups.into_iter().flatten().collect()))
}

/// Plain text, or a `*bold*` / `_italic_` group flattened into spans.
///
/// The two styles nest one level into each other (`*a _b_ c*`) and combine, so
/// `*_x_*` and `_*x*_` are both bold italic. A style cannot nest into itself:
/// the next `*` inside bold closes it.
fn span_group<'a>() -> impl Parser<'a, &'a str, Vec<TextSpan>, extra::Err<Rich<'a, char>>> {
    let bold = styled('*', TextStyle::Bold, styled_leaf('_', TextStyle::Italic));
    let italic = styled('_', TextStyle::Italic, styled_leaf('*', TextStyle::Bold));

    bold.or(italic)
        .or(plain().map(|span| vec![span]))
        .labelled("text")
}

/// Plain text and `nested` spans between two `marker`s, all gaining `style`.
fn styled<'a>(
    marker: char,
    style: TextStyle,
    nested: impl Parser<'a, &'a str, TextSpan, extra::Err<Rich<'a, char>>>,
) -> impl Parser<'a, &'a str, Vec<TextSpan>, extra::Err<Rich<'a, char>>> {
    nested
        .or(plain())
        .repeated()
        .at_least(1)
        .collect::<Vec<TextSpan>>()
        .delimited_by(just(marker), just(marker))
        .map(move |spans| {
            spans
                .into_iter()
                .map(|span| with_style(span, style))
                .collect()
        })
}

/// Plain text between two `marker`s: the innermost nesting level.
fn styled_leaf<'a>(
    marker: char,
    style: TextStyle,
) -> impl Parser<'a, &'a str, TextSpan, extra::Err<Rich<'a, char>>> {
    plain()
        .delimited_by(just(marker), just(marker))
        .map(move |span| with_style(span, style))
}

fn with_style(span: TextSpan, style: TextStyle) -> TextSpan {
    TextSpan {
        style: span.style.combine(style),
        ..span
    }
}

fn plain<'a>() -> impl Parser<'a, &'a str, TextSpan, extra::Err<Rich<'a, char>>> {
    text().map(|text| TextSpan::new(text, TextStyle::Normal))
}

/// One or more literal characters of span text.
///
/// `* _ < >` and line breaks are structural and end the text. Line breaks are
/// whatever [`newline`] accepts (including `\r`, form feed, and `\u{2028}`), so
/// text never swallows a break that separates lines. A backslash escapes one of
/// `\ * _ < > [ ]`; before any other character it is kept literally, as in
/// Markdown.
fn text<'a>() -> impl Parser<'a, &'a str, String, extra::Err<Rich<'a, char>>> {
    let escape = just('\\').ignore_then(one_of(r"\*_<>[]"));
    let plain_char = any().filter(|c: &char| !c.is_newline() && !"*_<>[]".contains(*c));
    let character = escape
        .or(reserved_bracket())
        .or(plain_char)
        .labelled("text");

    character.repeated().at_least(1).collect()
}

/// An unescaped `[` or `]`, reserved for future note syntax.
///
/// Reported as an error but kept as literal text, so parsing continues at any
/// nesting level (`*see [A]*`).
fn reserved_bracket<'a>() -> impl Parser<'a, &'a str, char, extra::Err<Rich<'a, char>>> {
    one_of("[]").validate(|bracket: char, e, emitter| {
        emitter.emit(Rich::custom(
            e.span(),
            format!("`{bracket}` is reserved for notes; write `\\{bracket}` for a literal bracket"),
        ));
        bracket
    })
}

/// Trim whitespace from the outer edges of a column, dropping spans left empty.
fn trim_column(spans: Vec<TextSpan>) -> Vec<TextSpan> {
    let leading_trimmed = trim_first(spans.into_iter(), str::trim_start);
    let mut trimmed = trim_first(leading_trimmed.into_iter().rev(), str::trim_end);
    trimmed.reverse();
    trimmed
}

/// Apply `trim` to the first span, dropping spans until one keeps some text.
fn trim_first(mut spans: impl Iterator<Item = TextSpan>, trim: fn(&str) -> &str) -> Vec<TextSpan> {
    let first_kept = spans.by_ref().find_map(|span| {
        let text: &str = span.text.as_ref();
        TextSpan::try_new(trim(text), span.style)
    });
    first_kept.into_iter().chain(spans).collect()
}
