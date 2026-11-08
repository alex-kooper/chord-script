//! SVG rendering of a [`Chart`].
//!
//! Pagination and vertical placement live in [`layout`], the size of rows and
//! the space between them in [`metrics`], and the drawing of individual rows
//! in [`line`] and [`bar_row`]. This module owns the generator
//! that wires them together: it paginates the chart, then builds one SVG
//! document per page.

use super::{BarRowStyle, RenderConfig, RenderError, Result};
use crate::model::{Chart, LineLevel};
use derive_more::{AsRef, Display, Into};
use svg::{Document, Node};

mod bar_row;
mod layout;
mod line;
mod metrics;

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
    /// page. Fails if a row is too tall to fit on any page, if a bar row is
    /// too cramped to draw between the margins, or if a font or chord size
    /// isn't a positive number or a space kept above or below rows isn't a
    /// non-negative one.
    pub fn render(&self, chart: &Chart) -> Result<Vec<SvgPage>> {
        check_styles(&self.config)?;
        let pages = layout::paginate(&chart.blocks, &self.config.layout, &self.config)?;
        pages.iter().map(|page| self.render_page(page)).collect()
    }

    fn render_page(&self, page: &Page) -> Result<SvgPage> {
        let geometry = &self.config.layout;

        let document = Document::new()
            .set(
                "viewBox",
                format!("0 0 {} {}", geometry.width, geometry.height),
            )
            .set("width", format!("{}pt", geometry.width))
            .set("height", format!("{}pt", geometry.height));

        let mut document = document;
        for placed in &page.rows {
            for element in self.render_row(placed)? {
                document = document.add(element);
            }
        }
        Ok(SvgPage(document.to_string()))
    }

    fn render_row(&self, placed: &PlacedRow) -> Result<Vec<Box<dyn Node>>> {
        match placed.row {
            Row::Text(line) => Ok(line::render(line, &self.config, placed.y)
                .into_iter()
                .map(Box::<dyn Node>::from)
                .collect()),
            Row::Bars(bars) => bar_row::render(bars, &self.config, placed.y),
        }
    }
}

/// Fails on the first style whose size isn't a positive number or whose
/// space above or below isn't a non-negative one.
fn check_styles(config: &RenderConfig) -> Result<()> {
    let size = |size: f64| size.is_finite() && size > 0.0;
    let space = |space: f64| space.is_finite() && space >= 0.0;

    let levels = [
        LineLevel::Header1,
        LineLevel::Header2,
        LineLevel::Header3,
        LineLevel::Text,
    ];
    for level in levels {
        let style = config.font_style_for_level(level);
        if !(size(style.size) && space(style.above) && space(style.below)) {
            return Err(RenderError::invalid_font_style(level, style));
        }
    }
    let BarRowStyle {
        chord_size,
        above,
        below,
    } = config.bar_row;
    if !(size(chord_size) && space(above) && space(below)) {
        return Err(RenderError::invalid_bar_row_style(&config.bar_row));
    }
    Ok(())
}
