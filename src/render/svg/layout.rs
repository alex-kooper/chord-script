//! Vertical page layout: the cursor that decides where lines sit.
//!
//! [`Layout`] knows only about vertical space — it advances a `y` cursor and
//! hands back baselines. It never learns what a line, column, or span is; that
//! is [`super::line`]'s concern. Pagination (page breaks, overflow) will grow
//! here without touching line rendering.

use super::config::SvgConfig;

/// A downward-moving vertical cursor over a page.
pub(super) struct Layout {
    y: f64,
}

impl Layout {
    /// Start a fresh layout at the top margin.
    pub(super) fn new(config: &SvgConfig) -> Self {
        Self {
            y: config.layout.margin_vertical,
        }
    }

    /// Advance the cursor by `height` and return the baseline for the content
    /// that occupies that space.
    pub(super) fn place(&mut self, height: f64) -> f64 {
        self.y += height;
        self.y
    }
}
