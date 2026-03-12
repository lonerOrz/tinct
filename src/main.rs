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
use tinct::core::{Mode, OutputFormat, TemplateEngine, ThemeLoader};
use tinct::output::FileOutput;
use tinct::palette::LegacyPaletteGenerator;
use tinct::template::TemplateProcessor;
use tinct::theme::JsonThemeLoader;

fn main() {
    let args = cli::CliArgs::parse();

    // Determine the config file path
    let config_path = path_resolver::resolve_config_file_path(args.config.as_ref());

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
    let theme_file = path_resolver::resolve_theme_path(&args.theme);

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
                    theme_file: &theme_file,
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
    theme_file: &'a str,
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

    // Load theme
    let theme = match ctx.theme_loader.load(ctx.theme_file) {
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
