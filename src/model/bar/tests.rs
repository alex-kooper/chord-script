use super::*;
use crate::model::{Letter, Note, RepeatCount};
use vec1::vec1;

fn span(width: usize) -> Cell {
    let width = NonZeroUsize::new(width).expect("a positive width");
    Span::try_new(width)
        .expect("a width within the limit")
        .into()
}

fn empty() -> Cell {
    Bar::Slots(Vec::new()).into()
}

#[test]
fn rows_display_as_written() {
    let chord = |letter| Slot::Chord(Chord::new(Note::natural(letter), None, None));
    let mut row = BarRow::new(vec1![
        span(2),
        Bar::Slots(vec![
            chord(Letter::A),
            Slot::Hold,
            chord(Letter::G),
            Slot::Slash
        ])
        .into(),
        Bar::Repeat.into(),
        empty(),
    ]);
    assert_eq!(row.to_string(), "| {2} | A _ G / | % | |");

    row.start = BarLine::RepeatStart;
    row.cells[0].end = BarLine::Double;
    row.cells[1].end = BarLine::RepeatBoth(None);
    row.cells[2].end = BarLine::RepeatEnd(None);
    row.cells[3].end = BarLine::Final;
    assert_eq!(row.to_string(), "||: {2} || A _ G / :||: % :|| |||");

    let count = |times| Some(RepeatCount::try_new(times).expect("a count within the limits"));
    row.cells[1].end = BarLine::RepeatBoth(count(2));
    row.cells[2].end = BarLine::RepeatEnd(count(13));
    assert_eq!(row.to_string(), "||: {2} || A _ G / :||:x2 % :||x13 |||");
}

#[test]
fn a_repeat_count_is_from_its_min_to_its_max() {
    assert!(RepeatCount::try_new(RepeatCount::MIN - 1).is_err());
    assert!(RepeatCount::try_new(RepeatCount::MIN).is_ok());
    assert!(RepeatCount::try_new(RepeatCount::MAX).is_ok());
    assert!(RepeatCount::try_new(RepeatCount::MAX + 1).is_err());
}

#[test]
fn only_a_repeat_end_has_a_count() {
    let count = RepeatCount::try_new(3).ok();
    assert_eq!(BarLine::RepeatEnd(count).count(), count);
    assert_eq!(BarLine::RepeatBoth(count).count(), count);
    assert_eq!(BarLine::RepeatEnd(None).count(), None);
    assert_eq!(BarLine::RepeatStart.count(), None);
}

#[test]
fn a_row_is_as_wide_as_its_bars_and_the_bars_its_spans_stand_for() {
    let row = BarRow::new(vec1![span(2), Bar::Repeat.into(), span(3)]);
    assert_eq!(row.width().get(), 6);
    assert_eq!(BarRow::new(vec1![Bar::Repeat]).width().get(), 1);
}

#[test]
fn a_row_has_its_starting_bar_line_then_one_closing_each_cell() {
    let mut row = BarRow::new(vec1![span(2), Bar::Repeat.into()]);
    row.start = BarLine::RepeatStart;
    row.cells[1].end = BarLine::Final;
    let lines: Vec<BarLine> = row.lines().collect();
    assert_eq!(
        lines,
        [BarLine::RepeatStart, BarLine::Single, BarLine::Final]
    );
}

#[test]
fn a_bar_line_is_beside_a_bar_if_either_cell_next_to_it_is_one() {
    let row = BarRow::new(vec1![span(2), Bar::Repeat.into(), empty(), span(3)]);
    let beside: Vec<bool> = (0..=row.cells.len())
        .map(|index| row.is_beside_bar(index))
        .collect();
    assert_eq!(beside, [false, true, true, true, false]);
    assert!(
        !row.is_beside_bar(row.cells.len() + 1),
        "past the row's end"
    );
}

#[test]
fn a_span_is_at_most_its_max_width() {
    let width = |bars| NonZeroUsize::new(bars).expect("a positive width");
    assert!(Span::try_new(width(Span::MAX_WIDTH)).is_ok());
    assert!(Span::try_new(width(Span::MAX_WIDTH + 1)).is_err());
}
