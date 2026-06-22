use chord_script::parser::parse_chart;

fn main() {
    // Each of these inputs is expected to fail; this example demonstrates the
    // rendered diagnostics rather than propagating a single error.
    let invalid_inputs = vec![
        ("Unclosed italic", "=== *Unclosed italic marker"),
        ("Unclosed bold", "=== **Unclosed bold marker"),
        ("Unclosed bold-italic", "=== ***Unclosed bold-italic marker"),
        ("No level marker", "This line has no level marker"),
    ];

    for (description, input) in invalid_inputs {
        println!("Testing: {description}");
        println!("Input: {input:?}");

        match parse_chart(input) {
            Ok(chart) => {
                println!("Parsed successfully: {} lines", chart.lines.len());
            }
            Err(error) => {
                print!("{}", error.report(description));
            }
        }
        println!();
    }
}
