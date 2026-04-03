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
mod path_resolver;

use clap::Parser;
use colored::*;
use serde_json::json;
use tinct::core::{Mode, OutputFormat, TemplateEngine, ThemeLoader};
use tinct::image::extract_source_color;
use tinct::output::FileOutput;
use tinct::palette::LegacyPaletteGenerator;
use tinct::template::TemplateProcessor;
use tinct::theme::JsonThemeLoader;

/// Convert Argb to hex string.
fn argb_to_hex(argb: material_colors::color::Argb) -> String {
    let material_colors::color::Argb {
        red, green, blue, ..
    } = argb;
    format!("#{:02X}{:02X}{:02X}", red, green, blue)
}

fn main() {
    let args = cli::CliArgs::parse();

    // Validate arguments
    if let Err(e) = args.validate() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }

    // Determine the config file path
    let config_path = path_resolver::resolve_config_file_path(args.config.as_ref());

    // Initialize global logger
    log::init_logger(match args.log_level {
        cli::LogLevel::Quiet => log::LogLevel::Quiet,
        cli::LogLevel::Normal => log::LogLevel::Normal,
        cli::LogLevel::Verbose => log::LogLevel::Verbose,
    });

    // Create theme data: either from seed, image, or from theme file
    let theme_data = if let Some(ref seed) = args.seed {
        // Create temporary theme from seed color
        json!({ "seed": seed })
    } else if let Some(ref image_path) = args.image {
        // Extract source color from wallpaper image
        let img_path = Path::new(image_path);
        if !img_path.exists() {
            eprintln!("Error: Image not found: {}", image_path);
            process::exit(1);
        }

        let scheme_type = args.scheme_type.0;

        if matches!(
            args.log_level,
            cli::LogLevel::Normal | cli::LogLevel::Verbose
        ) {
            println!(
                "{}: {} (scheme: {})",
                "Extracting".blue(),
                image_path,
                scheme_type.to_string().yellow()
            );
        }

        let source_argb = match extract_source_color(img_path, scheme_type) {
            Ok(argb) => argb,
            Err(e) => {
                eprintln!("Error extracting color from image: {}", e);
                process::exit(1);
            }
        };

        let hex = argb_to_hex(source_argb);

        if matches!(
            args.log_level,
            cli::LogLevel::Normal | cli::LogLevel::Verbose
        ) {
            println!("{}: {}", "Source color".blue(), hex.green());
            println!();
        }

        json!({ "seed": hex })
    } else {
        // Load theme from file
        let theme_file = path_resolver::resolve_theme_path(args.theme.as_ref().unwrap());
        match fs::read_to_string(&theme_file) {
            Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(json) => json,
                Err(e) => {
                    eprintln!("Error parsing theme JSON: {}", e);
                    process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("Error reading theme file: {}", e);
                process::exit(1);
            }
        }
    };

    // Print basic info
    if matches!(
        args.log_level,
        cli::LogLevel::Normal | cli::LogLevel::Verbose
    ) {
        println!("{}", "tinct - Theme Injector".bold());
        println!("{}: {}", "Config".blue(), config_path);
        if let Some(seed) = &args.seed {
            println!("{}: {}", "Seed".blue(), seed);
        } else if let Some(image) = &args.image {
            println!(
                "{}: {} (scheme: {})",
                "Image".blue(),
                image,
                args.scheme_type.0.to_string().yellow()
            );
        } else if let Some(theme) = &args.theme {
            println!("{}: {}", "Theme".blue(), theme);
        }
        println!("{}: {}", "Mode".blue(), args.mode.to_string().yellow());
        println!();
    }

    // Read TOML config
    let config_content = fs::read_to_string(&config_path).expect("Could not read config file");
    let config_root =
        config::ConfigRoot::parse(&config_content).expect("Invalid TOML format in config file");

    let alg_params = config_root.algorithm.clone();
    let mut config = config_root.into_flat_config();

    // Resolve paths relative to config file
    let config_dir = Path::new(&config_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_string_lossy()
        .to_string();

    for (_group_name, group) in config.iter_mut() {
        for (_section_name, section) in group.iter_mut() {
            path_resolver::resolve_config_paths(section, &config_dir);
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
        let preview_result = if let Some(ref seed) = args.seed {
            let temp_theme = json!({ "seed": seed });
            tinct::preview::show_color_preview_from_json(&temp_theme, &args.mode.to_string())
        } else if let Some(ref image_path) = args.image {
            // For image-based preview, extract color first
            let img_path = Path::new(image_path);
            let scheme_type = args.scheme_type.0;
            match extract_source_color(img_path, scheme_type) {
                Ok(argb) => {
                    let hex = argb_to_hex(argb);
                    let temp_theme = json!({ "seed": hex });
                    tinct::preview::show_color_preview_from_json(
                        &temp_theme,
                        &args.mode.to_string(),
                    )
                }
                Err(e) => {
                    eprintln!("Error extracting color from image: {}", e);
                    process::exit(1);
                }
            }
        } else {
            let theme_file = path_resolver::resolve_theme_path(args.theme.as_ref().unwrap());
            tinct::preview::show_color_preview(&theme_file, &args.mode.to_string())
        };

        match preview_result {
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
        let palette_gen = Arc::new(LegacyPaletteGenerator::new(tinct::AlgorithmParameters {
            contrast_threshold: alg_params.contrast_threshold,
            saturation_adjustment: alg_params.saturation_adjustment,
            lightness_adjustment: alg_params.lightness_adjustment,
            hue_shift: alg_params.hue_shift,
            min_contrast_ratio: alg_params.min_contrast_ratio,
        }));
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

                let ctx = ProcessContext {
                    section_name,
                    section,
                    theme_data: &theme_data,
                    mode,
                    theme_loader: &theme_loader,
                    template_engine: &template_engine,
                    output: &output,
                };
                let result = process_section_new(ctx, args.skip_sequences);

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

// Context struct for process_section to reduce parameter count
struct ProcessContext<'a> {
    section_name: &'a str,
    section: &'a config::ConfigSection,
    theme_data: &'a serde_json::Value,
    mode: Mode,
    theme_loader: &'a JsonThemeLoader,
    template_engine: &'a TemplateProcessor,
    output: &'a FileOutput,
}

/// Process a single section using the new architecture
fn process_section_new(ctx: ProcessContext, _skip_sequences: bool) -> bool {
    let input_path = &ctx.section.input_path;
    let output_path = &ctx.section.output_path;

    // Check if input file exists
    if !Path::new(input_path).exists() {
        crate::log::error::message(
            ctx.section_name,
            &format!("Input file '{}' does not exist. Skipping.", input_path),
        );
        return false;
    }

    // Ensure output directory exists
    if let Some(parent) = Path::new(output_path).parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            crate::log::error::message(
                ctx.section_name,
                &format!("Error creating output directory: {}. Skipping.", e),
            );
            return false;
        }
    }

    // Load theme from theme_data
    let theme = match ctx.theme_loader.load_value(ctx.theme_data) {
        Ok(t) => t,
        Err(e) => {
            crate::log::error::theme_error(ctx.section_name, &e.to_string());
            return false;
        }
    };

    // Read template
    let template_content = match fs::read_to_string(input_path) {
        Ok(c) => c,
        Err(e) => {
            crate::log::error::message(
                ctx.section_name,
                &format!("Error reading template file: {}. Skipping.", e),
            );
            return false;
        }
    };

    // Render template
    let output_content = match ctx
        .template_engine
        .render(&template_content, &theme, ctx.mode)
    {
        Ok(c) => c,
        Err(e) => {
            crate::log::error::theme_error(ctx.section_name, &e.to_string());
            return false;
        }
    };

    // Write output
    if let Err(e) = ctx.output.write(&output_content, output_path) {
        crate::log::error::message(ctx.section_name, &format!("Error writing output: {}", e));
        return false;
    }

    // Run post hook if specified
    if let Some(ref post_hook) = ctx.section.post_hook {
        if !post_hook.is_empty() {
            return cli::run_post_hook(
                post_hook,
                output_path,
                Some(ctx.section_name),
                cli::LogLevel::Normal,
            );
        }
    }

    true
}
