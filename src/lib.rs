//! tinct - A theme injector tool that applies Material Design 3 color palettes.
//!
//! Layered architecture:
//! - **domain** — [`core`] (color model, errors, `Mode`/`Theme`), [`palette`]
//!   (MD3 generation + ANSI), [`image`] (extraction), [`template`] (rendering)
//! - **infrastructure** — [`config`] (TOML + paths), [`output`] (file writing)
//! - **presentation** — [`ui`] (logging + preview)
//! - **application** — [`pipeline`] orchestrates everything

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
// `tinct::color`, `tinct::log`, `tinct::preview` and `tinct::path_resolver`.
pub use config::path as path_resolver;
pub use core::color;
pub use ui::{log, preview};

pub use config::*;
pub use core::color::{Color, Hsl, Rgb};
pub use ui::log::*;
pub use ui::preview::*;

pub use core::{Error, Mode, Result, Theme};

pub use output::FileOutput;
pub use palette::{AlgorithmParameters, ColorRole, LegacyPaletteGenerator, Palette};
pub use pipeline::{Pipeline, PipelineConfig};
pub use template::{ColorFilter, ColorProperty, FilterContext, TemplateProcessor};

pub use image::{ExtractedPalette, SchemeType, extract_source_color, extract_source_palette};
