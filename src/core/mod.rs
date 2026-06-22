//! Core abstractions and types for tinct
//!
//! This module provides the foundational traits and types that define
//! the tinct architecture.

mod error;
mod traits;
mod types;

pub use error::{Error, Result};
pub use traits::{OutputFormat, PaletteGenerator, TemplateEngine, ThemeLoader};
pub use types::{ColorFormat, Mode, Theme};
