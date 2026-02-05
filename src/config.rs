use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct AlgorithmConfig {
    /// Color contrast threshold (0.0-1.0)
    #[serde(default = "default_contrast_threshold")]
    pub contrast_threshold: f64,

    /// Saturation adjustment (-100 to 100)
    #[serde(default = "default_saturation_adjustment")]
    pub saturation_adjustment: i8,

    /// Lightness adjustment (-100 to 100)
    #[serde(default = "default_lightness_adjustment")]
    pub lightness_adjustment: i8,

    /// Hue shift (-180 to 180)
    #[serde(default = "default_hue_shift")]
    pub hue_shift: i16,

    /// Minimum contrast ratio for readability
    #[serde(default = "default_min_contrast_ratio")]
    pub min_contrast_ratio: f64,
}

fn default_contrast_threshold() -> f64 {
    0.15
}

fn default_saturation_adjustment() -> i8 {
    0
}

fn default_lightness_adjustment() -> i8 {
    0
}

fn default_hue_shift() -> i16 {
    0
}

fn default_min_contrast_ratio() -> f64 {
    4.5
}

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

// Use a completely different approach - parse algorithm and templates separately
#[derive(Debug)]
pub struct ConfigRoot {
    pub algorithm: AlgorithmConfig,
    pub groups: HashMap<String, HashMap<String, ConfigSection>>,
}

impl ConfigRoot {
    pub fn parse(config_content: &str) -> Result<Self, String> {
        use toml::Value;

        let value: Value =
            toml::from_str(config_content).map_err(|e| format!("Invalid TOML format: {}", e))?;

        // Extract algorithm config if present
        // Use serde defaults by deserializing, fallback to hardcoded defaults
        let mut algorithm = AlgorithmConfig {
            contrast_threshold: default_contrast_threshold(),
            saturation_adjustment: default_saturation_adjustment(),
            lightness_adjustment: default_lightness_adjustment(),
            hue_shift: default_hue_shift(),
            min_contrast_ratio: default_min_contrast_ratio(),
        };
        if let Some(table) = value.get("algorithm").and_then(|v| v.as_table()) {
            if let Some(v) = table.get("contrast_threshold").and_then(|v| v.as_float()) {
                algorithm.contrast_threshold = v;
            }
            if let Some(v) = table
                .get("saturation_adjustment")
                .and_then(|v| v.as_integer())
            {
                algorithm.saturation_adjustment = v as i8;
            }
            if let Some(v) = table
                .get("lightness_adjustment")
                .and_then(|v| v.as_integer())
            {
                algorithm.lightness_adjustment = v as i8;
            }
            if let Some(v) = table.get("hue_shift").and_then(|v| v.as_integer()) {
                algorithm.hue_shift = v as i16;
            }
            if let Some(v) = table.get("min_contrast_ratio").and_then(|v| v.as_float()) {
                algorithm.min_contrast_ratio = v;
            }
        }

        // Extract template groups - process tables that contain template sections
        let mut groups = HashMap::new();
        if let Value::Table(table) = &value {
            for (group_key, group_value) in table {
                // Skip the algorithm section
                if group_key != "algorithm" {
                    // Process if this is a table containing template sections
                    if let Value::Table(sub_table) = group_value {
                        let mut section_map = HashMap::new();

                        // Process each template section within this group
                        for (section_name, section_value) in sub_table {
                            // Only try to parse as ConfigSection if it has the required fields
                            if let Value::Table(section_fields) = section_value {
                                // Check if this table has the required fields for a ConfigSection
                                if section_fields.contains_key("input_path")
                                    && section_fields.contains_key("output_path")
                                {
                                    // Convert the table to a proper TOML string representation for parsing
                                    let toml_doc = toml::to_string(&section_fields)
                                        .unwrap_or_else(|_| String::from(""));

                                    if let Ok(config_section) =
                                        toml::from_str::<ConfigSection>(&toml_doc)
                                    {
                                        section_map.insert(section_name.clone(), config_section);
                                    }
                                }
                            }
                        }

                        if !section_map.is_empty() {
                            groups.insert(group_key.clone(), section_map);
                        }
                    }
                }
            }
        }

        // No debug output in release version - only for development

        Ok(ConfigRoot { algorithm, groups })
    }
}

// A representation of the entire config structure as a nested HashMap
pub type Config = HashMap<String, HashMap<String, ConfigSection>>;

impl ConfigRoot {
    pub fn to_flat_config(self) -> Config {
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
contrast_threshold = 0.2
saturation_adjustment = 10
lightness_adjustment = -5
hue_shift = 15
min_contrast_ratio = 4.0

[templates.test]
input_path = "input.css"
output_path = "output.css"
"#;
        let result = ConfigRoot::parse(config_content);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.algorithm.contrast_threshold, 0.2);
        assert_eq!(config.algorithm.saturation_adjustment, 10);
        assert_eq!(config.algorithm.lightness_adjustment, -5);
        assert_eq!(config.algorithm.hue_shift, 15);
        assert_eq!(config.algorithm.min_contrast_ratio, 4.0);
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
        assert!((config.algorithm.contrast_threshold - 0.15).abs() < 0.001);
        assert_eq!(config.algorithm.saturation_adjustment, 0);
        assert_eq!(config.algorithm.lightness_adjustment, 0);
        assert_eq!(config.algorithm.hue_shift, 0);
        assert!((config.algorithm.min_contrast_ratio - 4.5).abs() < 0.001);
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
        let flat = config_root.to_flat_config();
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
