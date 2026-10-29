//! SVG rendering of a [`Chart`].
//!
//! Pagination and vertical placement live in [`layout`]; the drawing of
//! individual rows in [`line`] and [`bar_row`]. This module owns the generator
//! that wires them together: it paginates the chart, then builds one SVG
//! document per page.

use super::{BarRowStyle, RenderConfig, Result};
use crate::model::Chart;
use derive_more::{AsRef, Display, Into};
use svg::{Document, Node};

mod bar_row;
mod layout;
mod line;

#[cfg(test)]
mod tests;

use layout::{Page, PlacedRow, Row};

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
    /// page. Fails if a row is too tall to fit on any page, or if the bar-row
    /// style's chord size isn't a positive number or its gap a non-negative
    /// one.
    pub fn render(&self, chart: &Chart) -> Result<Vec<SvgPage>> {
        let BarRowStyle { chord_size, gap } = self.config.bar_row;
        let valid = chord_size.is_finite() && chord_size > 0.0 && gap.is_finite() && gap >= 0.0;
        if !valid {
            return Err(super::RenderError::invalid_bar_row_style(
                &self.config.bar_row,
            ));
        }
        let pages = layout::paginate(&chart.blocks, &self.config.layout, |row| match row {
            Row::Text(line) => line::height_of(line, &self.config),
            Row::Bars(_) => bar_row::height_of(&self.config),
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
            .rows
            .iter()
            .flat_map(|placed| self.render_row(placed))
            .fold(document, |document, element| document.add(element));

        SvgPage(document.to_string())
    }

    fn render_row(&self, placed: &PlacedRow) -> Vec<Box<dyn Node>> {
        match placed.row {
            Row::Text(line) => line::render(line, &self.config, placed.y)
                .into_iter()
                .map(Box::<dyn Node>::from)
                .collect(),
            Row::Bars(bars) => bar_row::render(bars, &self.config, placed.y),
        }
    }
}
