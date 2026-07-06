//! Palette generation module
//!
//! This module provides Material Design 3 color palette generation
//! using the HCT (Hue-Chroma-Tone) color space via the material-colors crate.
//!
//! # Module Structure
//!
//! - `types` - Core color types (`ColorFormat`, `Palette`)
//! - `params` - Algorithm parameters configuration
//! - `generator` - Core palette generation logic using material-colors

mod adapter;
mod generator;
mod params;
mod types;

pub use adapter::LegacyPaletteGenerator;
pub use generator::{extract_seed_hex, generate_palette, generate_palette_with_params};
pub use params::{AlgorithmParameters, ColorHarmony};
pub use types::{ColorFormat, Palette};
