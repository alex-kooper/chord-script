//! Bar rows: `| Am _ _ A7 | % | G |`.
//!
//! The row is split at its `|`s first, and only then are each bar's words
//! checked. So `||`, `| % Am |`, or `Am_G` each gets one specific message
//! rather than whatever the grammar happened to expect where it stopped.

use super::Extra;
use super::symbol::{parse_symbol, shifted};
use crate::model::{Bar, BarRow, Chord, Slot, is_displayable};
use chumsky::input::Emitter;
use chumsky::prelude::*;
use chumsky::text::Char;
use vec1::Vec1;

/// A line of bars. Never fails once the first `|` is read: problems are
/// reported, and the parse fails at the end either way, but the rest of the
/// chart is still checked.
pub(super) fn bar_row<'a>() -> impl Parser<'a, &'a str, BarRow, Extra<'a>> {
    let segment = any()
        .filter(|c: &char| *c != '|' && !c.is_newline())
        .repeated()
        .to_slice()
        .map_with(|text: &str, e| Segment {
            text,
            span: e.span(),
        });

    just('|')
        .ignore_then(segment.separated_by(just('|')).at_least(1).collect())
        .validate(|segments: Vec<Segment<'a>>, e, emitter| {
            checked_row(&segments, e.span(), emitter)
        })
}

/// The text between two `|`s, or after the last one.
#[derive(Debug, Clone, Copy)]
struct Segment<'a> {
    text: &'a str,
    span: SimpleSpan,
}

/// The row read from `segments`: every one but the last is a bar, and the
/// last, after the final `|`, must be blank.
fn checked_row<'a>(
    segments: &[Segment<'a>],
    row: SimpleSpan,
    emitter: &mut Emitter<Rich<'a, char>>,
) -> BarRow {
    let (after_last, bars) = segments
        .split_last()
        .expect("a row has at least one segment");
    let bars = bars
        .iter()
        .map(|segment| {
            if has_bad_characters(segment, emitter) {
                Bar::Slots(Vec::new())
            } else {
                checked_bar(segment, emitter)
            }
        })
        .collect();
    let trailing_reported = has_bad_characters(after_last, emitter);
    let trailing = !after_last.text.trim_matches(is_separator).is_empty();
    if trailing && !trailing_reported {
        emitter.emit(Rich::custom(
            trimmed(after_last),
            "a bar row ends with `|`: close the last bar, or remove what follows it",
        ));
    }
    Vec1::try_from_vec(bars)
        .map(BarRow::new)
        .unwrap_or_else(|_| {
            if !trailing {
                emitter.emit(Rich::custom(
                    row,
                    "a bar row needs at least one bar, e.g. `| Am | G |`",
                ));
            }
            BarRow::new(Vec1::new(Bar::Slots(Vec::new())))
        })
}

/// Report every character in `segment` that a bar row cannot hold: unprintable
/// ones, and spaces other than space and tab. Returns whether there were any.
///
/// The characters are named by code point, so they never appear raw in a
/// message.
fn has_bad_characters<'a>(segment: &Segment<'a>, emitter: &mut Emitter<Rich<'a, char>>) -> bool {
    let mut found = false;
    for (offset, c) in segment.text.char_indices() {
        let code = u32::from(c);
        let message = if !is_displayable(c) {
            format!("unprintable character U+{code:04X} is not allowed in a bar row")
        } else if c.is_whitespace() && !is_separator(c) {
            format!("U+{code:04X} is not a space or tab: separate slots with spaces")
        } else {
            continue;
        };
        let start = segment.span.start + offset;
        emitter.emit(Rich::custom(
            SimpleSpan::from(start..start + c.len_utf8()),
            message,
        ));
        found = true;
    }
    found
}

/// Whether `c` separates slots, and may surround a bar line.
fn is_separator(c: char) -> bool {
    matches!(c, ' ' | '\t')
}

/// The bar written in `segment`: `%` alone, or slots separated by spaces.
fn checked_bar<'a>(segment: &Segment<'a>, emitter: &mut Emitter<Rich<'a, char>>) -> Bar {
    let Segment { text, span } = *segment;
    if text.is_empty() {
        emitter.emit(Rich::custom(
            SimpleSpan::from(span.start - 1..span.end + 1),
            "`||` is reserved for double bar lines: write `| |` for an empty bar",
        ));
        return Bar::Slots(Vec::new());
    }

    let words = words(text);
    if words.iter().any(|&(_, word)| word == "%") {
        if words.len() > 1 {
            emitter.emit(Rich::custom(
                trimmed(segment),
                "`%` repeats a whole bar, so it stands alone in it: `| % |`",
            ));
        }
        return Bar::Repeat;
    }

    let mut slots = Vec::new();
    for (offset, word) in words {
        let after = text[offset + word.len()..].chars().next().unwrap_or('|');
        slots.extend(checked_slot(word, span.start + offset, after, emitter));
    }
    Bar::Slots(slots)
}

/// The slot written as `word`, which starts `start` bytes into the source and
/// is followed there by `after`. A bad slot is reported and yields `None`.
fn checked_slot<'a>(
    word: &'a str,
    start: usize,
    after: char,
    emitter: &mut Emitter<Rich<'a, char>>,
) -> Option<Slot> {
    match word {
        "_" => return Some(Slot::Hold),
        "/" => return Some(Slot::Slash),
        _ => {}
    }
    if let Some(message) = slot_problem(word) {
        emitter.emit(Rich::custom(
            SimpleSpan::from(start..start + word.len()),
            message,
        ));
        return None;
    }
    checked_chord(word, start, after, emitter).map(Slot::Chord)
}

/// What is wrong with `word`, which is not `_` or `/`, if it can be told
/// without reading it as a chord.
fn slot_problem(word: &str) -> Option<String> {
    if word.starts_with(':') || word.ends_with(':') {
        return Some("repeat signs (`|:` and `:|`) are not supported yet".to_string());
    }
    if word.contains('_') {
        return Some(unspaced_slots(word));
    }
    if !word.starts_with(|c: char| ('A'..='G').contains(&c)) {
        return Some(format!("`{word}` is not a chord, `_`, `/` or `%`"));
    }
    word.strip_suffix('/').map(|chord| {
        format!(
            "a bass note (A to G) must follow `/`; for a beat slash, put a space before it: `{chord} /`"
        )
    })
}

/// Why `word`, which holds `_` but is more than one slot, is wrong, with the
/// two likely meanings: `Am_G` is `Am _ G` here, or two chords in one bar as
/// in chordsheet.com.
fn unspaced_slots(word: &str) -> String {
    let spaced = word.split('_').collect::<Vec<_>>().join(" _ ");
    let spaced = spaced.split_whitespace().collect::<Vec<_>>().join(" ");
    let chords = word.split('_').filter(|part| !part.is_empty());
    let chords = chords.collect::<Vec<_>>().join(" ");
    if chords.is_empty() {
        format!("separate slots with spaces: `{spaced}`")
    } else {
        format!("separate slots with spaces: `{spaced}`, or `{chords}` for just the chords")
    }
}

/// The chord written as `word`, which starts `start` bytes into the source.
fn checked_chord<'a>(
    word: &'a str,
    start: usize,
    after: char,
    emitter: &mut Emitter<Rich<'a, char>>,
) -> Option<Chord> {
    let (parsed, errors) = parse_symbol(word);
    for error in errors {
        emitter.emit(shifted(&error, start, after));
    }
    let (chord, rest) = parsed?;
    if !rest.is_empty() {
        let end = start + word.len();
        emitter.emit(Rich::custom(
            SimpleSpan::from(end - rest.len()..end),
            format!("`{rest}` follows `{chord}`: a bar holds chords, `_`, `/` or `%`, separated by spaces"),
        ));
        return None;
    }
    Some(chord)
}

/// The words of `text` between spaces and tabs, each with the byte offset
/// where it starts.
fn words(text: &str) -> Vec<(usize, &str)> {
    // `split` yields subslices of `text`, so the pointer difference is the
    // offset.
    text.split(is_separator)
        .filter(|word| !word.is_empty())
        .map(|word| (word.as_ptr() as usize - text.as_ptr() as usize, word))
        .collect()
}

/// The span of `segment` without the spaces and tabs around it.
fn trimmed(segment: &Segment<'_>) -> SimpleSpan {
    let leading = segment.text.len() - segment.text.trim_start_matches(is_separator).len();
    let start = segment.span.start + leading;
    SimpleSpan::from(start..start + segment.text.trim_matches(is_separator).len())
}
