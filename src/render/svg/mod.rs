//! SVG rendering of a [`Chart`].
//!
//! Pagination and vertical placement live in [`layout`]; the drawing of
//! individual lines in [`line`]. This module owns the generator that wires
//! them together: it paginates the chart, then builds one SVG document per
//! page.

use super::{RenderConfig, Result};
use crate::model::Chart;
use derive_more::{AsRef, Display, Into};
use svg::Document;

mod layout;
mod line;

#[cfg(test)]
mod tests;

use layout::Page;

/// One rendered page: a standalone SVG document.
#[derive(Debug, Clone, PartialEq, Eq, Display, AsRef, Into)]
#[as_ref(str)]
pub struct SvgPage(String);

/// SVG generator that renders charts to SVG format
pub struct SvgGenerator {
    config: RenderConfig,
}

impl SvgGenerator {
    /// Create a new SVG generator with the given configuration
    pub fn new(config: RenderConfig) -> Self {
        Self { config }
    }

    /// Create a new SVG generator with default configuration
    pub fn with_defaults() -> Self {
        Self::new(RenderConfig::default())
    }

    /// Render a chart to SVG, one document per page.
    ///
    /// There is always at least one page; an empty chart renders as one blank
    /// page. Fails if a line is too tall to fit on any page.
    pub fn render(&self, chart: &Chart) -> Result<Vec<SvgPage>> {
        let pages = layout::paginate(&chart.blocks, &self.config.layout, |chart_line| {
            line::height_of(chart_line, &self.config)
        })?;
        Ok(pages.iter().map(|page| self.render_page(page)).collect())
    }

    fn render_page(&self, page: &Page) -> SvgPage {
        let geometry = &self.config.layout;

        let document = Document::new()
            .set(
                "viewBox",
                format!("0 0 {} {}", geometry.width, geometry.height),
            )
            .set("width", format!("{}pt", geometry.width))
            .set("height", format!("{}pt", geometry.height));

        let document = page
            .lines
            .iter()
            .flat_map(|placed| line::render(placed.line, &self.config, placed.y))
            .fold(document, |document, element| document.add(element));

        SvgPage(document.to_string())
    }
}
