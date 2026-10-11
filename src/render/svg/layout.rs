//! Vertical page layout: which page each row lands on, and at what `y`.
//!
//! [`paginate`] decides the physical pages. An explicit [`Block::PageBreak`]
//! always starts a new page, and so does a row that would cross the bottom
//! margin. It knows only vertical space: each row's height comes from the
//! caller, so how rows are drawn stays in [`super::line`] and
//! [`super::bar_row`].

use crate::model::{BarRow, Block, Line};
use crate::render::{LayoutConfig, RenderError, Result};

/// One row of a page: a line of text or a row of bars.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Row<'c> {
    Text(&'c Line),
    Bars(&'c BarRow),
}

impl Row<'_> {
    /// Whether the row is an empty line, which only adds space.
    fn is_spacer(self) -> bool {
        matches!(self, Row::Text(line) if line.is_empty())
    }
}

/// A row positioned on a page, with the `y` of its bottom edge: the baseline
/// of a text line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct PlacedRow<'c> {
    pub(super) row: Row<'c>,
    pub(super) y: f64,
}

/// The rows of one physical page, top to bottom.
#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct Page<'c> {
    pub(super) rows: Vec<PlacedRow<'c>>,
}

/// Lay `blocks` out onto pages. There is always at least one page.
///
/// Explicit breaks are literal: a `#page_break` at the start or end, or right
/// after another one, produces an empty page, and spacers after it are kept.
/// An automatic break drops the spacers (empty lines) that would otherwise
/// start the next page, since that space only exists because of the break.
///
/// Fails if a row is taller than the space between a page's margins, since
/// no page could hold it.
pub(super) fn paginate<'c>(
    blocks: &'c [Block],
    page: &LayoutConfig,
    height_of: impl Fn(Row) -> f64,
) -> Result<Vec<Page<'c>>> {
    let mut paginator = Paginator::new(page);
    for block in blocks {
        let row = match block {
            Block::Text(line) => Row::Text(line),
            Block::Bars(bars) => Row::Bars(bars),
            Block::PageBreak => {
                paginator.break_page(Break::Explicit);
                continue;
            }
        };
        paginator.place(row, height_of(row))?;
    }
    Ok(paginator.finish())
}

/// Why a new page was started.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Break {
    /// The author wrote `#page_break`.
    Explicit,
    /// The next row did not fit.
    Automatic,
}

/// The running state of [`paginate`]: finished pages plus the one being filled.
struct Paginator<'c, 'p> {
    page: &'p LayoutConfig,
    finished: Vec<Page<'c>>,
    current: Page<'c>,
    y: f64,
    /// Set by an automatic break until the first row that is not a spacer is
    /// placed.
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

    /// Place a row below the previous one, first moving to a new page if its
    /// bottom would cross the bottom margin.
    fn place(&mut self, row: Row<'c>, height: f64) -> Result<()> {
        let available = self.page.height - 2.0 * self.page.margin_vertical;
        if height > available {
            return Err(match row {
                Row::Text(line) => RenderError::line_taller_than_page(line, height, available),
                Row::Bars(bars) => RenderError::bar_row_taller_than_page(bars, height, available),
            });
        }

        if self.y + height > self.page.height - self.page.margin_vertical {
            self.break_page(Break::Automatic);
        }
        if self.dropping_spacers && row.is_spacer() {
            return Ok(());
        }

        self.dropping_spacers = false;
        self.y += height;
        self.current.rows.push(PlacedRow { row, y: self.y });
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
