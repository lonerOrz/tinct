//! tinct - A theme injector tool that applies Material Design 3 color palettes
//!
//! This library provides a modular architecture for:
//! - Loading themes from various sources
//! - Generating Material Design 3 color palettes
//! - Processing templates with theme data
//! - Outputting to various formats

// Core abstractions
pub mod core;

// New modular architecture
pub mod output;
pub mod palette;
pub mod template;
pub mod theme;

// Image-based color extraction (wallpaper support)
pub mod image;

// Core utilities (kept for backward compatibility and shared functionality)
pub mod color;
pub mod config;
pub mod filter;
pub mod log;
pub mod path_resolver;
pub mod preview;

// Re-exports
pub use color::*;
pub use config::*;
pub use filter::{Filter as ColorFilter, FilterContext, FilterRegistry};
pub use log::*;
pub use preview::*;

// Core trait re-exports
pub use core::{
    ColorSpace, Error, Mode, OutputFormat, PaletteGenerator, Result, TemplateEngine, Theme,
    ThemeLoader,
};

// New module re-exports
pub use output::{FileOutput, OutputRegistry, TerminalOutput};
pub use palette::{
    AlgorithmParameters, ColorEntry, ColorFormat, ColorHarmony, LegacyPaletteGenerator, Palette,
};
pub use template::{Filter as TemplateFilter, TemplateEngineRegistry, TemplateProcessor};
pub use theme::{JsonThemeLoader, ThemeLoaderRegistry};
