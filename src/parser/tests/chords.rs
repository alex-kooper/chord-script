use super::styles::styles;
use super::{assert_columns, parse_chart, parse_line, report, texts};
use crate::model::{Chord, Inline, Letter, Note, TextStyle};
use TextStyle::{Bold, BoldItalic, Normal};

fn error_report(input: &str) -> String {
    report(&parse_chart(input).expect_err("input should be rejected"))
}

#[test]
fn chord_in_text_is_its_own_inline() {
    let line = parse_line("= Key of [Bb] minor");
    assert_eq!(texts(&line.left), ["Key of ", "[Bb]", " minor"]);
    assert_eq!(
        line.left[1],
        Inline::new(Chord::new(Note::flat(Letter::B), None, None), Normal)
    );
}

#[test]
fn full_chord_symbols_are_accepted() {
    for symbol in ["F#m7b5/C#", "C7(b9#13)/E", "Bbh7", "C^7", "Co7", "Cb5"] {
        let input = format!("- [{symbol}]");
        assert_eq!(texts(&parse_line(&input).left), [format!("[{symbol}]")]);
    }
}

#[test]
fn chords_fit_in_every_column() {
    assert_columns("= [Am] <[C]> [G7]", &["[Am]"], &["[C]"], &["[G7]"]);
}

#[test]
fn text_may_touch_a_chord() {
    assert_columns(
        "- [C]-[G]/[Am]",
        &["[C]", "-", "[G]", "/", "[Am]"],
        &[],
        &[],
    );
}

#[test]
fn column_edges_are_trimmed_around_chords() {
    assert_columns("= <  [Am] x  >", &[], &["[Am]", " x"], &[]);
}

#[test]
fn chords_take_the_surrounding_style() {
    assert_eq!(styles(&parse_line("= *[Am]*").left), [Bold]);
    assert_eq!(
        styles(&parse_line("= _see *[A]*_").left),
        [TextStyle::Italic, BoldItalic]
    );
    assert_eq!(styles(&parse_line("= *a [B] c*").left), [Bold, Bold, Bold]);
}

/// Assert that `input` fails with exactly the errors described by `messages`,
/// each found somewhere in the report.
fn assert_errors(input: &str, messages: &[&str]) {
    let error = parse_chart(input).expect_err("input should be rejected");
    let report = report(&error);
    assert_eq!(error.len(), messages.len(), "{input:?}: {report}");
    for message in messages {
        assert!(
            report.contains(message),
            "{input:?} lacks {message:?}: {report}"
        );
    }
}

#[test]
fn spaces_inside_brackets_are_one_error() {
    for input in ["= [ Am ]", "= [Am ]", "= [ Am]", "= *[\tAm]*"] {
        assert_errors(input, &["no spaces inside `[...]`: write `[Am]`"]);
    }
}

#[test]
fn empty_brackets_are_rejected() {
    for input in ["= a [] b", "= a [  ] b"] {
        assert_errors(input, &["empty `[]`"]);
    }
}

#[test]
fn invalid_chords_are_one_error_each() {
    for (input, message) in [
        ("= [am]", "found 'a' expected note letter (A to G)"),
        ("= [H7]", "found 'H' expected note letter (A to G)"),
        ("= [(Am7)]", "found '(' expected note letter (A to G)"),
        ("= [Cbb5]", "double accidentals"),
        ("= [C/]", "bass note (A to G)"),
        ("= [C/E7]", "single note"),
        ("= [Am", "`[` is not closed"),
        (
            "= [Am7 G]",
            "`[...]` holds a single chord, but ` G` follows `Am7`",
        ),
        (
            "= [Am7?]",
            "`[...]` holds a single chord, but `?` follows `Am7`",
        ),
        (
            "= [C7(b9]",
            "found ']' expected chord alteration, or `)` to close `(`",
        ),
        (
            "= [C7((b9))]",
            "expected chord alteration, or `)` to close `(`",
        ),
        (
            "= [C/E(b9)]",
            "the bass after `/` is a single note, but `(b9)` follows it",
        ),
    ] {
        assert_errors(input, &[message]);
    }
}

#[test]
fn unclosed_bracket_ends_at_structural_characters() {
    for input in ["= *see [Bb7*", "= <[Am> x", "= [Am [G]", "= _[C_"] {
        assert_errors(input, &["`[` is not closed"]);
    }
}

#[test]
fn space_before_closing_bracket_is_where_an_unfinished_chord_ends() {
    assert_errors(
        "= [C7(b9 ]",
        &[
            "no spaces inside `[...]`\n",
            "found ' ' expected chord alteration, or `)` to close `(`",
        ],
    );
    assert!(error_report("= [C7(b9 ]").contains(":1:9 "));
}

#[test]
fn problems_inside_one_bracket_are_reported_together() {
    assert_errors(
        "= [ H7 ]",
        &["no spaces inside `[...]`", "found 'H' expected note letter"],
    );
}

#[test]
fn chord_errors_point_into_the_brackets() {
    let report = error_report("- Key: [C/E7]");
    assert!(report.contains("1:12"), "{report}");
}

#[test]
fn unmatched_closing_bracket_is_rejected() {
    for input in ["= a ] b", "= *see A]*"] {
        let report = error_report(input);
        assert!(
            report.contains("without an opening `[`"),
            "{input:?}: {report}"
        );
    }
}

#[test]
fn parsing_continues_after_an_invalid_quality() {
    let error = parse_chart("= [Cbb5]\n= [C/E7]\n= *unclosed").expect_err("all lines are bad");
    assert_eq!(error.len(), 3, "{}", report(&error));
}
