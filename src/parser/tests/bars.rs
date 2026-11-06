use super::directives::kinds;
use super::{assert_errors, error_report, parse_chart};
use crate::model::{Block, Cell};

/// Each cell of each row in `input`, written back: a bar as its slots
/// separated by single spaces, or `%`, and a span as `{n}`.
fn rows(input: &str) -> Vec<Vec<String>> {
    let chart = parse_chart(input).unwrap_or_else(|e| panic!("{input:?} should parse: {e}"));
    chart
        .blocks
        .iter()
        .filter_map(|block| match block {
            Block::Bars(row) => Some(row.cells.iter().map(Cell::to_string).collect()),
            _ => None,
        })
        .collect()
}

#[test]
fn bars_hold_chords_holds_slashes_and_repeats() {
    assert_eq!(
        rows("| Am _ _ A7 | % | G G7 | / / / Dm |"),
        [["Am _ _ A7", "%", "G G7", "/ / / Dm"]]
    );
}

#[test]
fn spaces_around_bar_lines_are_optional() {
    for input in ["|Am|G|", "| Am | G |", "|  Am\t|\tG  |  "] {
        assert_eq!(rows(input), [["Am", "G"]], "{input:?}");
    }
}

#[test]
fn musical_nonsense_is_accepted() {
    assert_eq!(rows("| % | _ B _ _ | | / |"), [["%", "_ B _ _", "", "/"]]);
}

#[test]
fn spans_stand_anywhere_in_a_row() {
    assert_eq!(
        rows("| {2} | F | E7 |\n| Dm7 |{1}| G7 |\n|C| { 12 } |"),
        [
            vec!["{2}", "F", "E7"],
            vec!["Dm7", "{1}", "G7"],
            vec!["C", "{12}"]
        ]
    );
    assert_eq!(
        rows("| {4} |"),
        [["{4}"]],
        "a row of only spans is drawn empty"
    );
}

#[test]
fn bad_spans_are_one_error_each() {
    for (input, message) in [
        (
            "| {} |",
            "a span's width is a whole number of bars, e.g. `{2}`",
        ),
        ("| {two} |", "a span's width is a whole number"),
        ("| {-1} |", "a span's width is a whole number"),
        ("| {0} |", "a span is at least one bar wide, e.g. `{1}`"),
        ("| {2 Fine} |", "text in spans is not supported yet"),
        ("| {2 |", "a span ends with `}`, e.g. `{2}`"),
        ("| {2} Am |", "a span stands alone between two `|`"),
        ("| Am {2} |", "a span stands alone between two `|`"),
        ("| {17} |", "a span is at most 16 bars wide"),
        (
            "| {18446744073709551615} |",
            "a span is at most 16 bars wide",
        ),
        (
            "| {99999999999999999999} |",
            "a span is at most 16 bars wide",
        ),
        ("| Am} |", "`}` closes a span, which starts with `{`"),
        ("| C{2} |", "a span stands alone between two `|`"),
    ] {
        assert_errors(input, &[message]);
    }
}

#[test]
fn full_chord_symbols_are_accepted() {
    assert_eq!(
        rows("| F#m7b5/C# | C7(b9#13)/E | Bb^7 Am/E Am / |"),
        [["F#m7b5/C#", "C7(b9#13)/E", "Bb^7 Am/E Am /"]]
    );
}

#[test]
fn bar_rows_sit_between_other_lines() {
    let input = "= Verse\n| Am | G |\n#page_break\n| % |\n- end";
    assert_eq!(kinds(input), ["text", "bars", "page_break", "bars", "text"]);
    assert_eq!(rows(input), [vec!["Am", "G"], vec!["%"]]);
}

#[test]
fn bad_bars_are_one_error_each() {
    for (input, message) in [
        ("| Am || G |", "`||` is reserved for double bar lines"),
        ("|| Am |", "write `| |` for an empty bar"),
        ("| % Am |", "`%` repeats a whole bar, so it stands alone"),
        ("| Am %|", "`%` repeats a whole bar"),
        (
            "| Am | G",
            "a bar row ends with `|`: close the last bar, or remove what follows it",
        ),
        ("| Am | G | x2", "a bar row ends with `|`"),
        ("|", "a bar row needs at least one bar"),
        ("|  ", "a bar row needs at least one bar"),
    ] {
        assert_errors(input, &[message]);
    }
}

#[test]
fn bad_slots_are_one_error_each() {
    for (input, message) in [
        (
            "| Am_G |",
            "separate slots with spaces: `Am _ G`, or `Am G` for just the chords",
        ),
        ("| A__B7 |", "`A _ _ B7`, or `A B7`"),
        ("| Am_ |", "separate slots with spaces: `Am _`"),
        ("| __ |", "separate slots with spaces: `_ _`\n"),
        ("| am |", "`am` is not a chord, `_`, `/` or `%`"),
        (
            "| Am :|",
            "repeat signs (`|:` and `:|`) are not supported yet",
        ),
        ("|: Am |", "repeat signs"),
        ("|:Am | G |", "repeat signs"),
        ("| Am* |", "`*` follows `Am`: a bar holds chords"),
        ("| Am/ |", "for a beat slash, put a space before it: `Am /`"),
        ("| Cbb5 |", "double accidentals"),
        ("| C/E7 |", "single note"),
        ("| C7(b9 |", "found ' ' expected chord alteration"),
        ("|C7(b9|", "found '|' expected chord alteration"),
    ] {
        assert_errors(input, &[message]);
    }
}

#[test]
fn bad_characters_are_named_by_code_point() {
    for (input, message) in [
        (
            "| Am \u{1B}[31mX |",
            "unprintable character U+001B is not allowed in a bar row",
        ),
        ("|\u{1B}", "unprintable character U+001B"),
        (
            "| Am\u{A0}G |",
            "U+00A0 is not a space or tab: separate slots with spaces",
        ),
        ("| Am |\u{2003}", "U+2003 is not a space or tab"),
    ] {
        assert_errors(input, &[message]);
    }
}

#[test]
fn every_bad_slot_is_reported() {
    assert_errors(
        "| x Am y | Cbb |\n| Am_G |",
        &["`x` is not", "`y` is not", "double accidentals", "`Am _ G`"],
    );
}

#[test]
fn errors_point_into_the_bar() {
    let report = error_report("| Am | C/E7 |");
    assert!(report.contains(":1:11"), "{report}");
    let report = error_report("| Am || G |");
    assert!(report.contains(":1:6"), "{report}");
    let report = error_report("| Am |  {0} |");
    assert!(report.contains(":1:9"), "{report}");
}
