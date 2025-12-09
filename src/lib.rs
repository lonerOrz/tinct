pub mod color;
pub mod config;
pub mod theme;
pub mod log;
pub mod preview;

/// Public API for tinct
pub use color::*;
pub use config::*;
pub use theme::*;
pub use log::*;

/// Process a theme using a theme file, input template, and output path
/// This is the main entry point for the library functionality
pub fn process_theme_workflow(theme_path: &str, template_path: &str, output_path: &str, mode: &str) -> Result<(), String> {
    theme::process_theme(theme_path, template_path, output_path, mode)
}