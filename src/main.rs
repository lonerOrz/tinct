use std::env;
use std::fs;
use std::path::Path;
use std::process;

mod color;
mod config;
mod theme;
mod cli;
mod log;

use clap::Parser;
use colored::*;
use config::{Config, ConfigSection};

fn main() {
    let args = cli::CliArgs::parse();

    // Initialize global logger with the specified log level
    log::init_logger(match args.log_level {
        cli::LogLevel::Quiet => log::LogLevel::Quiet,
        cli::LogLevel::Normal => log::LogLevel::Normal,
        cli::LogLevel::Verbose => log::LogLevel::Verbose,
    });

    // Print basic info in a clean format
    if matches!(args.log_level, cli::LogLevel::Normal | cli::LogLevel::Verbose) {
        println!("{}", "tinct - Theme Injector".bold());
        println!("{}: {}", "Config".blue(), args.config);
        println!("{}: {}", "Theme".blue(), args.theme);
        println!("{}: {}", "Mode".blue(), args.mode.to_string().yellow());
        println!();
    }

    // Resolve config path
    let config_path = cli::resolve_path(Some(&args.config), Some("config.toml"), None)
        .expect("Config file path could not be resolved");

    if !Path::new(&config_path).exists() {
        eprintln!("Config file '{}' does not exist.", args.config);
        process::exit(1);
    }

    // Resolve theme path
    let theme_path = cli::resolve_path(Some(&args.theme), None, Some("themes"));
    let theme_file = if let Some(path) = theme_path {
        path
    } else {
        let theme_lookup_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("themes")
            .join(format!("{}.json", args.theme));
        eprintln!(
            "Theme '{}' not found in themes/ directory. Looking for {}",
            args.theme,
            theme_lookup_path.display()
        );
        process::exit(1);
    };

    // Read TOML config
    let config_content = fs::read_to_string(&config_path).expect("Could not read config file");

    let config: Config =
        toml::from_str(&config_content).expect("Invalid TOML format in config file");

    // Convert relative paths in config to absolute paths
    // Paths should be resolved relative to the config file location, not the project root
    let config_dir = Path::new(&config_path)
        .parent()
        .unwrap_or(Path::new(""))
        .to_string_lossy()
        .to_string();
    let mut processed_config = config;

    for (_group_name, group) in processed_config.iter_mut() {
        for (_section_name, section) in group.iter_mut() {
            if let Some(abs_path) = config::resolve_path_to_abs(&section.input_path, &config_dir) {
                section.input_path = abs_path;
            }

            if let Some(abs_path) = config::resolve_path_to_abs(&section.output_path, &config_dir) {
                section.output_path = abs_path;
            }

            if let Some(ref post_hook) = section.post_hook {
                if post_hook.starts_with("./") {
                    if let Some(abs_path) = config::resolve_path_to_abs(post_hook, &config_dir) {
                        // Update the post_hook in the config
                        section.post_hook = Some(abs_path);
                    }
                }
            }
        }
    }

    // Process each section in the config
    let mut success_count = 0;
    let mut total_count = 0;

    let mode_str = args.mode.to_string();
    for (group_name, group) in processed_config.iter() {
        if matches!(args.log_level, cli::LogLevel::Verbose) {
            println!("Processing group: {}", group_name);
        }
        for (section_name, section) in group.iter() {
            total_count += 1;

            if !cli::validate_config_section(section, section_name) {
                continue;
            }

            let result = cli::process_section(section_name, section, &theme_file, &mode_str, args.log_level.clone());

            if result {
                success_count += 1;
            }

            if matches!(args.log_level, cli::LogLevel::Normal | cli::LogLevel::Verbose) {
                if result {
                    log::success(section_name, "processed successfully");
                } else {
                    log::error(section_name, "failed to process");
                }
            }
        }
    }

    if matches!(args.log_level, cli::LogLevel::Normal | cli::LogLevel::Verbose) {
        println!();
        log::info(&format!(
            "{}: {} {} {} {}",
            "Summary".bold(),
            success_count.to_string().green().bold(),
            "of".white(),
            total_count.to_string().white().bold(),
            "sections processed successfully".green()
        ));
    }
}
