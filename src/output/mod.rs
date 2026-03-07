//! Output format handlers
//!
//! This module provides traits and implementations for writing
//! processed themes to various output formats.

mod file;
mod registry;
mod terminal;

pub use file::FileOutput;
pub use registry::OutputRegistry;
pub use terminal::TerminalOutput;
