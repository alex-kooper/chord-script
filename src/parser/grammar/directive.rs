//! Instruction lines: `#name value` directives and `//` comments.

use super::Extra;
use crate::model::Block;
use chumsky::prelude::*;
use chumsky::text::{Char, inline_whitespace};

/// Every directive name the parser understands, for error messages.
const KNOWN_DIRECTIVES: &[&str] = &["page_break"];

/// `#name` or `#name value`: a layout instruction, not content.
///
/// The name is everything up to the first whitespace, so a typo like
/// `#page-break` is reported as one bad name rather than a shorter name plus
/// stray characters. The name and value are checked by [`resolve`]. A bad
/// directive is reported but does not stop the parse, so the rest of the chart
/// is still checked. It then yields `None`.
pub(super) fn directive<'a>() -> impl Parser<'a, &'a str, Option<Block>, Extra<'a>> {
    // Every line break is also whitespace, so the name never runs past its line.
    let name = any()
        .filter(|c: &char| !c.is_whitespace())
        .repeated()
        .at_least(1)
        .to_slice()
        .labelled("directive name (e.g. page_break)");

    let value = inline_whitespace()
        .at_least(1)
        .ignore_then(rest_of_line())
        .map(str::trim)
        .or_not()
        .map(|value| value.filter(|value| !value.is_empty()));

    just('#')
        .ignore_then(name)
        .then(value)
        .validate(|(name, value), e, emitter| {
            resolve(name, value)
                .map_err(|message| emitter.emit(Rich::custom(e.span(), message)))
                .ok()
        })
}

/// A `//` line; its content is ignored.
pub(super) fn comment<'a>() -> impl Parser<'a, &'a str, (), Extra<'a>> {
    just("//").ignore_then(rest_of_line()).ignored()
}

/// Everything up to, but not including, the line break.
fn rest_of_line<'a>() -> impl Parser<'a, &'a str, &'a str, Extra<'a>> {
    any()
        .filter(|c: &char| !c.is_newline())
        .repeated()
        .to_slice()
}

/// Turn a directive's name and value into a block, or explain what is wrong.
fn resolve(name: &str, value: Option<&str>) -> Result<Block, String> {
    if !is_valid_name(name) {
        return Err(format!(
            "invalid directive name `#{name}`: use lowercase letters, digits, and `_`{}",
            did_you_mean(name)
        ));
    }
    match (name, value) {
        ("page_break", None) => Ok(Block::PageBreak),
        ("page_break", Some(value)) => {
            Err(format!("`#page_break` takes no value, but found `{value}`"))
        }
        (unknown, _) => Err(format!(
            "unknown directive `#{unknown}`{}; known directives: {}",
            did_you_mean(unknown),
            known_directives()
        )),
    }
}

/// Names are lowercase `snake_case`: a letter, then letters, digits, or `_`.
fn is_valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// A hint naming the known directive that `name` most likely meant, if any.
///
/// Names are compared ignoring case and punctuation, which catches the likely
/// slips: `#page-break`, `#pagebreak`, `#Page_Break`, `#page_break!`.
fn did_you_mean(name: &str) -> String {
    KNOWN_DIRECTIVES
        .iter()
        .find(|known| normalized(known) == normalized(name))
        .map(|known| format!("; did you mean `#{known}`?"))
        .unwrap_or_default()
}

fn normalized(name: &str) -> String {
    name.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

fn known_directives() -> String {
    KNOWN_DIRECTIVES
        .iter()
        .map(|name| format!("`#{name}`"))
        .collect::<Vec<_>>()
        .join(", ")
}
