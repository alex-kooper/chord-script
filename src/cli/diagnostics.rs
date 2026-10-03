//! Reporting failures on stderr.

use chord_script::parser::{ParseError, ReportStyle};
use std::env;
use std::io::{self, IsTerminal};
use std::path::Path;

/// Colour when stderr is a terminal and `NO_COLOR` is unset or empty.
pub(super) fn report_style() -> ReportStyle {
    let no_color = env::var_os("NO_COLOR").is_some_and(|value| !value.is_empty());
    if io::stderr().is_terminal() && !no_color {
        ReportStyle::Colored
    } else {
        ReportStyle::Plain
    }
}

/// Print why compiling `input` failed: parse diagnostics against the source,
/// any other error as one line with its causes.
pub(super) fn report_failure(input: &Path, error: &anyhow::Error, style: ReportStyle) {
    match error.downcast_ref::<ParseError>() {
        Some(parse) => eprint!("{}", parse.report(&input.display().to_string(), style)),
        None => eprintln!("error: {}: {error:#}", input.display()),
    }
}
