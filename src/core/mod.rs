//! Core domain types for tinct.
//!
//! This is the foundation layer: the color model, errors, and the aggregate
//! types (`Mode`, `Theme`) that the rest of the crate builds on.

pub mod color;

mod error;
mod types;

pub use color::{Color, Hsl, Rgb};
pub use error::{Error, Result};
pub use types::{Mode, Theme};
