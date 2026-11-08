use super::*;

fn three_times() -> Option<RepeatCount> {
    Some(RepeatCount::try_new(3).expect("a count within the limits"))
}

/// Where the baseline of the count of `line` drawn at `x` is centered, if it
/// has a count.
fn drawn_count(line: BarLine, x: f64, place: Place) -> Option<Point> {
    let sign = drawn(line, x, place);
    count(line, &sign, BASELINE, SIZE, COUNT_SIZE).map(|mark| match mark {
        Mark::Count { at, size, count } => {
            assert_eq!((size, count), (COUNT_SIZE, three_times().expect("a count")));
            at
        }
        other => panic!("a count: {other:?}"),
    })
}

/// The middle between the outer edges of the sign's strokes.
fn strokes_middle(sign: &Sign) -> f64 {
    let edges: Vec<f64> = sign
        .marks
        .iter()
        .filter_map(|mark| match mark {
            Mark::Stroke { from, width, .. } => Some([from.x - width / 2.0, from.x + width / 2.0]),
            _ => None,
        })
        .flatten()
        .collect();
    let min = edges.iter().copied().fold(f64::INFINITY, f64::min);
    let max = edges.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    (min + max) / 2.0
}

#[test]
fn a_count_stands_just_above_the_bar_lines() {
    let line = BarLine::RepeatEnd(three_times());
    let at = drawn_count(line, 100.0, Place::Inside).expect("a count");
    let top = BASELINE - ASCENT * SIZE;
    assert!(at.y < top && at.y > top - 0.2 * SIZE, "{at:?}");
}

#[test]
fn a_count_is_centered_on_the_strokes_of_its_sign_wherever_it_stands() {
    for line in [
        BarLine::RepeatEnd(three_times()),
        BarLine::RepeatBoth(three_times()),
    ] {
        for (x, place) in [
            (0.0, Place::Start),
            (100.0, Place::Inside),
            (300.0, Place::End),
        ] {
            let sign = drawn(line, x, place);
            let at = drawn_count(line, x, place).expect("a count");
            assert!(close(at.x, strokes_middle(&sign)), "{line} at {x}");
        }
    }

    let end = drawn(BarLine::RepeatEnd(three_times()), 100.0, Place::Inside);
    assert!(
        strokes_middle(&end) > (end.left + end.right) / 2.0,
        "the dots left out"
    );
    let both = drawn_count(BarLine::RepeatBoth(three_times()), 100.0, Place::Inside);
    assert!(
        close(both.expect("a count").x, 100.0),
        "on the thick stroke"
    );
}

#[test]
fn a_line_without_a_count_draws_none() {
    for line in [BarLine::RepeatEnd(None), BarLine::Final] {
        assert_eq!(drawn_count(line, 100.0, Place::Inside), None, "{line}");
    }
}

#[test]
fn a_count_takes_no_room_from_the_bars() {
    let mut row = BarRow::new(vec1![bar(), bar()]);
    row.cells[0].end = BarLine::RepeatEnd(None);
    let plain = row_marks(&row);
    row.cells[0].end = BarLine::RepeatEnd(three_times());
    let counted = row_marks(&row);
    assert_eq!(chord_starts(&counted), chord_starts(&plain));
    assert_eq!(counted.len(), plain.len() + 1);
}
