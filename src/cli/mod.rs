//! The `chords` command line: compile chart files to PDF or SVG.
//!
//! Application code: errors are `anyhow` values reported on stderr, and the
//! library's parse diagnostics are rendered against the source.

mod args;
mod compile;
mod diagnostics;
mod output;

pub use args::Args;

use std::process::ExitCode;

/// Compile every input, continuing past failures; fails if any input failed.
pub fn run(args: &Args) -> ExitCode {
    let style = diagnostics::report_style();
    let mut failed = false;
    for input in args.inputs() {
        if let Err(error) = compile::compile(input, args.output()) {
            diagnostics::report_failure(input, &error, style);
            failed = true;
        }
    }
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
