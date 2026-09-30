use super::*;
use crate::model::{Block, Chart, Line, LineLevel, TextSpan, TextStyle};
use crate::render::{FontStyle, LayoutConfig};

mod pagination;

/// A chart made only of text lines.
fn chart_of(lines: Vec<Line>) -> Chart {
    Chart::new(lines.into_iter().map(Block::from).collect())
}

/// A line with spans in the left column only.
fn left_line(level: LineLevel, spans: Vec<TextSpan>) -> Line {
    Line::new(level, spans, vec![], vec![])
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
        vec![TextSpan::plain("Left text")],
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
            TextSpan::plain("Normal "),
            TextSpan::new("bold", TextStyle::Bold),
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
        vec![TextSpan::plain("Title")],
    )]);
    let svg = render_default(&chart);

    assert!(svg.contains("font-family=\"Noto Sans, sans-serif\""));
    assert!(svg.contains("font-weight=\"normal\""));
    assert!(svg.contains("font-size=\"18\""));
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
            line_height: 40.0,
        },
        header2: FontStyle {
            size: 20.0,
            weight: "bold".to_string(),
            line_height: 30.0,
        },
        header3: FontStyle {
            size: 18.0,
            weight: "600".to_string(),
            line_height: 27.0,
        },
        text: FontStyle {
            size: 12.0,
            weight: "normal".to_string(),
            line_height: 18.0,
        },
    };
    let chart = chart_of(vec![left_line(
        LineLevel::Text,
        vec![TextSpan::plain("Test")],
    )]);

    let svg = render_one(&SvgGenerator::new(config), &chart);
    assert!(svg.contains("font-size=\"12\""));
}

#[test]
fn stacked_lines_get_distinct_increasing_y() {
    let chart = chart_of(vec![
        left_line(LineLevel::Text, vec![TextSpan::plain("first")]),
        left_line(LineLevel::Text, vec![TextSpan::plain("second")]),
    ]);
    let svg = render_default(&chart);

    // Defaults: margin_vertical = 28, Text line_height = 14 -> baselines 42 and 56.
    assert!(svg.contains("y=\"42\""));
    assert!(svg.contains("y=\"56\""));
}

#[test]
fn empty_line_takes_its_level_height() {
    let chart = chart_of(vec![
        left_line(LineLevel::Header2, vec![]),
        left_line(LineLevel::Text, vec![TextSpan::plain("after spacer")]),
    ]);
    let svg = render_default(&chart);

    // Defaults: margin 28 + empty H2 (20) + Text (14) -> baseline 62.
    assert!(svg.contains("y=\"62\""));
    assert_eq!(svg.matches("<text").count(), 1, "empty line draws nothing");
}

#[test]
fn spans_are_joined_without_separating_whitespace() {
    let chart = chart_of(vec![left_line(
        LineLevel::Text,
        vec![
            TextSpan::plain("un"),
            TextSpan::new("believ", TextStyle::Italic),
            TextSpan::plain("able"),
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
        vec![TextSpan::new("italic", TextStyle::Italic)],
        vec![TextSpan::new("both", TextStyle::BoldItalic)],
        vec![],
    )]);
    let svg = render_default(&chart);

    // Italic arm emits font-style; BoldItalic arm emits both weight and style.
    assert!(svg.contains("font-style=\"italic\""));
    assert!(svg.contains("font-weight=\"bold\""));
}
