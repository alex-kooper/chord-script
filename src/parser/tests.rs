use super::parse_chart;
use crate::model::{Line, LineLevel, TextSpan};

mod styles;

/// Parse input that must contain exactly one line.
fn parse_line(input: &str) -> Line {
    let chart = parse_chart(input).unwrap_or_else(|e| panic!("{input:?} should parse: {e}"));
    assert_eq!(chart.lines.len(), 1, "{input:?} should be one line");
    chart
        .lines
        .into_iter()
        .next()
        .expect("length checked above")
}

fn texts(spans: &[TextSpan]) -> Vec<&str> {
    spans.iter().map(|span| span.text.as_ref()).collect()
}

/// Assert the text of each column of a single-line input.
fn assert_columns(input: &str, left: &[&str], center: &[&str], right: &[&str]) {
    let line = parse_line(input);
    assert_eq!(texts(&line.left), left, "left column of {input:?}");
    assert_eq!(texts(&line.center), center, "center column of {input:?}");
    assert_eq!(texts(&line.right), right, "right column of {input:?}");
}

#[test]
fn test_parse_empty() {
    let result = parse_chart("");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().lines.len(), 0);
}

#[test]
fn test_parse_header1() {
    assert_eq!(parse_line("=== Left").level, LineLevel::Header1);
}

#[test]
fn test_alignment_left_only() {
    assert_columns("= Verse 1", &["Verse 1"], &[], &[]);
}

#[test]
fn test_alignment_center_only() {
    assert_columns(
        "=== <Rolling in the Deep>",
        &[],
        &["Rolling in the Deep"],
        &[],
    );
}

#[test]
fn test_alignment_right_only() {
    assert_columns("= <> page 1", &[], &[], &["page 1"]);
}

#[test]
fn test_alignment_left_and_right() {
    assert_columns("= Key: Am <> page 1", &["Key: Am"], &[], &["page 1"]);
}

#[test]
fn test_alignment_left_and_center() {
    assert_columns("= Left <Center>", &["Left"], &["Center"], &[]);
}

#[test]
fn test_alignment_center_and_right() {
    assert_columns("= <Center> Right", &[], &["Center"], &["Right"]);
}

#[test]
fn test_alignment_all_three() {
    assert_columns(
        "=== Left <Center> Right",
        &["Left"],
        &["Center"],
        &["Right"],
    );
}

#[test]
fn test_alignment_without_spaces() {
    assert_columns("= L<C>R", &["L"], &["C"], &["R"]);
}

#[test]
fn test_whitespace_only_center_is_not_an_error() {
    // A center holding only spaces must not panic and must not produce a span.
    assert_columns("= <   >", &[], &[], &[]);
}

#[test]
fn test_stray_closing_bracket_is_an_error() {
    assert!(parse_chart("= a > b").is_err());
}

#[test]
fn test_unclosed_center_is_an_error() {
    assert!(parse_chart("= Left <Center").is_err());
}

#[test]
fn test_second_center_is_an_error() {
    assert!(parse_chart("= a <b> c <d>").is_err());
}

#[test]
fn test_escapes_produce_literal_characters() {
    assert_columns(
        r"= 5 \* 3 \< 4 \> 2 \\ snake\_case \[A\]",
        &[r"5 * 3 < 4 > 2 \ snake_case [A]"],
        &[],
        &[],
    );
}

#[test]
fn test_unescaped_backslash_is_literal() {
    assert_columns(r"= C:\path", &[r"C:\path"], &[], &[]);
}

#[test]
fn test_square_brackets_are_reserved() {
    let error = parse_chart("= Key of [A]").expect_err("`[` should be rejected");
    assert!(error.report("test").contains("reserved for notes"));
    assert!(parse_chart("= a ] b").is_err());
}

#[test]
fn test_square_brackets_are_reserved_inside_styles() {
    for input in ["= *see [A]*", "= _see [A]_", "= *bold _see [A]_*"] {
        let error = parse_chart(input).expect_err("brackets inside styles should be rejected");
        assert!(
            error.report("test").contains("reserved for notes"),
            "reserved-bracket message for {input:?}"
        );
    }
}

#[test]
fn test_parsing_continues_after_reserved_bracket() {
    let error = parse_chart("= *see [A]*\n= *unclosed").expect_err("both lines are invalid");
    let report = error.report("test");
    assert!(report.contains("reserved for notes"), "{report}");
    assert!(report.contains("found end of input"), "{report}");
}

#[test]
fn test_bare_markers_are_empty_lines() {
    // Each bare marker is its own empty line; it never takes text from the next line.
    let chart = parse_chart("===\n==\n=\n-").expect("bare markers should parse");
    let levels: Vec<LineLevel> = chart.lines.iter().map(|line| line.level).collect();
    assert_eq!(
        levels,
        [
            LineLevel::Header1,
            LineLevel::Header2,
            LineLevel::Header3,
            LineLevel::Text
        ]
    );
    for line in &chart.lines {
        assert!(line.left.is_empty() && line.center.is_empty() && line.right.is_empty());
    }
}

#[test]
fn test_blank_lines_are_ignored() {
    let chart = parse_chart("\n= a\n\n   \n\t\n= b\n\n").expect("blank lines should parse");
    assert_eq!(chart.lines.len(), 2);
}

#[test]
fn test_crlf_line_endings() {
    let chart = parse_chart("= a\r\n= b\r\n").expect("CRLF input should parse");
    assert_eq!(chart.lines.len(), 2);
    assert_eq!(
        texts(&chart.lines[0].left),
        ["a"],
        "no \\r left in the text"
    );
    assert_eq!(texts(&chart.lines[1].left), ["b"]);
}

#[test]
fn test_every_line_break_separates_lines() {
    for break_char in [
        "\r", "\u{000B}", "\u{000C}", "\u{0085}", "\u{2028}", "\u{2029}",
    ] {
        let input = format!("= a{break_char}= b");
        let chart = parse_chart(&input).unwrap_or_else(|e| panic!("{input:?} should parse: {e}"));
        let lefts: Vec<Vec<&str>> = chart.lines.iter().map(|line| texts(&line.left)).collect();
        assert_eq!(lefts, [["a"], ["b"]], "lines of {input:?}");
    }
}

#[test]
fn test_indented_marker_is_an_error() {
    let error = parse_chart("= a\n  = b").expect_err("indented marker should be rejected");
    assert!(error.report("test").contains("beginning of the line"));
}

#[test]
fn test_marker_requires_space() {
    assert!(parse_chart("===Title").is_err());
    assert!(parse_chart("-note").is_err());
}

#[test]
fn test_marker_characters_mid_line_are_text() {
    assert_columns("= a == b - c", &["a == b - c"], &[], &[]);
}

#[test]
fn test_parse_multiline() {
    let input = "=== Song Title <Composer> 2024
== Verse 1
= Intro
- Piano only";

    let chart = parse_chart(input).expect("multiline chart should parse");
    let levels: Vec<LineLevel> = chart.lines.iter().map(|line| line.level).collect();
    assert_eq!(
        levels,
        [
            LineLevel::Header1,
            LineLevel::Header2,
            LineLevel::Header3,
            LineLevel::Text
        ]
    );

    let title = &chart.lines[0];
    assert_eq!(texts(&title.left), ["Song Title"]);
    assert_eq!(texts(&title.center), ["Composer"]);
    assert_eq!(texts(&title.right), ["2024"]);
    assert_eq!(texts(&chart.lines[3].left), ["Piano only"]);
}

#[test]
fn test_parse_invalid_input_returns_error() {
    let result = parse_chart("=== _Unclosed italic marker");
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
