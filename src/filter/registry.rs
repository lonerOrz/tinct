//! Filter registry for managing filter instances
//!
//! This module provides a registry pattern for organizing and applying filters.

use super::lightness::{DarkenFilter, LightenFilter};
use super::saturation::{DesaturateFilter, SaturateFilter};
use super::set_alpha::SetAlphaFilter;
use super::types::{ColorFormatType, Filter, FilterContext};

/// Registry for managing filters
pub struct FilterRegistry {
    filters: std::collections::HashMap<String, Box<dyn Filter>>,
}

impl Default for FilterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FilterRegistry {
    pub fn new() -> Self {
        let mut registry = FilterRegistry {
            filters: std::collections::HashMap::new(),
        };

        // Register default filters
        registry.register("set_alpha", Box::new(SetAlphaFilter));
        registry.register("lighten", Box::new(LightenFilter));
        registry.register("darken", Box::new(DarkenFilter));
        registry.register("saturate", Box::new(SaturateFilter));
        registry.register("desaturate", Box::new(DesaturateFilter));

        registry
    }

    pub fn register(&mut self, name: &str, filter: Box<dyn Filter>) {
        self.filters.insert(name.to_string(), filter);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Filter> {
        self.filters.get(name).map(|boxed| boxed.as_ref())
    }

    pub fn apply_filter(
        &self,
        value: &str,
        name: &str,
        param: Option<&str>,
        color_format: &crate::palette::ColorFormat,
        format_type: ColorFormatType,
    ) -> String {
        if let Some(filter) = self.get(name) {
            if filter.is_compatible(&format_type) {
                let ctx = FilterContext {
                    original_value: value.to_string(),
                    format_type,
                    color_format: color_format.clone(),
                };
                filter.apply(&ctx, param)
            } else {
                // Return original value if filter is not compatible with format
                value.to_string()
            }
        } else {
            // Return original value if filter not found
            value.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::ColorFormat;

    fn create_test_color() -> ColorFormat {
        ColorFormat {
            hex: "#FF5722".to_string(),
            hex_stripped: "FF5722".to_string(),
            hex8: "#FF5722FF".to_string(),
            hex8_stripped: "FF5722FF".to_string(),
            rgb: "rgb(255, 87, 34)".to_string(),
            rgba: "rgba(255, 87, 34, 1.0)".to_string(),
            hsl: "hsl(14, 100%, 57%)".to_string(),
            hsla: "hsla(14, 100%, 57%, 1.0)".to_string(),
            red: 255,
            green: 87,
            blue: 34,
            alpha: 1.0,
            hue: 14.0,
            saturation: 100.0,
            lightness: 57.0,
            original_hue: Some(14),
            original_saturation: Some(100),
            original_lightness: Some(57),
        }
    }

    #[test]
    fn test_registry_creation() {
        let registry = FilterRegistry::new();
        assert!(registry.get("set_alpha").is_some());
        assert!(registry.get("lighten").is_some());
        assert!(registry.get("darken").is_some());
        assert!(registry.get("saturate").is_some());
        assert!(registry.get("desaturate").is_some());
    }

    #[test]
    fn test_apply_filter_unknown_filter() {
        let registry = FilterRegistry::new();
        let color = create_test_color();
        let result =
            registry.apply_filter("test", "unknown_filter", None, &color, ColorFormatType::Rgb);
        assert_eq!(result, "test");
    }

    #[test]
    fn test_apply_filter_set_alpha() {
        let registry = FilterRegistry::new();
        let color = create_test_color();
        let result = registry.apply_filter(
            "rgb(255, 87, 34)",
            "set_alpha",
            Some("0.5"),
            &color,
            ColorFormatType::Rgba,
        );
        assert!(result.contains("0.5"));
    }
}
