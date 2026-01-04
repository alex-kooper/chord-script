use chord_script::parser::parse_chart;
use chord_script::render::SvgGenerator;
use std::env;
use std::fs;
use std::path::Path;
use std::process;

fn main() {
    // Get the filename from the first command-line argument
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input-file>", args[0]);
        eprintln!("Example: {} song.charts", args[0]);
        process::exit(1);
    }

    let input_file = &args[1];
    
    // Read the input file
    let input_content = match fs::read_to_string(input_file) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", input_file, err);
            process::exit(1);
        }
    };

    // Parse the chart
    let chart = match parse_chart(&input_content) {
        Ok(chart) => chart,
        Err(err) => {
            eprintln!("Parse error in '{}':", input_file);
            eprintln!("{}", err);
            process::exit(1);
        }
    };

    // Generate SVG
    let generator = SvgGenerator::with_defaults();
    let svg = generator.render(&chart);

    // Determine output filename (replace extension with .svg)
    let input_path = Path::new(input_file);
    let output_file = input_path.with_extension("svg");

    // Write the SVG file
    match fs::write(&output_file, svg) {
        Ok(_) => {
            println!("Successfully rendered: {}", output_file.display());
        }
        Err(err) => {
            eprintln!("Error writing to '{}': {}", output_file.display(), err);
            process::exit(1);
        }
    }
}
