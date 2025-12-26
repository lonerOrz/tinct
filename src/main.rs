use std::env;
use std::fs;
use std::path::Path;
use std::process;

mod cli;
mod color;
mod config;
mod log;
mod preview;
mod theme;

use clap::Parser;
use colored::*;
use config::Config;

fn main() {
    let args = cli::CliArgs::parse();

    // Determine the config file path
    let config_path = if let Some(config_arg) = &args.config {
        // Use the config file specified in the command line argument
        config_arg.clone()
    } else {
        // Use the default config file in user's home directory
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        format!("{}/.config/tinct/config.toml", home_dir)
    };

    // Initialize global logger with the specified log level
    log::init_logger(match args.log_level {
        cli::LogLevel::Quiet => log::LogLevel::Quiet,
        cli::LogLevel::Normal => log::LogLevel::Normal,
        cli::LogLevel::Verbose => log::LogLevel::Verbose,
    });

    // Print basic info in a clean format
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

    // Resolve theme path - check both project themes and user themes in ~/.config/tinct/themes/
    let theme_file = resolve_theme_path(&args.theme);

    fn resolve_theme_path(theme_name: &str) -> String {
        use std::env;

        // First, check if the theme path is provided as an absolute path
        if Path::new(theme_name).is_absolute() && Path::new(theme_name).exists() {
            return theme_name.to_string();
        }

        // Check if it's a relative path that exists from current directory
        if Path::new(theme_name).exists() {
            return Path::new(theme_name)
                .canonicalize()
                .unwrap_or_else(|_| std::path::PathBuf::from(theme_name))
                .to_string_lossy()
                .to_string();
        }

        // Check in project's themes directory
        let project_themes_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("themes")
            .join(format!("{}.json", theme_name));
        if project_themes_path.exists() {
            return project_themes_path.to_string_lossy().to_string();
        }

        // Check in user's config directory ~/.config/tinct/themes/
        if let Ok(home_dir) = env::var("HOME") {
            let user_themes_path = Path::new(&home_dir)
                .join(".config")
                .join("tinct")
                .join("themes")
                .join(format!("{}.json", theme_name));
            if user_themes_path.exists() {
                return user_themes_path.to_string_lossy().to_string();
            }
        }

        // If theme is not found anywhere, exit with error
        eprintln!(
            "Theme '{}' not found in any of these locations:\n  - Current directory\n  - Project themes/ directory\n  - ~/.config/tinct/themes/",
            theme_name
        );
        process::exit(1);
    }

    // If preview flag is set, show color preview and exit (before trying to load config)
    if args.preview {
        match preview::show_color_preview(&theme_file, &args.mode.to_string()) {
            Ok(()) => process::exit(0),
            Err(e) => {
                eprintln!("Error showing color preview: {}", e);
                process::exit(1);
            }
        }
    }

    // Check if the config file exists
    if !Path::new(&config_path).exists() {
        eprintln!("Config file '{}' does not exist.", config_path);
        process::exit(1);
    }

    // Read TOML config
    let config_content = fs::read_to_string(&config_path).expect("Could not read config file");

    let mut config: Config =
        toml::from_str(&config_content).expect("Invalid TOML format in config file");

    // Convert relative paths in config to absolute paths
    // Paths should be resolved relative to the config file location, not the project root
    let config_dir = Path::new(&config_path)
        .parent()
        .unwrap_or(Path::new(""))
        .to_string_lossy()
        .to_string();

    for (_group_name, group) in config.iter_mut() {
        for (_section_name, section) in group.iter_mut() {
            // Resolve input_path
            let expanded_input_path = shellexpand::tilde(&section.input_path).to_string();
            section.input_path = if Path::new(&expanded_input_path).is_absolute() {
                expanded_input_path
            } else {
                // If it's a relative path, resolve it relative to config file location
                Path::new(&config_dir)
                    .join(&expanded_input_path)
                    .canonicalize()
                    .unwrap_or_else(|_| Path::new(&config_dir).join(&expanded_input_path))
                    .to_string_lossy()
                    .to_string()
            };

            // Resolve output_path
            let expanded_output_path = shellexpand::tilde(&section.output_path).to_string();
            section.output_path = if Path::new(&expanded_output_path).is_absolute() {
                expanded_output_path
            } else {
                // If it's a relative path, resolve it relative to config file location
                Path::new(&config_dir)
                    .join(&expanded_output_path)
                    .canonicalize()
                    .unwrap_or_else(|_| Path::new(&config_dir).join(&expanded_output_path))
                    .to_string_lossy()
                    .to_string()
            };

            // Resolve post_hook if it exists - only for relative file paths starting with ./
            if let Some(ref mut post_hook) = section.post_hook {
                if post_hook.starts_with("./") {
                    // If it's a relative file path (starts with ./), resolve it relative to config file location
                    let expanded_post_hook = shellexpand::tilde(post_hook).to_string();
                    *post_hook = Path::new(&config_dir)
                        .join(&expanded_post_hook)
                        .canonicalize()
                        .unwrap_or_else(|_| Path::new(&config_dir).join(&expanded_post_hook))
                        .to_string_lossy()
                        .to_string();
                }
                // For other cases (absolute paths or shell commands), leave unchanged
            }
        }
    }

    // Process each section in the config
    let mut success_count = 0;
    let mut total_count = 0;

    let mode_str = args.mode.to_string();
    for (group_name, group) in config.iter() {
        if matches!(args.log_level, cli::LogLevel::Verbose) {
            println!("Processing group: {}", group_name);
        }
        for (section_name, section) in group.iter() {
            total_count += 1;

            if !cli::validate_config_section(section, section_name) {
                continue;
            }

            let result = cli::process_section(
                section_name,
                section,
                &theme_file,
                &mode_str,
                args.log_level.clone(),
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
