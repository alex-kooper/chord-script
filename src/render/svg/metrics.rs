//! How tall each row's box is, and how much space is kept between rows.
//!
//! A line's box runs from the top of its capitals to its baseline, and a bar
//! row's from the top of its bar lines past the room it keeps for a hanging
//! bass. Two rows are kept apart by the larger of the upper one's `below` and
//! the lower one's `above`, except a text line right after a bar row: it
//! comes as close to the bar lines as its `above`, leaving the room for the
//! bass to the author.

use super::layout::{Metrics, Row};
use crate::model::{LineLevel, TextStyle};
use crate::render::fonts::{self, Font};
use crate::render::{RenderConfig, bar_row};

impl Metrics for RenderConfig {
    fn height(&self, row: Row) -> f64 {
        match row {
            Row::Text(line) => {
                let size = self.font_style_for_level(line.level).size;
                fonts::cap_height(Font::Text(TextStyle::Normal), size)
            }
            Row::Bars(_) => bar_row::height(&self.bar_row),
        }
    }

    fn space_between(&self, upper: Row, lower: Row) -> f64 {
        match (upper, lower) {
            (Row::Bars(_), Row::Text(line)) if line.level == LineLevel::Text => {
                self.text.above - bar_row::room_below_bar_lines(&self.bar_row)
            }
            _ => below(self, upper).max(above(self, lower)),
        }
    }
}

fn above(config: &RenderConfig, row: Row) -> f64 {
    match row {
        Row::Text(line) => config.font_style_for_level(line.level).above,
        Row::Bars(_) => config.bar_row.above,
    }
}

fn below(config: &RenderConfig, row: Row) -> f64 {
    match row {
        Row::Text(line) => config.font_style_for_level(line.level).below,
        Row::Bars(_) => config.bar_row.below,
    }
}
