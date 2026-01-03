use chumsky::prelude::*;
use crate::model::{Chart, Line, LineLevel, TextSpan, TextStyle};
use ariadne::{Report, ReportKind, Label, Source};

/// Parse a complete chart from input text
pub fn parse_chart(input: &str) -> Result<Chart, Vec<String>> {
    chart_parser()
        .parse(input)
        .map(Chart::new)
        .map_err(|errors| {
            errors
                .into_iter()
                .map(|e| {
                    let mut output = Vec::new();
                    Report::build(ReportKind::Error, (), e.span().start)
                        .with_message("Parse error")
                        .with_label(
                            Label::new(e.span())
                                .with_message(e.to_string())
                        )
                        .finish()
                        .write(Source::from(input), &mut output)
                        .unwrap();
                    String::from_utf8(output).unwrap()
                })
                .collect()
        })
}

fn chart_parser() -> impl Parser<char, Vec<Line>, Error = Simple<char>> {
    line_parser()
        .padded()
        .repeated()
        .then_ignore(end())
}

fn line_parser() -> impl Parser<char, Line, Error = Simple<char>> {
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

fn columns_parser() -> impl Parser<char, (Vec<TextSpan>, Vec<TextSpan>, Vec<TextSpan>), Error = Simple<char>> {
    let text_span = styled_text_parser().boxed();
    
    // Parse optional left section
    let left_part = (just("<").ignored().then(text_span.clone().repeated()))
        .or(text_span.clone().repeated().map(|spans| ((), spans)))
        .map(|(_, spans)| spans);
    
    // Parse optional center section
    let center_part = (just("<>").ignored().then(text_span.clone().repeated()))
        .or(just("").to(((), Vec::new())))
        .map(|(_, spans)| spans);
    
    // Parse optional right section
    let right_part = (just(">").ignored().then(text_span.repeated()))
        .or(just("").to(((), Vec::new())))
        .map(|(_, spans)| spans);
    
    left_part
        .then(center_part)
        .then(right_part)
        .map(|((left, center), right)| (left, center, right))
}

fn styled_text_parser() -> impl Parser<char, TextSpan, Error = Simple<char>> {
    let bold_italic = just("***")
        .ignored()
        .then(none_of(['*'].as_ref()).repeated().at_least(1).collect::<String>())
        .then_ignore(just("***"))
        .map(|(_, text)| TextSpan {
            text: text.trim().to_string(),
            style: TextStyle::BoldItalic,
        });

    let bold = just("**")
        .ignored()
        .then(none_of(['*'].as_ref()).repeated().at_least(1).collect::<String>())
        .then_ignore(just("**"))
        .map(|(_, text)| TextSpan {
            text: text.trim().to_string(),
            style: TextStyle::Bold,
        });

    let italic = just("*")
        .ignored()
        .then(none_of(['*', '<', '>'].as_ref()).repeated().at_least(1).collect::<String>())
        .then_ignore(just("*"))
        .map(|(_, text)| TextSpan {
            text: text.trim().to_string(),
            style: TextStyle::Italic,
        });

    let plain = none_of(['<', '>', '*', '\n'].as_ref())
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
        
        let errors = result.unwrap_err();
        assert!(!errors.is_empty(), "Expected at least one error message");
    }
}
