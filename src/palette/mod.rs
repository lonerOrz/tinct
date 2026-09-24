//! Palette generation module.
//!
//! This module provides Material Design 3 color palette generation using the
//! HCT (Hue-Chroma-Tone) color space via the `material-colors` crate, plus the
//! terminal (ANSI) mapping derived from the generated scheme.

mod ansi;
mod dynamic;
mod params;
mod types;

#[cfg(test)]
pub(crate) mod test_support;

pub use ansi::AnsiParams;
pub use dynamic::{
    LegacyPaletteGenerator, build_palette, collect_theme_colors, extract_seed_hex,
    generate_palette, generate_palette_with_params,
};
pub use params::AlgorithmParameters;
pub use types::{ColorRole, Palette};
