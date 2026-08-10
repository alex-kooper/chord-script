//! Rendering configuration: page geometry and per-level font styling.
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

/// Layout configuration (page dimensions and margins)
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

/// Configuration for SVG rendering
#[derive(Debug, Clone)]
pub struct SvgConfig {
    // Layout
    pub layout: LayoutConfig,

    // Font (single font family for all text)
    pub font_family: String,

    // Font styles per level
    pub header1: FontStyle,
    pub header2: FontStyle,
    pub header3: FontStyle,
    pub text: FontStyle,
}

impl Default for SvgConfig {
    fn default() -> Self {
        Self {
            layout: LayoutConfig::default(),

            // Font (single font family for all text)
            font_family: "sans-serif".to_string(),

            // Font styles per level
            header1: FontStyle {
                size: 18.0,
                weight: "500".to_string(),
                line_height: 24.0,
            },
            header2: FontStyle {
                size: 14.0,
                weight: "450".to_string(),
                line_height: 20.0,
            },
            header3: FontStyle {
                size: 11.0,
                weight: "420".to_string(),
                line_height: 16.0,
            },
            text: FontStyle {
                size: 10.0,
                weight: "normal".to_string(),
                line_height: 14.0,
            },
        }
    }
}

impl SvgConfig {
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
