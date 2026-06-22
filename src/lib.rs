//! tinct - A theme injector tool that applies Material Design 3 color palettes
//!
//! This library provides a modular architecture for:
//! - Loading themes from various sources
//! - Generating Material Design 3 color palettes
//! - Processing templates with theme data
//! - Outputting to various formats

// Core abstractions
pub mod core;

// New modular architecture
pub mod output;
pub mod palette;
pub mod template;
pub mod theme;

// Image-based color extraction (wallpaper support)
pub mod image;

// Core utilities (kept for backward compatibility and shared functionality)
pub mod color;
pub mod config;
pub mod log;
pub mod path_resolver;
pub mod pipeline;
pub mod preview;

// Re-exports
pub use color::*;
pub use config::*;
pub use log::*;
pub use preview::*;

// Core trait re-exports
pub use core::{
    Error, Mode, OutputFormat, PaletteGenerator, Result, TemplateEngine, Theme, ThemeLoader,
};

// New module re-exports
pub use output::{FileOutput, TerminalOutput};
pub use palette::{
    AlgorithmParameters, ColorEntry, ColorFormat, ColorHarmony, LegacyPaletteGenerator, Palette,
};
pub use pipeline::{Pipeline, PipelineConfig};
pub use template::{ColorFormatType, FilterContext, FilterRegistry, TemplateProcessor};
pub use theme::JsonThemeLoader;

// Shared types between binary and library
pub use image::{extract_source_color, SchemeType};
pub use path_resolver::{resolve_config_file_path, resolve_config_paths, resolve_theme_path};

// Shared CLI types (used by pipeline and binary)
use std::env;
use std::fs;
use std::path::Path;

/// CLI wrapper for SchemeType with clap integration.
#[derive(Clone, Debug, PartialEq)]
pub struct SchemeTypeCli(pub SchemeType);

impl clap::ValueEnum for SchemeTypeCli {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self(SchemeType::TonalSpot),
            Self(SchemeType::Content),
            Self(SchemeType::FruitSalad),
            Self(SchemeType::Rainbow),
            Self(SchemeType::Monochrome),
            Self(SchemeType::Vibrant),
            Self(SchemeType::Faithful),
            Self(SchemeType::Dysfunctional),
            Self(SchemeType::Muted),
        ]
    }

    fn to_possible_value<'a>(&self) -> Option<clap::builder::PossibleValue> {
        let name = match self.0 {
            SchemeType::TonalSpot => "tonal-spot",
            SchemeType::Content => "content",
            SchemeType::FruitSalad => "fruit-salad",
            SchemeType::Rainbow => "rainbow",
            SchemeType::Monochrome => "monochrome",
            SchemeType::Vibrant => "vibrant",
            SchemeType::Faithful => "faithful",
            SchemeType::Dysfunctional => "dysfunctional",
            SchemeType::Muted => "muted",
        };
        Some(clap::builder::PossibleValue::new(name))
    }
}

impl std::str::FromStr for SchemeTypeCli {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        SchemeType::parse(s)
            .map(SchemeTypeCli)
            .ok_or_else(|| format!("Invalid scheme type: {}", s))
    }
}

/// Run a post-hook command after processing a section.
pub fn run_post_hook(post_hook: &str, output_file: &str, section_name: Option<&str>) -> bool {
    if post_hook.is_empty() {
        return true;
    }

    let post_hook_cmd = post_hook.replace("{{output_file}}", output_file);

    // Check if it's a script starting with ./
    if post_hook_cmd.starts_with("./") {
        let script_dir = Path::new(env!("CARGO_MANIFEST_DIR")).to_str().unwrap();
        let post_hook_path = Path::new(script_dir).join(&post_hook_cmd);

        if post_hook_path.exists() && is_executable(&post_hook_path) {
            if let Some(name) = section_name {
                log::hook::executing(name);
            }

            match std::process::Command::new(&post_hook_path).output() {
                Ok(result) => {
                    if result.status.success() {
                        if let Some(name) = section_name {
                            log::hook::success(name);
                        }
                        true
                    } else {
                        if let Some(name) = section_name {
                            log::error::message(name, "Error executing hook script");
                        }
                        false
                    }
                }
                Err(e) => {
                    if let Some(name) = section_name {
                        log::error::message(name, &format!("Error executing hook script: {}", e));
                    }
                    false
                }
            }
        } else {
            if let Some(name) = section_name {
                log::error::message(
                    name,
                    &format!(
                        "post_hook '{}' not found. Skipping.",
                        post_hook_path.display()
                    ),
                );
            }
            false
        }
    } else {
        if let Some(name) = section_name {
            log::hook::executing(name);
        }

        match std::process::Command::new("sh")
            .arg("-c")
            .arg(&post_hook_cmd)
            .output()
        {
            Ok(result) => {
                if result.status.success() {
                    if let Some(name) = section_name {
                        log::hook::success(name);
                    }
                    true
                } else {
                    if let Some(name) = section_name {
                        log::error::hook_error(
                            name,
                            String::from_utf8_lossy(&result.stderr).as_ref(),
                        );
                    }
                    false
                }
            }
            Err(e) => {
                if let Some(name) = section_name {
                    log::error::hook_error(name, &e.to_string());
                }
                false
            }
        }
    }
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(metadata) = fs::metadata(path) {
        metadata.permissions().mode() & 0o111 != 0
    } else {
        false
    }
}

#[cfg(not(unix))]
fn is_executable(_path: &Path) -> bool {
    true
}

/// Validate a config section has required fields.
pub fn validate_config_section(section: &config::ConfigSection, section_name: &str) -> bool {
    let mut is_valid = true;

    if section.input_path.is_empty() {
        eprintln!("[{}] Missing required key: input_path", section_name);
        is_valid = false;
    }

    if section.output_path.is_empty() {
        eprintln!("[{}] Missing required key: output_path", section_name);
        is_valid = false;
    }

    is_valid
}
