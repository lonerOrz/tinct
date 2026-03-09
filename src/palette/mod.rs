//! Palette generation module
//!
//! This module provides Material Design 3 color palette generation
//! using HCT (Hue-Chroma-Tone) color space.
//!
//! # Module Structure
//!
//! - `types` - Core color types (`ColorFormat`, `ColorEntry`, `Palette`)
//! - `params` - Algorithm parameters configuration
//! - `color_parser` - Color parsing and format conversion
//! - `algorithm` - Algorithm applications for HCT adjustment
//! - `generator` - Core palette generation logic
//! - `adapter` - Legacy adapter for backward compatibility

mod adapter;
mod types;
mod params;
mod color_parser;
mod algorithm;
mod generator;

pub use adapter::LegacyPaletteGenerator;
pub use types::{ColorFormat, ColorEntry, Palette};
pub use params::AlgorithmParameters;
pub use generator::{generate_palette, generate_palette_with_params};
