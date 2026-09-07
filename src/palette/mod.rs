//! Palette generation module
//!
//! This module provides Material Design 3 color palette generation
//! using the HCT (Hue-Chroma-Tone) color space via the material-colors crate.

mod adapter;
mod generator;
mod params;
mod types;

pub use adapter::LegacyPaletteGenerator;
pub use generator::{extract_seed_hex, generate_palette, generate_palette_with_params};
pub use params::{AlgorithmParameters, ColorHarmony};
pub use types::{ColorRole, Palette};
