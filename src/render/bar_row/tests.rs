use super::*;
use crate::model::{BarLine, Cell, Chord, ChordQuality, Letter, Note, Span};
use std::num::NonZeroUsize;
use vec1::Vec1;

const SIZE: f64 = 20.0;
const COUNT_SIZE: f64 = 12.0;
/// The row box's bottom edge; the chords' baseline is `DESCENT` above it.
const BOTTOM: f64 = 100.0;
const BASELINE: f64 = BOTTOM - DESCENT * SIZE;

fn style() -> BarRowStyle {
    BarRowStyle {
        chord_size: SIZE,
        above: 5.0,
        below: 5.0,
    }
}

fn chord(letter: Letter, quality: Option<&str>) -> Slot {
    let quality = quality.map(|text| ChordQuality::try_new(text).expect("valid quality"));
    Slot::Chord(Chord::new(Note::natural(letter), quality, None))
}

fn slots(slots: Vec<Slot>) -> Bar {
    Bar::Slots(slots)
}

/// The marks of a row of `bars` drawn from 0 to `width`.
fn marks(bars: Vec<Bar>, width: f64) -> Vec<Mark> {
    cell_marks(bars.into_iter().map(Cell::from).collect(), width)
}

fn cell_marks(cells: Vec<Cell>, width: f64) -> Vec<Mark> {
    let row = BarRow::new(Vec1::try_from_vec(cells).expect("at least one cell"));
    layout(&row, &style(), COUNT_SIZE, 0.0, width, BOTTOM).expect("room to draw the row")
}

fn span(width: usize) -> Cell {
    let width = NonZeroUsize::new(width).expect("a positive width");
    Span::try_new(width)
        .expect("a width within the limit")
        .into()
}

/// Where each bar line is.
fn bar_lines(marks: &[Mark]) -> Vec<f64> {
    marks
        .iter()
        .filter_map(|mark| match mark {
            Mark::Stroke { from, to, .. } if from.x == to.x => Some(from.x),
            _ => None,
        })
        .collect()
}

/// Each chord's start and squeeze.
fn chords(marks: &[Mark]) -> Vec<(f64, f64)> {
    marks
        .iter()
        .filter_map(|mark| match mark {
            Mark::Chord { at, squeeze, .. } => Some((at.x, *squeeze)),
            _ => None,
        })
        .collect()
}

/// Each beat slash's start on the baseline.
fn slashes(marks: &[Mark]) -> Vec<Point> {
    marks
        .iter()
        .filter_map(|mark| match mark {
            Mark::Stroke { from, to, .. } if from.x != to.x => Some(*from),
            _ => None,
        })
        .collect()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn a_row_is_its_chords_and_a_hanging_bass_tall() {
    assert!(close(height(&style()), 1.78 * SIZE));
}

#[test]
fn bars_split_the_row_evenly_between_bar_lines() {
    let marks = marks(vec![slots(vec![]), Bar::Repeat, slots(vec![])], 300.0);
    assert_eq!(bar_lines(&marks), [0.0, 100.0, 200.0, 300.0]);
}

#[test]
fn spans_take_room_without_bar_lines_of_their_own() {
    let bar = || Cell::from(slots(vec![chord(Letter::C, None)]));
    let leading = cell_marks(vec![span(2), bar(), bar()], 400.0);
    assert_eq!(bar_lines(&leading), [200.0, 300.0, 400.0]);
    assert_eq!(chords(&leading)[0].0, 200.0 + INSET * SIZE);

    let between = cell_marks(vec![bar(), span(1), bar(), span(1)], 400.0);
    assert_eq!(bar_lines(&between), [0.0, 100.0, 200.0, 300.0]);

    assert!(
        cell_marks(vec![span(3)], 300.0).is_empty(),
        "nothing to draw"
    );
}

/// `bars` one-chord bars across `width`, every bar line `line`.
fn row_of(bars: usize, line: BarLine) -> BarRow {
    let bar = Cell::from(slots(vec![chord(Letter::C, None)]));
    let mut row = BarRow::new(Vec1::from_vec_push(vec![bar.clone(); bars - 1], bar));
    row.start = line;
    for entry in row.cells.iter_mut() {
        entry.end = line;
    }
    row
}

#[test]
fn a_chord_squeezed_too_narrow_to_read_is_reported() {
    // About as narrow as 16 bars on the default page.
    let width = 16.0 * 0.9 * SIZE;
    let plain = row_of(16, BarLine::Single);
    let squeezed =
        layout(&plain, &style(), COUNT_SIZE, 0.0, width, BOTTOM).expect("plain lines leave room");
    assert!(chords(&squeezed).iter().all(|&(_, squeeze)| squeeze > 0.5));

    let mut repeats = row_of(16, BarLine::RepeatBoth(None));
    repeats.start = BarLine::RepeatStart;
    repeats.cells.last_mut().end = BarLine::RepeatEnd(None);
    let cramped = layout(&repeats, &style(), COUNT_SIZE, 0.0, width, BOTTOM);
    let Err(Cramped::Chord(chord)) = cramped else {
        panic!("repeat signs leave no room for the chords: {cramped:?}");
    };
    assert_eq!(chord.to_string(), "C");
}

#[test]
fn bar_lines_running_into_each_other_are_reported() {
    let empty = || Cell::from(slots(vec![]));
    let mut row = BarRow::new(Vec1::from_vec_push(vec![empty(); 29], empty()));
    for entry in row.cells.iter_mut() {
        entry.end = BarLine::RepeatBoth(None);
    }
    let cramped = layout(&row, &style(), COUNT_SIZE, 0.0, 30.0 * 0.6 * SIZE, BOTTOM);
    assert_eq!(cramped, Err(Cramped::BarLines));
}

#[test]
fn bar_lines_run_from_the_top_of_the_chords_into_the_bass() {
    let marks = marks(vec![slots(vec![])], 100.0);
    let Mark::Stroke { from, to, .. } = marks[0] else {
        panic!("the first mark is a bar line: {marks:?}");
    };
    assert!(close(from.y, BASELINE - ASCENT * SIZE));
    assert!(
        close(from.y, BOTTOM - height(&style())),
        "from the top of the box"
    );
    assert!(close(to.y, BASELINE + BAR_LINE_DROP * SIZE));
    assert!(close(BOTTOM - to.y, room_below_bar_lines(&style())));
    assert!(to.y < BOTTOM, "a hanging bass reaches below the bar line");
}

#[test]
fn content_starts_an_inset_into_its_slot_and_holds_draw_nothing() {
    let bar = slots(vec![
        chord(Letter::A, Some("m")),
        Slot::Hold,
        Slot::Slash,
        chord(Letter::D, None),
    ]);
    let marks = marks(vec![bar], 400.0);
    let inset = INSET * SIZE;
    assert_eq!(chords(&marks), [(inset, 1.0), (300.0 + inset, 1.0)]);
    assert_eq!(
        slashes(&marks),
        [Point {
            x: 200.0 + inset,
            y: BASELINE
        }]
    );
}

#[test]
fn a_bar_repeat_is_a_slash_between_two_dots_centered_in_its_bar() {
    let marks = marks(vec![slots(vec![]), Bar::Repeat], 200.0);
    let [.., slash, upper, lower] = marks.as_slice() else {
        panic!("bar lines, then the sign: {marks:?}");
    };
    let Mark::Stroke { from, to, width } = slash else {
        panic!("the sign's slash: {slash:?}");
    };
    assert!(close((from.x + to.x) / 2.0, 150.0) && from.x < to.x);
    assert!(
        close(from.y, BASELINE) && close(to.y, BASELINE - fonts::cap_height(Font::Chord, SIZE))
    );
    assert!(*width > SLASH_WIDTH * SIZE, "heavier than a beat slash");

    let (Mark::Dot { at: upper, .. }, Mark::Dot { at: lower, .. }) = (upper, lower) else {
        panic!("the sign's dots: {upper:?}, {lower:?}");
    };
    assert!(upper.x < 150.0 && lower.x > 150.0 && upper.y < lower.y);
    assert!(close(upper.x + lower.x, 300.0) && close(upper.y + lower.y, from.y + to.y));
}

#[test]
fn a_chord_reaches_over_the_holds_after_it() {
    let wide = || chord(Letter::C, Some("7(b9#13)"));
    let width = 4.0 * SIZE;
    let alone = marks(vec![slots(vec![wide(), chord(Letter::G, None)])], width);
    let held = marks(
        vec![slots(vec![
            wide(),
            Slot::Hold,
            Slot::Hold,
            chord(Letter::G, None),
        ])],
        2.0 * width,
    );
    assert!(chords(&alone)[0].1 < 1.0, "squeezed into half the bar");
    assert_eq!(chords(&held)[0].1, 1.0, "fits over the holds");
}

#[test]
fn a_squeezed_chord_ends_a_clearance_before_what_follows() {
    let marks = marks(
        vec![slots(vec![chord(Letter::C, Some("7(b9#13)")), Slot::Slash])],
        2.0 * SIZE,
    );
    let Mark::Chord {
        at,
        pieces,
        squeeze,
        ..
    } = &marks[2]
    else {
        panic!("the third mark is the chord: {marks:?}");
    };
    let slash = slashes(&marks)[0].x;
    let end = at.x + chord::width(pieces) * squeeze;
    assert!(close(end, slash - CLEARANCE * SIZE), "{end} vs {slash}");
}

#[test]
fn the_last_chord_of_a_bar_is_squeezed_to_the_bar_line() {
    let marks = marks(
        vec![slots(vec![chord(Letter::C, Some("7(b9#13)"))]), Bar::Repeat],
        2.0 * SIZE,
    );
    let Mark::Chord {
        at,
        pieces,
        squeeze,
        ..
    } = &marks[3]
    else {
        panic!("the fourth mark is the chord: {marks:?}");
    };
    let end = at.x + chord::width(pieces) * squeeze;
    assert!(close(end, SIZE - CLEARANCE * SIZE), "{end}");
}
