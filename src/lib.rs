//! tinct - A theme injector tool that applies Material Design 3 color palettes
//!
//! This library provides a modular architecture for:
//! - Loading themes from various sources
//! - Generating Material Design 3 color palettes
//! - Processing templates with theme data
//! - Outputting to various formats

pub mod color;
pub mod config;
pub mod core;
pub mod image;
pub mod log;
pub mod output;
pub mod palette;
pub mod path_resolver;
pub mod pipeline;
pub mod preview;
pub mod template;
pub mod theme;

pub use color::*;
pub use config::*;
pub use log::*;
pub use preview::*;

pub use core::{Error, Mode, Result, Theme};

pub use output::FileOutput;
pub use palette::{
    AlgorithmParameters, ColorFormat, ColorHarmony, LegacyPaletteGenerator, Palette,
};
pub use pipeline::{Pipeline, PipelineConfig};
pub use template::{ColorFilter, ColorFormatType, FilterContext, TemplateProcessor};
pub use theme::JsonThemeLoader;

pub use image::{SchemeType, extract_source_color};
pub use path_resolver::{resolve_config_file_path, resolve_config_paths, resolve_theme_path};
