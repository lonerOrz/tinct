//! Error types for tinct
//!
//! Provides a unified error handling approach using thiserror.

use thiserror::Error;

/// Main error type for tinct operations
#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Color conversion error: {0}")]
    Color(String),

    #[error("Theme error: {0}")]
    Theme(String),

    #[error("Template error: {0}")]
    Template(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Output error: {0}")]
    Output(String),

    #[error("Filter error: {0}")]
    Filter(String),

    #[error("Palette error: {0}")]
    Palette(String),
}

/// Result type alias for tinct operations
pub type Result<T> = std::result::Result<T, Error>;
