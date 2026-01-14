pub mod color;
pub mod config;
pub mod log;
pub mod preview;

// Module re-exports for backward compatibility
pub use color::*;
pub use config::*;
pub use log::*;

// New modular architecture
pub mod filter;
pub mod output_handler;
pub mod palette_generator;
pub mod template_processor;
pub mod theme_loader;

pub use filter::*;
pub use output_handler::*;
pub use palette_generator::*;
pub use template_processor::*;
/// Public API for tinct
pub use theme_loader::*;

/// Process a theme using a theme file, input template, and output path
/// This is the main entry point for the library functionality
pub fn process_theme_workflow(
    theme_path: &str,
    template_path: &str,
    output_path: &str,
    mode: &str,
) -> Result<(), String> {
    use crate::output_handler::save_output;
    use crate::palette_generator::generate_palette;
    use crate::template_processor::process_template;
    use crate::theme_loader::load_theme;

    // Load theme
    let theme_data = load_theme(theme_path)?;

    // Select theme mode
    let (theme, effective_mode) = crate::theme_loader::select_theme_mode(&theme_data, mode)?;

    // Generate palette
    let palette = generate_palette(&theme, effective_mode == "dark", false)?;

    // Load and process template
    let template_content = crate::template_processor::load_template(template_path)?;
    let processed_content = process_template(&template_content, &palette, &effective_mode);

    // Save output
    save_output(&processed_content, output_path)
}
