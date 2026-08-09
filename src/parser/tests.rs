use super::parse_chart;
use crate::model::{LineLevel, TextStyle};

#[test]
fn test_parse_empty() {
    let result = parse_chart("");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().lines.len(), 0);
}

#[test]
fn test_parse_header1() {
    let result = parse_chart("=== Left");
    assert!(result.is_ok());
    let chart = result.unwrap();
    assert_eq!(chart.lines.len(), 1);
    assert_eq!(chart.lines[0].level, LineLevel::Header1);
}

#[test]
fn test_parse_alignment() {
    let result = parse_chart("=== <Left <>Center >Right");
    assert!(result.is_ok());
    let chart = result.unwrap();
    assert_eq!(chart.lines.len(), 1);
    assert_eq!(chart.lines[0].level, LineLevel::Header1);

    // Check left column
    assert_eq!(chart.lines[0].left.len(), 1);
    assert_eq!(chart.lines[0].left[0].text.as_ref(), "Left");
    assert_eq!(chart.lines[0].left[0].style, TextStyle::Normal);

    // Check center column
    assert_eq!(chart.lines[0].center.len(), 1);
    assert_eq!(chart.lines[0].center[0].text.as_ref(), "Center");
    assert_eq!(chart.lines[0].center[0].style, TextStyle::Normal);

    // Check right column
    assert_eq!(chart.lines[0].right.len(), 1);
    assert_eq!(chart.lines[0].right[0].text.as_ref(), "Right");
    assert_eq!(chart.lines[0].right[0].style, TextStyle::Normal);
}

#[test]
fn test_parse_multiline() {
    let input = r#"=== <Song Title <>Composer >2024
== <Verse 1
= Intro
- Piano only"#;

    let result = parse_chart(input);
    assert!(result.is_ok());
    let chart = result.unwrap();
    assert_eq!(chart.lines.len(), 4);

    // Header1
    assert_eq!(chart.lines[0].level, LineLevel::Header1);
    assert_eq!(chart.lines[0].left[0].text.as_ref(), "Song Title");
    assert_eq!(chart.lines[0].center[0].text.as_ref(), "Composer");
    assert_eq!(chart.lines[0].right[0].text.as_ref(), "2024");

    // Header2
    assert_eq!(chart.lines[1].level, LineLevel::Header2);
    assert_eq!(chart.lines[1].left[0].text.as_ref(), "Verse 1");

    // Header3
    assert_eq!(chart.lines[2].level, LineLevel::Header3);
    assert_eq!(chart.lines[2].left[0].text.as_ref(), "Intro");

    // Text
    assert_eq!(chart.lines[3].level, LineLevel::Text);
    assert_eq!(chart.lines[3].left[0].text.as_ref(), "Piano only");
}

#[test]
fn test_parse_invalid_input_returns_error() {
    // Mismatched italic markers
    let result = parse_chart("=== *Unclosed italic marker");
    assert!(result.is_err(), "unclosed italic marker should be an error");

    let error = result.unwrap_err();
    assert!(
        !error.is_empty(),
        "error should carry at least one diagnostic"
    );
    assert!(
        !error.report("test").is_empty(),
        "report should produce output"
    );
}

#[test]
fn test_whitespace_only_span_is_not_an_error() {
    // A center marker followed by only spaces must not panic and must not
    // produce a span; it parses to a line with empty columns.
    let result = parse_chart("= <>   ");
    assert!(result.is_ok(), "whitespace-only span should parse cleanly");
    let chart = result.unwrap();
    assert_eq!(chart.lines.len(), 1);
    assert!(chart.lines[0].center.is_empty());
}
