//! Rendering configuration shared by every output format: page geometry and
//! per-level font styling.
//!
//! These types describe *how* a chart is drawn (sizes, weights, margins) and
//! carry no rendering logic beyond looking up the style for a line level.

use crate::model::LineLevel;

/// Font style configuration (size, weight, line-height)
#[derive(Debug, Clone)]
pub struct FontStyle {
    pub size: f64,
    pub weight: String,
    pub line_height: f64,
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
}

impl Default for RenderConfig {
    fn default() -> Self {
        // Every level uses the regular weight: the bundled font has only
        // regular and bold faces, and a regular header keeps inline `*bold*`
        // visible inside it.
        let regular = |size: f64, line_height: f64| FontStyle {
            size,
            weight: "normal".to_string(),
            line_height,
        };

        Self {
            layout: LayoutConfig::default(),
            // The generic fallback keeps SVG viewers without Noto Sans on a
            // sans-serif font; they would otherwise default to a serif one.
            font_family: format!("{}, sans-serif", super::fonts::FAMILY),
            header1: regular(18.0, 24.0),
            header2: regular(14.0, 20.0),
            header3: regular(11.0, 16.0),
            text: regular(10.0, 14.0),
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

    /// The line height to use for a given line level.
    pub fn line_height_for_level(&self, level: LineLevel) -> f64 {
        self.font_style_for_level(level).line_height
    }
}
