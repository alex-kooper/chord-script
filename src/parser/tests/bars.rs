use super::directives::kinds;
use super::{assert_errors, error_report, parse_chart};
use crate::model::{Bar, Block, Slot};

/// Each bar of each row in `input`, written back as its slots separated by
/// single spaces, or `%`.
fn rows(input: &str) -> Vec<Vec<String>> {
    let chart = parse_chart(input).unwrap_or_else(|e| panic!("{input:?} should parse: {e}"));
    chart
        .blocks
        .iter()
        .filter_map(|block| match block {
            Block::Bars(row) => Some(row.bars.iter().map(written).collect()),
            _ => None,
        })
        .collect()
}

fn written(bar: &Bar) -> String {
    match bar {
        Bar::Repeat => "%".to_string(),
        Bar::Slots(slots) => slots
            .iter()
            .map(|slot| match slot {
                Slot::Chord(chord) => chord.to_string(),
                Slot::Hold => "_".to_string(),
                Slot::Slash => "/".to_string(),
            })
            .collect::<Vec<_>>()
            .join(" "),
    }
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
}
