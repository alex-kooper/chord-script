use chord_script::model::{Chart, Line, LineLevel, TextSpan, TextStyle};
use chord_script::render::SvgGenerator;

fn main() {
    // Create a sample chart
    let chart = Chart::new(vec![
        Line {
            level: LineLevel::Header1,
            left: vec![],
            center: vec![TextSpan::plain("My Song Title")],
            right: vec![],
        },
        Line {
            level: LineLevel::Header2,
            left: vec![TextSpan::plain("Header 2")],
            center: vec![],
            right: vec![],
        },
        Line {
            level: LineLevel::Header3,
            left: vec![TextSpan::new("Verse 1", TextStyle::Italic)],
            center: vec![],
            right: vec![],
        },
        Line {
            level: LineLevel::Text,
            left: vec![
                TextSpan::plain("This is "),
                TextSpan::new("some", TextStyle::Bold),
                TextSpan::plain(" text with "),
                TextSpan::new("styling", TextStyle::Italic),
            ],
            center: vec![],
            right: vec![],
        },
        Line {
            level: LineLevel::Text,
            left: vec![],
            center: vec![TextSpan::plain("Centered text")],
            right: vec![],
        },
        Line {
            level: LineLevel::Text,
            left: vec![],
            center: vec![],
            right: vec![TextSpan::plain("Right aligned")],
        },
    ]);

    // Generate SVG
    let generator = SvgGenerator::with_defaults();
    let svg = generator.render(&chart);

    // Print to stdout
    println!("{}", svg);
}
