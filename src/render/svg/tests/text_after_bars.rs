use super::{left_line, polygons, render_default, render_one};
use crate::model::{Bar, BarRow, Block, Chart, Chord, Inline, Letter, LineLevel, Note, Slot};
use crate::render::{FontStyle, RenderConfig, SvgGenerator, bar_row};
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

/// The `y` of the text element drawing `word`.
fn baseline_of(svg: &str, word: &str) -> f64 {
    svg.split("<text")
        .find(|element| element.contains(&format!(">{word}<")))
        .and_then(|element| element.split(" y=\"").nth(1))
        .and_then(|rest| rest.split('"').next())
        .expect("the word is drawn")
        .parse()
        .expect("y is a number")
}

/// Where the bar lines end: the lowest point of any upright stroke. A slash
/// leans, and a hanging bass's reaches below the bar lines.
fn bar_lines_end(svg: &str) -> f64 {
    polygons(svg)
        .iter()
        .filter(|corners| corners[0].0 == corners[3].0)
        .flatten()
        .map(|&(_, y)| y)
        .fold(f64::MIN, f64::max)
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn a_text_line_after_bars_starts_where_their_bar_lines_end() {
    let svg = render_default(&Chart::new(vec![bars(), line(LineLevel::Text, "lyrics")]));
    let text = RenderConfig::default().text.line_height;
    assert!(close(
        baseline_of(&svg, "lyrics") - bar_lines_end(&svg),
        text
    ));
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
fn a_header_after_bars_keeps_its_height() {
    let svg = render_default(&Chart::new(vec![
        bars(),
        line(LineLevel::Header2, "Bridge"),
    ]));
    let config = RenderConfig::default();
    let row_bottom = config.layout.margin_vertical + bar_row::height(&config.bar_row);
    let expected = row_bottom + config.header2.line_height;
    assert!(close(baseline_of(&svg, "Bridge"), expected));
}

#[test]
fn a_short_text_line_after_bars_sits_on_the_bottom_of_the_row() {
    let config = RenderConfig {
        text: FontStyle {
            line_height: 1.0,
            ..RenderConfig::default().text
        },
        ..RenderConfig::default()
    };
    let chart = Chart::new(vec![bars(), line(LineLevel::Text, "lyrics")]);
    let svg = render_one(&SvgGenerator::new(config.clone()), &chart);
    let row_bottom = config.layout.margin_vertical + bar_row::height(&config.bar_row);
    assert!(close(baseline_of(&svg, "lyrics"), row_bottom));
}
