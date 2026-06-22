//! Image-based color extraction module.
//!
//! This module provides wallpaper image reading, color quantization,
//! and source color extraction — matching the Python `theming/` project's
//! extraction pipeline.
//!
//! # Supported Scheme Types
//!
//! ## M3 Schemes (Wu + WSMeans + Score)
//! - `tonal-spot` — Default Android 12-13 Material You scheme
//! - `content` — Preserves source color's chroma
//! - `fruit-salad` — Bold/playful with hue rotation
//! - `rainbow` — Chromatic accents with grayscale neutrals
//! - `monochrome` — Pure grayscale M3 scheme
//!
//! ## Non-M3 Schemes (K-means + custom scoring)
//! - `vibrant` — Prioritizes the most saturated colors
//! - `faithful` — Prioritizes dominant colors by area coverage
//! - `dysfunctional` — Picks 2nd most dominant color family
//! - `muted` — Like count but without chroma filtering (monochrome wallpapers)

mod extractor;
mod kmeans;
mod quantizer;
mod reader;
mod wsmeans;

pub use crate::color::Rgb;
pub use extractor::{extract_source_color, SchemeType};
pub use reader::{read_image, ResizeFilter};
