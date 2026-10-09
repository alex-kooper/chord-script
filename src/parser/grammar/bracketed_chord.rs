//! `[chord]` in text: exactly one chord symbol between the brackets.
//!
//! The brackets are read first, up to `]`, and only then is their content
//! checked as one chord. So `[ Am ]`, `[]`, or `[Am G]` each gets one specific
//! message rather than whatever the chord grammar happened to expect at the
//! point it stopped.

use super::Extra;
use super::chord::chord;
use crate::model::{Chord, Inline, TextStyle};
use chumsky::error::{LabelError, RichReason};
use chumsky::input::Emitter;
use chumsky::prelude::*;
use chumsky::text::Char;
use chumsky::util::MaybeRef;

/// `[chord]`. Never fails once `[` is read: problems are reported, and the
/// parse fails at the end either way, but the rest of the line is still
/// checked.
///
/// The content ends at any character that is structural in text, so a missing
/// `]` is reported as such and leaves the `*` of `*see [Am*` to close the bold.
pub(super) fn bracketed_chord<'a>() -> impl Parser<'a, &'a str, Inline, Extra<'a>> {
    let contents = any()
        .filter(|c: &char| !c.is_newline() && !"[]*_<>".contains(*c))
        .labelled("chord symbol")
        .repeated()
        .to_slice()
        .map_with(|text: &str, e| (text, e.span()));
    let closing = just(']').labelled("`]` to close `[`");

    just('[')
        .ignore_then(contents)
        .then(closing.or_not())
        .validate(|((text, span), closing), e, emitter| match closing {
            Some(_) => checked_chord(text, span, emitter),
            None => {
                emitter.emit(Rich::custom(
                    e.span(),
                    r"`[` is not closed: write `]` right after the chord, or `\[` for a literal bracket",
                ));
                invalid()
            }
        })
}

/// The chord written as `text`, which starts at `span` in the source.
fn checked_chord<'a>(
    text: &'a str,
    span: SimpleSpan,
    emitter: &mut Emitter<Rich<'a, char>>,
) -> Inline {
    let symbol = text.trim();
    if symbol.is_empty() {
        emitter.emit(Rich::custom(
            span,
            r"empty `[]`: write a chord inside, e.g. `[Am]`, or `\[\]` for literal brackets",
        ));
        return invalid();
    }
    let leading = text.len() - text.trim_start().len();
    let offset = span.start + leading;
    let after = text[leading + symbol.len()..].chars().next().unwrap_or(']');
    let (parsed, errors) = parse_symbol(symbol);

    if symbol != text {
        let valid = errors.is_empty() && parsed.as_ref().is_some_and(|(_, rest)| rest.is_empty());
        let hint = if valid {
            format!(": write `[{symbol}]`")
        } else {
            String::new()
        };
        emitter.emit(Rich::custom(
            span,
            format!("no spaces inside `[...]`{hint}"),
        ));
    }
    for error in errors {
        emitter.emit(shifted(&error, offset, after));
    }
    let Some((chord, rest)) = parsed else {
        return invalid();
    };
    if !rest.is_empty() {
        let start = offset + symbol.len() - rest.len();
        emitter.emit(Rich::custom(
            SimpleSpan::from(start..offset + symbol.len()),
            format!("`[...]` holds a single chord, but `{rest}` follows `{chord}`"),
        ));
    }
    Inline::new(chord, TextStyle::Normal)
}

/// The chord at the start of `symbol` and the text after it.
///
/// A chord that stops at a `(` stopped at a group it could not read, as in
/// `C7(b9` or `C7((b9))`. Then the chord grammar's own errors say what is
/// wrong with the group, and there is no chord.
fn parse_symbol(symbol: &str) -> (Option<(Chord, &str)>, Vec<Rich<'_, char>>) {
    let (parsed, errors) = chord()
        .then(any().repeated().to_slice())
        .parse(symbol)
        .into_output_errors();
    match parsed {
        Some((_, rest)) if rest.starts_with('(') => {
            (None, chord().then_ignore(end()).parse(symbol).into_errors())
        }
        parsed => (parsed, errors),
    }
}

/// `error`, found in a chord symbol that starts `offset` bytes into the
/// source. The symbol was parsed on its own, so where it ended the source has
/// `after`: the closing `]`, or a space before it.
fn shifted<'a>(error: &Rich<'a, char>, offset: usize, after: char) -> Rich<'a, char> {
    let start = error.span().start + offset;
    let end = error.span().end + offset;
    match error.reason() {
        RichReason::ExpectedFound { expected, found } => {
            let (found, end) = match found {
                Some(found) => (*found, end),
                None => (MaybeRef::Val(after), start + after.len_utf8()),
            };
            <Rich<'a, char> as LabelError<'a, &'a str, _>>::expected_found(
                expected.iter().cloned(),
                Some(found),
                SimpleSpan::from(start..end),
            )
        }
        RichReason::Custom(message) => Rich::custom(SimpleSpan::from(start..end), message),
    }
}

/// Stands in for a `[...]` that holds no valid chord; its error is already
/// reported, so the parse fails and this value is never seen.
fn invalid() -> Inline {
    Inline::plain("[]")
}
