//! Bar lines, `|` to `:||:`, and where they split a row.

use super::{Segment, is_separator};
use crate::model::{BarLine, BarRow};
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

/// The length of the bar line `text` starts with.
pub(super) fn line_len(text: &str) -> usize {
    text.find(|c| !is_line_char(c)).unwrap_or(text.len())
}

/// Where the next bar line in `text` starts, if there is one.
fn next_line(text: &str) -> Option<usize> {
    text.char_indices()
        .map(|(offset, _)| offset)
        .find(|&offset| {
            let rest = &text[offset..];
            rest.starts_with(is_line_char) && rest[..line_len(rest)].contains('|')
        })
}

/// The bar line written in `line`. A bad one is reported and read as `|`.
pub(super) fn checked_line<'a>(
    line: &Segment<'a>,
    emitter: &mut Emitter<Rich<'a, char>>,
) -> BarLine {
    let problem = match line.text {
        "|" => return BarLine::Single,
        "||" => return BarLine::Double,
        "|||" => return BarLine::Final,
        "||:" => return BarLine::RepeatStart,
        ":||" => return BarLine::RepeatEnd,
        ":||:" => return BarLine::RepeatBoth,
        "|:" => "write `||:` for a repeat start".to_string(),
        ":|" => "write `:||` for a repeat end".to_string(),
        ":|:" => "write `:||:` to end a repeat and start the next".to_string(),
        other => {
            format!("`{other}` is not a bar line: write `|`, `||`, `|||`, `||:`, `:||` or `:||:`")
        }
    };
    emitter.emit(Rich::custom(line.span, problem));
    BarLine::Single
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
        if pair == [BarLine::RepeatEnd, BarLine::RepeatStart] && blank {
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
