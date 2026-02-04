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

/// Layout configuration (page dimensions and margins)
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    pub width: f64,
    pub height: f64,
    pub margin_horizontal: f64,
    pub margin_vertical: f64,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            // A4 portrait: 595pt × 842pt (1pt = 1/72 inch)
            width: 595.0,
            height: 842.0,
            margin_horizontal: 28.0,  // ~10mm
            margin_vertical: 28.0,    // ~10mm
        }
    }
}

/// Configuration for SVG rendering
#[derive(Debug, Clone)]
pub struct SvgConfig {
    // Layout
    pub layout: LayoutConfig,
    
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
            layout: LayoutConfig::default(),
            
            // Font (single font family for all text)
            font_family: "sans-serif".to_string(),
            
            // Font styles per level
            header1: FontStyle {
                size: 18.0,
                weight: "500".to_string(),
                line_height: 24.0,
            },
            header2: FontStyle {
                size: 14.0,
                weight: "450".to_string(),
                line_height: 20.0,
            },
            header3: FontStyle {
                size: 11.0,
                weight: "420".to_string(),
                line_height: 16.0,
            },
            text: FontStyle {
                size: 10.0,
                weight: "normal".to_string(),
                line_height: 14.0,
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
        let layout = &self.config.layout;
        
        let mut document = Document::new()
            .set("viewBox", format!("0 0 {} {}", layout.width as i32, layout.height as i32))
            .set("width", format!("{}pt", layout.width))
            .set("height", format!("{}pt", layout.height));

        let mut y = layout.margin_vertical;

        for line in &chart.lines {
            y += self.line_height_for_level(line.level);
            
            // Left column
            if !line.left.is_empty() {
                let text_el = self.render_spans(&line.left, layout.margin_horizontal, y, line.level);
                document = document.add(text_el);
            }

            // Center column
            if !line.center.is_empty() {
                let text_el = self.render_spans(&line.center, layout.width / 2.0, y, line.level)
                    .set("text-anchor", "middle");
                document = document.add(text_el);
            }

            // Right column
            if !line.right.is_empty() {
                let text_el = self.render_spans(
                    &line.right,
                    layout.width - layout.margin_horizontal,
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
        let style = self.font_style_for_level(level);

        let mut text_el = SvgText::new("")
            .set("x", x)
            .set("y", y)
            .set("font-family", self.config.font_family.as_str())
            .set("font-size", style.size)
            .set("font-weight", style.weight.as_str());

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

    fn font_style_for_level(&self, level: LineLevel) -> &FontStyle {
        match level {
            LineLevel::Header1 => &self.config.header1,
            LineLevel::Header2 => &self.config.header2,
            LineLevel::Header3 => &self.config.header3,
            LineLevel::Text => &self.config.text,
        }
    }

    fn line_height_for_level(&self, level: LineLevel) -> f64 {
        self.font_style_for_level(level).line_height
    }
}

#[cfg(test)]
mod tests;
