//! The public pipeline: chart text in, one SVG document per page out.

use chord_script::parser::parse_chart;
use chord_script::render::SvgGenerator;

fn render(source: &str) -> Vec<String> {
    let chart = parse_chart(source).expect("source should parse");
    SvgGenerator::with_defaults()
        .render(&chart)
        .expect("chart should render")
        .into_iter()
        .map(String::from)
        .collect()
}

#[test]
fn page_break_splits_the_chart_into_svg_pages() {
    let pages = render("=== Title\n// the first page\n= Verse <> *Am*\n#page_break\n= Chorus");

    assert_eq!(pages.len(), 2);
    assert!(pages.iter().all(|page| page.starts_with("<svg")));
    assert!(pages[0].contains("Title") && pages[0].contains("Am"));
    assert!(
        !pages[0].contains("the first page"),
        "comments are not rendered"
    );
    assert!(pages[1].contains("Chorus") && !pages[1].contains("Verse"));
}

#[test]
fn chart_without_page_breaks_is_one_page() {
    assert_eq!(render("= one\n= two").len(), 1);
}
