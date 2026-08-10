//! SVG rendering of a [`Chart`].
//!
//! Configuration lives in [`config`]; this module owns the generator that turns
//! a chart into an SVG document.

use crate::model::{Chart, LineLevel, TextSpan, TextStyle};
use svg::node::element::{Text as SvgText, TSpan};
use svg::Document;

mod config;

#[cfg(test)]
mod tests;

pub use config::{FontStyle, LayoutConfig, SvgConfig};

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
            .set(
                "viewBox",
                format!("0 0 {} {}", layout.width as i32, layout.height as i32),
            )
            .set("width", format!("{}pt", layout.width))
            .set("height", format!("{}pt", layout.height));

        let mut y = layout.margin_vertical;

        for line in &chart.lines {
            y += self.config.line_height_for_level(line.level);

            // Left column
            if !line.left.is_empty() {
                let text_el = self.render_spans(&line.left, layout.margin_horizontal, y, line.level);
                document = document.add(text_el);
            }

            // Center column
            if !line.center.is_empty() {
                let text_el = self
                    .render_spans(&line.center, layout.width / 2.0, y, line.level)
                    .set("text-anchor", "middle");
                document = document.add(text_el);
            }

            // Right column
            if !line.right.is_empty() {
                let text_el = self
                    .render_spans(
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
        let style = self.config.font_style_for_level(level);

        let mut text_el = SvgText::new("")
            .set("x", x)
            .set("y", y)
            .set("font-family", self.config.font_family.as_str())
            .set("font-size", style.size)
            .set("font-weight", style.weight.as_str());

        for span in spans {
            let text: &str = span.text.as_ref();
            let mut tspan = TSpan::new(text);

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
}
