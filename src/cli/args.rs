//! Command-line arguments.

use clap::error::ErrorKind;
use clap::{CommandFactory, Parser};
use std::path::{Path, PathBuf};

/// Render chord-script charts to PDF or SVG.
///
/// Each chart is written next to its input as a PDF unless `--output` says
/// otherwise. A multi-page SVG is written as `name-1.svg`, `name-2.svg`, ...
#[derive(Debug, Parser)]
#[command(name = "chords", version)]
pub struct Args {
    /// Chart files to compile.
    #[arg(required = true, value_name = "INPUT")]
    inputs: Vec<PathBuf>,

    /// Output file; its extension (.pdf or .svg) selects the format.
    /// Only allowed with a single input.
    #[arg(short, long, value_name = "OUTPUT")]
    output: Option<PathBuf>,
}

impl Args {
    /// Parse the process arguments, exiting with a usage error if they are invalid.
    pub fn parse_or_exit() -> Self {
        let args = Self::parse();
        if args.output.is_some() && args.inputs.len() > 1 {
            Self::command()
                .error(
                    ErrorKind::ArgumentConflict,
                    "--output can only be used with a single input",
                )
                .exit();
        }
        args
    }

    pub(super) fn inputs(&self) -> &[PathBuf] {
        &self.inputs
    }

    pub(super) fn output(&self) -> Option<&Path> {
        self.output.as_deref()
    }
}
