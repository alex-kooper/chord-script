//! Bar lines, `|` to `:||:`, and where they split a row.

use super::{Segment, is_separator};
use crate::model::{BarLine, BarRow, RepeatCount};
use chumsky::input::Emitter;
use chumsky::prelude::*;

/// A row split at its bar lines: each bar line with the text after it, up to
/// the next one or the end of the row.
pub(super) struct Split<'a> {
    pub lines: Vec<Segment<'a>>,
    pub segments: Vec<Segment<'a>>,
}

/// Split `text`, a row starting with `|` at `start` in the source.
///
/// A bar line is a run of `|` and `:` holding at least one `|`, so `:|:` and
/// `|||:` are one (bad) bar line rather than a bar line and a stray `:`.
pub(super) fn split(text: &str, start: usize) -> Split<'_> {
    let mut split = Split {
        lines: Vec::new(),
        segments: Vec::new(),
    };
    let mut offset = 0;
    while offset < text.len() {
        let line = line_len(&text[offset..]);
        split.lines.push(segment(text, start, offset, line));
        offset += line;
        let between = next_line(&text[offset..]).unwrap_or(text.len() - offset);
        split.segments.push(segment(text, start, offset, between));
        offset += between;
    }
    split
}

fn segment(text: &str, start: usize, offset: usize, len: usize) -> Segment<'_> {
    let from = start + offset;
    Segment {
        text: &text[offset..offset + len],
        span: SimpleSpan::from(from..from + len),
    }
}

fn is_line_char(c: char) -> bool {
    matches!(c, '|' | ':')
}

/// The length of the bar line `text` starts with, with the repeat count
/// written against it, if any.
pub(super) fn line_len(text: &str) -> usize {
    let sign = sign_len(text);
    sign + count_len(&text[sign..])
}

/// The length of the bar line's sign `text` starts with, `|` to `:||:`.
fn sign_len(text: &str) -> usize {
    text.find(|c| !is_line_char(c)).unwrap_or(text.len())
}

/// The length of the repeat count `text` starts with, `x3`: an `x` and all
/// the letters and digits after it, in any script, so `x3b` and `xé` are each
/// one (bad) count.
fn count_len(text: &str) -> usize {
    match text.strip_prefix('x') {
        Some(rest) => {
            1 + rest
                .find(|c: char| !c.is_alphanumeric())
                .unwrap_or(rest.len())
        }
        None => 0,
    }
}

/// Whether `word`, written apart from any bar line, looks like a repeat
/// count, `x3`, meant for the bar line before it.
pub(super) fn is_detached_count(word: &str) -> bool {
    word.strip_prefix('x')
        .is_some_and(|digits| !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()))
}

/// Why a repeat count written apart from its bar line is wrong.
pub(super) const DETACHED_COUNT: &str =
    "a repeat count stands against its `:||` or `:||:`, e.g. `:||x3`";

/// Where the next bar line in `text` starts, if there is one.
fn next_line(text: &str) -> Option<usize> {
    text.char_indices()
        .map(|(offset, _)| offset)
        .find(|&offset| {
            let rest = &text[offset..];
            rest.starts_with(is_line_char) && rest[..line_len(rest)].contains('|')
        })
}

/// The bar line written in `line`, with its repeat count. A bad sign is
/// reported and read as `|`; a bad count is reported and left out.
pub(super) fn checked_line<'a>(
    line: &Segment<'a>,
    emitter: &mut Emitter<Rich<'a, char>>,
) -> BarLine {
    let (sign, count) = line.text.split_at(sign_len(line.text));
    let sign_end = line.span.start + sign.len();
    let read = match sign_line(sign) {
        Ok(read) => read,
        Err(problem) => {
            emitter.emit(Rich::custom(
                SimpleSpan::from(line.span.start..sign_end),
                problem,
            ));
            return BarLine::Single;
        }
    };
    if count.is_empty() {
        return read;
    }
    counted(read, count).unwrap_or_else(|problem| {
        emitter.emit(Rich::custom(
            SimpleSpan::from(sign_end..line.span.end),
            problem,
        ));
        read
    })
}

/// The bar line `sign` stands for, or why it stands for none.
fn sign_line(sign: &str) -> Result<BarLine, String> {
    match sign {
        "|" => Ok(BarLine::Single),
        "||" => Ok(BarLine::Double),
        "|||" => Ok(BarLine::Final),
        "||:" => Ok(BarLine::RepeatStart),
        ":||" => Ok(BarLine::RepeatEnd(None)),
        ":||:" => Ok(BarLine::RepeatBoth(None)),
        "|:" => Err("write `||:` for a repeat start".to_string()),
        ":|" => Err("write `:||` for a repeat end".to_string()),
        ":|:" => Err("write `:||:` to end a repeat and start the next".to_string()),
        other => Err(format!(
            "`{other}` is not a bar line: write `|`, `||`, `|||`, `||:`, `:||` or `:||:`"
        )),
    }
}

/// `line` played the times `count`, e.g. `x3`, says, or why it can't be.
fn counted(line: BarLine, count: &str) -> Result<BarLine, String> {
    let with_count: fn(Option<RepeatCount>) -> BarLine = match line {
        BarLine::RepeatEnd(_) => BarLine::RepeatEnd,
        BarLine::RepeatBoth(_) => BarLine::RepeatBoth,
        other => {
            return Err(format!(
                "a repeat count follows only `:||` or `:||:`, not `{other}`"
            ));
        }
    };
    let digits = count.strip_prefix('x').expect("a count starts with `x`");
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return Err(format!(
            "`{count}` is not a repeat count: write `x` and how many times the repeat is played, e.g. `:||x3`"
        ));
    }
    let (min, max) = (RepeatCount::MIN, RepeatCount::MAX);
    digits
        .parse()
        .ok()
        .and_then(|times| RepeatCount::try_new(times).ok())
        .map(|count| with_count(Some(count)))
        .ok_or_else(|| format!("a repeat is played from `x{min}` to `x{max}` times, not `{count}`"))
}

/// Report every blank bar between a repeat end and a repeat start,
/// `:|| ||:`: almost surely one sign written as two. `lines` are the row's bar
/// lines as read, `written` where each stands, and `segments` the text after
/// each.
pub(super) fn report_split_repeats<'a>(
    lines: &[BarLine],
    written: &[Segment<'a>],
    segments: &[Segment<'a>],
    emitter: &mut Emitter<Rich<'a, char>>,
) {
    for (index, pair) in lines.windows(2).enumerate() {
        let blank = segments[index].text.trim_matches(is_separator).is_empty();
        if matches!(pair, [BarLine::RepeatEnd(_), BarLine::RepeatStart]) && blank {
            emitter.emit(Rich::custom(
                SimpleSpan::from(written[index].span.start..written[index + 1].span.end),
                "write `:||:` to end a repeat and start the next, not an empty bar between them",
            ));
        }
    }
}

/// Report every bar line of `row` other than `|` with no bar on either side:
/// it would not be drawn. `written` is where each line of the row stands.
pub(super) fn report_lines_without_bars<'a>(
    row: &BarRow,
    written: &[Segment<'a>],
    emitter: &mut Emitter<Rich<'a, char>>,
) {
    for (index, (line, written)) in row.lines().zip(written).enumerate() {
        if line != BarLine::Single && !row.is_beside_bar(index) {
            emitter.emit(Rich::custom(
                written.span,
                format!("`{line}` has no bar next to it, so it would not be drawn"),
            ));
        }
    }
}
