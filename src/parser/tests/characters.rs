use super::{parse_chart, parse_line, report, texts};

fn error_report(input: &str) -> String {
    report(&parse_chart(input).expect_err("input should be rejected"))
}

#[test]
fn test_control_character_in_text_is_an_error() {
    for (input, code) in [
        ("= a\u{1}b", "U+0001"),
        ("= a\u{0}b", "U+0000"),
        ("- \u{1B}x", "U+001B"),
        ("= a\u{7F}", "U+007F"),
    ] {
        let error = parse_chart(input).expect_err("control characters are rejected");
        assert_eq!(error.len(), 1, "{input:?}: {}", report(&error));
        let report = report(&error);
        assert!(
            report.contains(&format!("unprintable character {code}")),
            "{report}"
        );
    }
}

#[test]
fn test_control_character_inside_styles_and_columns_is_an_error() {
    for input in ["= *a\u{1}b*", "= _x\u{1}_", "= <\u{1}>", "= a <> \u{1}"] {
        let report = error_report(input);
        assert!(report.contains("unprintable character U+0001"), "{report}");
    }
}

#[test]
fn test_noncharacter_is_an_error() {
    let report = error_report("= a\u{FFFF}");
    assert!(report.contains("U+FFFF"), "{report}");
}

#[test]
fn test_parsing_continues_after_control_character() {
    let error = parse_chart("= a\u{1}\n= *unclosed").expect_err("both lines are bad");
    assert_eq!(error.len(), 2, "{}", report(&error));
}

#[test]
fn test_tab_is_allowed_in_text() {
    assert_eq!(texts(&parse_line("= a\tb").left), ["a\tb"]);
}

#[test]
fn test_any_script_is_allowed_in_text() {
    assert_eq!(
        texts(&parse_line("- Ґанок 😀 你好").left),
        ["Ґанок 😀 你好"]
    );
}

#[test]
fn test_control_character_in_comment_is_ignored() {
    assert!(parse_chart("// a\u{1}b").is_ok());
}
