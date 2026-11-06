//! Vertical page layout: which page each row lands on, and at what `y`.
//!
//! [`paginate`] decides the physical pages. An explicit [`Block::PageBreak`]
//! always starts a new page, and so does a row that would cross the bottom
//! margin. It knows only vertical space: each row is a box whose height, and
//! the space between it and the row above, come from [`Metrics`], so how rows
//! are drawn stays in [`super::metrics`], [`super::line`] and
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

/// The vertical extent of rows, for [`paginate`].
pub(super) trait Metrics {
    /// The height of `row`'s box.
    fn height(&self, row: Row) -> f64;

    /// The space between the boxes of `upper` and of `lower` right below it.
    fn space_between(&self, upper: Row, lower: Row) -> f64;
}

/// A row positioned on a page, with the `y` of its box's bottom edge: the
/// baseline of a text line.
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
/// A row starting a page has its box at the top margin, with no space above
/// it; any other row sits the space between them below the row above.
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
    metrics: &impl Metrics,
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
        paginator.place(row, metrics)?;
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
            dropping_spacers: false,
        }
    }

    /// Place a row below the previous one, first moving to a new page if its
    /// bottom would cross the bottom margin.
    fn place(&mut self, row: Row<'c>, metrics: &impl Metrics) -> Result<()> {
        let height = metrics.height(row);
        let available = self.page.height - 2.0 * self.page.margin_vertical;
        if height > available {
            return Err(match row {
                Row::Text(line) => RenderError::line_taller_than_page(line, height, available),
                Row::Bars(bars) => RenderError::bar_row_taller_than_page(bars, height, available),
            });
        }

        let top = match self.current.rows.last() {
            Some(above) => above.y + metrics.space_between(above.row, row),
            None => self.page.margin_vertical,
        };
        let mut y = top + height;
        if y > self.page.height - self.page.margin_vertical {
            self.break_page(Break::Automatic);
            y = self.page.margin_vertical + height;
        }
        if self.dropping_spacers && row.is_spacer() {
            return Ok(());
        }

        self.dropping_spacers = false;
        self.current.rows.push(PlacedRow { row, y });
        Ok(())
    }

    fn break_page(&mut self, reason: Break) {
        self.finished.push(std::mem::take(&mut self.current));
        self.dropping_spacers = reason == Break::Automatic;
    }

    fn finish(mut self) -> Vec<Page<'c>> {
        self.finished.push(self.current);
        self.finished
    }
}
