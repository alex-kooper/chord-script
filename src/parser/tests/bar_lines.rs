use super::{assert_errors, error_report, parse_chart};
use crate::model::Block;

/// Each row in `input`, written back with single spaces.
fn rows(input: &str) -> Vec<String> {
    let chart = parse_chart(input).unwrap_or_else(|e| panic!("{input:?} should parse: {e}"));
    chart
        .blocks
        .iter()
        .filter_map(|block| match block {
            Block::Bars(row) => Some(row.to_string()),
            _ => None,
        })
        .collect()
}

#[test]
fn every_bar_line_reads_as_written() {
    let row = "||: Am | Dm || G7 :||: C :|| F |||";
    assert_eq!(rows(row), [row]);
}

#[test]
fn bar_lines_stand_anywhere_in_a_row() {
    assert_eq!(
        rows("|| Am | G ||\n||| Am ||:\n| {2} ||: F | E7 :||"),
        ["|| Am | G ||", "||| Am ||:", "| {2} ||: F | E7 :||"]
    );
}

#[test]
fn an_empty_bar_may_stand_between_other_signs() {
    for row in [
        "| Am || || G |",
        "| Am :|| | ||: G |",
        "||: Am ||: ||: G :||",
    ] {
        assert_eq!(rows(row), [row]);
    }
}

#[test]
fn spaces_around_bar_lines_are_optional() {
    assert_eq!(rows("||:Am|G:||:C:||"), ["||: Am | G :||: C :||"]);
}

#[test]
fn bad_bar_lines_are_one_error_each() {
    for (input, message) in [
        ("|: Am |", "write `||:` for a repeat start"),
        ("| Am :|", "write `:||` for a repeat end"),
        (
            "| Am :|: G |",
            "write `:||:` to end a repeat and start the next",
        ),
        (
            "| Am |||| G |",
            "`||||` is not a bar line: write `|`, `||`, `|||`, `||:`, `:||` or `:||:`",
        ),
        ("| Am |:| G |", "`|:|` is not a bar line"),
        ("| Am ::|| G |", "`::||` is not a bar line"),
        (
            "| Am : |",
            "a repeat sign's `:` stands against its bar line, e.g. `||: Am :||`",
        ),
        ("| Am: |", "a repeat sign's `:` stands against its bar line"),
        (
            "| {1} || {1} |",
            "`||` has no bar next to it, so it would not be drawn",
        ),
        ("||: {2} |", "`||:` has no bar next to it"),
        (
            "| Am :|| ||: G |",
            "write `:||:` to end a repeat and start the next, not an empty bar between them",
        ),
        ("| Am :||\t ||: G |", "write `:||:` to end a repeat"),
        (
            ":|| G | C |",
            "a bar row starts with `|`: end the repeat at the end of the row before",
        ),
        (":||: G | C |", "a bar row starts with `|`"),
        ("||", "a bar row needs at least one bar"),
        ("| Am ||| x", "a bar row ends with a bar line"),
    ] {
        assert_errors(input, &[message]);
    }
}

#[test]
fn errors_point_at_the_bar_line() {
    let report = error_report("| Am |: G |");
    assert!(report.contains(":1:6"), "{report}");
    let report = error_report("| {1} :|| {1} |");
    assert!(report.contains(":1:7"), "{report}");
    let report = error_report("| Am :|| ||: G |");
    assert!(report.contains(":1:6"), "{report}");
    let report = error_report("= Verse\n:||: G |");
    assert!(report.contains(":2:1"), "{report}");
}
