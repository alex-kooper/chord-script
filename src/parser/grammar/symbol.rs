//! Chord symbols that are read as text first and checked afterwards, as inside
//! `[...]` and in a bar: the chord grammar runs on the symbol alone, and its
//! errors are moved to where the symbol sits in the source.

use super::chord::chord;
use crate::model::Chord;
use chumsky::error::{LabelError, RichReason};
use chumsky::prelude::*;
use chumsky::util::MaybeRef;

/// The chord at the start of `symbol` and the text after it.
///
/// A chord that stops at a `(` stopped at a group it could not read, as in
/// `C7(b9` or `C7((b9))`. Then the chord grammar's own errors say what is
/// wrong with the group, and there is no chord.
pub(super) fn parse_symbol(symbol: &str) -> (Option<(Chord, &str)>, Vec<Rich<'_, char>>) {
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
/// `after`: what closes the symbol there, such as `]`, `|`, or a space.
pub(super) fn shifted<'a>(error: &Rich<'a, char>, offset: usize, after: char) -> Rich<'a, char> {
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
