//! Template processor implementation

use crate::color::Color;
use crate::core::{Mode, Result, Theme};
use crate::template::filters::{ColorFilter, ColorProperty, FilterContext};
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Pre-compiled regex per mode suffix — 3 compilations instead of 45.
static MODE_REGEXES: LazyLock<[Regex; 3]> = LazyLock::new(|| {
    let suffixes = ["default", "dark", "light"];
    suffixes.map(|s| {
        let pattern = format!(
            r"\{{\{{\s*colors\.([a-zA-Z0-9_]+)\.{s}\.([a-zA-Z0-9_]+)\s*(?:\|([a-zA-Z_]+)(?::([^}}]*))?)?\s*\}}\}}",
            s = s
        );
        Regex::new(&pattern).unwrap()
    })
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
        let mut content = template.to_string();

        // Process {{colors.XXX.default.XXX}} syntax - uses current mode colors
        let current_mode_colors = match mode {
            Mode::Dark => theme.dark_colors(),
            Mode::Light => theme.light_colors(),
        };
        let mut current_with_seed = current_mode_colors;
        if let Ok(c) = crate::color::Color::from_hex(&theme.source_color) {
            current_with_seed.insert("source_color".to_string(), c);
        }
        content =
            self.process_color_placeholders(content, &MODE_REGEXES[0], &current_with_seed)?;

        // Process {{colors.XXX.dark.XXX}} syntax - always uses dark colors
        let mut dark_with_seed = theme.dark_colors();
        if let Ok(c) = crate::color::Color::from_hex(&theme.source_color) {
            dark_with_seed.insert("source_color".to_string(), c);
        }
        content =
            self.process_color_placeholders(content, &MODE_REGEXES[1], &dark_with_seed)?;

        // Process {{colors.XXX.light.XXX}} syntax - always uses light colors
        let mut light_with_seed = theme.light_colors();
        if let Ok(c) = crate::color::Color::from_hex(&theme.source_color) {
            light_with_seed.insert("source_color".to_string(), c);
        }
        content =
            self.process_color_placeholders(content, &MODE_REGEXES[2], &light_with_seed)?;

        // Process mode placeholders
        let mode_str = match mode {
            Mode::Dark => "dark",
            Mode::Light => "light",
        };
        content = content.replace("{{mode}}", mode_str);
        content = content.replace("{{is_dark}}", if mode.is_dark() { "true" } else { "false" });
        content = content.replace(
            "{{is_light}}",
            if mode.is_light() { "true" } else { "false" },
        );

        Ok(content)
    }

    fn process_color_placeholders(
        &self,
        content: String,
        re: &Regex,
        colors: &HashMap<String, Color>,
    ) -> Result<String> {
        Ok(re
            .replace_all(&content, |caps: &regex::Captures| {
                let key = &caps[1];
                let prop = &caps[2];
                let filter_name = caps.get(3).map(|m| m.as_str());
                let filter_param = caps.get(4).map(|m| m.as_str());

                let prop_enum = match ColorProperty::from_property(prop) {
                    Some(p) => p,
                    None => return caps[0].to_string(),
                };

                if let Some(color) = colors.get(key) {
                    if let (Some(name), Some(param)) = (filter_name, filter_param) {
                        if let Some(filter) = ColorFilter::from_name(name, param) {
                            if filter.is_compatible(&prop_enum) {
                                let ctx = FilterContext {
                                    color: *color,
                                    format_type: prop_enum,
                                };
                                filter.apply_to(ctx.color, prop_enum)
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
            })
            .to_string())
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
        theme.dark_palette.insert(role, color.clone());
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

        let template = "Primary: {{colors.primary.default.hex|set_alpha:0.5}}";
        let result = processor.render(template, &theme, Mode::Dark).unwrap();
        // hex property outputs 6-digit hex; set_alpha modifies the color but format stays hex
        assert!(
            result.contains("Primary: #FF5722"),
            "Expected hex with alpha applied to color, got: {}",
            result
        );
    }

    #[test]
    fn test_template_processor_render_with_lighten_filter() {
        let processor = TemplateProcessor::new();
        let theme = make_theme_with_color(ColorRole::Primary, "#FF5722");

        let template = "Primary: {{colors.primary.default.rgb|lighten:10}}";
        let result = processor.render(template, &theme, Mode::Dark).unwrap();
        assert!(result.starts_with("Primary: rgb("), "got: {}", result);
    }
}
