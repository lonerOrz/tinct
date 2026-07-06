use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use toml::Value;

/// Image extraction configuration
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ImageConfig {
    /// Scheme type for color extraction (tonal-spot, vibrant, faithful, etc.)
    #[serde(default)]
    pub scheme_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct AlgorithmConfig {
    /// Saturation adjustment (-100 to 100)
    #[serde(default = "default_saturation_adjustment")]
    pub saturation_adjustment: i8,

    /// Hue shift (-180 to 180)
    #[serde(default = "default_hue_shift")]
    pub hue_shift: i16,

    /// MD3 contrast level (-1.0 to 1.0)
    #[serde(default = "default_contrast_level")]
    pub contrast_level: f64,

    /// Color harmony mode (md3, analogous, complementary, triadic, split-complementary)
    #[serde(default = "default_color_harmony")]
    pub color_harmony: String,
}

fn default_saturation_adjustment() -> i8 {
    0
}

fn default_hue_shift() -> i16 {
    0
}

fn default_contrast_level() -> f64 {
    0.0
}

fn default_color_harmony() -> String {
    "md3".to_string()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigSection {
    pub input_path: String,
    pub output_path: String,
    #[serde(rename = "post_hook", default)]
    pub post_hook: Option<String>,
}

/// Raw config with serde flatten — algorithm/image are deserialized by serde,
/// everything else (template groups) is captured as `toml::Value` for post-processing.
#[derive(Debug, Clone, Deserialize)]
struct ConfigRootRaw {
    #[serde(default)]
    algorithm: AlgorithmConfig,
    #[serde(default)]
    image: ImageConfig,
    #[serde(flatten)]
    other: HashMap<String, toml::Value>,
}

#[derive(Debug, Clone)]
pub struct ConfigRoot {
    pub algorithm: AlgorithmConfig,
    pub image: ImageConfig,
    pub groups: HashMap<String, HashMap<String, ConfigSection>>,
}

impl ConfigRoot {
    pub fn parse(config_content: &str) -> Result<Self, String> {
        let raw: ConfigRootRaw =
            toml::from_str(config_content).map_err(|e| format!("Invalid TOML format: {}", e))?;

        // Post-process flattened entries into template groups
        let mut groups: HashMap<String, HashMap<String, ConfigSection>> = HashMap::new();
        for (group_key, val) in raw.other {
            if let Value::Table(table) = val {
                let mut sections = HashMap::new();
                for (section_name, fields) in table {
                    if let Value::Table(field_map) = &fields {
                        // Silently skip sections without required fields
                        if field_map.contains_key("input_path")
                            && field_map.contains_key("output_path")
                        {
                            let s = toml::from_str::<ConfigSection>(
                                &toml::to_string(&fields).unwrap_or_default(),
                            );
                            if let Ok(s) = s {
                                sections.insert(section_name, s);
                            }
                        }
                    }
                }
                if !sections.is_empty() {
                    groups.insert(group_key, sections);
                }
            }
        }

        Ok(ConfigRoot {
            algorithm: raw.algorithm,
            image: raw.image,
            groups,
        })
    }
}

// A representation of the entire config structure as a nested HashMap
pub type Config = HashMap<String, HashMap<String, ConfigSection>>;

impl ConfigRoot {
    pub fn into_flat_config(self) -> Config {
        self.groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parse_minimal() {
        let config_content = r#"
[templates.test]
input_path = "input.css"
output_path = "output.css"
"#;
        let result = ConfigRoot::parse(config_content);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(config.groups.contains_key("templates"));
    }

    #[test]
    fn test_config_parse_with_algorithm() {
        let config_content = r#"
[algorithm]
saturation_adjustment = 10
hue_shift = 15

[templates.test]
input_path = "input.css"
output_path = "output.css"
"#;
        let result = ConfigRoot::parse(config_content);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.algorithm.saturation_adjustment, 10);
        assert_eq!(config.algorithm.hue_shift, 15);
    }

    #[test]
    fn test_config_parse_multiple_sections() {
        let config_content = r#"
[section1.test]
input_path = "input1.css"
output_path = "output1.css"

[section2.production]
input_path = "input2.css"
output_path = "output2.css"
post_hook = "./script.sh"
"#;
        let result = ConfigRoot::parse(config_content);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(config.groups.contains_key("section1"));
        assert!(config.groups.contains_key("section2"));
    }

    #[test]
    fn test_algorithm_config_defaults() {
        let config_content = r#"
[algorithm]
"#;
        let result = ConfigRoot::parse(config_content);
        assert!(result.is_ok());
        let config = result.unwrap();
        // Check default values
        assert_eq!(config.algorithm.saturation_adjustment, 0);
        assert_eq!(config.algorithm.hue_shift, 0);
    }

    #[test]
    fn test_config_parse_missing_required_fields() {
        let config_content = r#"
[incomplete.test]
some_field = "value"
"#;
        let result = ConfigRoot::parse(config_content);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(!config.groups.contains_key("incomplete"));
    }

    #[test]
    fn test_config_to_flat_config() {
        let config_content = r#"
[templates.test]
input_path = "input.css"
output_path = "output.css"
"#;
        let result = ConfigRoot::parse(config_content);
        assert!(result.is_ok());
        let config_root = result.unwrap();
        let flat = config_root.into_flat_config();
        assert!(flat.contains_key("templates"));
    }

    #[test]
    fn test_config_parse_with_nested_groups() {
        let config_content = r#"
[frontend.templates]
input_path = "src/templates/*.hbs"
output_path = "dist/"

[backend.templates]
input_path = "server/**/*.tmpl"
output_path = "generated/"
"#;
        let result = ConfigRoot::parse(config_content);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(config.groups.contains_key("frontend"));
        assert!(config.groups.contains_key("backend"));
    }
}
