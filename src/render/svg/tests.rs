use super::*;
use crate::model::{Block, Chart, Inline, Line, LineLevel, TextStyle};
use crate::render::fonts::{self, Font};
use crate::render::{BarRowStyle, FontStyle, LayoutConfig};

mod bars;
mod chords;
mod pages;
mod pagination;
mod text_and_bars;

/// A chart made only of text lines.
fn chart_of(lines: Vec<Line>) -> Chart {
    Chart::new(lines.into_iter().map(Block::from).collect())
}

/// A line with inlines in the left column only.
fn left_line(level: LineLevel, left: Vec<Inline>) -> Line {
    Line::new(level, left, vec![], vec![])
}

/// Render a chart that must fit on exactly one page.
fn render_one(generator: &SvgGenerator, chart: &Chart) -> String {
    let pages = generator.render(chart).expect("chart should render");
    assert_eq!(pages.len(), 1, "chart should fit on one page");
    pages
        .into_iter()
        .next()
        .expect("length checked above")
        .into()
}

fn render_default(chart: &Chart) -> String {
    render_one(&SvgGenerator::with_defaults(), chart)
}

/// The `y` of the text element drawing `word`.
pub(super) fn baseline_of(svg: &str, word: &str) -> f64 {
    svg.split("<text")
        .find(|element| element.contains(&format!(">{word}<")))
        .and_then(|element| element.split(" y=\"").nth(1))
        .and_then(|rest| rest.split('"').next())
        .expect("the word is drawn")
        .parse()
        .expect("y is a number")
}

pub(super) fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// Like [`close`], for a value read off a polygon, whose points are written
/// with three decimals.
pub(super) fn close_to_drawn(drawn: f64, exact: f64) -> bool {
    (drawn - exact).abs() <= 5e-4 + 1e-9
}

pub(super) fn text_cap_height(size: f64) -> f64 {
    fonts::cap_height(Font::Text(TextStyle::Normal), size)
}

/// The corners of every polygon in `svg`, whatever other attributes it has.
/// A stroke drawn upwards lists bottom left, bottom right, top right, top left.
pub(super) fn polygons(svg: &str) -> Vec<Vec<(f64, f64)>> {
    svg.split("<polygon")
        .skip(1)
        .map(|element| {
            let tag = element.split('>').next().expect("a closed tag");
            let points = tag
                .split(" points=\"")
                .nth(1)
                .and_then(|rest| rest.split('"').next())
                .expect("a polygon has points");
            points
                .split(' ')
                .map(|corner| {
                    let (x, y) = corner.split_once(',').expect("x,y");
                    (x.parse().expect("x"), y.parse().expect("y"))
                })
                .collect()
        })
        .collect()
}

#[test]
fn test_render_empty_chart_is_one_blank_page() {
    let svg = render_default(&Chart::new(vec![]));

    assert!(svg.contains("<svg"));
    assert!(svg.contains("viewBox"));
    assert!(!svg.contains("<text"));
}

#[test]
fn test_render_single_line() {
    let chart = chart_of(vec![left_line(
        LineLevel::Text,
        vec![Inline::plain("Left text")],
    )]);
    let svg = render_default(&chart);

    assert!(svg.contains("Left text"));
    assert!(svg.contains("font-family"));
}

#[test]
fn test_render_three_columns() {
    let chart = chart_of(vec![Line::plain_text(
        LineLevel::Header1,
        "Left",
        "Center",
        "Right",
    )]);
    let svg = render_default(&chart);

    assert!(svg.contains("Left"));
    assert!(svg.contains("Center"));
    assert!(svg.contains("Right"));
    assert!(svg.contains("text-anchor=\"middle\""));
    assert!(svg.contains("text-anchor=\"end\""));
}

#[test]
fn test_render_styled_spans() {
    let chart = chart_of(vec![left_line(
        LineLevel::Text,
        vec![
            Inline::plain("Normal "),
            Inline::text("bold", TextStyle::Bold),
        ],
    )]);
    let svg = render_default(&chart);

    assert!(svg.contains("Normal"));
    assert!(svg.contains("bold"));
    assert!(svg.contains("font-weight=\"bold\""));
}

#[test]
fn test_header_styling() {
    let chart = chart_of(vec![left_line(
        LineLevel::Header1,
        vec![Inline::plain("Title")],
    )]);
    let svg = render_default(&chart);

    assert!(svg.contains("font-family=\"Noto Sans, sans-serif\""));
    assert!(svg.contains("font-weight=\"normal\""));
    assert!(svg.contains("font-size=\"29\""));
}

#[test]
fn test_custom_config() {
    let config = RenderConfig {
        layout: LayoutConfig {
            width: 1000.0,
            height: 800.0,
            margin_horizontal: 50.0,
            margin_vertical: 30.0,
        },
        font_family: "sans-serif".to_string(),
        header1: FontStyle {
            size: 24.0,
            weight: "bold".to_string(),
            above: 20.0,
            below: 12.0,
        },
        header2: FontStyle {
            size: 20.0,
            weight: "bold".to_string(),
            above: 16.0,
            below: 10.0,
        },
        header3: FontStyle {
            size: 18.0,
            weight: "600".to_string(),
            above: 14.0,
            below: 9.0,
        },
        text: FontStyle {
            size: 12.0,
            weight: "normal".to_string(),
            above: 8.0,
            below: 8.0,
        },
        bar_row: BarRowStyle {
            chord_size: 16.0,
            above: 4.0,
            below: 4.0,
        },
    };
    let chart = chart_of(vec![left_line(
        LineLevel::Text,
        vec![Inline::plain("Test")],
    )]);

    let svg = render_one(&SvgGenerator::new(config), &chart);
    assert!(svg.contains("font-size=\"12\""));
}

#[test]
fn the_first_line_has_its_capitals_at_the_top_margin() {
    let chart = chart_of(vec![left_line(
        LineLevel::Text,
        vec![Inline::plain("first")],
    )]);
    let config = RenderConfig::default();
    let cap_height = text_cap_height(config.text.size);
    let baseline = baseline_of(&render_default(&chart), "first");
    assert!(close(baseline, config.layout.margin_vertical + cap_height));
}

#[test]
fn lines_are_kept_apart_by_the_larger_of_the_spaces_they_keep() {
    let config = RenderConfig::default();
    let baselines = |[upper, lower]: [LineLevel; 2]| {
        let svg = render_default(&chart_of(vec![
            left_line(upper, vec![Inline::plain("x")]),
            left_line(lower, vec![Inline::plain("y")]),
        ]));
        baseline_of(&svg, "y") - baseline_of(&svg, "x")
    };
    let text = |level| text_cap_height(config.font_style_for_level(level).size);

    let between_texts = config.text.below.max(config.text.above);
    let header_below = config.header2.below.max(config.text.above);
    let header_above = config.text.below.max(config.header2.above);
    let cases = [
        ([LineLevel::Text, LineLevel::Text], between_texts),
        ([LineLevel::Header2, LineLevel::Text], header_below),
        ([LineLevel::Text, LineLevel::Header2], header_above),
    ];
    for (levels, space) in cases {
        let expected = space + text(levels[1]);
        assert!(close(baselines(levels), expected), "{levels:?}");
    }
}

#[test]
fn an_empty_line_takes_the_space_of_its_level() {
    let chart = chart_of(vec![
        left_line(LineLevel::Header2, vec![]),
        left_line(LineLevel::Text, vec![Inline::plain("after spacer")]),
    ]);
    let svg = render_default(&chart);
    let config = RenderConfig::default();
    let space = config.header2.below.max(config.text.above);
    let expected = config.layout.margin_vertical
        + text_cap_height(config.header2.size)
        + space
        + text_cap_height(config.text.size);

    assert!(close(baseline_of(&svg, "after spacer"), expected));
    assert_eq!(svg.matches("<text").count(), 1, "empty line draws nothing");
}

#[test]
fn spans_are_joined_without_separating_whitespace() {
    let chart = chart_of(vec![left_line(
        LineLevel::Text,
        vec![
            Inline::plain("un"),
            Inline::text("believ", TextStyle::Italic),
            Inline::plain("able"),
        ],
    )]);
    let svg = render_default(&chart);

    assert!(
        svg.contains(
            r#"<tspan>un</tspan><tspan font-style="italic">believ</tspan><tspan>able</tspan>"#
        ),
        "tspans must be adjacent: {svg}"
    );
}

#[test]
fn renders_italic_and_bold_italic_styles() {
    let chart = chart_of(vec![Line::new(
        LineLevel::Text,
        vec![Inline::text("italic", TextStyle::Italic)],
        vec![Inline::text("both", TextStyle::BoldItalic)],
        vec![],
    )]);
    let svg = render_default(&chart);

    // Italic arm emits font-style; BoldItalic arm emits both weight and style.
    assert!(svg.contains("font-style=\"italic\""));
    assert!(svg.contains("font-weight=\"bold\""));
}
