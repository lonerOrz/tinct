//! Core abstractions and types for tinct
//!
//! This module provides the foundational traits and types that define
//! the tinct architecture.

mod error;
mod types;

pub use error::{Error, Result};
pub use types::{ColorFormat, Mode, Theme};
