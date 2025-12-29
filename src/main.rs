mod parser;
mod renderer;
mod exporter;
mod chart;

use clap::{Parser, ValueEnum};
use anyhow::{Context, Result};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "clean-chart")]
#[command(about = "Generate chord charts in SVG, PNG, and PDF formats")]
#[command(version)]
struct Cli {
    /// Input .cchart file
    #[arg(value_name = "FILE")]
    input: PathBuf,

    /// Output format
    #[arg(short, long, value_enum, default_value = "svg")]
    format: OutputFormat,

    /// Output file (defaults to input name with appropriate extension)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum OutputFormat {
    Svg,
    Png,
    Pdf,
}

impl OutputFormat {
    fn extension(&self) -> &str {
        match self {
            OutputFormat::Svg => "svg",
            OutputFormat::Png => "png",
            OutputFormat::Pdf => "pdf",
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Verify input file has .cchart extension
    if cli.input.extension().and_then(|s| s.to_str()) != Some("cchart") {
        anyhow::bail!("Input file must have .cchart extension");
    }

    // Read input file
    let content = std::fs::read_to_string(&cli.input)
        .with_context(|| format!("Failed to read input file: {}", cli.input.display()))?;

    // Parse the chart
    let chart = parser::parse_chart(&content)
        .with_context(|| "Failed to parse chart file")?;

    // Generate SVG
    let svg_content = renderer::render_to_svg(&chart)?;

    // Determine output file
    let output = cli.output.unwrap_or_else(|| {
        let mut path = cli.input.clone();
        path.set_extension(cli.format.extension());
        path
    });

    // Export based on format
    match cli.format {
        OutputFormat::Svg => {
            std::fs::write(&output, svg_content)
                .with_context(|| format!("Failed to write SVG file: {}", output.display()))?;
        }
        OutputFormat::Png => {
            exporter::export_png(&svg_content, &output)
                .with_context(|| "Failed to export PNG")?;
        }
        OutputFormat::Pdf => {
            exporter::export_pdf(&svg_content, &output)
                .with_context(|| "Failed to export PDF")?;
        }
    }

    println!("✓ Chart generated: {}", output.display());
    Ok(())
}
