//! Palette generation module
//!
//! This module provides Material Design 3 color palette generation
//! using HCT (Hue-Chroma-Tone) color space.

mod adapter;

pub use adapter::LegacyPaletteGenerator;

// Re-export from legacy palette_generator for backward compatibility
pub use crate::palette_generator::{
    generate_palette, generate_palette_with_params, AlgorithmParameters, Palette, ColorEntry, ColorFormat,
};
