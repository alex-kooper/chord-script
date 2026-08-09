//! Turning chord-script source text into a [`Chart`].
//!
//! [`parse_chart`] is the entire public surface of this module: text goes in,
//! a chart comes out, or a [`ParseError`] carrying every diagnostic found. The
//! grammar lives in a private submodule so the parsing library it uses stays an
//! implementation detail.

use crate::model::Chart;

mod error;
mod grammar;

#[cfg(test)]
mod tests;

pub use error::{ParseError, Result};

/// Parse a complete chart from input text.
pub fn parse_chart(input: &str) -> Result<Chart> {
    match grammar::parse(input) {
        Ok(lines) => Ok(Chart::new(lines)),
        Err(diagnostics) => Err(ParseError::new(input, diagnostics)),
    }
}
