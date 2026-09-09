use super::*;
use crate::model::{Chart, Line, LineLevel, TextSpan, TextStyle};

#[test]
fn test_render_empty_chart() {
    let chart = Chart::new(vec![]);
    let generator = SvgGenerator::with_defaults();
    let svg = generator.render(&chart);

    assert!(svg.contains("<svg"));
    assert!(svg.contains("viewBox"));
}

#[test]
fn test_render_single_line() {
    let chart = Chart::new(vec![Line {
        level: LineLevel::Text,
        left: vec![TextSpan::plain("Left text")],
        center: vec![],
        right: vec![],
    }]);
    let generator = SvgGenerator::with_defaults();
    let svg = generator.render(&chart);

    assert!(svg.contains("Left text"));
    assert!(svg.contains("font-family"));
}

#[test]
fn test_render_three_columns() {
    let chart = Chart::new(vec![Line {
        level: LineLevel::Header1,
        left: vec![TextSpan::plain("Left")],
        center: vec![TextSpan::plain("Center")],
        right: vec![TextSpan::plain("Right")],
    }]);
    let generator = SvgGenerator::with_defaults();
    let svg = generator.render(&chart);

    assert!(svg.contains("Left"));
    assert!(svg.contains("Center"));
    assert!(svg.contains("Right"));
    assert!(svg.contains("text-anchor=\"middle\""));
    assert!(svg.contains("text-anchor=\"end\""));
}

#[test]
fn test_render_styled_spans() {
    let chart = Chart::new(vec![Line {
        level: LineLevel::Text,
        left: vec![
            TextSpan::plain("Normal "),
            TextSpan::new("bold", TextStyle::Bold),
        ],
        center: vec![],
        right: vec![],
    }]);
    let generator = SvgGenerator::with_defaults();
    let svg = generator.render(&chart);

    assert!(svg.contains("Normal"));
    assert!(svg.contains("bold"));
    assert!(svg.contains("font-weight=\"bold\""));
}

#[test]
fn test_header_styling() {
    let chart = Chart::new(vec![Line {
        level: LineLevel::Header1,
        left: vec![TextSpan::plain("Title")],
        center: vec![],
        right: vec![],
    }]);
    let generator = SvgGenerator::with_defaults();
    let svg = generator.render(&chart);

    assert!(svg.contains("font-weight=\"500\""));
    assert!(svg.contains("font-size=\"18\""));
}

#[test]
fn test_custom_config() {
    let config = SvgConfig {
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

    let generator = SvgGenerator::new(config);
    let chart = Chart::new(vec![Line {
        level: LineLevel::Text,
        left: vec![TextSpan::plain("Test")],
        center: vec![],
        right: vec![],
    }]);

    let svg = generator.render(&chart);
    assert!(svg.contains("font-size=\"12\""));
}

#[test]
fn layout_accumulates_height_from_top_margin() {
    let config = SvgConfig::default();
    let margin = config.layout.margin_vertical;
    let mut layout = super::layout::Layout::new(&config);

    // First placement sits at margin + its own height, not at the bare margin.
    assert_eq!(layout.place(14.0), margin + 14.0);
    // Subsequent placements accumulate on top of that.
    assert_eq!(layout.place(20.0), margin + 14.0 + 20.0);
}

#[test]
fn stacked_lines_get_distinct_increasing_y() {
    let chart = Chart::new(vec![
        Line {
            level: LineLevel::Text,
            left: vec![TextSpan::plain("first")],
            center: vec![],
            right: vec![],
        },
        Line {
            level: LineLevel::Text,
            left: vec![TextSpan::plain("second")],
            center: vec![],
            right: vec![],
        },
    ]);
    let svg = SvgGenerator::with_defaults().render(&chart);

    // Defaults: margin_vertical = 28, Text line_height = 14 -> baselines 42 and 56.
    assert!(svg.contains("y=\"42\""));
    assert!(svg.contains("y=\"56\""));
}

#[test]
fn renders_italic_and_bold_italic_styles() {
    let chart = Chart::new(vec![Line {
        level: LineLevel::Text,
        left: vec![TextSpan::new("italic", TextStyle::Italic)],
        center: vec![TextSpan::new("both", TextStyle::BoldItalic)],
        right: vec![],
    }]);
    let svg = SvgGenerator::with_defaults().render(&chart);

    // Italic arm emits font-style; BoldItalic arm emits both weight and style.
    assert!(svg.contains("font-style=\"italic\""));
    assert!(svg.contains("font-weight=\"bold\""));
}
