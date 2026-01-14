use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
