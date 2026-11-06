use super::super::layout::{Metrics, Page, Row, paginate};
use super::left_line;
use crate::model::{Bar, BarRow, Block, Inline, LineLevel};
use crate::render::inline::typed_text;
use crate::render::{LayoutConfig, RenderError};
use vec1::vec1;

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

/// Fixed sizes: `Header1` lines are taller than a whole page, bar rows are
/// 30pt, and all other lines 20pt. Rows touch, except a line after a bar row
/// reaches 10pt up into it, and a `Header2` line keeps 5pt above it.
struct Fixed;

impl Metrics for Fixed {
    fn height(&self, row: Row) -> f64 {
        match row {
            Row::Text(line) if line.level == LineLevel::Header1 => 200.0,
            Row::Text(_) => 20.0,
            Row::Bars(_) => 30.0,
        }
    }

    fn space_between(&self, upper: Row, lower: Row) -> f64 {
        match (upper, lower) {
            (_, Row::Text(line)) if line.level == LineLevel::Header2 => 5.0,
            (Row::Bars(_), Row::Text(_)) => -10.0,
            _ => 0.0,
        }
    }
}

/// A one-bar row holding `%`.
fn bars() -> Block {
    BarRow::new(vec1![Bar::Repeat]).into()
}

fn text(content: &str) -> Block {
    left_line(LineLevel::Text, vec![Inline::plain(content)]).into()
}

/// An empty line, like a bare `-` marker.
fn spacer() -> Block {
    left_line(LineLevel::Text, vec![]).into()
}

fn tall(content: &str) -> Block {
    left_line(LineLevel::Header1, vec![Inline::plain(content)]).into()
}

/// Each page as its rows' `(text, y)` pairs; a spacer's text is `""`, and a
/// bar row's is the row as written.
fn layout(blocks: &[Block]) -> Vec<Vec<(String, f64)>> {
    paginate(blocks, &small_page(), &Fixed)
        .expect("blocks should fit on pages")
        .iter()
        .map(page_contents)
        .collect()
}

fn page_contents(page: &Page) -> Vec<(String, f64)> {
    page.rows
        .iter()
        .map(|placed| {
            let text = match placed.row {
                Row::Text(line) => line.left.iter().map(typed_text).collect(),
                Row::Bars(bars) => bars.to_string(),
            };
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
fn rows_keep_their_space_apart_but_not_at_the_top_of_a_page() {
    let header = || -> Block { left_line(LineLevel::Header2, vec![Inline::plain("h")]).into() };
    assert_eq!(
        layout(&[header(), text("a"), header(), Block::PageBreak, header()]),
        [
            vec![entry("h", 30.0), entry("a", 50.0), entry("h", 75.0)],
            vec![entry("h", 30.0)]
        ]
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
    let error = paginate(&[text("a"), tall("huge")], &small_page(), &Fixed)
        .expect_err("a 200pt line cannot fit in 80pt");
    assert_eq!(
        error,
        RenderError::RowTallerThanPage {
            row: "line \"huge\"".to_string(),
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
    let error = paginate(&[empty_tall], &small_page(), &Fixed).expect_err("too tall");
    assert!(
        error.to_string().starts_with("an empty Header1 line"),
        "{error}"
    );
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
fn bar_rows_stack_with_lines_and_break_like_them() {
    assert_eq!(
        layout(&[text("a"), bars(), text("b")]),
        [vec![
            entry("a", 30.0),
            entry("| % |", 60.0),
            entry("b", 70.0)
        ]]
    );
    let pages = layout(&[text("a"), text("b"), text("c"), bars()]);
    assert_eq!(pages[1], [entry("| % |", 40.0)]);
}

#[test]
fn a_line_after_bars_that_starts_a_page_sits_at_the_top_margin() {
    let pages = layout(&[text("a"), bars(), bars(), text("b")]);
    assert_eq!(pages[1], [entry("b", 30.0)]);
    let pages = layout(&[text("a"), bars(), bars(), spacer(), text("b")]);
    assert_eq!(pages[1], [entry("b", 30.0)]);
    let pages = layout(&[bars(), Block::PageBreak, text("b")]);
    assert_eq!(pages[1], [entry("b", 30.0)]);
}

#[test]
fn bar_row_taller_than_a_page_is_an_error_naming_the_row() {
    let page = LayoutConfig {
        height: 40.0,
        ..small_page()
    };
    let error = paginate(&[bars()], &page, &Fixed).expect_err("30pt cannot fit in 20pt");
    assert_eq!(
        error.to_string(),
        "bar row \"| % |\" is 30pt tall, but a page has only 20pt between its margins"
    );
}
