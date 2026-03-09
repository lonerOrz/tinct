//! Palette generation module
//!
//! This module provides Material Design 3 color palette generation
//! using the HCT (Hue-Chroma-Tone) color space via the material-colors crate.
//!
//! # Module Structure
//!
//! - `types` - Core color types (`ColorFormat`, `ColorEntry`, `Palette`)
//! - `params` - Algorithm parameters configuration
//! - `color_parser` - Color parsing and format conversion
//! - `generator` - Core palette generation logic using material-colors
//! - `constants` - Material Design 3 color constants

mod adapter;
mod color_parser;
pub mod constants;
mod generator;
mod params;
mod types;

pub use adapter::LegacyPaletteGenerator;
pub use generator::{generate_palette, generate_palette_with_params};
pub use params::AlgorithmParameters;
pub use types::{ColorEntry, ColorFormat, Palette};
