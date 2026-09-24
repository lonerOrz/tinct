//! tinct - A theme injector tool that applies Material Design 3 color palettes.
//!
//! Layered architecture:
//! - **domain** — [`core`] (color model, errors, `Mode`/`Theme`), [`palette`]
//!   (MD3 generation + ANSI), [`image`] (extraction), [`template`] (rendering)
//! - **infrastructure** — [`config`] (TOML + paths), [`output`] (file writing)
//! - **presentation** — [`ui`] (logging + preview)
//! - **application** — [`pipeline`] orchestrates everything

#![forbid(unsafe_code)]

pub mod config;
pub mod core;
pub mod image;
pub mod output;
pub mod palette;
pub mod pipeline;
pub mod template;
pub mod ui;

// Backwards-compatible module paths for modules that moved during the
// domain-driven reorganisation. Library consumers can keep using
// `tinct::color`, `tinct::log` and `tinct::preview`.
pub use core::color;
pub use ui::{log, preview};

pub use config::{
    AlgorithmConfig, AnsiAnchors, AnsiConfig, AnsiPalette, Config, ConfigLoad, ConfigSection,
    ImageConfig, resolve_theme_path,
};
pub use core::color::{Color, Hsl, Rgb};
pub use ui::log::{LogLevel, Logger, error, general, hook, info, init_logger};
pub use ui::preview::{
    show_color_preview, show_color_preview_from_json, show_color_preview_from_theme,
};

pub use core::{Error, Mode, Result, Theme};

pub use output::FileOutput;
pub use palette::{AlgorithmParameters, ColorRole, LegacyPaletteGenerator, Palette};
pub use pipeline::{Pipeline, PipelineConfig};
pub use template::{ColorFilter, ColorProperty, FilterContext, TemplateProcessor};

pub use image::{ExtractedPalette, SchemeType, extract_source_color, extract_source_palette};
