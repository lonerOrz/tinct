use clap::Parser;
use std::env;
use std::fs;
use std::path::Path;
use std::process;

mod color;
mod config;
mod theme;

use config::{Config, ConfigSection};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the TOML config file
    #[arg(long, default_value = "config.toml")]
    config: String,

    /// Path to theme.json file or theme name in themes/ folder
    #[arg(long, required = true)]
    theme: String,

    /// Theme mode override
    #[arg(long, value_enum, default_value = "dark")]
    mode: ThemeMode,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum ThemeMode {
    Dark,
    Light,
}

impl std::fmt::Display for ThemeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeMode::Dark => write!(f, "dark"),
            ThemeMode::Light => write!(f, "light"),
        }
    }
}

fn resolve_path(
    path: Option<&str>,
    default_file: Option<&str>,
    subfolder: Option<&str>,
) -> Option<String> {
    let script_dir = Path::new(env!("CARGO_MANIFEST_DIR")).to_str().unwrap();

    // Handle default file
    if path.is_none() {
        if let Some(default) = default_file {
            return Some(
                Path::new(script_dir)
                    .join(default)
                    .to_string_lossy()
                    .to_string(),
            );
        } else {
            return None;
        }
    }

    let path_str = path.unwrap();

    // Check if path contains separators, if not, look in subfolder
    if let Some(subfolder_name) = subfolder {
        if !path_str.contains('/') && !path_str.contains('\\') {
            let full_path = Path::new(script_dir)
                .join(subfolder_name)
                .join(format!("{}.json", path_str));

            if full_path.exists() {
                return Some(full_path.to_string_lossy().to_string());
            }
        }
    }

    // Check if path exists as provided
    let expanded_path = shellexpand::tilde(path_str).to_string();
    if Path::new(&expanded_path).exists() {
        return Some(
            Path::new(&expanded_path)
                .canonicalize()
                .ok()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or(expanded_path),
        );
    }

    // Try as relative to script directory
    let relative_path = Path::new(script_dir).join(path_str);
    if relative_path.exists() {
        return Some(
            relative_path
                .canonicalize()
                .ok()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or(relative_path.to_string_lossy().to_string()),
        );
    }

    None
}

fn run_post_hook(post_hook: &str, output_file: &str, section_name: Option<&str>) -> bool {
    if post_hook.is_empty() {
        return true;
    }

    let post_hook_cmd = post_hook.replace("{{output_file}}", output_file);

    // Check if it's a script starting with ./
    if post_hook_cmd.starts_with("./") {
        // Handle relative script path
        let script_dir = Path::new(env!("CARGO_MANIFEST_DIR")).to_str().unwrap();
        let post_hook_path = Path::new(script_dir).join(&post_hook_cmd);

        if post_hook_path.exists() && is_executable(&post_hook_path) {
            if let Some(name) = section_name {
                println!("[{}] ✓ Hook script executing...", name);
            }

            match std::process::Command::new(&post_hook_path).output() {
                Ok(result) => {
                    if result.status.success() {
                        if let Some(name) = section_name {
                            println!("[{}] ✓ Hook script executed successfully", name);
                        }
                        true
                    } else {
                        if let Some(name) = section_name {
                            eprintln!("[{}] ✗ Error executing hook script", name);
                        }
                        false
                    }
                }
                Err(e) => {
                    if let Some(name) = section_name {
                        eprintln!("[{}] ✗ Error executing hook script: {}", name, e);
                    }
                    false
                }
            }
        } else {
            if let Some(name) = section_name {
                eprintln!(
                    "[{}] ⚠ post_hook '{}' not found. Skipping.",
                    name,
                    post_hook_path.display()
                );
            }
            false
        }
    } else {
        // Handle command execution
        if let Some(name) = section_name {
            println!("[{}] ✓ Hook command executing...", name);
        }

        match std::process::Command::new("sh")
            .arg("-c")
            .arg(&post_hook_cmd)
            .output()
        {
            Ok(result) => {
                if result.status.success() {
                    if let Some(name) = section_name {
                        println!("[{}] ✓ Hook command executed successfully", name);
                    }
                    true
                } else {
                    if let Some(name) = section_name {
                        eprintln!(
                            "[{}] ✗ Error executing hook command: {}",
                            name,
                            String::from_utf8_lossy(&result.stderr)
                        );
                    }
                    false
                }
            }
            Err(e) => {
                if let Some(name) = section_name {
                    eprintln!("[{}] ✗ Error executing hook command: {}", name, e);
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
    // On Windows, we assume files with certain extensions are executable
    true
}

fn validate_config_section(section: &ConfigSection, section_name: &str) -> bool {
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

fn process_section(
    section_name: &str,
    section: &ConfigSection,
    theme_file: &str,
    mode: &str,
) -> bool {
    let input_path = &section.input_path;
    let output_path = &section.output_path;
    let post_hook = section.post_hook.as_deref().unwrap_or("");

    // Validate input file exists
    if !Path::new(input_path).exists() {
        eprintln!(
            "[{}] Input file '{}' does not exist. Skipping.",
            section_name, input_path
        );
        return false;
    }

    // Ensure output directory exists
    if let Some(parent) = Path::new(output_path).parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!(
                "[{}] Error creating output directory: {}. Skipping.",
                section_name, e
            );
            return false;
        }
    }

    // Process the theme
    match theme::process_theme(theme_file, input_path, output_path, mode) {
        Ok(()) => {
            // Run post hook if specified
            if !post_hook.is_empty() {
                run_post_hook(post_hook, output_path, Some(section_name));
            }
            true
        }
        Err(e) => {
            eprintln!("[{}] Error processing theme: {}", section_name, e);
            false
        }
    }
}

fn main() {
    let args = Args::parse();

    // Resolve config path
    let config_path = resolve_path(Some(&args.config), Some("config.toml"), None)
        .expect("Config file path could not be resolved");

    if !Path::new(&config_path).exists() {
        eprintln!("Config file '{}' does not exist.", args.config);
        process::exit(1);
    }

    // Resolve theme path
    let theme_path = resolve_path(Some(&args.theme), None, Some("themes"));
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

    println!("Config: {}", config_path);
    println!("Theme: {}", theme_file);
    println!("Mode: {}", args.mode);

    // Read TOML config
    let config_content = fs::read_to_string(&config_path).expect("Could not read config file");

    let config: Config =
        toml::from_str(&config_content).expect("Invalid TOML format in config file");

    println!("Configuration loaded from {}", config_path);

    // Convert relative paths in config to absolute paths
    let script_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .to_string_lossy()
        .to_string();
    let mut processed_config = config;

    for (_group_name, group) in processed_config.iter_mut() {
        for (_section_name, section) in group.iter_mut() {
            if let Some(abs_path) = config::resolve_path_to_abs(&section.input_path, &script_dir) {
                section.input_path = abs_path;
            }

            if let Some(abs_path) = config::resolve_path_to_abs(&section.output_path, &script_dir) {
                section.output_path = abs_path;
            }

            if let Some(ref post_hook) = section.post_hook {
                if post_hook.starts_with("./") {
                    if let Some(abs_path) = config::resolve_path_to_abs(post_hook, &script_dir) {
                        // Update the post_hook in the config
                        section.post_hook = Some(abs_path);
                    }
                }
            }
        }
    }

    // Process each section in the config
    let mode_str = args.mode.to_string();
    for (group_name, group) in processed_config.iter() {
        println!("Processing group: {}", group_name);
        for (section_name, section) in group.iter() {
            println!("Processing section: {}", section_name);

            if !validate_config_section(section, section_name) {
                continue;
            }

            process_section(section_name, section, &theme_file, &mode_str);
        }
    }
}
