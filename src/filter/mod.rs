//! Filter module for color transformations
//!
//! This module provides a modular filter system for transforming color values.
//! Filters can be applied to template color placeholders to modify colors dynamically.
//!
//! # Available Filters
//!
//! - `set_alpha` - Modify alpha transparency
//! - `lighten` - Increase color lightness
//! - `darken` - Decrease color lightness
//! - `saturate` - Increase color saturation
//! - `desaturate` - Decrease color saturation
//!
//! # Example
//!
//! ```rust
//! use tinct::filter::{FilterRegistry, ColorFormatType};
//! use tinct::palette::ColorFormat;
//!
//! let registry = FilterRegistry::new();
//! // let color = ColorFormat { /* ... */ };
//! // let result = registry.apply_filter(
//! //     "rgb(255, 0, 0)",
//! //     "set_alpha",
//! //     Some("0.5"),
//! //     &color,
//! //     ColorFormatType::Rgba,
//! // );
//! ```

mod lightness;
mod registry;
mod saturation;
mod set_alpha;
mod types;

pub use lightness::{DarkenFilter, LightenFilter};
pub use registry::FilterRegistry;
pub use saturation::{DesaturateFilter, SaturateFilter};
pub use set_alpha::SetAlphaFilter;
pub use types::{ColorFormatType, Filter, FilterContext};
