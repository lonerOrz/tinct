use crate::filter::FilterRegistry;
use crate::palette_generator::{ColorFormat, Palette};
use regex::Regex;
use std::collections::HashMap;

/// Load template file
pub fn load_template(template_path: &str) -> Result<String, String> {
    if crate::log::is_verbose() {
        eprintln!("Loading template from {}", template_path);
    }

    let template_content = std::fs::read_to_string(template_path)
        .map_err(|e| format!("Could not read template file '{}': {}", template_path, e))?;

    if crate::log::is_verbose() {
        eprintln!("Template loaded successfully from {}", template_path);
    }
    Ok(template_content)
}

/// Process template by replacing color placeholders and mode placeholders
pub fn process_template(template_content: &str, palette: &Palette, effective_mode: &str) -> String {
    if crate::log::is_verbose() {
        eprintln!("Processing template...");
    }

    let mut content = template_content.replace("{{mode}}", effective_mode);
    content = content.replace(
        "{{is_dark}}",
        if effective_mode == "dark" {
            "true"
        } else {
            "false"
        },
    );
    content = content.replace(
        "{{is_light}}",
        if effective_mode == "light" {
            "true"
        } else {
            "false"
        },
    );

    // Create a mapping of color names to color entries for dark mode
    let dark_color_map: HashMap<&str, &ColorFormat> = [
        ("primary", &palette.primary.dark),
        ("on_primary", &palette.on_primary.dark),
        ("primary_container", &palette.primary_container.dark),
        ("on_primary_container", &palette.on_primary_container.dark),
        ("secondary", &palette.secondary.dark),
        ("on_secondary", &palette.on_secondary.dark),
        ("secondary_container", &palette.secondary_container.dark),
        ("on_secondary_container", &palette.on_secondary_container.dark),
        ("tertiary", &palette.tertiary.dark),
        ("on_tertiary", &palette.on_tertiary.dark),
        ("tertiary_container", &palette.tertiary_container.dark),
        ("on_tertiary_container", &palette.on_tertiary_container.dark),
        ("error", &palette.error.dark),
        ("on_error", &palette.on_error.dark),
        ("error_container", &palette.error_container.dark),
        ("on_error_container", &palette.on_error_container.dark),
        ("background", &palette.background.dark),
        ("on_background", &palette.on_background.dark),
        ("surface", &palette.surface.dark),
        ("on_surface", &palette.on_surface.dark),
        ("surface_variant", &palette.surface_variant.dark),
        ("on_surface_variant", &palette.on_surface_variant.dark),
        ("surface_container_lowest", &palette.surface_container_lowest.dark),
        ("surface_container_low", &palette.surface_container_low.dark),
        ("surface_container", &palette.surface_container.dark),
        ("surface_container_high", &palette.surface_container_high.dark),
        ("surface_container_highest", &palette.surface_container_highest.dark),
        ("inverse_surface", &palette.inverse_surface.dark),
        ("inverse_on_surface", &palette.inverse_on_surface.dark),
        ("inverse_primary", &palette.inverse_primary.dark),
        ("surface_dim", &palette.surface_dim.dark),
        ("surface_bright", &palette.surface_bright.dark),
        ("scrim", &palette.scrim.dark),
        ("primary_fixed", &palette.primary_fixed.dark),
        ("primary_fixed_dim", &palette.primary_fixed_dim.dark),
        ("on_primary_fixed", &palette.on_primary_fixed.dark),
        ("on_primary_fixed_variant", &palette.on_primary_fixed_variant.dark),
        ("secondary_fixed", &palette.secondary_fixed.dark),
        ("secondary_fixed_dim", &palette.secondary_fixed_dim.dark),
        ("on_secondary_fixed", &palette.on_secondary_fixed.dark),
        ("on_secondary_fixed_variant", &palette.on_secondary_fixed_variant.dark),
        ("tertiary_fixed", &palette.tertiary_fixed.dark),
        ("tertiary_fixed_dim", &palette.tertiary_fixed_dim.dark),
        ("on_tertiary_fixed", &palette.on_tertiary_fixed.dark),
        ("on_tertiary_fixed_variant", &palette.on_tertiary_fixed_variant.dark),
        ("outline", &palette.outline.dark),
        ("outline_variant", &palette.outline_variant.dark),
        ("shadow", &palette.shadow.dark),
    ]
    .iter()
    .cloned()
    .collect();

    // Create a mapping of color names to color entries for light mode
    let light_color_map: HashMap<&str, &ColorFormat> = [
        ("primary", &palette.primary.light),
        ("on_primary", &palette.on_primary.light),
        ("primary_container", &palette.primary_container.light),
        ("on_primary_container", &palette.on_primary_container.light),
        ("secondary", &palette.secondary.light),
        ("on_secondary", &palette.on_secondary.light),
        ("secondary_container", &palette.secondary_container.light),
        ("on_secondary_container", &palette.on_secondary_container.light),
        ("tertiary", &palette.tertiary.light),
        ("on_tertiary", &palette.on_tertiary.light),
        ("tertiary_container", &palette.tertiary_container.light),
        ("on_tertiary_container", &palette.on_tertiary_container.light),
        ("error", &palette.error.light),
        ("on_error", &palette.on_error.light),
        ("error_container", &palette.error_container.light),
        ("on_error_container", &palette.on_error_container.light),
        ("background", &palette.background.light),
        ("on_background", &palette.on_background.light),
        ("surface", &palette.surface.light),
        ("on_surface", &palette.on_surface.light),
        ("surface_variant", &palette.surface_variant.light),
        ("on_surface_variant", &palette.on_surface_variant.light),
        ("surface_container_lowest", &palette.surface_container_lowest.light),
        ("surface_container_low", &palette.surface_container_low.light),
        ("surface_container", &palette.surface_container.light),
        ("surface_container_high", &palette.surface_container_high.light),
        ("surface_container_highest", &palette.surface_container_highest.light),
        ("inverse_surface", &palette.inverse_surface.light),
        ("inverse_on_surface", &palette.inverse_on_surface.light),
        ("inverse_primary", &palette.inverse_primary.light),
        ("surface_dim", &palette.surface_dim.light),
        ("surface_bright", &palette.surface_bright.light),
        ("scrim", &palette.scrim.light),
        ("primary_fixed", &palette.primary_fixed.light),
        ("primary_fixed_dim", &palette.primary_fixed_dim.light),
        ("on_primary_fixed", &palette.on_primary_fixed.light),
        ("on_primary_fixed_variant", &palette.on_primary_fixed_variant.light),
        ("secondary_fixed", &palette.secondary_fixed.light),
        ("secondary_fixed_dim", &palette.secondary_fixed_dim.light),
        ("on_secondary_fixed", &palette.on_secondary_fixed.light),
        ("on_secondary_fixed_variant", &palette.on_secondary_fixed_variant.light),
        ("tertiary_fixed", &palette.tertiary_fixed.light),
        ("tertiary_fixed_dim", &palette.tertiary_fixed_dim.light),
        ("on_tertiary_fixed", &palette.on_tertiary_fixed.light),
        ("on_tertiary_fixed_variant", &palette.on_tertiary_fixed_variant.light),
        ("outline", &palette.outline.light),
        ("outline_variant", &palette.outline_variant.light),
        ("shadow", &palette.shadow.light),
    ]
    .iter()
    .cloned()
    .collect();

    // Create a mapping of color names to color entries for default mode (based on effective_mode)
    let default_color_map: HashMap<&str, &ColorFormat> = if effective_mode == "dark" {
        dark_color_map.clone()
    } else {
        light_color_map.clone()
    };

    // Process filters in template
    let filter_registry = FilterRegistry::new();
    content = process_filters(content, &default_color_map, &filter_registry);

    // Replace all color property placeholders (without filters)
    let color_properties = [
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

    // Process templates for different mode suffixes: .default, .dark, .light
    for (mode_suffix, color_map) in [
        (".default", &default_color_map),
        (".dark", &dark_color_map),
        (".light", &light_color_map),
    ] {
        for prop in &color_properties {
            let pattern = format!(
                r"\{{\{{\s*colors\.([a-zA-Z0-9_]+){}\.{}\s*\}}\}}",
                regex::escape(mode_suffix),
                regex::escape(prop)
            );
            let re = Regex::new(&pattern).unwrap();

            content = re
                .replace_all(&content, |caps: &regex::Captures| {
                    let key = &caps[1];
                    if let Some(color_format) = color_map.get(key) {
                        match *prop {
                            "hex" => color_format.hex.clone(),
                            "hex_stripped" => color_format.hex_stripped.clone(),
                            "hex8" => color_format.hex8.clone(),
                            "hex8_stripped" => color_format.hex8_stripped.clone(),
                            "rgb" => color_format.rgb.clone(),
                            "rgba" => color_format.rgba.clone(),
                            "hsl" => color_format.hsl.clone(),
                            "hsla" => color_format.hsla.clone(),
                            "red" => color_format.red.to_string(),
                            "green" => color_format.green.to_string(),
                            "blue" => color_format.blue.to_string(),
                            "alpha" => color_format.alpha.to_string(),
                            "hue" => format!("{:.0}", color_format.hue),
                            "saturation" => format!("{:.0}", color_format.saturation),
                            "lightness" => format!("{:.0}", color_format.lightness),
                            _ => "#000000".to_string(), // default fallback
                        }
                    } else {
                        // Return default value if color key is not found
                        match *prop {
                            "hex" => "#000000".to_string(),
                            "hex_stripped" => "000000".to_string(),
                            "hex8" => "#00000000".to_string(),
                            "hex8_stripped" => "00000000".to_string(),
                            "red" | "green" | "blue" | "alpha" => "0".to_string(),
                            "hue" | "saturation" | "lightness" => "0".to_string(),
                            "rgb" => "rgb(0, 0, 0)".to_string(),
                            "rgba" => "rgba(0, 0, 0, 0)".to_string(),
                            "hsl" => "hsl(0, 0%, 0%)".to_string(),
                            "hsla" => "hsla(0, 0%, 0%, 1.0)".to_string(),
                            _ => "#000000".to_string(),
                        }
                    }
                })
                .to_string();
        }
    }

    if crate::log::is_verbose() {
        eprintln!("Template processed successfully");
    }
    content
}

/// Process filters in template content
fn process_filters(
    content: String,
    default_color_map: &HashMap<&str, &ColorFormat>,
    filter_registry: &FilterRegistry,
) -> String {
    // Pattern for filters: {{ colors.color_name.(default|dark|light).property | filter_name: param }}
    let filter_pattern = r"\{\{\s*colors\.([a-zA-Z0-9_]+)\.(default|dark|light)\.([a-zA-Z0-9_]+)\s*\|\s*([a-zA-Z0-9_]+)(?::\s*([^}]+))?\s*\}\}";
    let re = Regex::new(filter_pattern).unwrap();

    re.replace_all(&content, |caps: &regex::Captures| {
        let color_name = &caps[1];
        let _mode_suffix = &caps[2];
        let property = &caps[3];
        let filter_name = &caps[4];
        let filter_param = caps.get(5).map(|m| m.as_str().trim());

        // For filters, we use the default_color_map which is already selected based on effective_mode
        // This maintains backward compatibility with existing filter usage
        let color_format = default_color_map.get(color_name);

        if let Some(color_format) = color_format {
            let original_value = match property {
                "hex" => color_format.hex.clone(),
                "hex_stripped" => color_format.hex_stripped.clone(),
                "hex8" => color_format.hex8.clone(),
                "hex8_stripped" => color_format.hex8_stripped.clone(),
                "rgb" => color_format.rgb.clone(),
                "rgba" => color_format.rgba.clone(),
                "hsl" => color_format.hsl.clone(),
                "hsla" => color_format.hsla.clone(),
                "red" => color_format.red.to_string(),
                "green" => color_format.green.to_string(),
                "blue" => color_format.blue.to_string(),
                "alpha" => color_format.alpha.to_string(),
                "hue" => format!("{:.0}", color_format.hue),
                "saturation" => format!("{:.0}", color_format.saturation),
                "lightness" => format!("{:.0}", color_format.lightness),
                _ => "#000000".to_string(), // default fallback
            };

            let format_type = crate::filter::ColorFormatType::from_property(property);

            if let Some(format_type) = format_type {
                filter_registry.apply_filter(
                    &original_value,
                    filter_name,
                    filter_param,
                    color_format,
                    format_type,
                )
            } else {
                filter_registry.apply_filter(
                    &original_value,
                    filter_name,
                    filter_param,
                    color_format,
                    crate::filter::ColorFormatType::Hex,
                )
            }
        } else {
            // Return default value if color key is not found
            let default_value = match property {
                "hex" => "#000000".to_string(),
                "hex_stripped" => "000000".to_string(),
                "hex8" => "#00000000".to_string(),
                "hex8_stripped" => "00000000".to_string(),
                "red" | "green" | "blue" | "alpha" => "0".to_string(),
                "hue" | "saturation" | "lightness" => "0".to_string(),
                "rgb" => "rgb(0, 0, 0)".to_string(),
                "rgba" => "rgba(0, 0, 0, 0)".to_string(),
                "hsl" => "hsl(0, 0%, 0%)".to_string(),
                "hsla" => "hsla(0, 0%, 0%, 1.0)".to_string(),
                _ => "#000000".to_string(),
            };

            let format_type = crate::filter::ColorFormatType::from_property(property);

            if let Some(format_type) = format_type {
                filter_registry.apply_filter(
                    &default_value,
                    filter_name,
                    filter_param,
                    &ColorFormat {
                        hex: "#000000".to_string(),
                        hex_stripped: "000000".to_string(),
                        hex8: "#00000000".to_string(),
                        hex8_stripped: "00000000".to_string(),
                        rgb: "rgb(0, 0, 0)".to_string(),
                        rgba: "rgba(0, 0, 0, 0)".to_string(),
                        hsl: "hsl(0, 0%, 0%)".to_string(),
                        hsla: "hsla(0, 0%, 0%, 1.0)".to_string(),
                        red: 0,
                        green: 0,
                        blue: 0,
                        alpha: 0.0,
                        hue: 0.0,
                        saturation: 0.0,
                        lightness: 0.0,
                        original_hue: Some(0),
                        original_saturation: Some(0),
                        original_lightness: Some(0),
                    },
                    format_type,
                )
            } else {
                filter_registry.apply_filter(
                    &default_value,
                    filter_name,
                    filter_param,
                    &ColorFormat {
                        hex: "#000000".to_string(),
                        hex_stripped: "000000".to_string(),
                        hex8: "#00000000".to_string(),
                        hex8_stripped: "00000000".to_string(),
                        rgb: "rgb(0, 0, 0)".to_string(),
                        rgba: "rgba(0, 0, 0, 0)".to_string(),
                        hsl: "hsl(0, 0%, 0%)".to_string(),
                        hsla: "hsla(0, 0%, 0%, 1.0)".to_string(),
                        red: 0,
                        green: 0,
                        blue: 0,
                        alpha: 0.0,
                        hue: 0.0,
                        saturation: 0.0,
                        lightness: 0.0,
                        original_hue: Some(0),
                        original_saturation: Some(0),
                        original_lightness: Some(0),
                    },
                    crate::filter::ColorFormatType::Hex,
                )
            }
        }
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette_generator::{ColorEntry, ColorFormat, Palette};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_load_template() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let template_path = temp_dir.path().join("test_template.txt");

        let template_content = "Primary color: {{colors.primary.default.hex}}, Mode: {{mode}}";
        fs::write(&template_path, template_content).expect("Unable to write template file");

        let result = load_template(template_path.to_str().unwrap()).unwrap();
        assert_eq!(result, template_content);
    }

    #[test]
    fn test_process_template() {
        // Create a mock palette for testing
        let color_format = ColorFormat {
            hex: "#FF5722".to_string(),
            hex_stripped: "FF5722".to_string(),
            hex8: "#FF5722FF".to_string(), // Assuming full opacity
            hex8_stripped: "FF5722FF".to_string(), // Assuming full opacity
            rgb: "rgb(255, 87, 34)".to_string(),
            rgba: "rgba(255, 87, 34, 255)".to_string(),
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
        };

        let palette = Palette {
            primary: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_primary: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            primary_container: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_primary_container: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            primary_fixed: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            primary_fixed_dim: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_primary_fixed: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_primary_fixed_variant: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            secondary: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_secondary: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            secondary_container: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_secondary_container: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            secondary_fixed: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            secondary_fixed_dim: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_secondary_fixed: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_secondary_fixed_variant: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            tertiary: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_tertiary: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            tertiary_container: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_tertiary_container: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            tertiary_fixed: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            tertiary_fixed_dim: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_tertiary_fixed: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_tertiary_fixed_variant: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            error: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_error: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            error_container: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_error_container: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            background: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_background: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            surface: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_surface: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            surface_variant: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_surface_variant: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            surface_container_lowest: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            surface_container_low: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            surface_container: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            surface_container_high: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            surface_container_highest: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            inverse_surface: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            inverse_on_surface: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            inverse_primary: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            surface_dim: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            surface_bright: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            outline: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            outline_variant: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            shadow: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            scrim: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            // Terminal colors - using the same color format for all in tests
            black: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            red: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            green: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            yellow: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            blue: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            magenta: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            cyan: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            white: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            bright_black: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            bright_red: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            bright_green: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            bright_yellow: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            bright_blue: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            bright_magenta: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            bright_cyan: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            bright_white: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
        };

        let template_content = "Primary color: {{colors.primary.default.hex}}, Mode: {{mode}}";
        let result = process_template(template_content, &palette, "dark");

        // The result should contain the expected placeholders replaced
        assert!(result.contains("Primary color: #FF5722"));
        assert!(result.contains(", Mode: dark"));
    }

    #[test]
    fn test_set_alpha_filter() {
        let color_format = ColorFormat {
            hex: "#FF5722".to_string(),
            hex_stripped: "FF5722".to_string(),
            hex8: "#FF5722FF".to_string(), // Assuming full opacity
            hex8_stripped: "FF5722FF".to_string(), // Assuming full opacity
            rgb: "rgb(255, 87, 34)".to_string(),
            rgba: "rgba(255, 87, 34, 255)".to_string(),
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
        };

        let palette = Palette {
            primary: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_primary: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            primary_container: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_primary_container: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            primary_fixed: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            primary_fixed_dim: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_primary_fixed: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_primary_fixed_variant: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            secondary: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_secondary: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            secondary_container: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_secondary_container: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            secondary_fixed: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            secondary_fixed_dim: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_secondary_fixed: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_secondary_fixed_variant: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            tertiary: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_tertiary: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            tertiary_container: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_tertiary_container: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            tertiary_fixed: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            tertiary_fixed_dim: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_tertiary_fixed: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_tertiary_fixed_variant: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            error: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_error: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            error_container: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_error_container: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            background: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_background: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            surface: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_surface: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            surface_variant: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            on_surface_variant: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            surface_container_lowest: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            surface_container_low: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            surface_container: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            surface_container_high: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            surface_container_highest: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            inverse_surface: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            inverse_on_surface: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            inverse_primary: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            surface_dim: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            surface_bright: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            outline: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            outline_variant: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            shadow: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            scrim: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            // Terminal colors - using the same color format for all in tests
            black: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            red: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            green: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            yellow: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            blue: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            magenta: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            cyan: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            white: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            bright_black: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            bright_red: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            bright_green: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            bright_yellow: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            bright_blue: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            bright_magenta: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            bright_cyan: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
            bright_white: ColorEntry {
                default: color_format.clone(),
                dark: color_format.clone(),
                light: color_format.clone(),
            },
        };

        // Test set_alpha filter with rgba value
        let template_content = "Primary color: {{colors.primary.default.rgba | set_alpha: 0.5}}";
        let result = process_template(template_content, &palette, "dark");
        assert!(result.contains("rgba(255, 87, 34, 0."));
    }
}
