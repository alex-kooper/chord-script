use super::{parse_chart, parse_line, texts};
use crate::model::{TextSpan, TextStyle};
use TextStyle::{Bold, BoldItalic, Italic, Normal};

fn styles(spans: &[TextSpan]) -> Vec<TextStyle> {
    spans.iter().map(|span| span.style).collect()
}

/// Assert the text and style of every span in the left column.
fn assert_spans(input: &str, expected: &[(&str, TextStyle)]) {
    let line = parse_line(input);
    let (expected_texts, expected_styles): (Vec<&str>, Vec<TextStyle>) =
        expected.iter().copied().unzip();
    assert_eq!(texts(&line.left), expected_texts, "texts of {input:?}");
    assert_eq!(styles(&line.left), expected_styles, "styles of {input:?}");
}

#[test]
fn test_bold_and_italic() {
    assert_spans("= *bold*", &[("bold", Bold)]);
    assert_spans("= _italic_", &[("italic", Italic)]);
}

#[test]
fn test_nesting_order_does_not_matter() {
    assert_spans("= *_both_*", &[("both", BoldItalic)]);
    assert_spans("= _*both*_", &[("both", BoldItalic)]);
}

#[test]
fn test_mixed_nesting_flattens_into_spans() {
    assert_spans(
        "= *bold _both_ bold*",
        &[("bold ", Bold), ("both", BoldItalic), (" bold", Bold)],
    );
    assert_spans(
        "= _it *both* it_",
        &[("it ", Italic), ("both", BoldItalic), (" it", Italic)],
    );
}

#[test]
fn test_spaces_around_styled_spans_are_kept() {
    assert_spans(
        "= This is a *test* song",
        &[("This is a ", Normal), ("test", Bold), (" song", Normal)],
    );
    assert_spans("= *a* _b_", &[("a", Bold), (" ", Normal), ("b", Italic)]);
}

#[test]
fn test_styles_inside_words() {
    assert_spans(
        "= un_believ_able",
        &[("un", Normal), ("believ", Italic), ("able", Normal)],
    );
}

#[test]
fn test_column_edges_are_trimmed() {
    let line = parse_line("= *Key* <  _Title_  > *page 1*   ");
    assert_eq!(texts(&line.left), ["Key"]);
    assert_eq!(texts(&line.center), ["Title"]);
    assert_eq!(texts(&line.right), ["page 1"]);
}

#[test]
fn test_styled_text_inside_center() {
    let line = parse_line("= <*Title*>");
    assert_eq!(texts(&line.center), ["Title"]);
    assert_eq!(styles(&line.center), [Bold]);
}

#[test]
fn test_crossed_markers_are_an_error() {
    assert!(parse_chart("= *_x*_").is_err());
}

#[test]
fn test_markdown_double_star_is_an_error() {
    // `**` is an empty bold, so old Markdown-style charts fail loudly.
    assert!(parse_chart("= **bold**").is_err());
}

#[test]
fn test_unclosed_bold_is_an_error() {
    assert!(parse_chart("= *bold").is_err());
}

#[test]
fn test_bold_does_not_swallow_center_marker() {
    // `<` ends the bold text, so the bold is unclosed rather than absorbing `<Title>`.
    assert!(parse_chart("= *Key <Title>*").is_err());
}

#[test]
fn test_bold_does_not_span_lines() {
    assert!(parse_chart("= *a\n- b*").is_err());
}
