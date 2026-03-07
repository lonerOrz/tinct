use clap::Parser;
use std::env;
use std::fs;
use std::path::Path;

use crate::config::ConfigSection;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    /// Path to the TOML config file
    #[arg(short, long)]
    pub config: Option<String>,

    /// Path to theme.json file or theme name in themes/ folder
    #[arg(short, long, required = true)]
    pub theme: String,

    /// Theme mode override
    #[arg(short, long, value_enum, default_value = "dark")]
    pub mode: ThemeMode,

    /// Show color preview instead of processing templates
    #[arg(short, long)]
    pub preview: bool,

    /// Skip sending ANSI escape sequences to update terminal colors
    #[arg(long)]
    pub skip_sequences: bool,

    /// Logging level: quiet, normal, verbose
    #[arg(long, value_enum, default_value = "normal")]
    pub log_level: LogLevel,
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
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

pub fn run_post_hook(
    post_hook: &str,
    output_file: &str,
    section_name: Option<&str>,
    _log_level: LogLevel,
) -> bool {
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
                crate::log::hook::executing(name);
            }

            match std::process::Command::new(&post_hook_path).output() {
                Ok(result) => {
                    if result.status.success() {
                        if let Some(name) = section_name {
                            crate::log::hook::success(name);
                        }
                        true
                    } else {
                        if let Some(name) = section_name {
                            crate::log::error::message(name, "Error executing hook script");
                        }
                        false
                    }
                }
                Err(e) => {
                    if let Some(name) = section_name {
                        crate::log::error::message(
                            name,
                            &format!("Error executing hook script: {}", e),
                        );
                    }
                    false
                }
            }
        } else {
            if let Some(name) = section_name {
                crate::log::error::message(
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
        // Handle command execution
        if let Some(name) = section_name {
            crate::log::hook::executing(name);
        }

        match std::process::Command::new("sh")
            .arg("-c")
            .arg(&post_hook_cmd)
            .output()
        {
            Ok(result) => {
                if result.status.success() {
                    if let Some(name) = section_name {
                        crate::log::hook::success(name);
                    }
                    true
                } else {
                    if let Some(name) = section_name {
                        crate::log::error::hook_error(
                            name,
                            String::from_utf8_lossy(&result.stderr).as_ref(),
                        );
                    }
                    false
                }
            }
            Err(e) => {
                if let Some(name) = section_name {
                    crate::log::error::hook_error(name, &e.to_string());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConfigSection;

    #[test]
    fn test_validate_config_section_valid() {
        let section = ConfigSection {
            input_path: "input.css".to_string(),
            output_path: "output.css".to_string(),
            post_hook: None,
        };
        assert!(validate_config_section(&section, "test"));
    }

    #[test]
    fn test_validate_config_section_missing_input() {
        let section = ConfigSection {
            input_path: "".to_string(),
            output_path: "output.css".to_string(),
            post_hook: None,
        };
        assert!(!validate_config_section(&section, "test"));
    }

    #[test]
    fn test_validate_config_section_missing_output() {
        let section = ConfigSection {
            input_path: "input.css".to_string(),
            output_path: "".to_string(),
            post_hook: None,
        };
        assert!(!validate_config_section(&section, "test"));
    }

    #[test]
    fn test_validate_config_section_both_missing() {
        let section = ConfigSection {
            input_path: "".to_string(),
            output_path: "".to_string(),
            post_hook: None,
        };
        assert!(!validate_config_section(&section, "test"));
    }

    #[test]
    fn test_theme_mode_display() {
        assert_eq!(ThemeMode::Dark.to_string(), "dark");
        assert_eq!(ThemeMode::Light.to_string(), "light");
    }

    #[test]
    fn test_log_level_variants() {
        // Verify all log levels exist
        let _ = LogLevel::Quiet;
        let _ = LogLevel::Normal;
        let _ = LogLevel::Verbose;
    }

    #[test]
    fn test_cli_args_derive() {
        // Verify CliArgs can be created with derive
        let args = CliArgs {
            config: Some("custom.toml".to_string()),
            theme: "mytheme".to_string(),
            mode: ThemeMode::Light,
            preview: true,
            skip_sequences: false,
            log_level: LogLevel::Verbose,
        };
        assert_eq!(args.theme, "mytheme");
        assert_eq!(args.mode, ThemeMode::Light);
        assert!(args.preview);
    }

    #[test]
    fn test_run_post_hook_empty() {
        assert!(run_post_hook("", "output.css", None, LogLevel::Quiet));
    }

    #[test]
    fn test_is_executable_function_exists() {
        // Test that the platform-specific is_executable function exists
        let path = Path::new("/nonexistent/path");
        #[cfg(unix)]
        assert!(!is_executable(path));
        #[cfg(not(unix))]
        assert!(is_executable(path));
    }
}
