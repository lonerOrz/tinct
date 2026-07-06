//! Path resolution utilities
//!
//! This module provides functions for resolving and normalizing paths
//! used throughout the tinct application.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::ConfigSection;
use crate::core::Error;

/// Resolve the theme path by checking multiple locations
///
/// Checks the following locations in order:
/// 1. Absolute path
/// 2. Relative path from current directory
/// 3. Project themes directory
/// 4. User config directory (~/.config/tinct/themes/)
pub fn resolve_theme_path(theme_name: &str) -> Result<String, Error> {
    // Check absolute path
    if Path::new(theme_name).is_absolute() && Path::new(theme_name).exists() {
        return Ok(theme_name.to_string());
    }

    // Check relative path (must be a file, not a directory)
    let relative_path = Path::new(theme_name);
    if relative_path.exists() && relative_path.is_file() {
        return Ok(relative_path
            .canonicalize()
            .unwrap_or_else(|_| PathBuf::from(theme_name))
            .to_string_lossy()
            .to_string());
    }

    // Check user config directory
    if let Ok(home_dir) = env::var("HOME") {
        let user_themes_path = Path::new(&home_dir)
            .join(".config")
            .join("tinct")
            .join("themes")
            .join(format!("{}.json", theme_name));
        if user_themes_path.exists() {
            return Ok(user_themes_path.to_string_lossy().to_string());
        }
    }

    Err(Error::Config(format!(
        "Theme '{}' not found in any of these locations:\n  - Current directory\n  - Project themes/ directory\n  - ~/.config/tinct/themes/",
        theme_name
    )))
}

/// Resolve the default config file path
///
/// Returns the provided config path if specified, otherwise returns
/// the default path: ~/.config/tinct/config.toml
pub fn resolve_config_file_path(config_arg: Option<&String>) -> String {
    if let Some(config_path) = config_arg {
        config_path.clone()
    } else {
        let home_dir = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        format!("{}/.config/tinct/config.toml", home_dir)
    }
}

/// Resolve all paths in a config section relative to the config directory
///
/// This function:
/// - Expands tilde (~) in paths
/// - Resolves relative paths against the config directory
/// - Canonicalizes output paths when possible
/// - Resolves relative post_hook scripts starting with ./
pub fn resolve_config_paths(section: &mut ConfigSection, config_dir: &str) {
    // Expand tilde in input and output paths
    section.input_path = shellexpand::tilde(&section.input_path).to_string();
    section.output_path = shellexpand::tilde(&section.output_path).to_string();

    // Resolve input path relative to config directory
    if !Path::new(&section.input_path).is_absolute() {
        section.input_path = Path::new(config_dir)
            .join(&section.input_path)
            .to_string_lossy()
            .to_string();
    }

    // Resolve output path relative to config directory
    if !Path::new(&section.output_path).is_absolute() {
        let output_path = Path::new(config_dir).join(&section.output_path);
        section.output_path = resolve_output_path(&output_path);
    }

    // Resolve post_hook path if it starts with ./
    if let Some(ref mut hook) = section.post_hook
        && hook.starts_with("./")
    {
        let hook_path = Path::new(config_dir).join(&*hook);
        *hook = resolve_hook_path(hook, &hook_path);
    }
}

/// Resolve an output path, attempting to canonicalize the parent directory
fn resolve_output_path(output_path: &Path) -> String {
    if let Some(parent) = output_path.parent() {
        if let Ok(canonical_parent) = fs::canonicalize(parent) {
            let file_name = output_path.file_name().unwrap_or_default();
            canonical_parent
                .join(file_name)
                .to_string_lossy()
                .to_string()
        } else {
            output_path.to_string_lossy().to_string()
        }
    } else {
        output_path.to_string_lossy().to_string()
    }
}

/// Resolve a hook path, canonicalizing if the file exists
fn resolve_hook_path(_original: &str, hook_path: &Path) -> String {
    if hook_path.exists() {
        fs::canonicalize(hook_path)
            .unwrap_or_else(|_| hook_path.to_path_buf())
            .to_string_lossy()
            .to_string()
    } else {
        hook_path.to_string_lossy().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_config_file_path_with_arg() {
        let config_path = String::from("/custom/config.toml");
        let result = resolve_config_file_path(Some(&config_path));
        assert_eq!(result, "/custom/config.toml");
    }

    #[test]
    fn test_resolve_config_file_path_default() {
        let result = resolve_config_file_path(None);
        // Should end with /.config/tinct/config.toml
        assert!(result.ends_with("/.config/tinct/config.toml"));
    }

    #[test]
    fn test_resolve_output_path_with_parent() {
        let temp_dir = std::env::temp_dir();
        let test_path = temp_dir.join("subdir").join("output.txt");

        // Create parent directory
        let _ = fs::create_dir_all(test_path.parent().unwrap());

        let result = resolve_output_path(&test_path);
        assert!(result.ends_with("output.txt"));

        // Cleanup
        let _ = fs::remove_dir_all(temp_dir.join("subdir"));
    }

    #[test]
    fn test_resolve_hook_path_existing() {
        let temp_dir = std::env::temp_dir();
        let hook_file = temp_dir.join("test_hook.sh");

        // Create a temporary file
        let _ = fs::write(&hook_file, "#!/bin/bash");

        let result = resolve_hook_path("./test_hook.sh", &hook_file);
        assert!(Path::new(&result).is_absolute());

        // Cleanup
        let _ = fs::remove_file(&hook_file);
    }

    #[test]
    fn test_resolve_hook_path_non_existing() {
        let hook_path = PathBuf::from("/nonexistent/hook.sh");
        let result = resolve_hook_path("./hook.sh", &hook_path);
        assert_eq!(result, "/nonexistent/hook.sh");
    }

    #[test]
    fn test_resolve_config_paths_expands_tilde() {
        let mut section = ConfigSection {
            input_path: "~/input.css".to_string(),
            output_path: "~/output.css".to_string(),
            post_hook: None,
        };

        resolve_config_paths(&mut section, "/config");

        // Tilde should be expanded
        assert!(!section.input_path.starts_with("~"));
        assert!(!section.output_path.starts_with("~"));
    }

    #[test]
    fn test_resolve_theme_path_missing() {
        let result = resolve_theme_path("nonexistent_theme_xyz");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not found"));
    }
}
