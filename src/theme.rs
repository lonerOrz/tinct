use crate::color;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ColorFormat {
    pub hex: String,
    pub hex_stripped: String,
    pub rgb: String,
    pub rgba: String,
    pub hsl: String,
    pub hsla: String,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
    pub hue: f64,
    pub saturation: f64,
    pub lightness: f64,
}

#[derive(Debug)]
pub struct ColorEntry {
    pub default: ColorFormat,
}

#[derive(Debug)]
pub struct Palette {
    pub primary: ColorEntry,
    pub on_primary: ColorEntry,
    pub primary_container: ColorEntry,
    pub on_primary_container: ColorEntry,
    pub secondary: ColorEntry,
    pub on_secondary: ColorEntry,
    pub secondary_container: ColorEntry,
    pub on_secondary_container: ColorEntry,
    pub tertiary: ColorEntry,
    pub on_tertiary: ColorEntry,
    pub tertiary_container: ColorEntry,
    pub on_tertiary_container: ColorEntry,
    pub error: ColorEntry,
    pub on_error: ColorEntry,
    pub error_container: ColorEntry,
    pub on_error_container: ColorEntry,
    pub background: ColorEntry,
    pub on_background: ColorEntry,
    pub surface: ColorEntry,
    pub on_surface: ColorEntry,
    pub surface_variant: ColorEntry,
    pub on_surface_variant: ColorEntry,
    pub surface_container_lowest: ColorEntry,
    pub surface_container_low: ColorEntry,
    pub surface_container: ColorEntry,
    pub surface_container_high: ColorEntry,
    pub surface_container_highest: ColorEntry,
    pub outline: ColorEntry,
    pub outline_variant: ColorEntry,
    pub shadow: ColorEntry,
}

/// Create a color format from a hex string
fn create_color_format(hex: &str) -> Result<ColorFormat, String> {
    let rgb = color::hex_to_rgb(hex)?;
    let hsl = color::rgb_to_hsl(rgb.r as f64, rgb.g as f64, rgb.b as f64);

    Ok(ColorFormat {
        hex: hex.to_string(),
        hex_stripped: hex.trim_start_matches('#').to_string(),
        rgb: format!("rgb({}, {}, {})", rgb.r, rgb.g, rgb.b),
        rgba: format!("rgba({}, {}, {}, {})", rgb.r, rgb.g, rgb.b, 255),
        hsl: format!(
            "hsl({}, {}%, {}%)",
            (hsl.h as u32) % 360,
            (hsl.s as u32).min(100),
            (hsl.l as u32).min(100)
        ),
        hsla: format!(
            "hsla({}, {}%, {}%, 1.0)",
            (hsl.h as u32) % 360,
            (hsl.s as u32).min(100),
            (hsl.l as u32).min(100)
        ),
        red: rgb.r,
        green: rgb.g,
        blue: rgb.b,
        alpha: 255,
        hue: hsl.h,
        saturation: hsl.s,
        lightness: hsl.l,
    })
}

/// Load theme JSON file
pub fn load_theme(theme_path: &str) -> Result<Value, String> {
    if super::log::is_verbose() {
        eprintln!("Loading theme from {}", theme_path);
    }

    let content = fs::read_to_string(theme_path)
        .map_err(|e| format!("Could not read theme file '{}': {}", theme_path, e))?;

    let theme_data: Value = serde_json::from_str(&content)
        .map_err(|e| format!("Invalid JSON format in '{}': {}", theme_path, e))?;

    if super::log::is_verbose() {
        eprintln!("Theme loaded successfully from {}", theme_path);
    }
    Ok(theme_data)
}

/// Select theme mode, defaulting to dark if requested mode not found
pub fn select_theme_mode(theme_all: &Value, mode: &str) -> Result<(Value, String), String> {
    if let Some(theme_mode) = theme_all.get(mode) {
        Ok((theme_mode.clone(), mode.to_string()))
    } else {
        eprintln!("Mode '{}' not found in theme.json. Using 'dark'.", mode);
        if let Some(dark_mode) = theme_all.get("dark") {
            Ok((dark_mode.clone(), "dark".to_string()))
        } else {
            Err(
                "Error: 'dark' mode not available in theme.json and requested mode not found."
                    .to_string(),
            )
        }
    }
}

/// Load template file
pub fn load_template(template_path: &str) -> Result<String, String> {
    if super::log::is_verbose() {
        eprintln!("Loading template from {}", template_path);
    }

    let template_content = fs::read_to_string(template_path)
        .map_err(|e| format!("Could not read template file '{}': {}", template_path, e))?;

    if super::log::is_verbose() {
        eprintln!("Template loaded successfully from {}", template_path);
    }
    Ok(template_content)
}

/// Process template by replacing color placeholders and mode placeholders
pub fn process_template(template_content: &str, palette: &Palette, effective_mode: &str) -> String {
    if super::log::is_verbose() {
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
    let color_map: HashMap<&str, &ColorFormat> = [
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

    if super::log::is_verbose() {
        eprintln!("Template processed successfully");
    }
    content
}

/// Save processed content to output file
pub fn save_output(content: &str, output_path: &str) -> Result<(), String> {
    if super::log::is_verbose() {
        eprintln!("Saving output to {}", output_path);
    }

    let output_path = Path::new(output_path);

    // Ensure output directory exists
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Could not create output directory: {}", e))?;
    }

    fs::write(output_path, content).map_err(|e| {
        format!(
            "Could not write to output file '{}': {}",
            output_path.display(),
            e
        )
    })?;

    if super::log::is_verbose() {
        eprintln!("Output saved successfully to {}", output_path.display());
    }
    Ok(())
}

/// Generate color palette from theme data
pub fn generate_palette(
    theme: &Value,
    is_dark_mode: bool,
    _is_strict: bool,
) -> Result<Palette, String> {
    if super::log::is_verbose() {
        eprintln!("Generating color palette...");
    }

    // Get primary color from theme - try both "primary" and "mPrimary" keys
    let primary_hex = theme
        .get("primary")
        .and_then(|v| v.as_str())
        .or_else(|| theme.get("mPrimary").and_then(|v| v.as_str()))
        .ok_or("Primary color not found in theme")?;

    // Create default colors based on the primary
    let primary = create_color_format(primary_hex)?;

    // Generate other colors based on primary (simplified implementation)
    // In a real implementation, you would generate all colors according to Material Design guidelines
    let on_primary = if is_dark_mode {
        create_color_format("#ffffff")? // Light text on dark background
    } else {
        create_color_format("#000000")? // Dark text on light background
    };

    // For other colors, we'll create placeholder implementations
    // In a full implementation, these would be calculated based on MD3 guidelines
    let surface = if is_dark_mode {
        create_color_format("#121212")? // Dark surface
    } else {
        create_color_format("#ffffff")? // Light surface
    };

    let on_surface = if is_dark_mode {
        create_color_format("#e0e0e0")? // Light text on dark surface
    } else {
        create_color_format("#212121")? // Dark text on light surface
    };

    // Placeholder implementations for other colors
    let secondary = create_color_format("#64b5f6")?; // Blue
    let on_secondary = create_color_format("#000000")?;
    let tertiary = create_color_format("#ffcc80")?; // Orange
    let on_tertiary = create_color_format("#000000")?;
    let error = create_color_format("#f44336")?; // Red
    let on_error = create_color_format("#ffffff")?;
    let background = surface.clone();
    let on_background = on_surface.clone();
    let surface_variant = if is_dark_mode {
        create_color_format("#1e1e1e")?
    } else {
        create_color_format("#f5f5f5")?
    };
    let on_surface_variant = if is_dark_mode {
        create_color_format("#b0b0b0")?
    } else {
        create_color_format("#424242")?
    };
    let outline = if is_dark_mode {
        create_color_format("#808080")?
    } else {
        create_color_format("#9e9e9e")?
    };
    let shadow = if is_dark_mode {
        create_color_format("#000000")?
    } else {
        create_color_format("#000000")?
    };

    // Create placeholder implementations for container colors
    let primary_container = create_color_format("#303030")?;
    let on_primary_container = create_color_format("#ffffff")?;
    let secondary_container = create_color_format("#202020")?;
    let on_secondary_container = create_color_format("#ffffff")?;
    let tertiary_container = create_color_format("#302010")?;
    let on_tertiary_container = create_color_format("#ffffff")?;
    let error_container = create_color_format("#400000")?;
    let on_error_container = create_color_format("#ffffff")?;

    // Create placeholder implementations for surface container colors
    let surface_container_lowest = if is_dark_mode {
        create_color_format("#0f0f0f")?
    } else {
        create_color_format("#ffffff")?
    };
    let surface_container_low = if is_dark_mode {
        create_color_format("#171717")?
    } else {
        create_color_format("#fafafa")?
    };
    let surface_container = if is_dark_mode {
        create_color_format("#1a1a1a")?
    } else {
        create_color_format("#f5f5f5")?
    };
    let surface_container_high = if is_dark_mode {
        create_color_format("#202020")?
    } else {
        create_color_format("#f0f0f0")?
    };
    let surface_container_highest = if is_dark_mode {
        create_color_format("#272727")?
    } else {
        create_color_format("#eeeeee")?
    };
    let outline_variant = if is_dark_mode {
        create_color_format("#383838")?
    } else {
        create_color_format("#c5c5c5")?
    };

    let palette = Palette {
        primary: ColorEntry { default: primary },
        on_primary: ColorEntry {
            default: on_primary,
        },
        primary_container: ColorEntry {
            default: primary_container,
        },
        on_primary_container: ColorEntry {
            default: on_primary_container,
        },
        secondary: ColorEntry { default: secondary },
        on_secondary: ColorEntry {
            default: on_secondary,
        },
        secondary_container: ColorEntry {
            default: secondary_container,
        },
        on_secondary_container: ColorEntry {
            default: on_secondary_container,
        },
        tertiary: ColorEntry { default: tertiary },
        on_tertiary: ColorEntry {
            default: on_tertiary,
        },
        tertiary_container: ColorEntry {
            default: tertiary_container,
        },
        on_tertiary_container: ColorEntry {
            default: on_tertiary_container,
        },
        error: ColorEntry { default: error },
        on_error: ColorEntry { default: on_error },
        error_container: ColorEntry {
            default: error_container,
        },
        on_error_container: ColorEntry {
            default: on_error_container,
        },
        background: ColorEntry {
            default: background,
        },
        on_background: ColorEntry {
            default: on_background,
        },
        surface: ColorEntry { default: surface },
        on_surface: ColorEntry {
            default: on_surface,
        },
        surface_variant: ColorEntry {
            default: surface_variant,
        },
        on_surface_variant: ColorEntry {
            default: on_surface_variant,
        },
        surface_container_lowest: ColorEntry {
            default: surface_container_lowest,
        },
        surface_container_low: ColorEntry {
            default: surface_container_low,
        },
        surface_container: ColorEntry {
            default: surface_container,
        },
        surface_container_high: ColorEntry {
            default: surface_container_high,
        },
        surface_container_highest: ColorEntry {
            default: surface_container_highest,
        },
        outline: ColorEntry { default: outline },
        outline_variant: ColorEntry {
            default: outline_variant,
        },
        shadow: ColorEntry { default: shadow },
    };

    if super::log::is_verbose() {
        eprintln!("Color palette generated successfully");
    }
    Ok(palette)
}

/// Process theme - main function to generate theme from JSON and template
pub fn process_theme(
    theme_path: &str,
    template_path: &str,
    output_path: &str,
    mode: &str,
) -> Result<(), String> {
    if super::log::is_verbose() {
        eprintln!("Starting theme generation: mode={}", mode);
    }

    // Validate input files
    if !Path::new(theme_path).exists() {
        return Err(format!("Theme file '{}' does not exist.", theme_path));
    }
    if !Path::new(template_path).exists() {
        return Err(format!("Template file '{}' does not exist.", template_path));
    }

    // Read JSON theme
    let theme_all = load_theme(theme_path)?;
    let (theme, effective_mode) = select_theme_mode(&theme_all, mode)?;

    // Generate palette
    if super::log::is_verbose() {
        eprintln!("Generating color palette...");
    }
    let palette = generate_palette(&theme, effective_mode == "dark", false)?;
    if super::log::is_verbose() {
        eprintln!("Color palette generated successfully");
    }

    // Read template
    let template_content = load_template(template_path)?;

    // Replace placeholders
    let result_content = process_template(&template_content, &palette, &effective_mode);

    // Save output
    save_output(&result_content, output_path)?;

    if super::log::is_verbose() {
        eprintln!(
            "Theme generated successfully! Mode: {}, output: {}",
            effective_mode, output_path
        );
    }
    Ok(())
}
