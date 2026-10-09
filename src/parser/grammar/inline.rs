//! Inline content: plain and styled text, `[chord]`s, escapes, and column
//! trimming.

use super::Extra;
use super::bracketed_chord::bracketed_chord;
use crate::model::{Content, Inline, TextStyle, is_displayable};
use chumsky::prelude::*;
use chumsky::text::Char;

/// Parse one column: a run of text and chords.
///
/// Whitespace at the column's edges (e.g. around `<` and `>`) is insignificant
/// and trimmed; whitespace between inlines is kept. A whitespace-only column,
/// like the inside of `<   >`, becomes empty rather than an error.
pub(super) fn column<'a>() -> impl Parser<'a, &'a str, Vec<Inline>, Extra<'a>> {
    inline_group()
        .repeated()
        .collect::<Vec<Vec<Inline>>>()
        .map(|groups| trim_column(groups.into_iter().flatten().collect()))
}

/// Unstyled content, or a `*bold*` / `_italic_` group flattened into inlines.
///
/// The two styles nest one level into each other (`*a _b_ c*`) and combine, so
/// `*_x_*` and `_*x*_` are both bold italic. A style cannot nest into itself:
/// the next `*` inside bold closes it. Chords take the style around them.
fn inline_group<'a>() -> impl Parser<'a, &'a str, Vec<Inline>, Extra<'a>> {
    let bold = styled('*', TextStyle::Bold, styled_leaf('_', TextStyle::Italic));
    let italic = styled('_', TextStyle::Italic, styled_leaf('*', TextStyle::Bold));

    bold.or(italic)
        .or(unstyled().map(|inline| vec![inline]))
        .labelled("text")
}

/// Unstyled content and `nested` groups between two `marker`s, all gaining
/// `style`.
fn styled<'a>(
    marker: char,
    style: TextStyle,
    nested: impl Parser<'a, &'a str, Vec<Inline>, Extra<'a>>,
) -> impl Parser<'a, &'a str, Vec<Inline>, Extra<'a>> {
    nested
        .or(unstyled().map(|inline| vec![inline]))
        .repeated()
        .at_least(1)
        .collect::<Vec<Vec<Inline>>>()
        .delimited_by(just(marker), just(marker))
        .map(move |groups| {
            groups
                .into_iter()
                .flatten()
                .map(|inline| with_style(inline, style))
                .collect()
        })
}

/// Unstyled content between two `marker`s: the innermost nesting level.
fn styled_leaf<'a>(
    marker: char,
    style: TextStyle,
) -> impl Parser<'a, &'a str, Vec<Inline>, Extra<'a>> {
    unstyled()
        .repeated()
        .at_least(1)
        .collect::<Vec<Inline>>()
        .delimited_by(just(marker), just(marker))
        .map(move |inlines| {
            inlines
                .into_iter()
                .map(|inline| with_style(inline, style))
                .collect()
        })
}

fn with_style(inline: Inline, added: TextStyle) -> Inline {
    Inline {
        style: inline.style.combine(added),
        ..inline
    }
}

fn unstyled<'a>() -> impl Parser<'a, &'a str, Inline, Extra<'a>> {
    bracketed_chord().or(text().map(Inline::plain))
}

/// One or more literal characters of text.
///
/// `* _ < > [` and line breaks are structural and end the text. Line breaks
/// are whatever [`chumsky::text::newline`] accepts (including `\r`, form feed,
/// and `\u{2028}`), so text never swallows a break that separates lines. A
/// backslash escapes one of `\ * _ < > [ ]`; before any other character it is
/// kept literally, as in Markdown.
fn text<'a>() -> impl Parser<'a, &'a str, String, Extra<'a>> {
    let escape = just('\\').ignore_then(one_of(r"\*_<>[]"));
    let plain_char =
        any().filter(|c: &char| !c.is_newline() && is_displayable(*c) && !"*_<>[]".contains(*c));
    let character = escape
        .or(unmatched_closing_bracket())
        .or(undisplayable())
        .or(plain_char)
        .labelled("text");

    character.repeated().at_least(1).collect()
}

/// A `]` that closes no chord.
///
/// Reported as an error but kept as literal text, so parsing continues at any
/// nesting level (`*see A]*`).
fn unmatched_closing_bracket<'a>() -> impl Parser<'a, &'a str, char, Extra<'a>> {
    just(']').validate(|bracket: char, e, emitter| {
        emitter.emit(Rich::custom(
            e.span(),
            r"`]` without an opening `[`; write `\]` for a literal bracket",
        ));
        bracket
    })
}

/// A character that cannot appear in chart text, such as a control character.
///
/// Reported as an error. It is replaced so the text stays valid while the rest
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

/// Trim whitespace from the outer edges of a column, dropping text left empty.
fn trim_column(inlines: Vec<Inline>) -> Vec<Inline> {
    let leading_trimmed = trim_first(inlines.into_iter(), str::trim_start);
    let mut trimmed = trim_first(leading_trimmed.into_iter().rev(), str::trim_end);
    trimmed.reverse();
    trimmed
}

/// Apply `trim` to the first text, dropping inlines until one keeps some text.
/// A chord is kept as-is and ends the trimming.
fn trim_first(mut inlines: impl Iterator<Item = Inline>, trim: fn(&str) -> &str) -> Vec<Inline> {
    let first_kept = inlines.by_ref().find_map(|inline| match &inline.content {
        Content::Text(text) => Inline::try_text(trim(text.as_ref()), inline.style),
        Content::Chord(_) => Some(inline),
    });
    first_kept.into_iter().chain(inlines).collect()
}
