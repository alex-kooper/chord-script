use super::*;
use crate::model::{Block, Chord, Inline, Letter, Line, LineLevel, Note, TextStyle};

fn text(content: &str) -> Block {
    Line::new(
        LineLevel::Text,
        vec![Inline::plain(content)],
        vec![],
        vec![],
    )
    .into()
}

fn render(generator: &PdfGenerator, blocks: Vec<Block>) -> String {
    let document = generator
        .render(&Chart::new(blocks))
        .expect("chart should render");
    let bytes: &[u8] = document.as_ref();
    String::from_utf8_lossy(bytes).into_owned()
}

fn render_default(blocks: Vec<Block>) -> String {
    render(&PdfGenerator::with_defaults(), blocks)
}

/// Page objects are `/Type/Page`; the page tree root is `/Type/Pages`.
fn page_count(pdf: &str) -> usize {
    pdf.matches("/Type/Page").count() - pdf.matches("/Type/Pages").count()
}

fn with_layout(width: f64, height: f64) -> PdfGenerator {
    PdfGenerator::new(RenderConfig {
        layout: LayoutConfig {
            width,
            height,
            ..LayoutConfig::default()
        },
        ..RenderConfig::default()
    })
}

#[test]
fn output_is_a_complete_pdf_file() {
    let pdf = render_default(vec![text("hello")]);
    assert!(pdf.starts_with("%PDF-"), "{}", &pdf[..20]);
    assert!(pdf.trim_end().ends_with("%%EOF"));
}

#[test]
fn chart_pages_become_pages_of_one_document() {
    let pdf = render_default(vec![text("first"), Block::PageBreak, text("second")]);
    assert_eq!(page_count(&pdf), 2);
}

#[test]
fn empty_chart_is_one_blank_page() {
    assert_eq!(page_count(&render_default(vec![])), 1);
}

#[test]
fn default_page_is_a4_in_points() {
    let pdf = render_default(vec![text("a")]);
    assert!(pdf.contains("/MediaBox[0 0 595 842]"), "A4 is 595 × 842pt");
}

#[test]
fn page_size_follows_the_layout() {
    let pdf = render(&with_layout(500.0, 300.0), vec![text("a")]);
    assert!(pdf.contains("/MediaBox[0 0 500 300]"));
}

#[test]
fn text_is_embedded_in_the_bundled_font_faces() {
    let line = Line::new(
        LineLevel::Text,
        vec![
            Inline::plain("Привіт, "),
            Inline::text("жирний", TextStyle::Bold),
            Inline::text(" курсив", TextStyle::Italic),
        ],
        vec![],
        vec![],
    );
    let pdf = render_default(vec![line.into()]);

    // Subset fonts are named `ABCDEF+PostScriptName`.
    for face in ["NotoSans-Regular", "NotoSans-Bold", "NotoSans-Italic"] {
        assert!(
            pdf.contains(&format!("+{face}")),
            "{face} should be embedded"
        );
    }
    assert!(
        !pdf.contains("+NotoSans-BoldItalic"),
        "unused faces are left out"
    );
    assert!(
        pdf.contains("/FontFile2"),
        "font data is embedded, not referenced"
    );
}

#[test]
fn rendering_is_deterministic() {
    let blocks = || vec![text("same"), Block::PageBreak, text("output")];
    assert_eq!(render_default(blocks()), render_default(blocks()));
}

#[test]
fn page_without_positive_size_is_an_error() {
    let chart = Chart::new(vec![text("a")]);
    let error = with_layout(0.0, 842.0)
        .render(&chart)
        .expect_err("a zero-width page cannot exist");
    assert_eq!(
        error,
        RenderError::InvalidPageSize {
            width: 0.0,
            height: 842.0
        }
    );
}

fn with_font_family(font_family: &str) -> PdfGenerator {
    PdfGenerator::new(RenderConfig {
        font_family: font_family.to_string(),
        ..RenderConfig::default()
    })
}

#[test]
fn font_family_without_a_bundled_font_is_an_error() {
    let error = with_font_family("Times New Roman")
        .render(&Chart::new(vec![text("a")]))
        .expect_err("the PDF would have no text");
    assert_eq!(
        error,
        RenderError::FontUnavailable {
            family: "Times New Roman".to_string(),
            bundled: "Noto Sans",
        }
    );
}

#[test]
fn font_family_list_falls_back_to_generic_sans_serif() {
    let pdf = render(
        &with_font_family("Times New Roman, sans-serif"),
        vec![text("a")],
    );
    assert!(pdf.contains("+NotoSans-Regular"));
}

#[test]
fn character_missing_from_the_bundled_font_is_an_error() {
    let key = Line::new(
        LineLevel::Text,
        vec![Inline::plain("Key: ")],
        vec![],
        vec![Inline::text("B♭", TextStyle::Bold)],
    );
    let error = PdfGenerator::with_defaults()
        .render(&Chart::new(vec![text("fine"), key.into()]))
        .expect_err("a flat cannot be drawn");
    assert_eq!(
        error.to_string(),
        "line \"Key: B♭\" contains `♭` (U+266D), which the bundled font cannot draw"
    );
}

#[test]
fn chords_are_checked_and_described_by_their_symbol() {
    let chord = Inline::new(
        Chord::new(Note::flat(Letter::B), None, None),
        TextStyle::Normal,
    );
    let key = Line::new(
        LineLevel::Text,
        vec![Inline::plain("Key: "), chord, Inline::plain(" ♭")],
        vec![],
        vec![],
    );
    let error = PdfGenerator::with_defaults()
        .render(&Chart::new(vec![key.into()]))
        .expect_err("a typed flat cannot be drawn");
    assert_eq!(
        error.to_string(),
        "line \"Key: Bb ♭\" contains `♭` (U+266D), which the bundled font cannot draw"
    );
}

#[test]
fn emoji_and_cjk_are_errors() {
    for content in ["smile 😀", "你好"] {
        let result = PdfGenerator::with_defaults().render(&Chart::new(vec![text(content)]));
        assert!(
            matches!(result, Err(RenderError::UnsupportedCharacter { .. })),
            "{content}"
        );
    }
}

#[test]
fn tab_is_drawn_as_space_not_rejected() {
    assert_eq!(page_count(&render_default(vec![text("a\tb")])), 1);
}

#[test]
fn fractional_page_size_is_kept_exactly_in_svg_and_pdf() {
    let config = RenderConfig {
        layout: LayoutConfig {
            width: 595.28,
            height: 841.89,
            ..LayoutConfig::default()
        },
        ..RenderConfig::default()
    };
    let chart = Chart::new(vec![text("a")]);

    let svg_pages = SvgGenerator::new(config.clone())
        .render(&chart)
        .expect("renders");
    let svg: &str = svg_pages[0].as_ref();
    assert!(svg.contains("viewBox=\"0 0 595.28 841.89\""), "{svg}");

    let pdf = render(&PdfGenerator::new(config), chart.blocks);
    assert!(pdf.contains("/MediaBox[0 0 595.28 841.89]"));
}

#[test]
fn layout_errors_are_reported_as_for_svg() {
    // 20pt between the 28pt margins; a 14pt text line fits, a 24pt title does not.
    let title = Line::new(
        LineLevel::Header1,
        vec![Inline::plain("Title")],
        vec![],
        vec![],
    );
    let error = with_layout(595.0, 76.0)
        .render(&Chart::new(vec![title.into()]))
        .expect_err("title is taller than the page");
    assert!(
        matches!(error, RenderError::LineTallerThanPage { .. }),
        "{error}"
    );
}
