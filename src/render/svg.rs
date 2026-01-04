use crate::model::{Chart, LineLevel, TextSpan, TextStyle};
use svg::node::element::{Text as SvgText, TSpan};
use svg::Document;

/// Font style configuration (size, weight, line-height)
#[derive(Debug, Clone)]
pub struct FontStyle {
    pub size: f64,
    pub weight: String,
    pub line_height: f64,
}

/// Configuration for SVG rendering
#[derive(Debug, Clone)]
pub struct SvgConfig {
    // Layout
    pub width: f64,
    pub height: f64,
    pub margin_horizontal: f64,
    pub margin_vertical: f64,
    
    // Font (single font family for all text)
    pub font_family: String,
    
    // Font styles per level
    pub header1: FontStyle,
    pub header2: FontStyle,
    pub header3: FontStyle,
    pub text: FontStyle,
}

impl Default for SvgConfig {
    fn default() -> Self {
        Self {
            // A4 portrait (ISO): 210mm × 297mm (1:1 coordinate system)
            width: 210.0,
            height: 297.0,
            margin_horizontal: 10.0,
            margin_vertical: 10.0,
            
            // Font (single font family for all text)
            font_family: "sans-serif".to_string(),
            
            // Font styles per level
            header1: FontStyle {
                size: 7.5,
                weight: "500".to_string(),
                line_height: 11.0,
            },
            header2: FontStyle {
                size: 7.0,
                weight: "450".to_string(),
                line_height: 10.0,
            },
            header3: FontStyle {
                size: 6.0,
                weight: "420".to_string(),
                line_height: 9.0,
            },
            text: FontStyle {
                size: 4.3,
                weight: "normal".to_string(),
                line_height: 6.0,
            },
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
            .set("viewBox", format!("0 0 {} {}", self.config.width as i32, self.config.height as i32))
            .set("width", format!("{}mm", self.config.width))
            .set("height", format!("{}mm", self.config.height));

        let mut y = self.config.margin_vertical;

        for line in &chart.lines {
            y += self.line_height_for_level(line.level);
            
            // Left column
            if !line.left.is_empty() {
                let text_el = self.render_spans(&line.left, self.config.margin_horizontal, y, line.level);
                document = document.add(text_el);
            }

            // Center column
            if !line.center.is_empty() {
                let text_el = self.render_spans(&line.center, self.config.width / 2.0, y, line.level)
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
        }

        document.to_string()
    }

    /// Render a sequence of styled text spans as a single SVG text element with tspans
    fn render_spans(&self, spans: &[TextSpan], x: f64, y: f64, level: LineLevel) -> SvgText {
        let base_font_size = self.font_size_for_level(level);
        let base_font_weight = self.font_weight_for_level(level);

        let mut text_el = SvgText::new("")
            .set("x", x)
            .set("y", y)
            .set("font-family", self.config.font_family.as_str())
            .set("font-size", base_font_size)
            .set("font-weight", base_font_weight);

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

    fn font_size_for_level(&self, level: LineLevel) -> f64 {
        match level {
            LineLevel::Header1 => self.config.header1.size,
            LineLevel::Header2 => self.config.header2.size,
            LineLevel::Header3 => self.config.header3.size,
            LineLevel::Text => self.config.text.size,
        }
    }

    fn font_weight_for_level(&self, level: LineLevel) -> &str {
        match level {
            LineLevel::Header1 => &self.config.header1.weight,
            LineLevel::Header2 => &self.config.header2.weight,
            LineLevel::Header3 => &self.config.header3.weight,
            LineLevel::Text => &self.config.text.weight,
        }
    }

    fn line_height_for_level(&self, level: LineLevel) -> f64 {
        match level {
            LineLevel::Header1 => self.config.header1.line_height,
            LineLevel::Header2 => self.config.header2.line_height,
            LineLevel::Header3 => self.config.header3.line_height,
            LineLevel::Text => self.config.text.line_height,
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

        assert!(svg.contains("font-weight=\"500\""));
        assert!(svg.contains("font-size=\"7.5\""));
    }

    #[test]
    fn test_custom_config() {
        let config = SvgConfig {
            width: 1000.0,
            height: 800.0,
            margin_horizontal: 50.0,
            margin_vertical: 30.0,
            font_family: "sans-serif".to_string(),
            header1: FontStyle {
                size: 24.0,
                weight: "bold".to_string(),
                line_height: 40.0,
            },
            header2: FontStyle {
                size: 20.0,
                weight: "bold".to_string(),
                line_height: 30.0,
            },
            header3: FontStyle {
                size: 18.0,
                weight: "600".to_string(),
                line_height: 27.0,
            },
            text: FontStyle {
                size: 12.0,
                weight: "normal".to_string(),
                line_height: 18.0,
            },
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
