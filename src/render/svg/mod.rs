//! SVG rendering of a [`Chart`].
//!
//! Configuration lives in [`config`]; vertical placement in [`layout`]; the
//! drawing of individual lines in [`line`]. This module owns the generator that
//! wires them together: it builds the SVG document and runs the layout loop.

use crate::model::Chart;
use svg::Document;

mod config;
mod layout;
mod line;

#[cfg(test)]
mod tests;

pub use config::{FontStyle, LayoutConfig, SvgConfig};

use layout::Layout;

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
        let page = &self.config.layout;

        let mut document = Document::new()
            .set(
                "viewBox",
                format!("0 0 {} {}", page.width as i32, page.height as i32),
            )
            .set("width", format!("{}pt", page.width))
            .set("height", format!("{}pt", page.height));

        let mut layout = Layout::new(&self.config);

        for chart_line in &chart.lines {
            let height = line::height_of(chart_line, &self.config);
            let y = layout.place(height);
            for element in line::render(chart_line, &self.config, y) {
                document = document.add(element);
            }
        }

        document.to_string()
    }
}
