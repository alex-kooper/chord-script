use chord_script::parser::{ReportStyle, parse_chart};

fn main() {
    // Each of these inputs is expected to fail; this example demonstrates the
    // rendered diagnostics rather than propagating a single error.
    let invalid_inputs = vec![
        ("Unclosed italic", "=== _Unclosed italic marker"),
        ("Unclosed bold", "=== *Unclosed bold marker"),
        ("Crossed markers", "=== *_bold italic*_"),
        ("Markdown double star", "=== **bold**"),
        ("Reserved bracket", "= Key of [A] minor"),
        ("No level marker", "This line has no level marker"),
        ("Unclosed center", "= Left <Center"),
        ("Stray closing bracket", "= 3 > 2"),
        ("Second center", "= Left <Center> Right <Again>"),
        ("Indented marker", "= Verse\n   = Chorus"),
        ("No space after marker", "===Title"),
        ("Unknown directive", "#pagebreak"),
        ("Directive with unexpected value", "#page_break now"),
    ];

    for (description, input) in invalid_inputs {
        println!("Testing: {description}");
        println!("Input: {input:?}");

        match parse_chart(input) {
            Ok(chart) => {
                println!("Parsed successfully: {} blocks", chart.blocks.len());
            }
            Err(error) => {
                print!("{}", error.report(description, ReportStyle::Colored));
            }
        }
        println!();
    }
}
