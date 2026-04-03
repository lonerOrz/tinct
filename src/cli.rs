use clap::Parser;
use std::env;
use std::fs;
use std::path::Path;

use crate::config::ConfigSection;
use tinct::image::SchemeType;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    /// Path to the TOML config file
    #[arg(short, long)]
    pub config: Option<String>,

    /// Path to theme.json file or theme name in themes/ folder
    #[arg(short, long)]
    pub theme: Option<String>,

    /// Seed color for generating palette (e.g., "#7aa2f7")
    #[arg(short, long)]
    pub seed: Option<String>,

    /// Path to wallpaper image for color extraction (PNG/JPG/WebP)
    #[arg(long)]
    pub image: Option<String>,

    /// Color scheme type for image extraction
    #[arg(long, value_name = "SCHEME", default_value = "tonal-spot")]
    pub scheme_type: SchemeTypeCli,

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

/// CLI wrapper for SchemeType with clap integration.
#[derive(Clone, Debug, PartialEq)]
pub struct SchemeTypeCli(pub SchemeType);

impl clap::ValueEnum for SchemeTypeCli {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self(SchemeType::TonalSpot),
            Self(SchemeType::Content),
            Self(SchemeType::FruitSalad),
            Self(SchemeType::Rainbow),
            Self(SchemeType::Monochrome),
            Self(SchemeType::Vibrant),
            Self(SchemeType::Faithful),
            Self(SchemeType::Dysfunctional),
            Self(SchemeType::Muted),
        ]
    }

    fn to_possible_value<'a>(&self) -> Option<clap::builder::PossibleValue> {
        let name = match self.0 {
            SchemeType::TonalSpot => "tonal-spot",
            SchemeType::Content => "content",
            SchemeType::FruitSalad => "fruit-salad",
            SchemeType::Rainbow => "rainbow",
            SchemeType::Monochrome => "monochrome",
            SchemeType::Vibrant => "vibrant",
            SchemeType::Faithful => "faithful",
            SchemeType::Dysfunctional => "dysfunctional",
            SchemeType::Muted => "muted",
        };
        Some(clap::builder::PossibleValue::new(name))
    }
}

impl std::str::FromStr for SchemeTypeCli {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SchemeType::parse(s)
            .map(SchemeTypeCli)
            .ok_or_else(|| format!("Invalid scheme type: {}", s))
    }
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

impl CliArgs {
    /// Validate that either --theme, --seed, or --image is provided
    pub fn validate(&self) -> Result<(), String> {
        let has_theme = self.theme.is_some();
        let has_seed = self.seed.is_some();
        let has_image = self.image.is_some();

        let count = [has_theme, has_seed, has_image]
            .iter()
            .filter(|&&x| x)
            .count();

        if count == 0 {
            return Err("Either --theme, --seed, or --image must be provided".to_string());
        }
        if count > 1 {
            return Err(
                "--theme, --seed, and --image are mutually exclusive, use only one".to_string(),
            );
        }
        Ok(())
    }
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
            theme: Some("mytheme".to_string()),
            seed: None,
            image: None,
            scheme_type: SchemeTypeCli(SchemeType::TonalSpot),
            mode: ThemeMode::Light,
            preview: true,
            skip_sequences: false,
            log_level: LogLevel::Verbose,
        };
        assert_eq!(args.theme, Some("mytheme".to_string()));
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
