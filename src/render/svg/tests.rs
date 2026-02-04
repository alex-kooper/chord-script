use super::*;
use crate::model::Line;

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
