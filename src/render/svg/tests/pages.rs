use super::{baseline_of, chart_of, close, left_line, text_cap_height};
use crate::model::{Block, Chart, Inline, LineLevel};
use crate::render::{FontStyle, LayoutConfig, RenderConfig, RenderError, SvgGenerator};

fn text(content: &str) -> Block {
    left_line(LineLevel::Text, vec![Inline::plain(content)]).into()
}

fn render(generator: &SvgGenerator, chart: &Chart) -> Vec<String> {
    generator
        .render(chart)
        .expect("chart should render")
        .into_iter()
        .map(String::from)
        .collect()
}

#[test]
fn render_fails_when_a_line_cannot_fit_on_any_page() {
    // 4pt between the margins: far less than any line's capitals.
    let config = RenderConfig {
        layout: LayoutConfig {
            width: 100.0,
            height: 40.0,
            margin_horizontal: 10.0,
            margin_vertical: 18.0,
        },
        ..RenderConfig::default()
    };
    let chart = chart_of(vec![left_line(
        LineLevel::Text,
        vec![Inline::plain("Title")],
    )]);
    let error = SvgGenerator::new(config)
        .render(&chart)
        .expect_err("no page can hold the line");
    assert!(
        matches!(error, RenderError::RowTallerThanPage { .. }),
        "{error:?}"
    );
}

#[test]
fn a_font_size_or_space_that_is_not_a_fitting_number_is_an_error() {
    let chart = Chart::new(vec![]);
    let cases = [
        (0.0, 6.0, 6.0),
        (f64::NAN, 6.0, 6.0),
        (16.0, -200.0, 6.0),
        (16.0, 6.0, f64::INFINITY),
    ];
    for (size, above, below) in cases {
        let defaults = RenderConfig::default();
        let config = RenderConfig {
            header3: FontStyle {
                size,
                above,
                below,
                ..defaults.header3.clone()
            },
            ..defaults
        };
        let error = SvgGenerator::new(config)
            .render(&chart)
            .expect_err("the lines cannot be drawn");
        let RenderError::InvalidFontStyle { level, .. } = error else {
            panic!("{error:?} for {size}, {above}, {below}");
        };
        assert_eq!(level, LineLevel::Header3);
    }
}

#[test]
fn each_rendered_page_is_a_standalone_svg() {
    let chart = Chart::new(vec![text("first"), Block::PageBreak, text("second")]);
    let pages = render(&SvgGenerator::with_defaults(), &chart);

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
    // Far more than a page holds, however the default sizes are tuned.
    const LINES: usize = 200;
    let lines = (0..LINES)
        .map(|n| left_line(LineLevel::Text, vec![Inline::plain(format!("line {n}"))]))
        .collect();
    let pages = render(&SvgGenerator::with_defaults(), &chart_of(lines));
    assert!(pages.len() > 1);

    let config = RenderConfig::default();
    let top = config.layout.margin_vertical + text_cap_height(config.text.size);
    let mut placed = 0;
    for page in &pages {
        let first = format!("line {placed}");
        assert!(
            close(baseline_of(page, &first), top),
            "{first} starts a page"
        );
        placed += page.matches("<text").count();
    }
    assert_eq!(placed, LINES, "every line lands on some page");
}
