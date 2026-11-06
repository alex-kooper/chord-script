//! Rendering configuration shared by every output format: page geometry and
//! per-level font styling.
//!
//! These types describe *how* a chart is drawn (sizes, weights, margins) and
//! carry no rendering logic beyond looking up the style for a line level.

use crate::model::LineLevel;

/// How the lines of one level are drawn, in points.
///
/// A line's box runs from the top of its capitals down to its baseline;
/// descenders, accents and parentheses reach out of it. Two rows one after
/// the other are kept apart by the larger of the upper one's `below` and the
/// lower one's `above`, measured between their boxes.
#[derive(Debug, Clone)]
pub struct FontStyle {
    pub size: f64,
    pub weight: String,
    /// The least space kept above the line's capitals
    pub above: f64,
    /// The least space kept below the line's baseline
    pub below: f64,
}

/// Layout configuration (page dimensions and margins), in points
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    pub width: f64,
    pub height: f64,
    pub margin_horizontal: f64,
    pub margin_vertical: f64,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            // A4 portrait: 595pt × 842pt (1pt = 1/72 inch)
            width: 595.0,
            height: 842.0,
            margin_horizontal: 28.0, // ~10mm
            margin_vertical: 28.0,   // ~10mm
        }
    }
}

/// How bar rows are drawn, in points.
///
/// A bar row's box runs from the top of its bar lines down past them, by the
/// room it keeps for a hanging bass. It is kept apart from its neighbours
/// like a line is (see [`FontStyle`]), except a text line right after it:
/// that line ignores `below` and keeps only the text's `above` from the bar
/// lines, reaching up into the room for the bass.
#[derive(Debug, Clone)]
pub struct BarRowStyle {
    /// The font size of the chords; everything else in a bar row, from the
    /// row's height to the bar lines' thickness, scales with it.
    pub chord_size: f64,
    /// The least space kept above the bar lines
    pub above: f64,
    /// The least space kept below the room for a hanging bass
    pub below: f64,
}

impl Default for BarRowStyle {
    fn default() -> Self {
        Self {
            chord_size: 38.0,
            above: 10.0,
            below: 10.0,
        }
    }
}

/// How a chart is drawn, whatever the output format.
#[derive(Debug, Clone)]
pub struct RenderConfig {
    // Layout
    pub layout: LayoutConfig,

    /// Font family list for all text, in CSS syntax and tried in order, e.g.
    /// `Noto Sans, sans-serif`.
    ///
    /// SVG viewers may use any font installed where the SVG is opened. PDF
    /// output can only use the bundled Noto Sans (also reached through
    /// `sans-serif`), and fails if no family in the list is bundled.
    pub font_family: String,

    // Font styles per level
    pub header1: FontStyle,
    pub header2: FontStyle,
    pub header3: FontStyle,
    pub text: FontStyle,

    pub bar_row: BarRowStyle,
}

impl Default for RenderConfig {
    fn default() -> Self {
        // Every level uses the regular weight: the bundled font has only
        // regular and bold faces, and a regular header keeps inline `*bold*`
        // visible inside it.
        let regular = |size: f64, above: f64, below: f64| FontStyle {
            size,
            weight: "normal".to_string(),
            above,
            below,
        };

        // A header keeps more space above it than below, so it reads with
        // what follows it. Each level's `below` leaves about the same space
        // under its descenders.
        Self {
            layout: LayoutConfig::default(),
            // The generic fallback keeps SVG viewers without Noto Sans on a
            // sans-serif font; they would otherwise default to a serif one.
            font_family: format!("{}, sans-serif", super::fonts::FAMILY),
            header1: regular(29.0, 17.0, 14.0),
            header2: regular(22.0, 16.0, 12.0),
            header3: regular(18.0, 13.0, 11.0),
            text: regular(16.0, 10.5, 10.0),
            bar_row: BarRowStyle::default(),
        }
    }
}

impl RenderConfig {
    /// The font style to use for a given line level.
    pub fn font_style_for_level(&self, level: LineLevel) -> &FontStyle {
        match level {
            LineLevel::Header1 => &self.header1,
            LineLevel::Header2 => &self.header2,
            LineLevel::Header3 => &self.header3,
            LineLevel::Text => &self.text,
        }
    }
}
