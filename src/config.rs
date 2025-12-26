use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigSection {
    pub input_path: String,
    pub output_path: String,
    #[serde(rename = "post_hook", default)]
    pub post_hook: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(dead_code)]
pub struct ConfigGroup {
    pub sections: HashMap<String, ConfigSection>,
}

// A representation of the entire config structure as a nested HashMap
pub type Config = HashMap<String, HashMap<String, ConfigSection>>;

pub fn resolve_path_to_abs(path: &str, base_dir: &str) -> Option<String> {
    if path.is_empty() {
        return None;
    }

    let expanded = shellexpand::tilde(path).to_string();

    if !Path::new(&expanded).is_absolute() {
        // Always treat relative paths as relative to the base directory
        // This handles paths like "templates/file.json", "./file.json", etc.
        return Some(
            std::fs::canonicalize(Path::new(base_dir).join(&expanded))
                .ok()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| {
                    Path::new(base_dir)
                        .join(&expanded)
                        .to_string_lossy()
                        .to_string()
                }),
        );
    }

    // If it's already absolute, return as is
    Some(expanded)
}
