use anyhow::{Context, Result, bail};
use chord_script::model::Chart;
use chord_script::parser::parse_chart;
use chord_script::render::{PdfGenerator, SvgGenerator};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        bail!("usage: {} <input-file>", args[0]);
    }

    let input_file = Path::new(&args[1]);
    let input_content = fs::read_to_string(input_file)
        .with_context(|| format!("reading input file '{}'", input_file.display()))?;

    let chart = match parse_chart(&input_content) {
        Ok(chart) => chart,
        Err(error) => {
            eprint!("{}", error.report(&input_file.display().to_string()));
            bail!("failed to parse '{}'", input_file.display());
        }
    };

    write_svg_pages(&chart, input_file)?;
    write_pdf(&chart, input_file)
}

/// Temporary naming until the CLI decides on output paths: song-1.svg, song-2.svg, ...
fn write_svg_pages(chart: &Chart, input: &Path) -> Result<()> {
    let pages = SvgGenerator::with_defaults()
        .render(chart)
        .context("rendering chart to SVG")?;

    for (index, page) in pages.iter().enumerate() {
        let stem = input.file_stem().unwrap_or_default().to_string_lossy();
        let output = input.with_file_name(format!("{stem}-{}.svg", index + 1));
        write(&output, page.as_ref() as &str)?;
    }
    Ok(())
}

fn write_pdf(chart: &Chart, input: &Path) -> Result<()> {
    let document = PdfGenerator::with_defaults()
        .render(chart)
        .context("rendering chart to PDF")?;
    write(&input.with_extension("pdf"), document.as_ref() as &[u8])
}

fn write(output: &PathBuf, contents: impl AsRef<[u8]>) -> Result<()> {
    fs::write(output, contents).with_context(|| format!("writing '{}'", output.display()))?;
    println!("Rendered: {}", output.display());
    Ok(())
}
