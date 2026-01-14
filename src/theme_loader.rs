use serde_json::Value;
use std::fs;

/// Load theme JSON file
pub fn load_theme(theme_path: &str) -> Result<Value, String> {
    if crate::log::is_verbose() {
        eprintln!("Loading theme from {}", theme_path);
    }

    let content = fs::read_to_string(theme_path)
        .map_err(|e| format!("Could not read theme file '{}': {}", theme_path, e))?;

    let theme_data: Value = serde_json::from_str(&content)
        .map_err(|e| format!("Invalid JSON format in '{}': {}", theme_path, e))?;

    if crate::log::is_verbose() {
        eprintln!("Theme loaded successfully from {}", theme_path);
    }
    Ok(theme_data)
}

/// Select theme mode, defaulting to dark if requested mode not found
pub fn select_theme_mode(theme_all: &Value, mode: &str) -> Result<(Value, String), String> {
    if let Some(theme_mode) = theme_all.get(mode) {
        Ok((theme_mode.clone(), mode.to_string()))
    } else {
        eprintln!("Mode '{}' not found in theme.json. Using 'dark'.", mode);
        if let Some(dark_mode) = theme_all.get("dark") {
            Ok((dark_mode.clone(), "dark".to_string()))
        } else {
            Err(
                "Error: 'dark' mode not available in theme.json and requested mode not found."
                    .to_string(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_load_theme() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let theme_path = temp_dir.path().join("test_theme.json");

        let theme_content = serde_json::json!({
            "dark": {
                "primary": "#FF5722",
                "secondary": "#607D8B"
            },
            "light": {
                "primary": "#E91E63",
                "secondary": "#00BCD4"
            }
        })
        .to_string();

        fs::write(&theme_path, theme_content).expect("Unable to write theme file");

        let result = load_theme(theme_path.to_str().unwrap());
        assert!(result.is_ok());

        let theme = result.unwrap();
        assert!(theme.get("dark").is_some());
        assert!(theme.get("light").is_some());
    }

    #[test]
    fn test_select_theme_mode() {
        let theme_content = serde_json::json!({
            "dark": {
                "primary": "#FF5722"
            },
            "light": {
                "primary": "#E91E63"
            }
        });

        // Test selecting dark mode
        let (theme, mode) = select_theme_mode(&theme_content, "dark").unwrap();
        assert_eq!(mode, "dark");
        assert_eq!(theme.get("primary").unwrap().as_str().unwrap(), "#FF5722");

        // Test selecting light mode
        let (theme, mode) = select_theme_mode(&theme_content, "light").unwrap();
        assert_eq!(mode, "light");
        assert_eq!(theme.get("primary").unwrap().as_str().unwrap(), "#E91E63");

        // Test fallback to dark when requested mode doesn't exist
        let (theme, mode) = select_theme_mode(&theme_content, "nonexistent").unwrap();
        assert_eq!(mode, "dark");
        assert_eq!(theme.get("primary").unwrap().as_str().unwrap(), "#FF5722");
    }
}
