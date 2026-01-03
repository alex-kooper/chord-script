use chord_script::parser::parse_chart;

fn main() {
    // Test various invalid inputs
    let invalid_inputs = vec![
        ("Unclosed italic", "=== *Unclosed italic marker"),
        ("Unclosed bold", "=== **Unclosed bold marker"),
        ("Unclosed bold-italic", "=== ***Unclosed bold-italic marker"),
        ("No level marker", "This line has no level marker"),
        ("Right and left columns without center", "-Left column >right column"),
        ("Center without left", "= <>Center only"),
    ];

    for (description, input) in invalid_inputs {
        println!("Testing: {}", description);
        println!("Input: {:?}", input);
        
        match parse_chart(input) {
            Ok(lines) => {
                println!("✓ Parsed successfully: {} lines", lines.len());
            }
            Err(errors) => {
                println!("✗ Parse errors:");
                for error in errors {
                    println!("  - {}", error);
                }
            }
        }
        println!();
    }
}
