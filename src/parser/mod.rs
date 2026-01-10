use chumsky::prelude::*;
use crate::model::{Chart, Line, LineLevel, TextSpan, TextStyle};
use std::fmt;

/// Parser error type
#[derive(Debug)]
pub struct ParseError {
    /// Individual parse errors from the parser
    pub errors: Vec<String>,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for error in &self.errors {
            writeln!(f, "{}", error)?;
        }
        Ok(())
    }
}

impl std::error::Error for ParseError {}

/// Result type alias for parser operations
pub type Result<T> = std::result::Result<T, ParseError>;

/// Parse a complete chart from input text
pub fn parse_chart(input: &str) -> Result<Chart> {
    let result = chart_parser().parse(input);
    match result.into_result() {
        Ok(lines) => Ok(Chart::new(lines)),
        Err(errors) => {
            let error_messages: Vec<String> = errors
                .into_iter()
                .map(|e| format!("Parse error: {}", e))
                .collect();
            Err(ParseError { errors: error_messages })
        }
    }
}

fn chart_parser<'a>() -> impl Parser<'a, &'a str, Vec<Line>> {
    line_parser()
        .padded()
        .repeated()
        .collect()
        .then_ignore(end())
}

fn line_parser<'a>() -> impl Parser<'a, &'a str, Line> {
    let header1 = just("===").ignored().to(LineLevel::Header1);
    let header2 = just("==").ignored().to(LineLevel::Header2);
    let header3 = just("=").ignored().to(LineLevel::Header3);
    let text_level = just("-").ignored().to(LineLevel::Text);

    let level = header1.or(header2).or(header3).or(text_level);

    (level.padded())
        .then(columns_parser())
        .map(|(level, (left, center, right))| Line {
            level,
            left,
            center,
            right,
        })
}

fn columns_parser<'a>() -> impl Parser<'a, &'a str, (Vec<TextSpan>, Vec<TextSpan>, Vec<TextSpan>)> {
    // Try center marker first (since <> starts with <, it must be checked before <)
    let with_center = just("<>")
        .ignore_then(styled_text_parser().repeated().collect::<Vec<_>>())
        .then(just(">").ignore_then(styled_text_parser().repeated().collect::<Vec<_>>()).or_not())
        .map(|(center, right)| {
            (Vec::new(), center, right.unwrap_or_default())
        });
    
    // Try left marker with optional center and right
    let with_left = just("<")
        .ignore_then(styled_text_parser().repeated().collect::<Vec<_>>())
        .then(just("<>").ignore_then(styled_text_parser().repeated().collect::<Vec<_>>()).or_not())
        .then(just(">").ignore_then(styled_text_parser().repeated().collect::<Vec<_>>()).or_not())
        .map(|((left, center), right)| {
            (left, center.unwrap_or_default(), right.unwrap_or_default())
        });
    
    // Try right marker only (starts with >)
    let with_right = just(">")
        .ignore_then(styled_text_parser().repeated().collect::<Vec<_>>())
        .map(|right| {
            (Vec::new(), Vec::new(), right)
        });
    
    // No markers - just left content
    let no_markers = styled_text_parser()
        .repeated()
        .collect::<Vec<_>>()
        .map(|left| (left, Vec::new(), Vec::new()));
    
    with_center.or(with_left).or(with_right).or(no_markers)
}

fn styled_text_parser<'a>() -> impl Parser<'a, &'a str, TextSpan> {
    let bold_italic = just("***")
        .ignored()
        .then(none_of("*").repeated().at_least(1).collect::<String>())
        .then_ignore(just("***"))
        .map(|(_, text)| TextSpan {
            text: text.trim().to_string(),
            style: TextStyle::BoldItalic,
        });

    let bold = just("**")
        .ignored()
        .then(none_of("*").repeated().at_least(1).collect::<String>())
        .then_ignore(just("**"))
        .map(|(_, text)| TextSpan {
            text: text.trim().to_string(),
            style: TextStyle::Bold,
        });

    let italic = just("*")
        .ignored()
        .then(none_of("*<>\n").repeated().at_least(1).collect::<String>())
        .then_ignore(just("*"))
        .map(|(_, text)| TextSpan {
            text: text.trim().to_string(),
            style: TextStyle::Italic,
        });

    let plain = none_of("<>*\n")
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(|text| TextSpan {
            text: text.trim().to_string(),
            style: TextStyle::Normal,
        });

    bold_italic.or(bold).or(italic).or(plain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let result = parse_chart("");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().lines.len(), 0);
    }

    #[test]
    fn test_parse_header1() {
        let result = parse_chart("=== Left");
        assert!(result.is_ok());
        let chart = result.unwrap();
        assert_eq!(chart.lines.len(), 1);
        assert_eq!(chart.lines[0].level, LineLevel::Header1);
    }

    #[test]
    fn test_parse_alignment() {
        let result = parse_chart("=== <Left <>Center >Right");
        assert!(result.is_ok());
        let chart = result.unwrap();
        assert_eq!(chart.lines.len(), 1);
        assert_eq!(chart.lines[0].level, LineLevel::Header1);
        
        // Check left column
        assert_eq!(chart.lines[0].left.len(), 1);
        assert_eq!(chart.lines[0].left[0].text, "Left");
        assert_eq!(chart.lines[0].left[0].style, TextStyle::Normal);
        
        // Check center column
        assert_eq!(chart.lines[0].center.len(), 1);
        assert_eq!(chart.lines[0].center[0].text, "Center");
        assert_eq!(chart.lines[0].center[0].style, TextStyle::Normal);
        
        // Check right column
        assert_eq!(chart.lines[0].right.len(), 1);
        assert_eq!(chart.lines[0].right[0].text, "Right");
        assert_eq!(chart.lines[0].right[0].style, TextStyle::Normal);
    }

    #[test]
    fn test_parse_multiline() {
        let input = r#"=== <Song Title <>Composer >2024
== <Verse 1
= Intro
- Piano only"#;
        
        let result = parse_chart(input);
        assert!(result.is_ok());
        let chart = result.unwrap();
        assert_eq!(chart.lines.len(), 4);
        
        // Header1
        assert_eq!(chart.lines[0].level, LineLevel::Header1);
        assert_eq!(chart.lines[0].left[0].text, "Song Title");
        assert_eq!(chart.lines[0].center[0].text, "Composer");
        assert_eq!(chart.lines[0].right[0].text, "2024");
        
        // Header2
        assert_eq!(chart.lines[1].level, LineLevel::Header2);
        assert_eq!(chart.lines[1].left[0].text, "Verse 1");
        
        // Header3
        assert_eq!(chart.lines[2].level, LineLevel::Header3);
        assert_eq!(chart.lines[2].left[0].text, "Intro");
        
        // Text
        assert_eq!(chart.lines[3].level, LineLevel::Text);
        assert_eq!(chart.lines[3].left[0].text, "Piano only");
    }

    #[test]
    fn test_parse_invalid_input_returns_error() {
        // Test input with mismatched italic markers
        let invalid_input = "=== *Unclosed italic marker";
        
        let result = parse_chart(invalid_input);
        assert!(result.is_err(), "Expected parser to return an error for unclosed italic marker");
        
        let error = result.unwrap_err();
        assert!(!error.errors.is_empty(), "Expected at least one error message");
    }
}
