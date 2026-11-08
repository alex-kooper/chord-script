use super::*;
use crate::model::{Bar, BarRow, Cell, Chord, Letter, Note, Slot, Span};
use crate::render::BarRowStyle;
use crate::render::bar_row::{INSET, layout};
use std::num::NonZeroUsize;
use vec1::vec1;

const SIZE: f64 = 20.0;
const BASELINE: f64 = 100.0;

fn drawn(line: BarLine, x: f64, place: Place) -> Sign {
    sign(line, x, place, BASELINE, SIZE)
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// The sign's pieces left to right: `|` a thin stroke, `I` a thick one, `:`
/// a pair of dots.
fn shape(sign: &Sign) -> String {
    let mut pieces: Vec<(f64, char)> = Vec::new();
    for mark in &sign.marks {
        match mark {
            Mark::Stroke { from, width, .. } => {
                let thick = *width > BAR_LINE_WIDTH * SIZE;
                pieces.push((from.x, if thick { 'I' } else { '|' }));
            }
            Mark::Dot { at, .. } if !pieces.contains(&(at.x, ':')) => pieces.push((at.x, ':')),
            Mark::Dot { .. } => {}
            Mark::Chord { .. } => panic!("a bar line draws no chord"),
        }
    }
    pieces.sort_by(|a, b| a.0.total_cmp(&b.0));
    pieces.into_iter().map(|(_, piece)| piece).collect()
}

/// Where the sign's thick stroke stands, or its only stroke.
fn heaviest(sign: &Sign) -> f64 {
    let strokes = sign.marks.iter().filter_map(|mark| match mark {
        Mark::Stroke { from, width, .. } => Some((from.x, *width)),
        _ => None,
    });
    strokes
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .expect("a stroke")
        .0
}

#[test]
fn each_sign_is_drawn_as_its_strokes_and_dots() {
    for (line, expected) in [
        (BarLine::Single, "|"),
        (BarLine::Double, "||"),
        (BarLine::Final, "|I"),
        (BarLine::RepeatStart, "I|:"),
        (BarLine::RepeatEnd, ":|I"),
        (BarLine::RepeatBoth, ":|I|:"),
    ] {
        assert_eq!(
            shape(&drawn(line, 100.0, Place::Inside)),
            expected,
            "{line}"
        );
    }
}

#[test]
fn a_plain_line_takes_no_room() {
    let plain = drawn(BarLine::Single, 100.0, Place::Inside);
    assert_eq!((plain.left, plain.right), (100.0, 100.0));
}

#[test]
fn inside_a_row_a_sign_stands_on_its_heaviest_stroke() {
    let half_thick = THICK_WIDTH / 2.0 * SIZE;
    let start = drawn(BarLine::RepeatStart, 100.0, Place::Inside);
    assert!(close(heaviest(&start), 100.0) && close(start.left, 100.0 - half_thick));
    assert!(
        start.right > 100.0 + SIZE * 0.3,
        "reaches into the bar it opens"
    );

    let end = drawn(BarLine::RepeatEnd, 100.0, Place::Inside);
    assert!(close(heaviest(&end), 100.0) && close(end.right, 100.0 + half_thick));
    assert!(
        close(100.0 - end.left, start.right - 100.0),
        "mirror images"
    );

    let both = drawn(BarLine::RepeatBoth, 100.0, Place::Inside);
    assert!(close(heaviest(&both), 100.0) && close(both.left + both.right, 200.0));

    let double = drawn(BarLine::Double, 100.0, Place::Inside);
    assert!(close(double.right, 100.0 + BAR_LINE_WIDTH / 2.0 * SIZE));
    assert!(
        double.left < 100.0 - SIZE * 0.1,
        "reaches into the bar it closes"
    );
}

#[test]
fn at_the_ends_of_a_row_a_sign_is_pushed_inward() {
    let half_thin = BAR_LINE_WIDTH / 2.0 * SIZE;
    let opening = drawn(BarLine::Double, 0.0, Place::Start);
    assert!(
        close(opening.left, -half_thin),
        "its outer stroke on the edge"
    );
    assert!(opening.right > 0.0);

    let ending = drawn(BarLine::RepeatStart, 300.0, Place::End);
    assert!(close(ending.right, 300.0), "its dots inside the row");

    for line in [BarLine::Final, BarLine::RepeatEnd] {
        let closing = drawn(line, 300.0, Place::End);
        assert!(close(heaviest(&closing), 300.0), "{line} already fits");
    }
}

#[test]
fn repeat_dots_sit_either_side_of_the_middle_of_a_root() {
    let middle = BASELINE - fonts::cap_height(Font::Chord, SIZE) / 2.0;
    let dots: Vec<Point> = drawn(BarLine::RepeatStart, 100.0, Place::Inside)
        .marks
        .iter()
        .filter_map(|mark| match mark {
            Mark::Dot { at, .. } => Some(*at),
            _ => None,
        })
        .collect();
    let [upper, lower] = dots.as_slice() else {
        panic!("two dots: {dots:?}");
    };
    assert_eq!(upper.x, lower.x);
    assert!(close(upper.y + lower.y, 2.0 * middle) && upper.y < lower.y);
}

fn bar() -> Cell {
    Bar::Slots(vec![Slot::Chord(Chord::new(
        Note::natural(Letter::C),
        None,
        None,
    ))])
    .into()
}

fn span() -> Cell {
    Span::try_new(NonZeroUsize::MIN)
        .expect("one bar is within the limit")
        .into()
}

/// The marks of `row` drawn from 0 to 200.
fn row_marks(row: &BarRow) -> Vec<Mark> {
    let style = BarRowStyle {
        chord_size: SIZE,
        above: 5.0,
        below: 5.0,
    };
    layout(row, &style, 0.0, 200.0, BASELINE).expect("room to draw the row")
}

fn chord_starts(marks: &[Mark]) -> Vec<f64> {
    marks
        .iter()
        .filter_map(|mark| match mark {
            Mark::Chord { at, .. } => Some(at.x),
            _ => None,
        })
        .collect()
}

fn dots(marks: &[Mark]) -> Vec<f64> {
    marks
        .iter()
        .filter_map(|mark| match mark {
            Mark::Dot { at, .. } => Some(at.x),
            _ => None,
        })
        .collect()
}

#[test]
fn a_sign_takes_room_from_the_bars_beside_it() {
    let mut row = BarRow::new(vec1![bar(), bar()]);
    row.start = BarLine::RepeatStart;
    let opening = drawn(BarLine::RepeatStart, 0.0, Place::Start);
    assert_eq!(
        chord_starts(&row_marks(&row)),
        [opening.right + INSET * SIZE, 100.0 + INSET * SIZE]
    );
}

#[test]
fn a_sign_beside_a_span_reaches_into_its_empty_room() {
    let mut after_span = BarRow::new(vec1![span(), bar()]);
    after_span.cells[0].end = BarLine::RepeatEnd;
    let marks = row_marks(&after_span);
    let closing = drawn(BarLine::RepeatEnd, 100.0, Place::Inside);
    let left_of_bar = dots(&marks);
    assert!(
        left_of_bar.len() == 2 && left_of_bar.iter().all(|&x| x < 100.0),
        "{marks:?}"
    );
    assert_eq!(chord_starts(&marks), [closing.right + INSET * SIZE]);

    let mut before_span = BarRow::new(vec1![bar(), span()]);
    before_span.cells[0].end = BarLine::RepeatStart;
    let marks = row_marks(&before_span);
    let right_of_bar = dots(&marks);
    assert!(
        right_of_bar.len() == 2 && right_of_bar.iter().all(|&x| x > 100.0),
        "{marks:?}"
    );
}
