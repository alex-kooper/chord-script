use super::{parse_chart, parse_lines, report};
use crate::model::{Block, LineLevel};

/// The kind of each block, as a short label: `"text"` or `"page_break"`.
fn kinds(input: &str) -> Vec<&'static str> {
    let chart = parse_chart(input).unwrap_or_else(|e| panic!("{input:?} should parse: {e}"));
    chart
        .blocks
        .iter()
        .map(|block| match block {
            Block::Text(_) => "text",
            Block::PageBreak => "page_break",
        })
        .collect()
}

fn error_report(input: &str) -> String {
    report(&parse_chart(input).expect_err("input should be rejected"))
}

#[test]
fn test_page_break_between_lines() {
    assert_eq!(
        kinds("= a\n#page_break\n= b"),
        ["text", "page_break", "text"]
    );
}

#[test]
fn test_page_break_allows_trailing_whitespace() {
    assert_eq!(kinds("#page_break   "), ["page_break"]);
}

#[test]
fn test_page_breaks_are_kept_literally() {
    assert_eq!(
        kinds("#page_break\n#page_break\n= a\n#page_break"),
        ["page_break", "page_break", "text", "page_break"]
    );
}

#[test]
fn test_page_break_takes_no_value() {
    let report = error_report("#page_break now");
    assert!(report.contains("takes no value"), "{report}");
}

#[test]
fn test_unknown_directive_lists_known_ones() {
    let report = error_report("#pagebreak");
    assert!(
        report.contains("unknown directive `#pagebreak`"),
        "{report}"
    );
    assert!(report.contains("`#page_break`"), "{report}");
}

#[test]
fn test_unknown_directive_suggests_a_close_name() {
    let report = error_report("#pagebreak");
    assert!(report.contains("did you mean `#page_break`?"), "{report}");
}

#[test]
fn test_unrelated_unknown_directive_has_no_suggestion() {
    let report = error_report("#tempo 120");
    assert!(report.contains("unknown directive `#tempo`"), "{report}");
    assert!(!report.contains("did you mean"), "{report}");
}

#[test]
fn test_directive_name_must_be_lowercase() {
    let report = error_report("#Page_Break");
    assert!(
        report.contains("invalid directive name `#Page_Break`"),
        "{report}"
    );
    assert!(parse_chart("#PAGE_BREAK").is_err());
    assert!(parse_chart("#1page").is_err());
}

#[test]
fn test_bare_hash_expects_a_directive_name() {
    let report = error_report("#");
    assert!(report.contains("directive name"), "{report}");
}

#[test]
fn test_mistyped_name_is_one_error_with_a_suggestion() {
    for input in ["#page-break", "#page_break!", "#Page-Break"] {
        let error = parse_chart(input).expect_err("mistyped name should be rejected");
        assert_eq!(error.len(), 1, "{input}: {}", report(&error));

        let report = report(&error);
        assert!(report.contains("invalid directive name"), "{report}");
        assert!(report.contains("did you mean `#page_break`?"), "{report}");
    }
}

#[test]
fn test_directive_name_ends_at_whitespace() {
    let report = error_report("#page-break now");
    assert!(
        report.contains("invalid directive name `#page-break`:"),
        "{report}"
    );
}

#[test]
fn test_parsing_continues_after_bad_directive() {
    let report = error_report("#pagebreak\n= *unclosed");
    assert!(report.contains("unknown directive"), "{report}");
    assert!(report.contains("found end of input"), "{report}");
}

#[test]
fn test_indented_directive_is_an_error() {
    let report = error_report("  #page_break");
    assert!(report.contains("beginning of the line"), "{report}");
}

#[test]
fn test_hash_inside_text_is_plain_text() {
    let lines = parse_lines("= Take #2 <> F#m");
    assert_eq!(super::texts(&lines[0].left), ["Take #2"]);
    assert_eq!(super::texts(&lines[0].right), ["F#m"]);
}

#[test]
fn test_comments_are_ignored() {
    let lines = parse_lines("// title first\n=== Title\n//\n// = not a line");
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].level, LineLevel::Header1);
}

#[test]
fn test_comment_may_contain_markup_characters() {
    assert_eq!(parse_lines("// *unclosed [A] <center").len(), 0);
}

#[test]
fn test_slashes_inside_text_are_plain_text() {
    let lines = parse_lines("= Verse 1 // quiet");
    assert_eq!(super::texts(&lines[0].left), ["Verse 1 // quiet"]);
}

#[test]
fn test_indented_comment_is_an_error() {
    let report = error_report("   // note");
    assert!(report.contains("beginning of the line"), "{report}");
}

#[test]
fn test_single_slash_is_not_a_comment() {
    assert!(parse_chart("/ note").is_err());
}
