use anyhow::{Context, Result, bail};
use chord_script::parser::parse_chart;
use chord_script::render::SvgGenerator;
use std::env;
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        bail!("usage: {} <input-file>", args[0]);
    }

    let input_file = &args[1];
    let input_content = fs::read_to_string(input_file)
        .with_context(|| format!("reading input file '{input_file}'"))?;

    let chart = match parse_chart(&input_content) {
        Ok(chart) => chart,
        Err(error) => {
            eprint!("{}", error.report(input_file));
            bail!("failed to parse '{input_file}'");
        }
    };

    let generator = SvgGenerator::with_defaults();
    let svg = generator.render(&chart);

    let output_file = Path::new(input_file).with_extension("svg");
    fs::write(&output_file, svg)
        .with_context(|| format!("writing SVG to '{}'", output_file.display()))?;

    println!("Successfully rendered: {}", output_file.display());
    Ok(())
}
