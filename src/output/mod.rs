//! Output format handlers
//!
//! This module provides traits and implementations for writing
//! processed themes to various output formats.

mod file;
mod terminal;

pub use file::FileOutput;
pub use terminal::TerminalOutput;
