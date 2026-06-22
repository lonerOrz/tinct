//! Pipeline module — single entry point for the entire tinct workflow.
//!
//! `Pipeline::run(config)` handles theme creation, palette generation,
//! template rendering, output, and post-hooks. One interface, one place to test.

use std::env;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use colored::*;
use rayon::prelude::*;
use serde_json::json;

use crate::config::{AlgorithmConfig, ConfigSection};
use crate::core::{Mode, OutputFormat, TemplateEngine, Theme, ThemeLoader};
use crate::image::{extract_source_color, SchemeType};
use crate::log;
use crate::palette::{AlgorithmParameters, ColorHarmony, LegacyPaletteGenerator};
use crate::path_resolver;
use crate::template::TemplateProcessor;
use crate::theme::JsonThemeLoader;
use crate::FileOutput;

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
    pub log_level: crate::log::LogLevel,
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
        log::init_logger(log_level);

        // Create theme data from source
        let theme_data = Self::create_theme_data(&theme_source)?;

        // Build theme once — shared by preview and processing
        let theme = Self::build_theme(&theme_data, &algorithm)?;

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
            Self::run_preview(&theme, mode)?;
        } else {
            Self::run_processing(&theme, mode, &flat_config, log_level)?;
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
                let resolved = path_resolver::resolve_theme_path(theme_path)?;
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

    /// Build a Theme from JSON data and algorithm config.
    ///
    /// Single entry point for theme construction — used by both preview and processing.
    fn build_theme(
        theme_data: &serde_json::Value,
        algorithm: &AlgorithmConfig,
    ) -> crate::Result<Theme> {
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
        let theme_loader = JsonThemeLoader::new(palette_gen);
        theme_loader
            .load_value(theme_data)
            .map_err(|e| crate::core::Error::Config(format!("Theme loading error: {}", e)))
    }

    /// Show color preview and exit.
    fn run_preview(theme: &Theme, mode: Mode) -> crate::Result<()> {
        let colors = match mode {
            Mode::Dark => theme.dark_colors(),
            Mode::Light => theme.light_colors(),
        };
        crate::preview::show_color_preview_from_theme(&colors, mode);
        Ok(())
    }

    /// Process all config sections in parallel.
    fn run_processing(
        theme: &Theme,
        mode: Mode,
        flat_config: &crate::config::Config,
        log_level: crate::log::LogLevel,
    ) -> crate::Result<()> {
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
                let (success, error) =
                    process_section(section, theme, mode, &template_engine, &output);
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
                    run_post_hook(post_hook, output_path, Some(section_name));
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
    theme: &Theme,
    mode: Mode,
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

    let template_content = match fs::read_to_string(input_path) {
        Ok(c) => c,
        Err(e) => return (false, Some(format!("Error reading template: {}", e))),
    };

    let output_content = match template_engine.render(&template_content, theme, mode) {
        Ok(c) => c,
        Err(e) => return (false, Some(format!("Template rendering error: {}", e))),
    };

    if let Err(e) = output.write(&output_content, output_path) {
        return (false, Some(format!("Error writing output: {}", e)));
    }

    (true, None)
}

/// Validate a config section has required fields.
fn validate_config_section(section: &crate::config::ConfigSection, section_name: &str) -> bool {
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

/// Run a post-hook command after processing a section.
fn run_post_hook(post_hook: &str, output_file: &str, section_name: Option<&str>) -> bool {
    if post_hook.is_empty() {
        return true;
    }

    let post_hook_cmd = post_hook.replace("{{output_file}}", output_file);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AlgorithmConfig, ConfigSection};
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn default_algorithm() -> AlgorithmConfig {
        AlgorithmConfig::default()
    }

    fn seed_theme_data() -> serde_json::Value {
        json!({ "seed": "#6750A4" })
    }

    #[test]
    fn test_validate_config_section_valid() {
        let section = ConfigSection {
            input_path: "input.css".to_string(),
            output_path: "output.css".to_string(),
            post_hook: None,
        };
        assert!(validate_config_section(&section, "test_section"));
    }

    #[test]
    fn test_validate_config_section_empty_input() {
        let section = ConfigSection {
            input_path: String::new(),
            output_path: "output.css".to_string(),
            post_hook: None,
        };
        assert!(!validate_config_section(&section, "test_section"));
    }

    #[test]
    fn test_validate_config_section_empty_output() {
        let section = ConfigSection {
            input_path: "input.css".to_string(),
            output_path: String::new(),
            post_hook: None,
        };
        assert!(!validate_config_section(&section, "test_section"));
    }

    #[test]
    fn test_validate_config_section_both_empty() {
        let section = ConfigSection {
            input_path: String::new(),
            output_path: String::new(),
            post_hook: None,
        };
        assert!(!validate_config_section(&section, "test_section"));
    }

    #[test]
    fn test_validate_config_all_valid() {
        let mut config: crate::config::Config = HashMap::new();
        let mut group = HashMap::new();
        group.insert(
            "section1".to_string(),
            ConfigSection {
                input_path: "a.css".to_string(),
                output_path: "b.css".to_string(),
                post_hook: None,
            },
        );
        config.insert("group1".to_string(), group);
        assert!(Pipeline::validate_config(&config));
    }

    #[test]
    fn test_validate_config_one_invalid() {
        let mut config: crate::config::Config = HashMap::new();
        let mut group = HashMap::new();
        group.insert(
            "bad_section".to_string(),
            ConfigSection {
                input_path: String::new(),
                output_path: "b.css".to_string(),
                post_hook: None,
            },
        );
        config.insert("group1".to_string(), group);
        assert!(!Pipeline::validate_config(&config));
    }

    #[test]
    fn test_build_theme_from_seed() {
        let data = seed_theme_data();
        let result = Pipeline::build_theme(&data, &default_algorithm());
        assert!(result.is_ok());
        let theme = result.unwrap();
        let dark = theme.dark_colors();
        let light = theme.light_colors();
        assert!(!dark.is_empty());
        assert!(!light.is_empty());
        assert!(dark.contains_key("primary"));
        assert!(light.contains_key("primary"));
    }

    #[test]
    fn test_build_theme_dark_has_more_colors() {
        let data = seed_theme_data();
        let theme = Pipeline::build_theme(&data, &default_algorithm()).unwrap();
        let dark = theme.dark_colors();
        let light = theme.light_colors();
        assert_eq!(dark.len(), light.len());
    }

    #[test]
    fn test_create_theme_data_seed() {
        let source = ThemeSource::Seed("#FF0000".to_string());
        let data = Pipeline::create_theme_data(&source).unwrap();
        assert_eq!(data["seed"], "#FF0000");
    }

    #[test]
    fn test_create_theme_data_image_missing() {
        let source = ThemeSource::Image {
            path: "/nonexistent/image.png".to_string(),
            scheme_type: SchemeType::TonalSpot,
        };
        let result = Pipeline::create_theme_data(&source);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Image not found"));
    }

    #[test]
    fn test_create_theme_data_file_missing() {
        let source = ThemeSource::File("/nonexistent/theme.json".to_string());
        let result = Pipeline::create_theme_data(&source);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_theme_data_file_valid() {
        let tmp = TempDir::new().unwrap();
        let theme_path = tmp.path().join("theme.json");
        fs::write(
            &theme_path,
            "{\"seed\": \"#123456\", \"primary\": \"#ABCDEF\"}",
        )
        .unwrap();

        let source = ThemeSource::File(theme_path.to_str().unwrap().to_string());
        let data = Pipeline::create_theme_data(&source).unwrap();
        assert_eq!(data["seed"], "#123456");
    }

    #[test]
    fn test_process_section_missing_input() {
        let tmp = TempDir::new().unwrap();
        let section = ConfigSection {
            input_path: tmp
                .path()
                .join("nonexistent.css")
                .to_str()
                .unwrap()
                .to_string(),
            output_path: tmp.path().join("out.css").to_str().unwrap().to_string(),
            post_hook: None,
        };

        let theme = Pipeline::build_theme(&seed_theme_data(), &default_algorithm()).unwrap();
        let engine = Arc::new(TemplateProcessor::new());
        let output = Arc::new(FileOutput::new());

        let (success, error) = process_section(&section, &theme, Mode::Dark, &engine, &output);
        assert!(!success);
        assert!(error.unwrap().contains("does not exist"));
    }

    #[test]
    fn test_process_section_happy_path() {
        let tmp = TempDir::new().unwrap();
        let input = tmp.path().join("input.css");
        let output_path = tmp.path().join("output.css");

        fs::write(&input, "color: {{colors.primary.default.hex}};").unwrap();

        let section = ConfigSection {
            input_path: input.to_str().unwrap().to_string(),
            output_path: output_path.to_str().unwrap().to_string(),
            post_hook: None,
        };

        let theme = Pipeline::build_theme(&seed_theme_data(), &default_algorithm()).unwrap();
        let engine = Arc::new(TemplateProcessor::new());
        let output = Arc::new(FileOutput::new());

        let (success, error) = process_section(&section, &theme, Mode::Dark, &engine, &output);
        assert!(success, "process_section failed: {:?}", error);
        assert!(output_path.exists());

        let content = fs::read_to_string(&output_path).unwrap();
        assert!(content.starts_with("color: #"));
    }

    #[test]
    fn test_process_section_creates_parent_dirs() {
        let tmp = TempDir::new().unwrap();
        let input = tmp.path().join("input.css");
        let output_path = tmp.path().join("deep").join("nested").join("out.css");

        fs::write(&input, "body { }").unwrap();

        let section = ConfigSection {
            input_path: input.to_str().unwrap().to_string(),
            output_path: output_path.to_str().unwrap().to_string(),
            post_hook: None,
        };

        let theme = Pipeline::build_theme(&seed_theme_data(), &default_algorithm()).unwrap();
        let engine = Arc::new(TemplateProcessor::new());
        let output = Arc::new(FileOutput::new());

        let (success, error) = process_section(&section, &theme, Mode::Dark, &engine, &output);
        assert!(success, "process_section failed: {:?}", error);
        assert!(output_path.exists());
    }

    #[test]
    fn test_post_hook_empty_returns_true() {
        assert!(run_post_hook("", "output.css", None));
    }

    #[test]
    fn test_is_executable_nonexistent() {
        let path = Path::new("/nonexistent/file");
        assert!(!is_executable(path));
    }
}
