use std::env;
use std::fs;
use std::path::Path;
use std::process;

mod cli;
mod config;
mod log;

use clap::Parser;
use colored::*;
use shellexpand;
use tinct::preview;

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

    // Read TOML config
    let config_content = fs::read_to_string(&config_path).expect("Could not read config file");

    // Parse the config with algorithm parameters using custom method
    let config_root = config::ConfigRoot::parse(&config_content)
        .expect("Invalid TOML format in config file");

    // Extract algorithm parameters
    let _alg_params = config_root.algorithm.clone();

    // Convert to flat config structure
    let mut config = config_root.to_flat_config();

    // Convert relative paths in config to absolute paths
    // Paths should be resolved relative to the config file location, not the project root
    let config_dir = Path::new(&config_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_string_lossy()
        .to_string();

    for (_group_name, group) in config.iter_mut() {
        for (_section_name, section) in group.iter_mut() {
            // Expand tilde in paths
            section.input_path = shellexpand::tilde(&section.input_path).to_string();
            section.output_path = shellexpand::tilde(&section.output_path).to_string();

            // Convert relative paths to absolute paths relative to config file location
            // For input_path: just join with config_dir, don't canonicalize (template might not exist yet)
            if !Path::new(&section.input_path).is_absolute() {
                section.input_path = Path::new(&config_dir)
                    .join(&section.input_path)
                    .to_string_lossy()
                    .to_string();
            }

            // For output_path: join with config_dir and try to canonicalize parent directory
            if !Path::new(&section.output_path).is_absolute() {
                let output_path = Path::new(&config_dir).join(&section.output_path);

                // Try to canonicalize the parent directory if it exists
                if let Some(parent) = output_path.parent() {
                    if let Ok(canonical_parent) = std::fs::canonicalize(parent) {
                        let file_name = output_path.file_name().unwrap_or_default();
                        section.output_path = canonical_parent.join(file_name).to_string_lossy().to_string();
                    } else {
                        // If canonicalize fails, just use the joined path
                        section.output_path = output_path.to_string_lossy().to_string();
                    }
                } else {
                    // If no parent exists, just use the joined path
                    section.output_path = output_path.to_string_lossy().to_string();
                }
            }

            // Also handle post_hook if it exists
            if let Some(ref mut hook) = section.post_hook {
                if hook.starts_with("./") {
                    // If the hook starts with ./, treat it as relative to config file location
                    let hook_clone = hook.clone(); // Clone to avoid move issues
                    let hook_path = Path::new(&config_dir).join(&hook_clone);

                    // Try to canonicalize if the hook file exists
                    if hook_path.exists() {
                        *hook = std::fs::canonicalize(&hook_path)
                            .unwrap_or_else(|_| hook_path)
                            .to_string_lossy()
                            .to_string();
                    } else {
                        // If the hook doesn't exist, just join with config_dir
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
        match preview::show_color_preview(&theme_file, &args.mode.to_string()) {
            Ok(()) => process::exit(0),
            Err(e) => {
                eprintln!("Error showing color preview: {}", e);
                process::exit(1);
            }
        }
    }

    // Process sections if not in preview mode
    if !args.preview {
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
                    args.skip_sequences,
                    _alg_params.clone(),  // Pass the algorithm parameters
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

/// Resolve theme path - check both project themes and user themes in ~/.config/tinct/themes/
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