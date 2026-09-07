//! Template processor implementation

use crate::color::Color;
use crate::core::{Mode, Result, Theme};
use crate::template::filters::{ColorFilter, ColorProperty};
use regex::Regex;
use std::sync::LazyLock;

/// Unified regex for all color placeholders across all mode suffixes.
/// Capture groups: $1=color role, $2=mode (default|dark|light), $3=property,
/// $4=filter name (optional), $5=filter param (optional).
static COLOR_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\{\{\s*colors\.([a-zA-Z0-9_]+)\.(default|dark|light)\.([a-zA-Z0-9_]+)\s*(?:\|([a-zA-Z_]+)(?::([^}]*))?)?\s*\}\}",
    )
    .unwrap()
});

/// Default template processor implementation
pub struct TemplateProcessor;

impl TemplateProcessor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TemplateProcessor {
    fn default() -> Self {
        Self
    }
}

impl TemplateProcessor {
    pub fn render(&self, template: &str, theme: &Theme, mode: Mode) -> Result<String> {
        // Build color maps once, inject source_color into both.
        let mut dark_colors = theme.dark_colors();
        let mut light_colors = theme.light_colors();
        if let Ok(c) = Color::from_hex(&theme.source_color) {
            dark_colors.insert("source_color".to_string(), c);
            light_colors.insert("source_color".to_string(), c);
        }

        let content = COLOR_REGEX.replace_all(template, |caps: &regex::Captures| {
            let key = &caps[1];
            let mode_suffix = &caps[2];
            let prop = &caps[3];
            let filter_name = caps.get(4).map(|m| m.as_str());
            let filter_param = caps.get(5).map(|m| m.as_str());

            let prop_enum = match ColorProperty::from_property(prop) {
                Some(p) => p,
                None => return caps[0].to_string(),
            };

            let colors = match mode_suffix {
                "dark" => &dark_colors,
                "light" => &light_colors,
                "default" => match mode {
                    Mode::Dark => &dark_colors,
                    Mode::Light => &light_colors,
                },
                _ => return caps[0].to_string(),
            };

            if let Some(color) = colors.get(key) {
                if let (Some(name), Some(param)) = (filter_name, filter_param) {
                    if let Some(filter) = ColorFilter::from_name(name, param) {
                        if filter.is_compatible(&prop_enum) {
                            filter.apply_to(*color, prop_enum)
                        } else {
                            color.format(&prop_enum)
                        }
                    } else {
                        color.format(&prop_enum)
                    }
                } else {
                    color.format(&prop_enum)
                }
            } else {
                crate::log::general::info(&format!(
                    "Warning: color '{}' not found in palette, using #000000",
                    key
                ));
                "#000000".to_string()
            }
        });

        let mut output = content.into_owned();
        let mode_str = match mode {
            Mode::Dark => "dark",
            Mode::Light => "light",
        };
        output = output.replace("{{mode}}", mode_str);
        output = output.replace("{{is_dark}}", if mode.is_dark() { "true" } else { "false" });
        output = output.replace(
            "{{is_light}}",
            if mode.is_light() { "true" } else { "false" },
        );

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;
    use crate::palette::ColorRole;

    fn make_theme_with_color(role: ColorRole, hex: &str) -> Theme {
        let mut theme = Theme::new("test".to_string(), "#FF5722".to_string());
        let color = Color::from_hex(hex).unwrap();
        theme.dark_palette.insert(role, color);
        theme.light_palette.insert(role, color);
        theme
    }

    #[test]
    fn test_template_processor_new() {
        let _processor = TemplateProcessor::new();
    }

    #[test]
    fn test_template_processor_render_basic() {
        let processor = TemplateProcessor::new();
        let theme = make_theme_with_color(ColorRole::Primary, "#FF5722");

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

        theme
            .dark_palette
            .insert(ColorRole::Background, Color::from_hex("#111111").unwrap());
        theme
            .light_palette
            .insert(ColorRole::Background, Color::from_hex("#EEEEEE").unwrap());

        let template =
            "Dark: {{colors.background.dark.hex}}, Light: {{colors.background.light.hex}}";
        let result = processor.render(template, &theme, Mode::Dark).unwrap();

        assert!(result.contains("Dark: #111111"), "got: {}", result);
        assert!(result.contains("Light: #EEEEEE"), "got: {}", result);
    }

    #[test]
    fn test_template_processor_render_with_filter() {
        let processor = TemplateProcessor::new();
        let theme = make_theme_with_color(ColorRole::Primary, "#FF5722");

        // hex stays 6-digit even when alpha is modified via filter
        let template = "Primary: {{colors.primary.default.hex|set_alpha:0.5}}";
        let result = processor.render(template, &theme, Mode::Dark).unwrap();
        assert!(
            result.contains("Primary: #FF5722"),
            "Expected 6-digit hex, got: {}",
            result
        );

        // hex8 produces 8-digit hex when alpha is applied
        let template8 = "Primary: {{colors.primary.default.hex8|set_alpha:0.5}}";
        let result8 = processor.render(template8, &theme, Mode::Dark).unwrap();
        assert!(
            result8.contains("Primary: #FF572280"),
            "Expected hex8 with alpha applied, got: {}",
            result8
        );
    }

    #[test]
    fn test_template_processor_render_with_lighten_filter() {
        let processor = TemplateProcessor::new();
        let theme = make_theme_with_color(ColorRole::Primary, "#FF5722");

        let template = "Primary: {{colors.primary.default.rgb|lighten:0.15}}";
        let result = processor.render(template, &theme, Mode::Dark).unwrap();
        assert!(result.starts_with("Primary: rgb("), "got: {}", result);
    }
}
