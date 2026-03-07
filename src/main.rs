//! tinct - Theme Injector
//!
//! A theme injector tool that applies Material Design 3 color palettes
//! to various configuration files.

use std::fs;
use std::path::Path;
use std::process;
use std::sync::Arc;

mod cli;
mod config;
mod log;

use clap::Parser;
use colored::*;
use tinct::core::{Mode, OutputFormat, TemplateEngine, ThemeLoader};
use tinct::output::FileOutput;
use tinct::palette::LegacyPaletteGenerator;
use tinct::template::TemplateProcessor;
use tinct::theme::JsonThemeLoader;

fn main() {
    let args = cli::CliArgs::parse();

    // Determine the config file path
    let config_path = if let Some(config_arg) = &args.config {
        config_arg.clone()
    } else {
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        format!("{}/.config/tinct/config.toml", home_dir)
    };

    // Initialize global logger
    log::init_logger(match args.log_level {
        cli::LogLevel::Quiet => log::LogLevel::Quiet,
        cli::LogLevel::Normal => log::LogLevel::Normal,
        cli::LogLevel::Verbose => log::LogLevel::Verbose,
    });

    // Print basic info
    if matches!(
        args.log_level,
        cli::LogLevel::Normal | cli::LogLevel::Verbose
    ) {
        println!("{}", "tinct - Theme Injector".bold());
        println!("{}: {}", "Config".blue(), config_path);
        println!("{}: {}", "Theme".blue(), args.theme);
        println!("{}: {}", "Mode".blue(), args.mode.to_string().yellow());
        println!();
    }

    // Resolve theme path
    let theme_file = resolve_theme_path(&args.theme);

    // Read TOML config
    let config_content = fs::read_to_string(&config_path).expect("Could not read config file");
    let config_root = config::ConfigRoot::parse(&config_content)
        .expect("Invalid TOML format in config file");

    let alg_params = config_root.algorithm.clone();
    let mut config = config_root.to_flat_config();

    // Resolve paths relative to config file
    let config_dir = Path::new(&config_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_string_lossy()
        .to_string();

    for (_group_name, group) in config.iter_mut() {
        for (_section_name, section) in group.iter_mut() {
            section.input_path = shellexpand::tilde(&section.input_path).to_string();
            section.output_path = shellexpand::tilde(&section.output_path).to_string();

            if !Path::new(&section.input_path).is_absolute() {
                section.input_path = Path::new(&config_dir)
                    .join(&section.input_path)
                    .to_string_lossy()
                    .to_string();
            }

            if !Path::new(&section.output_path).is_absolute() {
                let output_path = Path::new(&config_dir).join(&section.output_path);
                if let Some(parent) = output_path.parent() {
                    if let Ok(canonical_parent) = std::fs::canonicalize(parent) {
                        let file_name = output_path.file_name().unwrap_or_default();
                        section.output_path = canonical_parent.join(file_name).to_string_lossy().to_string();
                    } else {
                        section.output_path = output_path.to_string_lossy().to_string();
                    }
                } else {
                    section.output_path = output_path.to_string_lossy().to_string();
                }
            }

            if let Some(ref mut hook) = section.post_hook {
                if hook.starts_with("./") {
                    let hook_clone = hook.clone();
                    let hook_path = Path::new(&config_dir).join(&hook_clone);
                    if hook_path.exists() {
                        *hook = std::fs::canonicalize(&hook_path)
                            .unwrap_or_else(|_| hook_path)
                            .to_string_lossy()
                            .to_string();
                    } else {
                        *hook = hook_path.to_string_lossy().to_string();
                    }
                }
            }
        }
    }

    // Validate config sections
    let mut is_valid = true;
    for (_group_name, group) in config.iter() {
        for (section_name, section) in group.iter() {
            if !cli::validate_config_section(section, section_name) {
                is_valid = false;
            }
        }
    }

    if !is_valid && !args.preview {
        eprintln!("Configuration validation failed. Exiting.");
        process::exit(1);
    }

    // Show preview if requested
    if args.preview {
        match tinct::preview::show_color_preview(&theme_file, &args.mode.to_string()) {
            Ok(()) => process::exit(0),
            Err(e) => {
                eprintln!("Error showing color preview: {}", e);
                process::exit(1);
            }
        }
    }

    // Process sections using new architecture
    if !args.preview {
        let mut success_count = 0;
        let mut total_count = 0;

        let mode = match args.mode {
            cli::ThemeMode::Dark => Mode::Dark,
            cli::ThemeMode::Light => Mode::Light,
        };

        // Create new architecture components
        let palette_gen = Arc::new(LegacyPaletteGenerator::new(
            tinct::AlgorithmParameters {
                contrast_threshold: alg_params.contrast_threshold,
                saturation_adjustment: alg_params.saturation_adjustment,
                lightness_adjustment: alg_params.lightness_adjustment,
                hue_shift: alg_params.hue_shift,
                min_contrast_ratio: alg_params.min_contrast_ratio,
            }
        ));
        let theme_loader = JsonThemeLoader::new(palette_gen);
        let template_engine = TemplateProcessor::new();
        let output = FileOutput::new();

        for (group_name, group) in config.iter() {
            if matches!(args.log_level, cli::LogLevel::Verbose) {
                println!("Processing group: {}", group_name);
            }
            for (section_name, section) in group.iter() {
                total_count += 1;

                if !cli::validate_config_section(section, section_name) {
                    continue;
                }

                let result = process_section_new(
                    section_name,
                    section,
                    &theme_file,
                    mode,
                    &theme_loader,
                    &template_engine,
                    &output,
                    args.skip_sequences,
                );

                if result {
                    success_count += 1;
                }

                if matches!(
                    args.log_level,
                    cli::LogLevel::Normal | cli::LogLevel::Verbose
                ) {
                    if result {
                        crate::log::info::processed_successfully(section_name);
                    } else {
                        crate::log::error::message(section_name, "failed to process");
                    }
                }
            }
        }

        if matches!(
            args.log_level,
            cli::LogLevel::Normal | cli::LogLevel::Verbose
        ) {
            println!();
            crate::log::general::summary(success_count, total_count);
        }
    }
}

/// Process a single section using the new architecture
fn process_section_new(
    section_name: &str,
    section: &config::ConfigSection,
    theme_file: &str,
    mode: Mode,
    theme_loader: &JsonThemeLoader,
    template_engine: &TemplateProcessor,
    output: &FileOutput,
    _skip_sequences: bool,
) -> bool {
    let input_path = &section.input_path;
    let output_path = &section.output_path;

    // Check if input file exists
    if !Path::new(input_path).exists() {
        crate::log::error::message(
            section_name,
            &format!("Input file '{}' does not exist. Skipping.", input_path),
        );
        return false;
    }

    // Ensure output directory exists
    if let Some(parent) = Path::new(output_path).parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            crate::log::error::message(
                section_name,
                &format!("Error creating output directory: {}. Skipping.", e),
            );
            return false;
        }
    }

    // Load theme
    let theme = match theme_loader.load(theme_file) {
        Ok(t) => t,
        Err(e) => {
            crate::log::error::theme_error(section_name, &e.to_string());
            return false;
        }
    };

    // Read template
    let template_content = match fs::read_to_string(input_path) {
        Ok(c) => c,
        Err(e) => {
            crate::log::error::message(
                section_name,
                &format!("Error reading template file: {}. Skipping.", e),
            );
            return false;
        }
    };

    // Render template
    let output_content = match template_engine.render(&template_content, &theme, mode) {
        Ok(c) => c,
        Err(e) => {
            crate::log::error::theme_error(section_name, &e.to_string());
            return false;
        }
    };

    // Write output
    if let Err(e) = output.write(&output_content, output_path) {
        crate::log::error::message(section_name, &format!("Error writing output: {}", e));
        return false;
    }

    // Run post hook if specified
    if let Some(ref post_hook) = section.post_hook {
        if !post_hook.is_empty() {
            return cli::run_post_hook(post_hook, output_path, Some(section_name), cli::LogLevel::Normal);
        }
    }

    true
}

/// Resolve theme path - check both project themes and user themes
fn resolve_theme_path(theme_name: &str) -> String {
    // Check absolute path
    if Path::new(theme_name).is_absolute() && Path::new(theme_name).exists() {
        return theme_name.to_string();
    }

    // Check relative path
    if Path::new(theme_name).exists() {
        return Path::new(theme_name)
            .canonicalize()
            .unwrap_or_else(|_| std::path::PathBuf::from(theme_name))
            .to_string_lossy()
            .to_string();
    }

    // Check project themes directory
    let project_themes_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("themes")
        .join(format!("{}.json", theme_name));
    if project_themes_path.exists() {
        return project_themes_path.to_string_lossy().to_string();
    }

    // Check user config directory
    if let Ok(home_dir) = std::env::var("HOME") {
        let user_themes_path = Path::new(&home_dir)
            .join(".config")
            .join("tinct")
            .join("themes")
            .join(format!("{}.json", theme_name));
        if user_themes_path.exists() {
            return user_themes_path.to_string_lossy().to_string();
        }
    }

    eprintln!(
        "Theme '{}' not found in any of these locations:\n  - Current directory\n  - Project themes/ directory\n  - ~/.config/tinct/themes/",
        theme_name
    );
    process::exit(1);
}
