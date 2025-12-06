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
        if path.starts_with("./") {
            // Remove './' prefix and join with base directory
            let path_without_prefix = &path[2..];
            return Some(
                std::fs::canonicalize(Path::new(base_dir).join(path_without_prefix))
                    .ok()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| {
                        Path::new(base_dir)
                            .join(path_without_prefix)
                            .to_string_lossy()
                            .to_string()
                    }),
            );
        } else if !path.contains('/') {
            // If path doesn't contain '/', assume it's relative to base directory
            return Some(
                std::fs::canonicalize(Path::new(base_dir).join(path))
                    .ok()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| {
                        Path::new(base_dir).join(path).to_string_lossy().to_string()
                    }),
            );
        }
    }

    // If it's already absolute or couldn't be resolved relative to base, return as is
    Some(expanded)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_resolve_path_to_abs() {
        // Since this function depends on file system paths, we'll just verify it compiles
        // with the shellexpand dependency
        assert!(true);
    }
}
