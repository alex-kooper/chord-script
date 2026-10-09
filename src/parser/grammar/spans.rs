//! Inline text: plain and styled spans, escapes, and column trimming.

use super::Extra;
use crate::model::{TextSpan, TextStyle, is_displayable};
use chumsky::prelude::*;
use chumsky::text::Char;

/// Parse one column: a run of plain and styled spans.
///
/// Whitespace at the column's edges (e.g. around `<` and `>`) is insignificant
/// and trimmed; whitespace between spans is kept. A whitespace-only column,
/// like the inside of `<   >`, becomes empty rather than an error.
pub(super) fn spans<'a>() -> impl Parser<'a, &'a str, Vec<TextSpan>, Extra<'a>> {
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
fn span_group<'a>() -> impl Parser<'a, &'a str, Vec<TextSpan>, Extra<'a>> {
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
    nested: impl Parser<'a, &'a str, TextSpan, Extra<'a>>,
) -> impl Parser<'a, &'a str, Vec<TextSpan>, Extra<'a>> {
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
) -> impl Parser<'a, &'a str, TextSpan, Extra<'a>> {
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

fn plain<'a>() -> impl Parser<'a, &'a str, TextSpan, Extra<'a>> {
    text().map(|text| TextSpan::new(text, TextStyle::Normal))
}

/// One or more literal characters of span text.
///
/// `* _ < >` and line breaks are structural and end the text. Line breaks are
/// whatever [`chumsky::text::newline`] accepts (including `\r`, form feed, and
/// `\u{2028}`), so text never swallows a break that separates lines. A
/// backslash escapes one of `\ * _ < > [ ]`; before any other character it is
/// kept literally, as in Markdown.
fn text<'a>() -> impl Parser<'a, &'a str, String, Extra<'a>> {
    let escape = just('\\').ignore_then(one_of(r"\*_<>[]"));
    let plain_char =
        any().filter(|c: &char| !c.is_newline() && is_displayable(*c) && !"*_<>[]".contains(*c));
    let character = escape
        .or(reserved_bracket())
        .or(undisplayable())
        .or(plain_char)
        .labelled("text");

    character.repeated().at_least(1).collect()
}

/// A character that cannot appear in chart text, such as a control character.
///
/// Reported as an error. It is replaced so the span stays valid while the rest
/// of the chart is still checked; the parse fails either way.
fn undisplayable<'a>() -> impl Parser<'a, &'a str, char, Extra<'a>> {
    any()
        .filter(|c: &char| !c.is_newline() && !is_displayable(*c))
        .validate(|c: char, e, emitter| {
            emitter.emit(Rich::custom(
                e.span(),
                format!(
                    "unprintable character U+{:04X} is not allowed in text",
                    u32::from(c)
                ),
            ));
            char::REPLACEMENT_CHARACTER
        })
}

/// An unescaped `[` or `]`, reserved for future note syntax.
///
/// Reported as an error but kept as literal text, so parsing continues at any
/// nesting level (`*see [A]*`).
fn reserved_bracket<'a>() -> impl Parser<'a, &'a str, char, Extra<'a>> {
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
