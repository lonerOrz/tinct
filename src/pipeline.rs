//! Pipeline module — single entry point for the entire tinct workflow.
//!
//! `Pipeline::run(config)` handles theme creation, palette generation,
//! template rendering, output, and post-hooks. One interface, one place to test.

use std::fs;
use std::path::Path;
use std::sync::Arc;

use colored::*;
use rayon::prelude::*;
use serde_json::json;

use crate::config::{AlgorithmConfig, ConfigSection};
use crate::core::{Mode, OutputFormat, TemplateEngine, ThemeLoader};
use crate::image::{extract_source_color, SchemeType};
use crate::log;
use crate::palette::{AlgorithmParameters, ColorHarmony, LegacyPaletteGenerator};
use crate::path_resolver;
use crate::template::TemplateProcessor;
use crate::theme::JsonThemeLoader;
use crate::{validate_config_section, FileOutput};

/// Pre-parsed configuration for the pipeline.
///
/// Constructed by `main` after CLI parsing and config resolution.
/// The pipeline owns all execution logic; main owns I/O and arg parsing.
pub struct PipelineConfig {
    pub config_path: String,
    pub flat_config: crate::config::Config,
    pub config_dir: String,
    pub mode: Mode,
    pub preview: bool,
    pub log_level: LogVerbosity,
    pub algorithm: AlgorithmConfig,
    pub image_scheme_type: Option<SchemeType>,
    pub theme_source: ThemeSource,
}

/// Where the theme data comes from.
pub enum ThemeSource {
    Seed(String),
    Image {
        path: String,
        scheme_type: SchemeType,
    },
    File(String),
}

/// Log verbosity level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogVerbosity {
    Quiet,
    Normal,
    Verbose,
}

impl LogVerbosity {
    pub fn is_quiet(&self) -> bool {
        matches!(self, LogVerbosity::Quiet)
    }
}

impl From<LogVerbosity> for crate::log::LogLevel {
    fn from(v: LogVerbosity) -> Self {
        match v {
            LogVerbosity::Quiet => crate::log::LogLevel::Quiet,
            LogVerbosity::Normal => crate::log::LogLevel::Normal,
            LogVerbosity::Verbose => crate::log::LogLevel::Verbose,
        }
    }
}

/// The pipeline: one method handles the entire tinct workflow.
pub struct Pipeline;

impl Pipeline {
    /// Run the full tinct pipeline.
    pub fn run(config: PipelineConfig) -> crate::Result<()> {
        let PipelineConfig {
            config_path,
            mut flat_config,
            config_dir,
            mode,
            preview,
            log_level,
            algorithm,
            image_scheme_type: _,
            theme_source,
        } = config;

        // Initialize logger
        log::init_logger(log_level.into());

        // Create theme data from source
        let theme_data = Self::create_theme_data(&theme_source)?;

        // Print info
        if !log_level.is_quiet() {
            Self::print_info(&config_path, &theme_source, mode);
        }

        // Resolve paths relative to config file
        for (_group_name, group) in flat_config.iter_mut() {
            for (_section_name, section) in group.iter_mut() {
                path_resolver::resolve_config_paths(section, &config_dir);
            }
        }

        // Validate config sections
        let is_valid = Self::validate_config(&flat_config);
        if !is_valid && !preview {
            return Err(crate::core::Error::Config(
                "Configuration validation failed".to_string(),
            ));
        }

        // Preview or process
        if preview {
            Self::run_preview(&theme_data, mode)?;
        } else {
            Self::run_processing(&theme_data, mode, &flat_config, &algorithm, log_level)?;
        }

        Ok(())
    }

    /// Create theme JSON value from the source.
    fn create_theme_data(source: &ThemeSource) -> crate::Result<serde_json::Value> {
        match source {
            ThemeSource::Seed(seed) => Ok(json!({ "seed": seed })),
            ThemeSource::Image { path, scheme_type } => {
                let img_path = Path::new(path);
                if !img_path.exists() {
                    return Err(crate::core::Error::Config(format!(
                        "Image not found: {}",
                        path
                    )));
                }

                let source_argb = extract_source_color(img_path, *scheme_type).map_err(|e| {
                    crate::core::Error::Config(format!("Error extracting color from image: {}", e))
                })?;

                let material_colors::color::Argb {
                    red, green, blue, ..
                } = source_argb;
                let hex = format!("#{:02X}{:02X}{:02X}", red, green, blue);

                Ok(json!({ "seed": hex }))
            }
            ThemeSource::File(theme_path) => {
                let resolved = path_resolver::resolve_theme_path(theme_path);
                let content = fs::read_to_string(&resolved).map_err(|e| {
                    crate::core::Error::Config(format!("Error reading theme file: {}", e))
                })?;
                serde_json::from_str::<serde_json::Value>(&content).map_err(|e| {
                    crate::core::Error::Config(format!("Error parsing theme JSON: {}", e))
                })
            }
        }
    }

    /// Print basic info to stdout.
    fn print_info(config_path: &str, source: &ThemeSource, mode: Mode) {
        println!("{}", "tinct - Theme Injector".bold());
        println!("{}: {}", "Config".blue(), config_path);

        match source {
            ThemeSource::Seed(seed) => {
                println!("{}: {}", "Seed".blue(), seed);
            }
            ThemeSource::Image { path, scheme_type } => {
                println!(
                    "{}: {} (scheme: {})",
                    "Image".blue(),
                    path,
                    scheme_type.to_string().yellow()
                );
            }
            ThemeSource::File(theme) => {
                println!("{}: {}", "Theme".blue(), theme);
            }
        }

        println!("{}: {}", "Mode".blue(), mode.to_string().yellow());
        println!();
    }

    /// Validate all config sections.
    fn validate_config(config: &crate::config::Config) -> bool {
        let mut is_valid = true;
        for (_group_name, group) in config.iter() {
            for (section_name, section) in group.iter() {
                if !validate_config_section(section, section_name) {
                    is_valid = false;
                }
            }
        }
        is_valid
    }

    /// Show color preview and exit.
    fn run_preview(theme_data: &serde_json::Value, mode: Mode) -> crate::Result<()> {
        let mode_str = mode.to_string();
        let result = crate::preview::show_color_preview_from_json(theme_data, &mode_str);
        match result {
            Ok(()) => Ok(()),
            Err(e) => Err(crate::core::Error::Config(format!(
                "Error showing color preview: {}",
                e
            ))),
        }
    }

    /// Process all config sections in parallel.
    fn run_processing(
        theme_data: &serde_json::Value,
        mode: Mode,
        flat_config: &crate::config::Config,
        algorithm: &AlgorithmConfig,
        log_level: LogVerbosity,
    ) -> crate::Result<()> {
        let harmony = ColorHarmony::parse(&algorithm.color_harmony).unwrap_or(ColorHarmony::Md3);

        let palette_gen = Arc::new(LegacyPaletteGenerator::new(AlgorithmParameters {
            contrast_threshold: algorithm.contrast_threshold,
            saturation_adjustment: algorithm.saturation_adjustment,
            lightness_adjustment: algorithm.lightness_adjustment,
            hue_shift: algorithm.hue_shift,
            min_contrast_ratio: algorithm.min_contrast_ratio,
            contrast_level: algorithm.contrast_level,
            color_harmony: harmony,
        }));
        let theme_loader = Arc::new(JsonThemeLoader::new(palette_gen));
        let template_engine = Arc::new(TemplateProcessor::new());
        let output = Arc::new(FileOutput::new());

        // Flatten all sections into a single list for parallel processing
        let sections: Vec<_> = flat_config
            .values()
            .flat_map(|group| {
                group
                    .iter()
                    .map(|(name, section)| (name.clone(), section))
                    .collect::<Vec<_>>()
            })
            .collect();

        let mut success_count = 0;
        let total_count = sections.len();

        let results: Vec<_> = sections
            .par_iter()
            .map(|(section_name, section)| {
                let (success, error) = process_section(
                    section,
                    theme_data,
                    mode,
                    &theme_loader,
                    &template_engine,
                    &output,
                );
                (section_name.clone(), success, error)
            })
            .collect();

        for (section_name, success, error) in &results {
            if *success {
                success_count += 1;
            }

            if !log_level.is_quiet() {
                if *success {
                    log::info::processed_successfully(section_name);
                } else if let Some(ref msg) = error {
                    log::error::message(section_name, msg);
                } else {
                    log::error::message(section_name, "failed to process");
                }
            }
        }

        // Run post-hooks sequentially after all processing
        for (section_name, section) in sections.iter() {
            if let Some(ref post_hook) = section.post_hook {
                if !post_hook.is_empty() {
                    let output_path = &section.output_path;
                    crate::run_post_hook(post_hook, output_path, Some(section_name));
                }
            }
        }

        if !log_level.is_quiet() {
            println!();
            log::general::summary(success_count, total_count);
        }

        Ok(())
    }
}

/// Process a single config section.
fn process_section(
    section: &ConfigSection,
    theme_data: &serde_json::Value,
    mode: Mode,
    theme_loader: &Arc<JsonThemeLoader>,
    template_engine: &Arc<TemplateProcessor>,
    output: &Arc<FileOutput>,
) -> (bool, Option<String>) {
    let input_path = &section.input_path;
    let output_path = &section.output_path;

    if !Path::new(input_path).exists() {
        return (
            false,
            Some(format!("Input file '{}' does not exist", input_path)),
        );
    }

    if let Some(parent) = Path::new(output_path).parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return (
                false,
                Some(format!("Error creating output directory: {}", e)),
            );
        }
    }

    let theme = match theme_loader.load_value(theme_data) {
        Ok(t) => t,
        Err(e) => return (false, Some(format!("Theme loading error: {}", e))),
    };

    let template_content = match fs::read_to_string(input_path) {
        Ok(c) => c,
        Err(e) => return (false, Some(format!("Error reading template: {}", e))),
    };

    let output_content = match template_engine.render(&template_content, &theme, mode) {
        Ok(c) => c,
        Err(e) => return (false, Some(format!("Template rendering error: {}", e))),
    };

    if let Err(e) = output.write(&output_content, output_path) {
        return (false, Some(format!("Error writing output: {}", e)));
    }

    (true, None)
}
