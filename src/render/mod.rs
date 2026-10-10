//! Rendering a [`Chart`](crate::model::Chart) to output formats.
//!
//! Each format has its own generator with the same shape: `new(RenderConfig)`
//! (or `with_defaults()`) and `render(&Chart) -> Result<_>`. They share the
//! configuration, the errors, and the bundled fonts.

mod chord;
mod config;
mod error;
mod fonts;
mod inline;
mod pdf;
mod svg;

pub use config::{FontStyle, LayoutConfig, RenderConfig};
pub use error::{RenderError, Result};
pub use pdf::{PdfDocument, PdfGenerator};
pub use svg::{SvgGenerator, SvgPage};
