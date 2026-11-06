use super::{
    baseline_of, close, close_to_drawn, left_line, polygons, render_default, render_one,
    text_cap_height,
};
use crate::model::{Bar, BarRow, Block, Chart, Chord, Inline, Letter, LineLevel, Note, Slot};
use crate::render::{BarRowStyle, RenderConfig, SvgGenerator, bar_row};
use vec1::vec1;

fn bars() -> Block {
    let chord = Chord::new(Note::natural(Letter::C), None, None);
    BarRow::new(vec1![Bar::Slots(vec![Slot::Chord(chord)])]).into()
}

fn line(level: LineLevel, content: &str) -> Block {
    let inlines = if content.is_empty() {
        vec![]
    } else {
        vec![Inline::plain(content)]
    };
    left_line(level, inlines).into()
}

/// The `y`s of the upright strokes' corners: the bar lines'. A slash leans,
/// and a hanging bass's reaches below the bar lines.
fn bar_line_ys(svg: &str) -> impl Iterator<Item = f64> {
    polygons(svg)
        .into_iter()
        .filter(|corners| corners[0].0 == corners[3].0)
        .flatten()
        .map(|(_, y)| y)
}

fn with_chord_size(chord_size: f64) -> RenderConfig {
    let defaults = RenderConfig::default();
    RenderConfig {
        bar_row: BarRowStyle {
            chord_size,
            ..defaults.bar_row
        },
        ..defaults
    }
}

/// Whatever the chord size, even one whose room for a bass is larger than the
/// text's `above`, the line keeps exactly its `above` from the bar lines.
#[test]
fn a_text_line_after_bars_keeps_its_space_from_their_bar_lines() {
    let chart = Chart::new(vec![bars(), line(LineLevel::Text, "lyrics")]);
    for chord_size in [38.0, 60.0, 23.7] {
        let config = with_chord_size(chord_size);
        let room = bar_row::room_below_bar_lines(&config.bar_row);
        let svg = render_one(&SvgGenerator::new(config.clone()), &chart);
        let bar_lines_end = bar_line_ys(&svg).fold(f64::MIN, f64::max);
        let cap_top = baseline_of(&svg, "lyrics") - text_cap_height(config.text.size);
        let reach = format!("{chord_size}: the bass room is {room}");
        assert!(
            close_to_drawn(bar_lines_end + config.text.above, cap_top),
            "{reach}"
        );
    }
}

#[test]
fn bar_lines_keep_the_larger_space_from_the_text_line_above() {
    let svg = render_default(&Chart::new(vec![line(LineLevel::Text, "Rubato"), bars()]));
    let config = RenderConfig::default();
    let bar_lines_start = bar_line_ys(&svg).fold(f64::MAX, f64::min);
    let space = config.text.below.max(config.bar_row.above);
    let baseline = baseline_of(&svg, "Rubato");
    assert!(close_to_drawn(bar_lines_start, baseline + space));
}

#[test]
fn an_empty_line_between_bar_rows_adds_as_much_as_one_with_text() {
    let between = |middle: Block| {
        let svg = render_default(&Chart::new(vec![bars(), middle, bars()]));
        polygons(&svg)
    };
    let empty = between(line(LineLevel::Text, ""));
    assert_eq!(empty, between(line(LineLevel::Text, "Rubato")));
    assert_ne!(empty, between(line(LineLevel::Header3, "")));
}

#[test]
fn a_header_after_bars_keeps_clear_of_the_room_for_a_bass() {
    let svg = render_default(&Chart::new(vec![
        bars(),
        line(LineLevel::Header2, "Bridge"),
    ]));
    let config = RenderConfig::default();
    let row_bottom = config.layout.margin_vertical + bar_row::height(&config.bar_row);
    let space = config.bar_row.below.max(config.header2.above);
    let expected = row_bottom + space + text_cap_height(config.header2.size);
    assert!(close(baseline_of(&svg, "Bridge"), expected));
}
