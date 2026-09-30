//! Vertical page layout: which page each line lands on, and at what `y`.
//!
//! [`paginate`] decides the physical pages. An explicit [`Block::PageBreak`]
//! always starts a new page, and so does a line that would cross the bottom
//! margin. It knows only vertical space: each line's height comes from the
//! caller, so how lines are drawn stays in [`super::line`].

use crate::model::{Block, Line};
use crate::render::{LayoutConfig, RenderError, Result};

/// A line positioned on a page at the baseline `y` it is drawn at.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct PlacedLine<'c> {
    pub(super) line: &'c Line,
    pub(super) y: f64,
}

/// The lines of one physical page, top to bottom.
#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct Page<'c> {
    pub(super) lines: Vec<PlacedLine<'c>>,
}

/// Lay `blocks` out onto pages. There is always at least one page.
///
/// Explicit breaks are literal: a `#page_break` at the start or end, or right
/// after another one, produces an empty page, and spacers after it are kept.
/// An automatic break drops the spacers (empty lines) that would otherwise
/// start the next page, since that space only exists because of the break.
///
/// Fails if a line is taller than the space between a page's margins, since
/// no page could hold it.
pub(super) fn paginate<'c>(
    blocks: &'c [Block],
    page: &LayoutConfig,
    height_of: impl Fn(&Line) -> f64,
) -> Result<Vec<Page<'c>>> {
    let mut paginator = Paginator::new(page);
    for block in blocks {
        match block {
            Block::Text(line) => paginator.place(line, height_of(line))?,
            Block::PageBreak => paginator.break_page(Break::Explicit),
        }
    }
    Ok(paginator.finish())
}

/// Why a new page was started.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Break {
    /// The author wrote `#page_break`.
    Explicit,
    /// The next line did not fit.
    Automatic,
}

/// The running state of [`paginate`]: finished pages plus the one being filled.
struct Paginator<'c, 'p> {
    page: &'p LayoutConfig,
    finished: Vec<Page<'c>>,
    current: Page<'c>,
    y: f64,
    /// Set by an automatic break until the first line with text is placed.
    dropping_spacers: bool,
}

impl<'c, 'p> Paginator<'c, 'p> {
    fn new(page: &'p LayoutConfig) -> Self {
        Self {
            page,
            finished: Vec::new(),
            current: Page::default(),
            y: page.margin_vertical,
            dropping_spacers: false,
        }
    }

    /// Place a line below the previous one, first moving to a new page if its
    /// baseline would cross the bottom margin.
    fn place(&mut self, line: &'c Line, height: f64) -> Result<()> {
        let available = self.page.height - 2.0 * self.page.margin_vertical;
        if height > available {
            return Err(RenderError::line_taller_than_page(line, height, available));
        }

        if self.y + height > self.page.height - self.page.margin_vertical {
            self.break_page(Break::Automatic);
        }
        if self.dropping_spacers && line.is_empty() {
            return Ok(());
        }

        self.dropping_spacers = false;
        self.y += height;
        self.current.lines.push(PlacedLine { line, y: self.y });
        Ok(())
    }

    fn break_page(&mut self, reason: Break) {
        self.finished.push(std::mem::take(&mut self.current));
        self.y = self.page.margin_vertical;
        self.dropping_spacers = reason == Break::Automatic;
    }

    fn finish(mut self) -> Vec<Page<'c>> {
        self.finished.push(self.current);
        self.finished
    }
}
