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
mod types;
mod params;
mod color_parser;
mod generator;
pub mod constants;

pub use adapter::LegacyPaletteGenerator;
pub use types::{ColorFormat, ColorEntry, Palette};
pub use params::AlgorithmParameters;
pub use generator::{generate_palette, generate_palette_with_params};
