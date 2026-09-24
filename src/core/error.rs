//! Error types for tinct
//!
//! Provides a unified error handling approach using thiserror.

use thiserror::Error;

/// Main error type for tinct operations
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// An I/O operation failed.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// A color value could not be parsed or converted.
    #[error("color error: {0}")]
    Color(String),

    /// A theme was malformed or unsupported.
    #[error("theme error: {0}")]
    Theme(String),

    /// A template could not be rendered.
    #[error("template error: {0}")]
    Template(String),

    /// The configuration was invalid.
    #[error("configuration error: {0}")]
    Config(String),

    /// Writing output failed.
    #[error("output error: {0}")]
    Output(String),

    /// A color filter could not be applied.
    #[error("filter error: {0}")]
    Filter(String),

    /// Palette generation failed.
    #[error("palette error: {0}")]
    Palette(String),

    /// A theme file could not be read from disk.
    #[error("failed to read theme file")]
    ThemeRead(#[source] std::io::Error),

    /// A theme file did not contain valid JSON.
    #[error("invalid theme JSON: {0}")]
    ThemeJson(#[source] serde_json::Error),
}

/// Result type alias for tinct operations
pub type Result<T> = std::result::Result<T, Error>;
