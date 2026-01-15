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
        
        let value: Value = toml::from_str(config_content)
            .map_err(|e| format!("Invalid TOML format: {}", e))?;

        // Extract algorithm config if present
        let mut algorithm = AlgorithmConfig::default();
        if let Some(table) = value.get("algorithm").and_then(|v| v.as_table()) {
            if let Some(v) = table.get("contrast_threshold").and_then(|v| v.as_float()) {
                algorithm.contrast_threshold = v;
            }
            if let Some(v) = table.get("saturation_adjustment").and_then(|v| v.as_integer()) {
                algorithm.saturation_adjustment = v as i8;
            }
            if let Some(v) = table.get("lightness_adjustment").and_then(|v| v.as_integer()) {
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
                                if section_fields.contains_key("input_path") && section_fields.contains_key("output_path") {
                                    // Convert the table to a proper TOML string representation for parsing
                                    let toml_doc = toml::to_string(&section_fields)
                                        .unwrap_or_else(|_| String::from(""));

                                    if let Ok(config_section) = toml::from_str::<ConfigSection>(&toml_doc) {
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

        Ok(ConfigRoot {
            algorithm,
            groups,
        })
    }
}

// A representation of the entire config structure as a nested HashMap
pub type Config = HashMap<String, HashMap<String, ConfigSection>>;

impl ConfigRoot {
    pub fn to_flat_config(self) -> Config {
        self.groups
    }
}