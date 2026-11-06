//! Spans: `{2}`, room for bars that are not drawn.

use super::{Segment, is_separator, trimmed};
use crate::model::{Bar, Cell, Span};
use chumsky::input::Emitter;
use chumsky::prelude::*;
use std::num::{IntErrorKind, NonZeroUsize};

/// The span written in `segment`, `{n}` alone between two `|`. A bad one is
/// reported and read as an empty bar.
pub(super) fn checked_span<'a>(
    segment: &Segment<'a>,
    emitter: &mut Emitter<Rich<'a, char>>,
) -> Cell {
    let text = segment.text.trim_matches(is_separator);
    match span(text) {
        Ok(span) => span.into(),
        Err(message) => {
            emitter.emit(Rich::custom(trimmed(segment), message));
            Bar::Slots(Vec::new()).into()
        }
    }
}

/// The span written as `text`, which starts with `{`, or what is wrong with
/// it.
fn span(text: &str) -> Result<Span, String> {
    let width = written_width(text)?;
    let too_wide = || format!("a span is at most {} bars wide", Span::MAX_WIDTH);
    match width.parse::<NonZeroUsize>() {
        Ok(width) => Span::try_new(width).map_err(|_| too_wide()),
        Err(error) if *error.kind() == IntErrorKind::Zero => {
            Err("a span is at least one bar wide, e.g. `{1}`".to_string())
        }
        Err(_) => Err(too_wide()),
    }
}

/// The digits of the width written in the span `text`, or what is wrong with
/// them.
fn written_width(text: &str) -> Result<&str, &'static str> {
    let Some((inside, after)) = text[1..].split_once('}') else {
        return Err("a span ends with `}`, e.g. `{2}`");
    };
    if !after.is_empty() {
        return Err("a span stands alone between two `|`, e.g. `| {2} | Am |`");
    }
    let inside = inside.trim_matches(is_separator);
    let width = inside.split(is_separator).next().unwrap_or_default();
    if width.is_empty() || !width.chars().all(|c| c.is_ascii_digit()) {
        return Err("a span's width is a whole number of bars, e.g. `{2}`");
    }
    if width != inside {
        return Err("text in spans is not supported yet: write just the width, e.g. `{2}`");
    }
    Ok(width)
}
