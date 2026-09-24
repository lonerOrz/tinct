//! Pipeline module — single entry point for the entire tinct workflow.
//!
//! `Pipeline::run(config)` handles theme creation, palette generation,
//! template rendering, output, and post-hooks. One interface, one place to test.

use colored::*;
use rayon::prelude::*;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::FileOutput;
use crate::config::{AnsiConfig, Config, ConfigSection, ImageConfig, resolve_theme_path};
use crate::core::color::Color;
use crate::core::{Mode, Theme};
use crate::image::{ImageOptions, SchemeType, extract_source_palette_with};
use crate::palette::{
    AlgorithmParameters, AnsiParams, LegacyPaletteGenerator, collect_theme_colors,
};
use crate::template::TemplateProcessor;
use crate::ui::log;
use crate::ui::preview::show_color_preview_from_theme;

/// Pre-parsed configuration for the pipeline.
///
/// Constructed by `main` after CLI parsing. The pipeline owns all execution
/// logic; main only assembles this struct.
pub struct PipelineConfig {
    pub config: Config,
    pub mode: Mode,
    pub preview: bool,
    pub log_level: log::LogLevel,
    /// MD3 scheme variant used for palette generation (all theme sources).
    pub scheme_type: SchemeType,
    pub theme_source: ThemeSource,
}

/// Where the theme data comes from.
pub enum ThemeSource {
    Seed(String),
    Image(PathBuf),
    File(String),
}

/// The pipeline: one method handles the entire tinct workflow.
pub struct Pipeline;

impl Pipeline {
    /// Run the full tinct pipeline.
    pub fn run(pipeline: PipelineConfig) -> crate::Result<()> {
        let PipelineConfig {
            config,
            mode,
            preview,
            log_level,
            scheme_type,
            theme_source,
        } = pipeline;

        // Initialize logger
        log::init_logger(log_level);

        // Create theme data from source (image sources also yield clusters)
        let (theme_data, source_colors) =
            Self::create_theme_data(&theme_source, scheme_type, &config.image)?;

        // Build theme once — shared by preview and processing
        let theme = Self::build_theme(
            &theme_data,
            &source_colors,
            &config.algorithm,
            scheme_type,
            &config.ansi,
        )?;

        // Print info
        if !log_level.is_quiet() {
            Self::print_info(&config.config_path, &theme_source, mode, scheme_type);
        }

        // Validate config sections
        let is_valid = Self::validate_config(&config.sections);
        if !is_valid && !preview {
            return Err(crate::core::Error::Config(
                "Configuration validation failed".to_string(),
            ));
        }

        // Preview or process
        if preview {
            Self::run_preview(&theme, mode)?;
        } else {
            Self::run_processing(&theme, mode, &config.sections, log_level)?;
        }

        Ok(())
    }

    /// Create theme JSON value from the source.
    ///
    /// `scheme_type` selects the extraction algorithm for image sources.
    fn create_theme_data(
        source: &ThemeSource,
        scheme_type: SchemeType,
        image: &ImageConfig,
    ) -> crate::Result<(serde_json::Value, Vec<Color>)> {
        match source {
            ThemeSource::Seed(seed) => {
                let value = json!({ "seed": seed });
                let colors = collect_theme_colors(&value);
                Ok((value, colors))
            }
            ThemeSource::Image(path) => {
                if !path.exists() {
                    return Err(crate::core::Error::Config(format!(
                        "Image not found: {}",
                        path.display()
                    )));
                }

                let options = ImageOptions {
                    max_colors: image.max_colors,
                    min_population: image.min_population,
                    filter: image.filter,
                    quantizer: image.quantizer,
                };
                let extracted =
                    extract_source_palette_with(path, scheme_type, &options).map_err(|e| {
                        crate::core::Error::Config(format!(
                            "Error extracting color from image: {}",
                            e
                        ))
                    })?;

                let material_colors::color::Argb {
                    red, green, blue, ..
                } = extracted.source;
                let hex = format!("#{:02X}{:02X}{:02X}", red, green, blue);

                Ok((json!({ "seed": hex }), extracted.colors))
            }
            ThemeSource::File(theme_path) => {
                let resolved = resolve_theme_path(theme_path)?;
                let content = fs::read_to_string(&resolved).map_err(|e| {
                    crate::core::Error::Config(format!("Error reading theme file: {}", e))
                })?;
                let value = serde_json::from_str::<serde_json::Value>(&content).map_err(|e| {
                    crate::core::Error::Config(format!("Error parsing theme JSON: {}", e))
                })?;
                // A theme file's own colors drive the terminal palette.
                let colors = collect_theme_colors(&value);
                Ok((value, colors))
            }
        }
    }

    /// Print basic info to stdout.
    fn print_info(config_path: &Path, source: &ThemeSource, mode: Mode, scheme_type: SchemeType) {
        println!("{}", "tinct - Theme Injector".bold());
        println!("{}: {}", "Config".blue(), config_path.display());

        match source {
            ThemeSource::Seed(seed) => {
                println!("{}: {}", "Seed".blue(), seed);
            }
            ThemeSource::Image(path) => {
                println!("{}: {}", "Image".blue(), path.display());
            }
            ThemeSource::File(theme) => {
                println!("{}: {}", "Theme".blue(), theme);
            }
        }

        println!("{}: {}", "Scheme".blue(), scheme_type.to_string().yellow());
        println!("{}: {}", "Mode".blue(), mode.to_string().yellow());
        println!();
    }

    /// Validate all config sections.
    fn validate_config(sections: &HashMap<String, ConfigSection>) -> bool {
        let mut is_valid = true;
        for (section_name, section) in sections {
            if !validate_config_section(section, section_name) {
                is_valid = false;
            }
        }
        is_valid
    }

    /// Build a Theme from JSON data and algorithm config.
    ///
    /// Single entry point for theme construction — used by both preview and processing.
    fn build_theme(
        theme_data: &serde_json::Value,
        source_colors: &[Color],
        algorithm: &crate::config::AlgorithmConfig,
        scheme_type: SchemeType,
        ansi: &AnsiConfig,
    ) -> crate::Result<Theme> {
        let dark_scheme = algorithm.variant_dark.unwrap_or(scheme_type);
        let light_scheme = algorithm.variant_light.unwrap_or(scheme_type);
        let palette_gen = LegacyPaletteGenerator::new(
            AlgorithmParameters {
                saturation_adjustment: algorithm.saturation_adjustment,
                hue_shift: algorithm.hue_shift,
                contrast_level: algorithm.contrast_level,
                seed_tone: algorithm.seed_tone,
                chroma_floor: algorithm.chroma_floor,
            },
            dark_scheme,
        )
        .with_light_scheme(light_scheme)
        .with_ansi(AnsiParams::from_config(ansi))
        .with_source_colors(source_colors.to_vec());
        Theme::from_json_value(theme_data, &palette_gen)
            .map_err(|e| crate::core::Error::Config(format!("Theme loading error: {}", e)))
    }

    /// Show color preview and exit.
    fn run_preview(theme: &Theme, mode: Mode) -> crate::Result<()> {
        let palette = match mode {
            Mode::Dark => &theme.dark_palette,
            Mode::Light => &theme.light_palette,
        };
        show_color_preview_from_theme(palette, mode)
            .map_err(|e| crate::core::Error::Config(format!("Preview error: {}", e)))?;
        Ok(())
    }

    /// Process all config sections in parallel.
    fn run_processing(
        theme: &Theme,
        mode: Mode,
        sections: &HashMap<String, ConfigSection>,
        log_level: log::LogLevel,
    ) -> crate::Result<()> {
        let template_engine = TemplateProcessor::new();
        let output = FileOutput::new();

        let entries: Vec<(&String, &ConfigSection)> = sections.iter().collect();

        let results: Vec<_> = entries
            .par_iter()
            .map(|(section_name, section)| {
                let (success, error) =
                    process_section(section, theme, mode, &template_engine, &output);
                ((*section_name).clone(), success, error)
            })
            .collect();

        let total_count = results.len();
        let mut success_count = 0;

        for (section_name, success, error) in &results {
            if *success {
                success_count += 1;
            }

            if !log_level.is_quiet() {
                if *success {
                    log::info::processed_successfully(section_name);
                } else if let Some(msg) = error {
                    log::error::message(section_name, msg);
                } else {
                    log::error::message(section_name, "failed to process");
                }
            }
        }

        // Run post-hooks sequentially after all processing
        for (section_name, section) in &entries {
            if let Some(ref post_hook) = section.post_hook
                && !post_hook.is_empty()
            {
                run_post_hook(post_hook, &section.output_path, Some(section_name));
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
    template_engine: &TemplateProcessor,
    output: &FileOutput,
) -> (bool, Option<String>) {
    let input_path = &section.input_path;
    let output_path = &section.output_path;

    if !input_path.exists() {
        return (
            false,
            Some(format!(
                "Input file '{}' does not exist",
                input_path.display()
            )),
        );
    }

    if let Some(parent) = output_path.parent()
        && let Err(e) = fs::create_dir_all(parent)
    {
        return (
            false,
            Some(format!("Error creating output directory: {}", e)),
        );
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
fn validate_config_section(section: &ConfigSection, section_name: &str) -> bool {
    let mut is_valid = true;

    if section.input_path.as_os_str().is_empty() {
        eprintln!("[{}] Missing required key: input_path", section_name);
        is_valid = false;
    }

    if section.output_path.as_os_str().is_empty() {
        eprintln!("[{}] Missing required key: output_path", section_name);
        is_valid = false;
    }

    is_valid
}

/// Run a post-hook command after processing a section.
fn run_post_hook(post_hook: &str, output_file: &Path, section_name: Option<&str>) -> bool {
    if post_hook.is_empty() {
        return true;
    }

    let post_hook_cmd = post_hook.replace("{{output_file}}", &output_file.to_string_lossy());

    if post_hook_cmd.starts_with("./") {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let post_hook_path = cwd.join(&post_hook_cmd);

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
    use crate::config::AlgorithmConfig;
    use tempfile::TempDir;

    fn default_algorithm() -> AlgorithmConfig {
        AlgorithmConfig::default()
    }

    fn seed_theme_data() -> serde_json::Value {
        json!({ "seed": "#6750A4" })
    }

    /// Build a theme with no image clusters and the default Tonal Spot scheme.
    fn build_theme(data: &serde_json::Value) -> crate::Result<Theme> {
        Pipeline::build_theme(
            data,
            &[],
            &default_algorithm(),
            SchemeType::TonalSpot,
            &AnsiConfig::default(),
        )
    }

    fn section(input: impl Into<PathBuf>, output: impl Into<PathBuf>) -> ConfigSection {
        ConfigSection {
            input_path: input.into(),
            output_path: output.into(),
            post_hook: None,
        }
    }

    #[test]
    fn test_variant_dark_overrides_only_dark_mode() {
        let mut algorithm = default_algorithm();
        algorithm.variant_dark = Some(SchemeType::Monochrome);

        let base = Pipeline::build_theme(
            &seed_theme_data(),
            &[],
            &default_algorithm(),
            SchemeType::TonalSpot,
            &AnsiConfig::default(),
        )
        .unwrap();
        let overridden = Pipeline::build_theme(
            &seed_theme_data(),
            &[],
            &algorithm,
            SchemeType::TonalSpot,
            &AnsiConfig::default(),
        )
        .unwrap();

        assert_ne!(
            base.dark_palette.get("primary").unwrap().hex(),
            overridden.dark_palette.get("primary").unwrap().hex(),
            "variant_dark should change the dark palette"
        );
        assert_eq!(
            base.light_palette.get("primary").unwrap().hex(),
            overridden.light_palette.get("primary").unwrap().hex(),
            "light mode must keep the base scheme when variant_light is unset"
        );
    }

    #[test]
    fn test_validate_config_section_valid() {
        assert!(validate_config_section(
            &section("input.css", "output.css"),
            "test_section"
        ));
    }

    #[test]
    fn test_validate_config_section_empty_input() {
        assert!(!validate_config_section(
            &section("", "output.css"),
            "test_section"
        ));
    }

    #[test]
    fn test_validate_config_section_empty_output() {
        assert!(!validate_config_section(
            &section("input.css", ""),
            "test_section"
        ));
    }

    #[test]
    fn test_validate_config_section_both_empty() {
        assert!(!validate_config_section(&section("", ""), "test_section"));
    }

    #[test]
    fn test_validate_config_all_valid() {
        let mut config = HashMap::new();
        config.insert("group1.section1".to_string(), section("a.css", "b.css"));
        assert!(Pipeline::validate_config(&config));
    }

    #[test]
    fn test_validate_config_one_invalid() {
        let mut config = HashMap::new();
        config.insert("group1.bad_section".to_string(), section("", "b.css"));
        assert!(!Pipeline::validate_config(&config));
    }

    #[test]
    fn test_build_theme_from_seed() {
        let data = seed_theme_data();
        let result = build_theme(&data);
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
        let theme = build_theme(&data).unwrap();
        let dark = theme.dark_colors();
        let light = theme.light_colors();
        assert_eq!(dark.len(), light.len());
    }

    #[test]
    fn test_create_theme_data_seed() {
        let source = ThemeSource::Seed("#FF0000".to_string());
        let (data, _) =
            Pipeline::create_theme_data(&source, SchemeType::TonalSpot, &ImageConfig::default())
                .unwrap();
        assert_eq!(data["seed"], "#FF0000");
    }

    #[test]
    fn test_create_theme_data_image_missing() {
        let source = ThemeSource::Image(PathBuf::from("/nonexistent/image.png"));
        let result =
            Pipeline::create_theme_data(&source, SchemeType::TonalSpot, &ImageConfig::default());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Image not found"));
    }

    #[test]
    fn test_create_theme_data_file_missing() {
        let source = ThemeSource::File("/nonexistent/theme.json".to_string());
        let result =
            Pipeline::create_theme_data(&source, SchemeType::TonalSpot, &ImageConfig::default());
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
        let (data, colors) =
            Pipeline::create_theme_data(&source, SchemeType::TonalSpot, &ImageConfig::default())
                .unwrap();
        assert_eq!(data["seed"], "#123456");
        // The theme file's own colors are captured so the terminal palette can
        // be generated from them, not just from the seed.
        assert_eq!(colors.len(), 2);
    }

    #[test]
    fn test_create_theme_data_seed_yields_seed_color() {
        let source = ThemeSource::Seed("#FF0000".to_string());
        let (_, colors) =
            Pipeline::create_theme_data(&source, SchemeType::TonalSpot, &ImageConfig::default())
                .unwrap();
        assert_eq!(colors.len(), 1);
    }

    #[test]
    fn test_process_section_missing_input() {
        let tmp = TempDir::new().unwrap();
        let section = section(
            tmp.path().join("nonexistent.css"),
            tmp.path().join("out.css"),
        );

        let theme = build_theme(&seed_theme_data()).unwrap();
        let engine = TemplateProcessor::new();
        let output = FileOutput::new();

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

        let section = section(input, output_path.clone());

        let theme = build_theme(&seed_theme_data()).unwrap();
        let engine = TemplateProcessor::new();
        let output = FileOutput::new();

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

        let section = section(input, output_path.clone());

        let theme = build_theme(&seed_theme_data()).unwrap();
        let engine = TemplateProcessor::new();
        let output = FileOutput::new();

        let (success, error) = process_section(&section, &theme, Mode::Dark, &engine, &output);
        assert!(success, "process_section failed: {:?}", error);
        assert!(output_path.exists());
    }

    #[test]
    fn test_post_hook_empty_returns_true() {
        assert!(run_post_hook("", Path::new("output.css"), None));
    }

    #[test]
    fn test_is_executable_nonexistent() {
        let path = Path::new("/nonexistent/file");
        assert!(!is_executable(path));
    }
}
