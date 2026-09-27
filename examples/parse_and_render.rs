use anyhow::{Context, Result, bail};
use chord_script::parser::parse_chart;
use chord_script::render::SvgGenerator;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

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

    let pages = SvgGenerator::with_defaults()
        .render(&chart)
        .context("rendering chart")?;

    // Temporary naming until the CLI decides on output paths: song-1.svg, song-2.svg, ...
    for (index, page) in pages.iter().enumerate() {
        let output_file = page_path(Path::new(input_file), index + 1);
        fs::write(&output_file, page.as_ref() as &str)
            .with_context(|| format!("writing SVG to '{}'", output_file.display()))?;
        println!("Rendered: {}", output_file.display());
    }
    Ok(())
}

fn page_path(input: &Path, page_number: usize) -> PathBuf {
    let stem = input.file_stem().unwrap_or_default().to_string_lossy();
    input.with_file_name(format!("{stem}-{page_number}.svg"))
}
