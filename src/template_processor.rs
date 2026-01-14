use crate::palette_generator::{ColorFormat, Palette};
use regex::Regex;
use std::fs;

/// Load template file
pub fn load_template(template_path: &str) -> Result<String, String> {
    if crate::log::is_verbose() {
        eprintln!("Loading template from {}", template_path);
    }

    let template_content = fs::read_to_string(template_path)
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

    // Create a mapping of color names to color entries
    let color_map: std::collections::HashMap<&str, &ColorFormat> = [
        ("primary", &palette.primary.default),
        ("on_primary", &palette.on_primary.default),
        ("primary_container", &palette.primary_container.default),
        (
            "on_primary_container",
            &palette.on_primary_container.default,
        ),
        ("secondary", &palette.secondary.default),
        ("on_secondary", &palette.on_secondary.default),
        ("secondary_container", &palette.secondary_container.default),
        (
            "on_secondary_container",
            &palette.on_secondary_container.default,
        ),
        ("tertiary", &palette.tertiary.default),
        ("on_tertiary", &palette.on_tertiary.default),
        ("tertiary_container", &palette.tertiary_container.default),
        (
            "on_tertiary_container",
            &palette.on_tertiary_container.default,
        ),
        ("error", &palette.error.default),
        ("on_error", &palette.on_error.default),
        ("error_container", &palette.error_container.default),
        ("on_error_container", &palette.on_error_container.default),
        ("background", &palette.background.default),
        ("on_background", &palette.on_background.default),
        ("surface", &palette.surface.default),
        ("on_surface", &palette.on_surface.default),
        ("surface_variant", &palette.surface_variant.default),
        ("on_surface_variant", &palette.on_surface_variant.default),
        (
            "surface_container_lowest",
            &palette.surface_container_lowest.default,
        ),
        (
            "surface_container_low",
            &palette.surface_container_low.default,
        ),
        ("surface_container", &palette.surface_container.default),
        (
            "surface_container_high",
            &palette.surface_container_high.default,
        ),
        (
            "surface_container_highest",
            &palette.surface_container_highest.default,
        ),
        ("inverse_surface", &palette.inverse_surface.default),
        ("inverse_on_surface", &palette.inverse_on_surface.default),
        ("inverse_primary", &palette.inverse_primary.default),
        ("surface_dim", &palette.surface_dim.default),
        ("surface_bright", &palette.surface_bright.default),
        ("scrim", &palette.scrim.default),
        ("primary_fixed", &palette.primary_fixed.default),
        ("primary_fixed_dim", &palette.primary_fixed_dim.default),
        ("on_primary_fixed", &palette.on_primary_fixed.default),
        (
            "on_primary_fixed_variant",
            &palette.on_primary_fixed_variant.default,
        ),
        ("secondary_fixed", &palette.secondary_fixed.default),
        ("secondary_fixed_dim", &palette.secondary_fixed_dim.default),
        ("on_secondary_fixed", &palette.on_secondary_fixed.default),
        (
            "on_secondary_fixed_variant",
            &palette.on_secondary_fixed_variant.default,
        ),
        ("tertiary_fixed", &palette.tertiary_fixed.default),
        ("tertiary_fixed_dim", &palette.tertiary_fixed_dim.default),
        ("on_tertiary_fixed", &palette.on_tertiary_fixed.default),
        (
            "on_tertiary_fixed_variant",
            &palette.on_tertiary_fixed_variant.default,
        ),
        ("outline", &palette.outline.default),
        ("outline_variant", &palette.outline_variant.default),
        ("shadow", &palette.shadow.default),
    ]
    .iter()
    .cloned()
    .collect();

    // Replace all color property placeholders
    let color_properties = [
        "hex",
        "hex_stripped",
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

    for prop in &color_properties {
        let pattern = format!(
            r"\{{\{{\s*colors\.([a-zA-Z0-9_]+)\.default\.{}\s*\}}\}}",
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

    if crate::log::is_verbose() {
        eprintln!("Template processed successfully");
    }
    content
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
            rgb: "rgb(255, 87, 34)".to_string(),
            rgba: "rgba(255, 87, 34, 255)".to_string(),
            hsl: "hsl(14, 100%, 57%)".to_string(),
            hsla: "hsla(14, 100%, 57%, 1.0)".to_string(),
            red: 255,
            green: 87,
            blue: 34,
            alpha: 255,
            hue: 14.0,
            saturation: 100.0,
            lightness: 57.0,
        };

        let palette = Palette {
            primary: ColorEntry {
                default: color_format.clone(),
            },
            on_primary: ColorEntry {
                default: color_format.clone(),
            },
            primary_container: ColorEntry {
                default: color_format.clone(),
            },
            on_primary_container: ColorEntry {
                default: color_format.clone(),
            },
            primary_fixed: ColorEntry {
                default: color_format.clone(),
            },
            primary_fixed_dim: ColorEntry {
                default: color_format.clone(),
            },
            on_primary_fixed: ColorEntry {
                default: color_format.clone(),
            },
            on_primary_fixed_variant: ColorEntry {
                default: color_format.clone(),
            },
            secondary: ColorEntry {
                default: color_format.clone(),
            },
            on_secondary: ColorEntry {
                default: color_format.clone(),
            },
            secondary_container: ColorEntry {
                default: color_format.clone(),
            },
            on_secondary_container: ColorEntry {
                default: color_format.clone(),
            },
            secondary_fixed: ColorEntry {
                default: color_format.clone(),
            },
            secondary_fixed_dim: ColorEntry {
                default: color_format.clone(),
            },
            on_secondary_fixed: ColorEntry {
                default: color_format.clone(),
            },
            on_secondary_fixed_variant: ColorEntry {
                default: color_format.clone(),
            },
            tertiary: ColorEntry {
                default: color_format.clone(),
            },
            on_tertiary: ColorEntry {
                default: color_format.clone(),
            },
            tertiary_container: ColorEntry {
                default: color_format.clone(),
            },
            on_tertiary_container: ColorEntry {
                default: color_format.clone(),
            },
            tertiary_fixed: ColorEntry {
                default: color_format.clone(),
            },
            tertiary_fixed_dim: ColorEntry {
                default: color_format.clone(),
            },
            on_tertiary_fixed: ColorEntry {
                default: color_format.clone(),
            },
            on_tertiary_fixed_variant: ColorEntry {
                default: color_format.clone(),
            },
            error: ColorEntry {
                default: color_format.clone(),
            },
            on_error: ColorEntry {
                default: color_format.clone(),
            },
            error_container: ColorEntry {
                default: color_format.clone(),
            },
            on_error_container: ColorEntry {
                default: color_format.clone(),
            },
            background: ColorEntry {
                default: color_format.clone(),
            },
            on_background: ColorEntry {
                default: color_format.clone(),
            },
            surface: ColorEntry {
                default: color_format.clone(),
            },
            on_surface: ColorEntry {
                default: color_format.clone(),
            },
            surface_variant: ColorEntry {
                default: color_format.clone(),
            },
            on_surface_variant: ColorEntry {
                default: color_format.clone(),
            },
            surface_container_lowest: ColorEntry {
                default: color_format.clone(),
            },
            surface_container_low: ColorEntry {
                default: color_format.clone(),
            },
            surface_container: ColorEntry {
                default: color_format.clone(),
            },
            surface_container_high: ColorEntry {
                default: color_format.clone(),
            },
            surface_container_highest: ColorEntry {
                default: color_format.clone(),
            },
            inverse_surface: ColorEntry {
                default: color_format.clone(),
            },
            inverse_on_surface: ColorEntry {
                default: color_format.clone(),
            },
            inverse_primary: ColorEntry {
                default: color_format.clone(),
            },
            surface_dim: ColorEntry {
                default: color_format.clone(),
            },
            surface_bright: ColorEntry {
                default: color_format.clone(),
            },
            outline: ColorEntry {
                default: color_format.clone(),
            },
            outline_variant: ColorEntry {
                default: color_format.clone(),
            },
            shadow: ColorEntry {
                default: color_format.clone(),
            },
            scrim: ColorEntry {
                default: color_format,
            },
        };

        let template_content = "Primary color: {{colors.primary.default.hex}}, Mode: {{mode}}";
        let result = process_template(template_content, &palette, "dark");

        // The result should contain the expected placeholders replaced
        assert!(result.contains("Primary color: #FF5722"));
        assert!(result.contains(", Mode: dark"));
    }
}
