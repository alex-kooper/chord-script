//! PDF rendering of a [`Chart`]: one document, one PDF page per chart page.
//!
//! Text is embedded as selectable text in the bundled font (see
//! [`super::fonts`]), so the file looks the same on every device, whatever
//! fonts it has installed.
//!
//! Pages are currently drawn as SVG and converted with krilla. That is an
//! implementation detail: the interface speaks only of charts, configuration,
//! and PDF bytes.

use super::svg::{SvgGenerator, SvgPage};
use super::{LayoutConfig, RenderConfig, RenderError, Result, fonts};
use crate::model::{Block, Chart};
use derive_more::{AsRef, Into};
use krilla::Document;
use krilla::geom::Size;
use krilla::page::PageSettings;
use krilla_svg::{SurfaceExt, SvgSettings};

#[cfg(test)]
mod tests;

/// A rendered chart: the bytes of a complete PDF file.
#[derive(Debug, Clone, PartialEq, Eq, AsRef, Into)]
#[as_ref([u8])]
pub struct PdfDocument(Vec<u8>);

/// PDF generator that renders charts to a single multi-page PDF
pub struct PdfGenerator {
    pages: SvgGenerator,
    config: RenderConfig,
}

impl PdfGenerator {
    /// Create a new PDF generator with the given configuration
    pub fn new(config: RenderConfig) -> Self {
        Self {
            pages: SvgGenerator::new(config.clone()),
            config,
        }
    }

    /// Create a new PDF generator with default configuration
    pub fn with_defaults() -> Self {
        Self::new(RenderConfig::default())
    }

    /// Render a chart to one PDF document.
    ///
    /// There is always at least one page; an empty chart renders as one blank
    /// page. Fails rather than produce a PDF that differs from the chart: if
    /// the page size is not positive, if the font family is not bundled, if
    /// the text uses a character the bundled font cannot draw, if a line is
    /// too tall to fit on any page, or if the PDF cannot be written.
    pub fn render(&self, chart: &Chart) -> Result<PdfDocument> {
        let page_size = page_size(&self.config.layout)?;
        check_font_family(&self.config.font_family)?;
        check_characters(chart)?;

        let options = usvg::Options {
            font_family: fonts::FAMILY.to_string(),
            fontdb: fonts::database(),
            ..usvg::Options::default()
        };

        let mut document = Document::new();
        for page in self.pages.render(chart)? {
            draw_page(&mut document, &page, page_size, &options);
        }
        document
            .finish()
            .map(PdfDocument)
            .map_err(|error| RenderError::PdfExport {
                message: error.to_string(),
            })
    }
}

fn page_size(layout: &LayoutConfig) -> Result<Size> {
    Size::from_wh(layout.width as f32, layout.height as f32).ok_or(RenderError::InvalidPageSize {
        width: layout.width,
        height: layout.height,
    })
}

fn check_font_family(family: &str) -> Result<()> {
    if fonts::has_family(family) {
        return Ok(());
    }
    Err(RenderError::FontUnavailable {
        family: family.to_string(),
        bundled: fonts::FAMILY,
    })
}

/// Fail on the first character, in reading order, that no bundled font has.
///
/// Tabs are exempt: they are drawn as spaces.
fn check_characters(chart: &Chart) -> Result<()> {
    let lines = chart.blocks.iter().filter_map(|block| match block {
        Block::Text(line) => Some(line),
        Block::PageBreak => None,
    });
    for line in lines {
        let unsupported = line
            .spans()
            .flat_map(|span| -> std::str::Chars<'_> { span.text.as_ref().chars() })
            .find(|&c| c != '\t' && !fonts::can_draw(c));
        if let Some(character) = unsupported {
            return Err(RenderError::unsupported_character(line, character));
        }
    }
    Ok(())
}

/// Add a PDF page of `size` points and draw `page` scaled to fill it.
fn draw_page(document: &mut Document, page: &SvgPage, size: Size, options: &usvg::Options) {
    // The SVG is our own output, so a parse failure is a bug, not bad input.
    let tree =
        usvg::Tree::from_str(page.as_ref(), options).expect("generated SVG should always parse");

    let mut pdf_page = document.start_page_with(PageSettings::new(size));
    let mut surface = pdf_page.surface();
    surface.draw_svg(&tree, size, SvgSettings::default());
    surface.finish();
    pdf_page.finish();
}
