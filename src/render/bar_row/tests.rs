use super::*;
use crate::model::{Chord, ChordQuality, Letter, Note};
use crate::render::fonts::{self, Font};
use vec1::Vec1;

const SIZE: f64 = 20.0;
const GAP: f64 = 5.0;
/// The row box's bottom edge; the chords' baseline is `DESCENT` above it.
const BOTTOM: f64 = 100.0;
const BASELINE: f64 = BOTTOM - DESCENT * SIZE;

fn style() -> BarRowStyle {
    BarRowStyle {
        chord_size: SIZE,
        gap: GAP,
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
    let row = BarRow::new(Vec1::try_from_vec(bars).expect("at least one bar"));
    layout(&row, &style(), 0.0, width, BOTTOM)
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
fn a_row_is_the_gap_its_chords_and_a_hanging_bass_tall() {
    assert!(close(height(&style()), GAP + 1.9 * SIZE));
}

#[test]
fn bars_split_the_row_evenly_between_bar_lines() {
    let marks = marks(vec![slots(vec![]), Bar::Repeat, slots(vec![])], 300.0);
    assert_eq!(bar_lines(&marks), [0.0, 100.0, 200.0, 300.0]);
}

#[test]
fn bar_lines_run_from_the_top_of_the_chords_into_the_bass() {
    let marks = marks(vec![slots(vec![])], 100.0);
    let Mark::Stroke { from, to, .. } = marks[0] else {
        panic!("the first mark is a bar line: {marks:?}");
    };
    assert!(close(from.y, BASELINE - ASCENT * SIZE));
    assert!(close(to.y, BASELINE + BAR_LINE_DROP * SIZE));
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
fn a_bar_repeat_is_centered_in_its_bar() {
    let marks = marks(vec![slots(vec![]), Bar::Repeat], 200.0);
    let repeats: Vec<f64> = marks
        .iter()
        .filter_map(|mark| match mark {
            Mark::Repeat { at, .. } => Some(at.x),
            _ => None,
        })
        .collect();
    assert_eq!(repeats, [150.0]);
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

#[test]
fn the_music_font_draws_the_bar_repeat_sign() {
    assert!(fonts::has_glyph(Font::Music, REPEAT_SIGN));
}
