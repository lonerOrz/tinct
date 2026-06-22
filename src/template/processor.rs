//! Template processor implementation

use crate::core::{ColorFormat, Mode, Result, TemplateEngine, Theme};
use regex::Regex;
use std::collections::HashMap;

// Type alias for filter functions to reduce type complexity
type FilterFn = Box<dyn Fn(&str, &HashMap<String, String>) -> Result<String> + Send + Sync>;

/// Default template processor implementation
pub struct TemplateProcessor {
    #[allow(dead_code)]
    filters: HashMap<String, FilterFn>,
}

impl TemplateProcessor {
    pub fn new() -> Self {
        let mut processor = Self {
            filters: HashMap::new(),
        };
        processor.register_builtin_filters();
        processor
    }

    fn register_builtin_filters(&mut self) {
        // Register built-in filters here
    }

    #[allow(dead_code)]
    pub fn register_filter<F>(&mut self, name: &str, filter: F)
    where
        F: Fn(&str, &HashMap<String, String>) -> Result<String> + Send + Sync + 'static,
    {
        self.filters.insert(name.to_string(), Box::new(filter));
    }
}

impl Default for TemplateProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateEngine for TemplateProcessor {
    fn render(&self, template: &str, theme: &Theme, mode: Mode) -> Result<String> {
        let mut content = template.to_string();

        // Process {{colors.XXX.default.XXX}} syntax - uses current mode colors
        let current_mode_colors = match mode {
            Mode::Dark => theme.dark_colors(),
            Mode::Light => theme.light_colors(),
        };
        content = self.process_color_placeholders(content, &current_mode_colors, "default")?;

        // Process {{colors.XXX.dark.XXX}} syntax - always uses dark colors
        let dark_colors = theme.dark_colors();
        content = self.process_color_placeholders(content, &dark_colors, "dark")?;

        // Process {{colors.XXX.light.XXX}} syntax - always uses light colors
        let light_colors = theme.light_colors();
        content = self.process_color_placeholders(content, &light_colors, "light")?;

        // Process mode placeholders
        content = content.replace("{{mode}}", &mode.to_string());
        content = content.replace("{{is_dark}}", if mode.is_dark() { "true" } else { "false" });
        content = content.replace(
            "{{is_light}}",
            if mode.is_light() { "true" } else { "false" },
        );

        Ok(content)
    }
}

impl TemplateProcessor {
    fn process_color_placeholders(
        &self,
        content: String,
        colors: &HashMap<String, ColorFormat>,
        mode_suffix: &str,
    ) -> Result<String> {
        let mut result = content;

        let properties = [
            "hex",
            "hex_stripped",
            "hex8",
            "hex8_stripped",
            "rgb",
            "rgba",
            "hsl",
            "hsla",
            "red",
            "green",
            "blue",
            "alpha",
            "hue",
            "saturation",
            "lightness",
        ];

        for prop in &properties {
            // Match {{colors.NAME.SUFFIX.PROP}} where SUFFIX is the mode suffix
            let pattern = format!(
                r"\{{\{{\s*colors\.([a-zA-Z0-9_]+)\.{}\.{}\s*\}}\}}",
                mode_suffix, prop
            );
            let re = Regex::new(&pattern).map_err(|e| {
                crate::core::Error::Template(format!("Invalid regex pattern: {}", e))
            })?;

            result = re
                .replace_all(&result, |caps: &regex::Captures| {
                    let key = &caps[1];
                    if let Some(color) = colors.get(key) {
                        match *prop {
                            "hex" => color.hex.clone(),
                            "hex_stripped" => color.hex_stripped.clone(),
                            "hex8" => color.hex8.clone(),
                            "hex8_stripped" => color.hex8_stripped.clone(),
                            "rgb" => color.rgb.clone(),
                            "rgba" => color.rgba.clone(),
                            "hsl" => color.hsl.clone(),
                            "hsla" => color.hsla.clone(),
                            "red" => color.red.to_string(),
                            "green" => color.green.to_string(),
                            "blue" => color.blue.to_string(),
                            "alpha" => color.alpha.to_string(),
                            "hue" => format!("{:.0}", color.hue),
                            "saturation" => format!("{:.0}", color.saturation),
                            "lightness" => format!("{:.0}", color.lightness),
                            _ => "#000000".to_string(),
                        }
                    } else {
                        "#000000".to_string()
                    }
                })
                .to_string();
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::{ColorEntry, ColorFormat};

    fn create_test_color(hex: &str) -> ColorFormat {
        ColorFormat {
            hex: hex.to_string(),
            hex_stripped: hex.trim_start_matches('#').to_string(),
            hex8: format!("{}FF", hex),
            hex8_stripped: format!("{}FF", hex.trim_start_matches('#')),
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
    fn test_template_processor_new() {
        let processor = TemplateProcessor::new();
        assert!(processor.filters.is_empty());
    }

    #[test]
    fn test_template_processor_render_basic() {
        let processor = TemplateProcessor::new();
        let mut theme = Theme::new("test".to_string(), "#FF5722".to_string());

        let color = create_test_color("#FF5722");
        theme.dark_palette.primary = ColorEntry {
            default: color.clone(),
        };
        theme.light_palette.primary = ColorEntry { default: color };

        let template = "Primary: {{colors.primary.default.hex}}";
        let result = processor.render(template, &theme, Mode::Dark).unwrap();

        assert!(result.contains("Primary: #FF5722"));
    }

    #[test]
    fn test_template_processor_render_mode_placeholders() {
        let processor = TemplateProcessor::new();
        let theme = Theme::new("test".to_string(), "#FF5722".to_string());

        let template = "Mode: {{mode}}, Is Dark: {{is_dark}}, Is Light: {{is_light}}";

        let result_dark = processor.render(template, &theme, Mode::Dark).unwrap();
        assert!(result_dark.contains("Mode: dark"));
        assert!(result_dark.contains("Is Dark: true"));
        assert!(result_dark.contains("Is Light: false"));

        let result_light = processor.render(template, &theme, Mode::Light).unwrap();
        assert!(result_light.contains("Mode: light"));
        assert!(result_light.contains("Is Dark: false"));
        assert!(result_light.contains("Is Light: true"));
    }

    #[test]
    fn test_template_processor_render_dark_light_suffix() {
        let processor = TemplateProcessor::new();
        let mut theme = Theme::new("test".to_string(), "#FF5722".to_string());

        let dark_color = create_test_color("#111111");
        let light_color = create_test_color("#EEEEEE");

        theme.dark_palette.background = ColorEntry {
            default: dark_color,
        };
        theme.light_palette.background = ColorEntry {
            default: light_color,
        };

        let template =
            "Dark: {{colors.background.dark.hex}}, Light: {{colors.background.light.hex}}";
        let result = processor.render(template, &theme, Mode::Dark).unwrap();

        assert!(
            result.contains("Dark: #111111"),
            "Expected dark color, got: {}",
            result
        );
        assert!(
            result.contains("Light: #EEEEEE"),
            "Expected light color, got: {}",
            result
        );
    }
}
