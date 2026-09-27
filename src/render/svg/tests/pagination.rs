use super::super::layout::{Page, paginate};
use super::{chart_of, left_line};
use crate::model::{Block, Chart, Line, LineLevel, TextSpan};
use crate::render::svg::{LayoutConfig, RenderError, SvgConfig, SvgGenerator};

/// A 100pt-tall page with 10pt margins: baselines may go down to y = 90, and
/// 80pt fit between the margins.
fn small_page() -> LayoutConfig {
    LayoutConfig {
        width: 100.0,
        height: 100.0,
        margin_horizontal: 10.0,
        margin_vertical: 10.0,
    }
}

/// Fixed heights: `Header1` lines are taller than a whole page, all others 20pt.
fn height_of(line: &Line) -> f64 {
    match line.level {
        LineLevel::Header1 => 200.0,
        _ => 20.0,
    }
}

fn text(content: &str) -> Block {
    left_line(LineLevel::Text, vec![TextSpan::plain(content)]).into()
}

/// An empty line, like a bare `-` marker.
fn spacer() -> Block {
    left_line(LineLevel::Text, vec![]).into()
}

fn tall(content: &str) -> Block {
    left_line(LineLevel::Header1, vec![TextSpan::plain(content)]).into()
}

/// Each page as its lines' `(text, y)` pairs; a spacer's text is `""`.
fn layout(blocks: &[Block]) -> Vec<Vec<(String, f64)>> {
    paginate(blocks, &small_page(), height_of)
        .expect("blocks should fit on pages")
        .iter()
        .map(page_contents)
        .collect()
}

fn page_contents(page: &Page) -> Vec<(String, f64)> {
    page.lines
        .iter()
        .map(|placed| {
            let text: String = placed
                .line
                .left
                .iter()
                .map(|span| -> &str { span.text.as_ref() })
                .collect();
            (text, placed.y)
        })
        .collect()
}

fn entry(text: &str, y: f64) -> (String, f64) {
    (text.to_string(), y)
}

/// Four 20pt lines exactly fill the small page.
fn full_page() -> [Block; 4] {
    [text("a"), text("b"), text("c"), text("d")]
}

fn after_full_page(rest: &[Block]) -> Vec<Block> {
    full_page()
        .into_iter()
        .chain(rest.iter().cloned())
        .collect()
}

#[test]
fn lines_stack_down_from_the_top_margin() {
    assert_eq!(
        layout(&[text("a"), text("b")]),
        [vec![entry("a", 30.0), entry("b", 50.0)]]
    );
}

#[test]
fn line_ending_exactly_at_the_bottom_margin_fits() {
    let pages = layout(&full_page());
    assert_eq!(pages.len(), 1);
    assert_eq!(pages[0].last(), Some(&entry("d", 90.0)));
}

#[test]
fn line_that_does_not_fit_starts_a_new_page() {
    let pages = layout(&after_full_page(&[text("e")]));
    assert_eq!(pages.len(), 2);
    assert_eq!(pages[1], [entry("e", 30.0)]);
}

#[test]
fn line_taller_than_a_page_is_an_error() {
    let error = paginate(&[text("a"), tall("huge")], &small_page(), height_of)
        .expect_err("a 200pt line cannot fit in 80pt");
    assert_eq!(
        error,
        RenderError::LineTallerThanPage {
            line: "line \"huge\"".to_string(),
            height: 200.0,
            available: 80.0,
        }
    );
    assert_eq!(
        error.to_string(),
        "line \"huge\" is 200pt tall, but a page has only 80pt between its margins"
    );
}

#[test]
fn empty_line_taller_than_a_page_is_described_by_its_level() {
    let empty_tall: Block = left_line(LineLevel::Header1, vec![]).into();
    let error = paginate(&[empty_tall], &small_page(), height_of).expect_err("too tall");
    assert!(
        error.to_string().starts_with("an empty Header1 line"),
        "{error}"
    );
}

#[test]
fn render_fails_when_a_line_cannot_fit_on_any_page() {
    // 20pt between the margins; a default H1 line is 24pt tall.
    let config = SvgConfig {
        layout: LayoutConfig {
            height: 40.0,
            ..small_page()
        },
        ..SvgConfig::default()
    };
    let chart = chart_of(vec![left_line(
        LineLevel::Header1,
        vec![TextSpan::plain("Title")],
    )]);
    assert!(SvgGenerator::new(config).render(&chart).is_err());
}

#[test]
fn page_break_starts_a_new_page() {
    assert_eq!(
        layout(&[text("a"), Block::PageBreak, text("b")]),
        [vec![entry("a", 30.0)], vec![entry("b", 30.0)]]
    );
}

#[test]
fn empty_chart_is_one_empty_page() {
    assert_eq!(layout(&[]), [Vec::<(String, f64)>::new()]);
}

#[test]
fn page_breaks_are_literal() {
    let empty = Vec::<(String, f64)>::new;
    assert_eq!(layout(&[Block::PageBreak]), [empty(), empty()]);
    assert_eq!(
        layout(&[text("a"), Block::PageBreak, Block::PageBreak, text("b")]),
        [vec![entry("a", 30.0)], empty(), vec![entry("b", 30.0)]]
    );
    assert_eq!(
        layout(&[text("a"), Block::PageBreak]),
        [vec![entry("a", 30.0)], empty()]
    );
}

#[test]
fn spacers_at_an_automatic_break_are_dropped() {
    let pages = layout(&after_full_page(&[spacer(), spacer(), text("e")]));
    assert_eq!(pages.len(), 2);
    assert_eq!(pages[1], [entry("e", 30.0)]);
}

#[test]
fn spacer_that_fits_at_the_bottom_stays() {
    let pages = layout(&[text("a"), text("b"), text("c"), spacer()]);
    assert_eq!(pages.len(), 1);
    assert_eq!(pages[0].last(), Some(&entry("", 90.0)));
}

#[test]
fn spacers_after_an_explicit_break_are_kept() {
    assert_eq!(
        layout(&[text("a"), Block::PageBreak, spacer(), text("b")]),
        [
            vec![entry("a", 30.0)],
            vec![entry("", 30.0), entry("b", 50.0)]
        ]
    );
}

#[test]
fn spacers_below_content_on_a_new_page_are_kept() {
    let pages = layout(&after_full_page(&[text("e"), spacer(), text("f")]));
    assert_eq!(
        pages[1],
        [entry("e", 30.0), entry("", 50.0), entry("f", 70.0)]
    );
}

#[test]
fn each_rendered_page_is_a_standalone_svg() {
    let chart = Chart::new(vec![text("first"), Block::PageBreak, text("second")]);
    let pages: Vec<String> = SvgGenerator::with_defaults()
        .render(&chart)
        .expect("chart should render")
        .into_iter()
        .map(String::from)
        .collect();

    assert_eq!(pages.len(), 2);
    for page in &pages {
        assert!(page.starts_with("<svg"), "{page}");
        assert!(page.ends_with("</svg>"), "{page}");
    }
    assert!(pages[0].contains("first") && !pages[0].contains("second"));
    assert!(pages[1].contains("second") && !pages[1].contains("first"));
}

#[test]
fn overflow_renders_onto_following_pages() {
    // Defaults: 842pt page, 28pt margins, 14pt text lines -> 56 lines per page.
    let lines = (0..60)
        .map(|n| left_line(LineLevel::Text, vec![TextSpan::plain(format!("line {n}"))]))
        .collect();
    let pages = SvgGenerator::with_defaults()
        .render(&chart_of(lines))
        .expect("chart should render");

    assert_eq!(pages.len(), 2);
    let second: &str = pages[1].as_ref();
    assert!(second.contains(">line 56<"), "{second}");
    assert!(second.contains("y=\"42\""), "restarts at the top margin");
}
