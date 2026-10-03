//! Compiling one chart file: read, parse, render, write.

use super::output::{Format, Output};
use anyhow::{Context, Result, ensure};
use chord_script::model::Chart;
use chord_script::parser::parse_chart;
use chord_script::render::{PdfGenerator, SvgGenerator};
use std::fs;
use std::path::Path;

/// Compile `input` to the requested output, or to a PDF next to it.
///
/// A parse failure is returned as the library's `ParseError`, so the caller
/// can render its diagnostics against the source.
pub(super) fn compile(input: &Path, requested: Option<&Path>) -> Result<()> {
    let output = Output::resolve(input, requested)?;
    let source =
        fs::read_to_string(input).with_context(|| format!("reading '{}'", input.display()))?;
    let chart = parse_chart(&source)?;

    match output.format() {
        Format::Pdf => write_pdf(&chart, input, &output),
        Format::Svg => write_svg(&chart, input, &output),
    }
}

fn write_pdf(chart: &Chart, input: &Path, output: &Output) -> Result<()> {
    let document = PdfGenerator::with_defaults()
        .render(chart)
        .context("rendering PDF")?;
    write(input, output.path(), document.as_ref())
}

fn write_svg(chart: &Chart, input: &Path, output: &Output) -> Result<()> {
    let pages = SvgGenerator::with_defaults()
        .render(chart)
        .context("rendering SVG")?;
    let paths = output.page_paths(pages.len());
    for (page, path) in pages.iter().zip(&paths) {
        write(input, path, page.as_ref() as &str)?;
    }
    Ok(())
}

fn write(input: &Path, path: &Path, contents: impl AsRef<[u8]>) -> Result<()> {
    ensure!(
        !is_same_file(input, path),
        "output '{}' would overwrite the input",
        path.display()
    );
    fs::write(path, contents).with_context(|| format!("writing '{}'", path.display()))
}

fn is_same_file(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}
