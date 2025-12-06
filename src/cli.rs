use clap::Parser;
use std::env;
use std::fs;
use std::path::Path;

use crate::config::ConfigSection;
use crate::theme;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    /// Path to the TOML config file
    #[arg(long, default_value = "config.toml")]
    pub config: String,

    /// Path to theme.json file or theme name in themes/ folder
    #[arg(long, required = true)]
    pub theme: String,

    /// Theme mode override
    #[arg(long, value_enum, default_value = "dark")]
    pub mode: ThemeMode,

    /// Logging level: quiet, normal, verbose
    #[arg(long, value_enum, default_value = "normal")]
    pub log_level: LogLevel,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum ThemeMode {
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

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
pub enum LogLevel {
    Quiet,
    Normal,
    Verbose,
}

// Functions to resolve paths
pub fn resolve_path(
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

// Hook execution functions
pub fn run_post_hook(post_hook: &str, output_file: &str, section_name: Option<&str>, log_level: LogLevel) -> bool {
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
                if log_level == LogLevel::Verbose {
                    println!("[{}] ✓ Hook script executing...", name);
                }
            }

            match std::process::Command::new(&post_hook_path).output() {
                Ok(result) => {
                    if result.status.success() {
                        if let Some(name) = section_name {
                            if log_level != LogLevel::Quiet {
                                println!("[{}] ✓ Hook script executed successfully", name);
                            }
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
            if log_level == LogLevel::Verbose {
                println!("[{}] ✓ Hook command executing...", name);
            }
        }

        match std::process::Command::new("sh")
            .arg("-c")
            .arg(&post_hook_cmd)
            .output()
        {
            Ok(result) => {
                if result.status.success() {
                    if let Some(name) = section_name {
                        if log_level != LogLevel::Quiet {
                            println!("[{}] ✓ Hook command executed successfully", name);
                        }
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

// Configuration validation
pub fn validate_config_section(section: &ConfigSection, section_name: &str) -> bool {
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

// Section processing
pub fn process_section(
    section_name: &str,
    section: &ConfigSection,
    theme_file: &str,
    mode: &str,
    log_level: LogLevel,
) -> bool {
    let input_path = &section.input_path;
    let output_path = &section.output_path;
    let post_hook = section.post_hook.as_deref().unwrap_or("");

    // Validate input file exists
    if !Path::new(input_path).exists() {
        if log_level != LogLevel::Quiet {
            eprintln!(
                "[{}] Input file '{}' does not exist. Skipping.",
                section_name, input_path
            );
        }
        return false;
    }

    // Ensure output directory exists
    if let Some(parent) = Path::new(output_path).parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            if log_level != LogLevel::Quiet {
                eprintln!(
                    "[{}] Error creating output directory: {}. Skipping.",
                    section_name, e
                );
            }
            return false;
        }
    }

    // Process the theme
    match theme::process_theme(theme_file, input_path, output_path, mode) {
        Ok(()) => {
            // Run post hook if specified
            if !post_hook.is_empty() {
                run_post_hook(post_hook, output_path, Some(section_name), log_level);
            }
            true
        }
        Err(e) => {
            if log_level != LogLevel::Quiet {
                eprintln!("[{}] Error processing theme: {}", section_name, e);
            }
            false
        }
    }
}