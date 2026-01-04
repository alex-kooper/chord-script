use crate::model::{Chart, LineLevel, TextSpan, TextStyle};
use svg::node::element::{Text as SvgText, TSpan};
use svg::Document;

/// Configuration for SVG rendering
#[derive(Debug, Clone)]
pub struct SvgConfig {
    pub width: u32,
    pub height: u32,
    pub margin_horizontal: u32,
    pub margin_vertical: u32,
    pub header1_font_size: u32,
    pub header1_line_height: u32,
    pub header2_font_size: u32,
    pub header2_line_height: u32,
    pub header3_font_size: u32,
    pub header3_line_height: u32,
    pub text_font_size: u32,
    pub text_line_height: u32,
}

impl Default for SvgConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            margin_horizontal: 20,
            margin_vertical: 20,
            header1_font_size: 18,
            header1_line_height: 30,
            header2_font_size: 16,
            header2_line_height: 25,
            header3_font_size: 14,
            header3_line_height: 22,
            text_font_size: 14,
            text_line_height: 20,
        }
    }
}

/// SVG generator that renders charts to SVG format
pub struct SvgGenerator {
    config: SvgConfig,
}

impl SvgGenerator {
    /// Create a new SVG generator with the given configuration
    pub fn new(config: SvgConfig) -> Self {
        Self { config }
    }

    /// Create a new SVG generator with default configuration
    pub fn with_defaults() -> Self {
        Self::new(SvgConfig::default())
    }

    /// Render a Chart to SVG string
    pub fn render(&self, chart: &Chart) -> String {
        let mut document = Document::new()
            .set("viewBox", (0, 0, self.config.width, self.config.height))
            .set("width", self.config.width)
            .set("height", self.config.height);

        let mut y = self.config.margin_vertical;

        for line in &chart.lines {
            // Left column
            if !line.left.is_empty() {
                let text_el = self.render_spans(&line.left, self.config.margin_horizontal, y, line.level);
                document = document.add(text_el);
            }

            // Center column
            if !line.center.is_empty() {
                let text_el = self.render_spans(&line.center, self.config.width / 2, y, line.level)
                    .set("text-anchor", "middle");
                document = document.add(text_el);
            }

            // Right column
            if !line.right.is_empty() {
                let text_el = self.render_spans(
                    &line.right,
                    self.config.width - self.config.margin_horizontal,
                    y,
                    line.level,
                )
                .set("text-anchor", "end");
                document = document.add(text_el);
            }

            y += self.line_height_for_level(line.level);
        }

        document.to_string()
    }

    /// Render a sequence of styled text spans as a single SVG text element with tspans
    fn render_spans(&self, spans: &[TextSpan], x: u32, y: u32, level: LineLevel) -> SvgText {
        let base_font_size = self.font_size_for_level(level);
        let font_weight = match level {
            LineLevel::Header1 | LineLevel::Header2 | LineLevel::Header3 => "bold",
            LineLevel::Text => "normal",
        };

        let mut text_el = SvgText::new("")
            .set("x", x)
            .set("y", y)
            .set("font-family", "sans-serif")
            .set("font-size", base_font_size)
            .set("font-weight", font_weight);

        for span in spans {
            let mut tspan = TSpan::new(&span.text);

            // Apply text styling
            tspan = match span.style {
                TextStyle::Normal => tspan,
                TextStyle::Bold => tspan.set("font-weight", "bold"),
                TextStyle::Italic => tspan.set("font-style", "italic"),
                TextStyle::BoldItalic => tspan
                    .set("font-weight", "bold")
                    .set("font-style", "italic"),
            };

            text_el = text_el.add(tspan);
        }

        text_el
    }

    fn font_size_for_level(&self, level: LineLevel) -> u32 {
        match level {
            LineLevel::Header1 => self.config.header1_font_size,
            LineLevel::Header2 => self.config.header2_font_size,
            LineLevel::Header3 => self.config.header3_font_size,
            LineLevel::Text => self.config.text_font_size,
        }
    }

    fn line_height_for_level(&self, level: LineLevel) -> u32 {
        match level {
            LineLevel::Header1 => self.config.header1_line_height,
            LineLevel::Header2 => self.config.header2_line_height,
            LineLevel::Header3 => self.config.header3_line_height,
            LineLevel::Text => self.config.text_line_height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Line;

    #[test]
    fn test_render_empty_chart() {
        let chart = Chart::new(vec![]);
        let generator = SvgGenerator::with_defaults();
        let svg = generator.render(&chart);

        assert!(svg.contains("<svg"));
        assert!(svg.contains("viewBox"));
    }

    #[test]
    fn test_render_single_line() {
        let chart = Chart::new(vec![Line {
            level: LineLevel::Text,
            left: vec![TextSpan::plain("Left text")],
            center: vec![],
            right: vec![],
        }]);
        let generator = SvgGenerator::with_defaults();
        let svg = generator.render(&chart);

        assert!(svg.contains("Left text"));
        assert!(svg.contains("font-family"));
    }

    #[test]
    fn test_render_three_columns() {
        let chart = Chart::new(vec![Line {
            level: LineLevel::Header1,
            left: vec![TextSpan::plain("Left")],
            center: vec![TextSpan::plain("Center")],
            right: vec![TextSpan::plain("Right")],
        }]);
        let generator = SvgGenerator::with_defaults();
        let svg = generator.render(&chart);

        assert!(svg.contains("Left"));
        assert!(svg.contains("Center"));
        assert!(svg.contains("Right"));
        assert!(svg.contains("text-anchor=\"middle\""));
        assert!(svg.contains("text-anchor=\"end\""));
    }

    #[test]
    fn test_render_styled_spans() {
        let chart = Chart::new(vec![Line {
            level: LineLevel::Text,
            left: vec![
                TextSpan::plain("Normal "),
                TextSpan::new("bold", TextStyle::Bold),
            ],
            center: vec![],
            right: vec![],
        }]);
        let generator = SvgGenerator::with_defaults();
        let svg = generator.render(&chart);

        assert!(svg.contains("Normal"));
        assert!(svg.contains("bold"));
        assert!(svg.contains("font-weight=\"bold\""));
    }

    #[test]
    fn test_header_styling() {
        let chart = Chart::new(vec![Line {
            level: LineLevel::Header1,
            left: vec![TextSpan::plain("Title")],
            center: vec![],
            right: vec![],
        }]);
        let generator = SvgGenerator::with_defaults();
        let svg = generator.render(&chart);

        assert!(svg.contains("font-weight=\"bold\""));
        assert!(svg.contains("font-size=\"18\""));
    }

    #[test]
    fn test_custom_config() {
        let config = SvgConfig {
            width: 1000,
            height: 800,
            margin_horizontal: 50,
            margin_vertical: 30,
            header1_font_size: 24,
            header1_line_height: 40,
            text_font_size: 12,
            text_line_height: 18,
            ..Default::default()
        };

        let generator = SvgGenerator::new(config);
        let chart = Chart::new(vec![Line {
            level: LineLevel::Text,
            left: vec![TextSpan::plain("Test")],
            center: vec![],
            right: vec![],
        }]);

        let svg = generator.render(&chart);
        assert!(svg.contains("font-size=\"12\""));
    }
}
