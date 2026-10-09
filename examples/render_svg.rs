use anyhow::{Context, Result};
use chord_script::model::{Block, Chart, Inline, Line, LineLevel, TextStyle};
use chord_script::render::SvgGenerator;

fn main() -> Result<()> {
    // Create a sample chart
    let lines = vec![
        Line {
            level: LineLevel::Header1,
            left: vec![],
            center: vec![Inline::plain("My Song Title")],
            right: vec![],
        },
        Line {
            level: LineLevel::Header2,
            left: vec![Inline::plain("Header 2")],
            center: vec![],
            right: vec![],
        },
        Line {
            level: LineLevel::Header3,
            left: vec![Inline::text("Verse 1", TextStyle::Italic)],
            center: vec![],
            right: vec![],
        },
        Line {
            level: LineLevel::Text,
            left: vec![
                Inline::plain("This is "),
                Inline::text("some", TextStyle::Bold),
                Inline::plain(" text with "),
                Inline::text("styling", TextStyle::Italic),
            ],
            center: vec![],
            right: vec![],
        },
        Line {
            level: LineLevel::Text,
            left: vec![],
            center: vec![Inline::plain("Centered text")],
            right: vec![],
        },
        Line {
            level: LineLevel::Text,
            left: vec![],
            center: vec![],
            right: vec![Inline::plain("Right aligned")],
        },
    ];
    let chart = Chart::new(lines.into_iter().map(Block::from).collect());

    // Print each page's SVG to stdout
    let pages = SvgGenerator::with_defaults()
        .render(&chart)
        .context("rendering chart")?;
    for page in pages {
        println!("{page}");
    }
    Ok(())
}
