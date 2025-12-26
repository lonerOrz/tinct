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
    // Fixed accent colors
    pub primary_fixed: ColorEntry,
    pub primary_fixed_dim: ColorEntry,
    pub on_primary_fixed: ColorEntry,
    pub on_primary_fixed_variant: ColorEntry,

    pub secondary: ColorEntry,
    pub on_secondary: ColorEntry,
    pub secondary_container: ColorEntry,
    pub on_secondary_container: ColorEntry,
    // Fixed accent colors
    pub secondary_fixed: ColorEntry,
    pub secondary_fixed_dim: ColorEntry,
    pub on_secondary_fixed: ColorEntry,
    pub on_secondary_fixed_variant: ColorEntry,

    pub tertiary: ColorEntry,
    pub on_tertiary: ColorEntry,
    pub tertiary_container: ColorEntry,
    pub on_tertiary_container: ColorEntry,
    // Fixed accent colors
    pub tertiary_fixed: ColorEntry,
    pub tertiary_fixed_dim: ColorEntry,
    pub on_tertiary_fixed: ColorEntry,
    pub on_tertiary_fixed_variant: ColorEntry,

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

    // Surface container colors
    pub surface_container_lowest: ColorEntry,
    pub surface_container_low: ColorEntry,
    pub surface_container: ColorEntry,
    pub surface_container_high: ColorEntry,
    pub surface_container_highest: ColorEntry,

    // Inverse colors
    pub inverse_surface: ColorEntry,
    pub inverse_on_surface: ColorEntry,
    pub inverse_primary: ColorEntry,

    // Bright and dim surface colors
    pub surface_dim: ColorEntry,
    pub surface_bright: ColorEntry,

    // Outline colors
    pub outline: ColorEntry,
    pub outline_variant: ColorEntry,

    // Other colors
    pub shadow: ColorEntry,
    pub scrim: ColorEntry,
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
    if crate::log::is_verbose() {
        eprintln!("Loading theme from {}", theme_path);
    }

    let content = fs::read_to_string(theme_path)
        .map_err(|e| format!("Could not read theme file '{}': {}", theme_path, e))?;

    let theme_data: Value = serde_json::from_str(&content)
        .map_err(|e| format!("Invalid JSON format in '{}': {}", theme_path, e))?;

    if crate::log::is_verbose() {
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

    if crate::log::is_verbose() {
        eprintln!("Template processed successfully");
    }
    content
}

/// Save processed content to output file
pub fn save_output(content: &str, output_path: &str) -> Result<(), String> {
    if crate::log::is_verbose() {
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

    if crate::log::is_verbose() {
        eprintln!("Output saved successfully to {}", output_path.display());
    }
    Ok(())
}

/// Generate color palette from theme data using HCT (Hue-Chroma-Tone) color space
pub fn generate_palette(
    theme: &Value,
    is_dark_mode: bool,
    _is_strict: bool,
) -> Result<Palette, String> {
    if crate::log::is_verbose() {
        eprintln!("Generating color palette...");
    }

    // Get colors from theme - try both standard and m-prefixed keys
    let primary_hex = theme
        .get("primary")
        .and_then(|v| v.as_str())
        .or_else(|| theme.get("mPrimary").and_then(|v| v.as_str()))
        .ok_or("Primary color not found in theme")?;

    let secondary_hex = theme
        .get("secondary")
        .and_then(|v| v.as_str())
        .or_else(|| theme.get("mSecondary").and_then(|v| v.as_str()))
        .unwrap_or(primary_hex); // Fallback to primary if not specified

    let tertiary_hex = theme
        .get("tertiary")
        .and_then(|v| v.as_str())
        .or_else(|| theme.get("mTertiary").and_then(|v| v.as_str()))
        .unwrap_or(secondary_hex); // Fallback to secondary if not specified

    let error_hex = theme
        .get("error")
        .and_then(|v| v.as_str())
        .or_else(|| theme.get("mError").and_then(|v| v.as_str()))
        .unwrap_or("#f44336"); // Standard error color if not specified

    // Try to get surface colors from theme, fallback to generated ones if not available
    let surface_hex = theme
        .get("surface")
        .and_then(|v| v.as_str())
        .or_else(|| theme.get("mSurface").and_then(|v| v.as_str()));

    let surface_variant_hex = theme
        .get("surface_variant")
        .and_then(|v| v.as_str())
        .or_else(|| theme.get("mSurfaceVariant").and_then(|v| v.as_str()));

    // Convert hex to HCT for primary
    let primary_rgb = color::hex_to_rgb(primary_hex)?;
    let primary_hct = color::rgb_to_hct(primary_rgb.r, primary_rgb.g, primary_rgb.b);

    // Convert hex to HCT for secondary and tertiary
    let secondary_rgb = color::hex_to_rgb(secondary_hex)?;
    let secondary_hct = color::rgb_to_hct(secondary_rgb.r, secondary_rgb.g, secondary_rgb.b);

    let tertiary_rgb = color::hex_to_rgb(tertiary_hex)?;
    let tertiary_hct = color::rgb_to_hct(tertiary_rgb.r, tertiary_rgb.g, tertiary_rgb.b);

    let error_rgb = color::hex_to_rgb(error_hex)?;
    let error_hct = color::rgb_to_hct(error_rgb.r, error_rgb.g, error_rgb.b);

    // Create primary colors using HCT
    let primary = create_color_format(&primary_hct.to_hex())?;
    let on_primary = if is_dark_mode {
        // Try to get specific on_primary color, fallback to standard
        theme
            .get("on_primary")
            .and_then(|v| v.as_str())
            .or_else(|| theme.get("mOnPrimary").and_then(|v| v.as_str()))
            .map(create_color_format)
            .unwrap_or_else(|| create_color_format("#ffffff"))
    } else {
        theme
            .get("on_primary")
            .and_then(|v| v.as_str())
            .or_else(|| theme.get("mOnPrimary").and_then(|v| v.as_str()))
            .map(create_color_format)
            .unwrap_or_else(|| create_color_format("#000000"))
    }?;

    // Create secondary and tertiary colors
    let secondary = create_color_format(&secondary_hct.to_hex())?;
    let on_secondary = if is_dark_mode {
        theme
            .get("on_secondary")
            .and_then(|v| v.as_str())
            .or_else(|| theme.get("mOnSecondary").and_then(|v| v.as_str()))
            .map(create_color_format)
            .unwrap_or_else(|| create_color_format("#ffffff"))
    } else {
        theme
            .get("on_secondary")
            .and_then(|v| v.as_str())
            .or_else(|| theme.get("mOnSecondary").and_then(|v| v.as_str()))
            .map(create_color_format)
            .unwrap_or_else(|| create_color_format("#000000"))
    }?;

    let tertiary = create_color_format(&tertiary_hct.to_hex())?;
    let on_tertiary = if is_dark_mode {
        theme
            .get("on_tertiary")
            .and_then(|v| v.as_str())
            .or_else(|| theme.get("mOnTertiary").and_then(|v| v.as_str()))
            .map(create_color_format)
            .unwrap_or_else(|| create_color_format("#ffffff"))
    } else {
        theme
            .get("on_tertiary")
            .and_then(|v| v.as_str())
            .or_else(|| theme.get("mOnTertiary").and_then(|v| v.as_str()))
            .map(create_color_format)
            .unwrap_or_else(|| create_color_format("#000000"))
    }?;

    // Generate container colors (lower chroma, adjusted tone)
    let primary_container_hct = color::Hct::from_hct(
        primary_hct.h,
        primary_hct.c * 0.4,              // Much less chroma
        if is_dark_mode { 30.0 } else { 90.0 }   // Lower tone for container
    );
    let primary_container = create_color_format(&primary_container_hct.to_hex())?;
    let on_primary_container = if is_dark_mode {
        theme
            .get("on_primary_container")
            .and_then(|v| v.as_str())
            .or_else(|| theme.get("mOnPrimary").and_then(|v| v.as_str())) // Use mOnPrimary as fallback
            .map(create_color_format)
            .unwrap_or_else(|| create_color_format("#ffffff"))
    } else {
        theme
            .get("on_primary_container")
            .and_then(|v| v.as_str())
            .or_else(|| theme.get("mOnPrimary").and_then(|v| v.as_str())) // Use mOnPrimary as fallback
            .map(create_color_format)
            .unwrap_or_else(|| create_color_format("#000000"))
    }?;

    let secondary_container_hct = color::Hct::from_hct(
        secondary_hct.h,
        secondary_hct.c * 0.4,
        if is_dark_mode { 20.0 } else { 95.0 }
    );
    let secondary_container = create_color_format(&secondary_container_hct.to_hex())?;
    let on_secondary_container = if is_dark_mode {
        create_color_format("#ffffff")?
    } else {
        create_color_format("#000000")?
    };

    let tertiary_container_hct = color::Hct::from_hct(
        tertiary_hct.h,
        tertiary_hct.c * 0.4,
        if is_dark_mode { 25.0 } else { 95.0 }
    );
    let tertiary_container = create_color_format(&tertiary_container_hct.to_hex())?;
    let on_tertiary_container = if is_dark_mode {
        create_color_format("#ffffff")?
    } else {
        create_color_format("#000000")?
    };

    // Use provided surface colors if available, otherwise generate
    let (surface, on_surface, surface_hct) = if let Some(hex) = surface_hex {
        let surface = create_color_format(hex)?;
        let on_surface = if is_dark_mode {
            theme
                .get("on_surface")
                .and_then(|v| v.as_str())
                .or_else(|| theme.get("mOnSurface").and_then(|v| v.as_str()))
                .map(create_color_format)
                .unwrap_or_else(|| create_color_format("#e0e0e0"))?  // Light text on dark surface
        } else {
            theme
                .get("on_surface")
                .and_then(|v| v.as_str())
                .or_else(|| theme.get("mOnSurface").and_then(|v| v.as_str()))
                .map(create_color_format)
                .unwrap_or_else(|| create_color_format("#1f1f1f"))?  // Dark text on light surface
        };
        // Create HCT from the provided surface color for use in other calculations
        let surface_rgb = color::hex_to_rgb(hex)?;
        let surface_hct = color::rgb_to_hct(surface_rgb.r, surface_rgb.g, surface_rgb.b);
        (surface, on_surface, surface_hct)
    } else {
        // Generate surface colors based on the theme
        let surface_tone = if is_dark_mode { 6.0 } else { 98.0 };
        let surface_hct = color::Hct::from_hct(primary_hct.h, 5.0, surface_tone); // Low chroma for surface
        let surface = create_color_format(&surface_hct.to_hex())?;
        let on_surface = if is_dark_mode {
            create_color_format("#e0e0e0")?  // Light text on dark surface
        } else {
            create_color_format("#1f1f1f")?  // Dark text on light surface
        };
        (surface, on_surface, surface_hct)
    };

    let background = surface.clone();
    let on_background = on_surface.clone();

    // Use provided surface variant color if available, otherwise generate
    let (surface_variant, on_surface_variant) = if let Some(hex) = surface_variant_hex {
        let surface_variant = create_color_format(hex)?;
        let on_surface_variant = if is_dark_mode {
            theme
                .get("on_surface_variant")
                .and_then(|v| v.as_str())
                .or_else(|| theme.get("mOnSurfaceVariant").and_then(|v| v.as_str()))
                .map(create_color_format)
                .unwrap_or_else(|| create_color_format("#c4c4c4"))?
        } else {
            theme
                .get("on_surface_variant")
                .and_then(|v| v.as_str())
                .or_else(|| theme.get("mOnSurfaceVariant").and_then(|v| v.as_str()))
                .map(create_color_format)
                .unwrap_or_else(|| create_color_format("#49454f"))?
        };
        (surface_variant, on_surface_variant)
    } else {
        // Generate surface variant (slightly different hue)
        let surface_variant_hct = color::Hct::from_hct(
            (surface_hct.h + 15.0) % 360.0,  // Slight hue shift from actual surface
            5.0,
            if is_dark_mode { 10.0 } else { 94.0 }
        );
        let surface_variant = create_color_format(&surface_variant_hct.to_hex())?;
        let on_surface_variant = if is_dark_mode {
            create_color_format("#c4c4c4")?
        } else {
            create_color_format("#49454f")?
        };
        (surface_variant, on_surface_variant)
    };

    // Surface container colors (different tones for hierarchy)
    let surface_container_lowest_hct = color::Hct::from_hct(primary_hct.h, 5.0, if is_dark_mode { 4.0 } else { 100.0 });
    let surface_container_low_hct = color::Hct::from_hct(primary_hct.h, 5.0, if is_dark_mode { 6.0 } else { 98.0 });
    let surface_container_hct = color::Hct::from_hct(primary_hct.h, 5.0, if is_dark_mode { 8.0 } else { 96.0 });
    let surface_container_high_hct = color::Hct::from_hct(primary_hct.h, 5.0, if is_dark_mode { 10.0 } else { 92.0 });
    let surface_container_highest_hct = color::Hct::from_hct(primary_hct.h, 5.0, if is_dark_mode { 12.0 } else { 87.0 });

    let surface_container_lowest = create_color_format(&surface_container_lowest_hct.to_hex())?;
    let surface_container_low = create_color_format(&surface_container_low_hct.to_hex())?;
    let surface_container = create_color_format(&surface_container_hct.to_hex())?;
    let surface_container_high = create_color_format(&surface_container_high_hct.to_hex())?;
    let surface_container_highest = create_color_format(&surface_container_highest_hct.to_hex())?;

    // Fixed accent colors (maintain consistent appearance across themes)
    let primary_fixed_hct = color::Hct::from_hct(primary_hct.h, primary_hct.c * 0.9, 90.0);
    let primary_fixed_dim_hct = color::Hct::from_hct(primary_hct.h, primary_hct.c * 0.7, 75.0);
    let primary_fixed = create_color_format(&primary_fixed_hct.to_hex())?;
    let primary_fixed_dim = create_color_format(&primary_fixed_dim_hct.to_hex())?;
    let on_primary_fixed = create_color_format("#000000")?;
    let on_primary_fixed_variant = if is_dark_mode {
        create_color_format("#9a87ff")?  // Based on primary
    } else {
        create_color_format("#43389d")?  // Based on primary
    };

    let secondary_fixed_hct = color::Hct::from_hct(secondary_hct.h, secondary_hct.c * 0.9, 90.0);
    let secondary_fixed_dim_hct = color::Hct::from_hct(secondary_hct.h, secondary_hct.c * 0.7, 75.0);
    let secondary_fixed = create_color_format(&secondary_fixed_hct.to_hex())?;
    let secondary_fixed_dim = create_color_format(&secondary_fixed_dim_hct.to_hex())?;
    let on_secondary_fixed = create_color_format("#000000")?;
    let on_secondary_fixed_variant = if is_dark_mode {
        create_color_format("#67daff")?  // Based on secondary
    } else {
        create_color_format("#006b60")?  // Based on secondary
    };

    let tertiary_fixed_hct = color::Hct::from_hct(tertiary_hct.h, tertiary_hct.c * 0.9, 90.0);
    let tertiary_fixed_dim_hct = color::Hct::from_hct(tertiary_hct.h, tertiary_hct.c * 0.7, 75.0);
    let tertiary_fixed = create_color_format(&tertiary_fixed_hct.to_hex())?;
    let tertiary_fixed_dim = create_color_format(&tertiary_fixed_dim_hct.to_hex())?;
    let on_tertiary_fixed = create_color_format("#000000")?;
    let on_tertiary_fixed_variant = if is_dark_mode {
        create_color_format("#f8c26d")?  // Based on tertiary
    } else {
        create_color_format("#442a51")?  // Based on tertiary
    };

    // Inverse colors
    let inverse_surface_hct = color::Hct::from_hct(surface_hct.h, surface_hct.c, if is_dark_mode { 90.0 } else { 20.0 });
    let inverse_surface = create_color_format(&inverse_surface_hct.to_hex())?;
    let inverse_on_surface = if is_dark_mode {
        create_color_format("#313031")?  // Dark text on light inverse
    } else {
        create_color_format("#e3e1e3")?  // Light text on dark inverse
    };
    let inverse_primary = if is_dark_mode {
        create_color_format("#6200ee")?  // Light theme primary for dark theme inverse
    } else {
        create_color_format("#bb86fc")?  // Dark theme primary for light theme inverse
    };

    // Bright and dim surface colors
    let surface_dim_hct = color::Hct::from_hct(surface_hct.h, surface_hct.c, if is_dark_mode { 6.0 } else { 87.0 });
    let surface_bright_hct = color::Hct::from_hct(surface_hct.h, surface_hct.c, if is_dark_mode { 24.0 } else { 100.0 });
    let surface_dim = create_color_format(&surface_dim_hct.to_hex())?;
    let surface_bright = create_color_format(&surface_bright_hct.to_hex())?;

    // Error colors
    let error = create_color_format(&error_hct.to_hex())?;
    let on_error = if is_dark_mode {
        theme
            .get("on_error")
            .and_then(|v| v.as_str())
            .or_else(|| theme.get("mOnError").and_then(|v| v.as_str()))
            .map(create_color_format)
            .unwrap_or_else(|| create_color_format("#410002"))?  // Dark text on light error
    } else {
        theme
            .get("on_error")
            .and_then(|v| v.as_str())
            .or_else(|| theme.get("mOnError").and_then(|v| v.as_str()))
            .map(create_color_format)
            .unwrap_or_else(|| create_color_format("#ffffff"))?  // Light text on dark error
    };

    let error_container_hct = color::Hct::from_hct(error_hct.h, 30.0, if is_dark_mode { 30.0 } else { 95.0 });
    let error_container = create_color_format(&error_container_hct.to_hex())?;
    let on_error_container = if is_dark_mode {
        create_color_format("#ffdad6")?  // Light text on dark error container
    } else {
        create_color_format("#410002")?  // Dark text on light error container
    };

    // Outline colors - try to use mOutline if available
    let outline = theme
        .get("outline")
        .and_then(|v| v.as_str())
        .or_else(|| theme.get("mOutline").and_then(|v| v.as_str()))
        .map(create_color_format)
        .unwrap_or_else(|| {
            let outline_hct = color::Hct::from_hct(surface_hct.h, 10.0, if is_dark_mode { 60.0 } else { 50.0 });
            create_color_format(&outline_hct.to_hex())
        })?;

    let outline_variant = {
        let outline_variant_hct = color::Hct::from_hct(surface_hct.h, 5.0, if is_dark_mode { 30.0 } else { 80.0 });
        create_color_format(&outline_variant_hct.to_hex())?
    };

    // Other colors
    let shadow = theme
        .get("shadow")
        .and_then(|v| v.as_str())
        .or_else(|| theme.get("mShadow").and_then(|v| v.as_str()))
        .map(create_color_format)
        .unwrap_or_else(|| create_color_format("#000000"))?; // Use mShadow if available, otherwise black

    let scrim = create_color_format("#000000")?; // Always black

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
        primary_fixed: ColorEntry {
            default: primary_fixed,
        },
        primary_fixed_dim: ColorEntry {
            default: primary_fixed_dim,
        },
        on_primary_fixed: ColorEntry {
            default: on_primary_fixed,
        },
        on_primary_fixed_variant: ColorEntry {
            default: on_primary_fixed_variant,
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
        secondary_fixed: ColorEntry {
            default: secondary_fixed,
        },
        secondary_fixed_dim: ColorEntry {
            default: secondary_fixed_dim,
        },
        on_secondary_fixed: ColorEntry {
            default: on_secondary_fixed,
        },
        on_secondary_fixed_variant: ColorEntry {
            default: on_secondary_fixed_variant,
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
        tertiary_fixed: ColorEntry {
            default: tertiary_fixed,
        },
        tertiary_fixed_dim: ColorEntry {
            default: tertiary_fixed_dim,
        },
        on_tertiary_fixed: ColorEntry {
            default: on_tertiary_fixed,
        },
        on_tertiary_fixed_variant: ColorEntry {
            default: on_tertiary_fixed_variant,
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
        inverse_surface: ColorEntry {
            default: inverse_surface,
        },
        inverse_on_surface: ColorEntry {
            default: inverse_on_surface,
        },
        inverse_primary: ColorEntry {
            default: inverse_primary,
        },
        surface_dim: ColorEntry {
            default: surface_dim,
        },
        surface_bright: ColorEntry {
            default: surface_bright,
        },
        outline: ColorEntry { default: outline },
        outline_variant: ColorEntry {
            default: outline_variant,
        },
        shadow: ColorEntry { default: shadow },
        scrim: ColorEntry { default: scrim },
    };

    if crate::log::is_verbose() {
        eprintln!("Color palette generated successfully");
    }
    Ok(palette)
}

/// Generate a harmonious color based on the source color with a hue shift
fn generate_harmonious_color(source_hex: &str, hue_shift: f64, saturation_change: f64) -> Result<ColorFormat, String> {
    let source_rgb = color::hex_to_rgb(source_hex)?;
    let source_hsl = color::rgb_to_hsl(source_rgb.r as f64, source_rgb.g as f64, source_rgb.b as f64);

    // Apply hue shift (keeping within 0-360 range)
    let new_hue = (source_hsl.h + hue_shift) % 360.0;
    let new_hue = if new_hue < 0.0 { new_hue + 360.0 } else { new_hue };

    // Apply saturation change (keeping within 0-100 range)
    let new_saturation = color::clamp(source_hsl.s + saturation_change, 0.0, 100.0);

    let new_rgb = color::hsl_to_rgb(new_hue, new_saturation, source_hsl.l);
    let new_hex = color::rgb_to_hex(new_rgb.r as f64, new_rgb.g as f64, new_rgb.b as f64);

    create_color_format(&new_hex)
}

/// Generate a color with appropriate contrast based on the background
fn generate_contrast_color(background_hex: &str, is_dark_mode: bool) -> Result<ColorFormat, String> {
    let luminance = color::get_luminance(background_hex)?;

    // For dark mode, if background is dark, use light text; if background is light, use dark text
    // For light mode, if background is light, use dark text; if background is dark, use light text
    if (is_dark_mode && luminance < 0.5) || (!is_dark_mode && luminance > 0.5) {
        // Background is dark in dark mode or light in light mode
        // Use opposite text color for contrast
        if color::get_contrast_ratio(background_hex, "#ffffff")? >= 4.5 {
            create_color_format("#ffffff")
        } else {
            // If white doesn't provide enough contrast, use a light gray
            create_color_format("#e6e1e5")
        }
    } else {
        // Background is light in dark mode or dark in light mode
        // Use opposite text color for contrast
        if color::get_contrast_ratio(background_hex, "#000000")? >= 4.5 {
            create_color_format("#000000")
        } else {
            // If black doesn't provide enough contrast, use a dark gray
            create_color_format("#1c1b1f")
        }
    }
}

/// Generate a container color based on the source color and theme
fn generate_container_color(source_hex: &str, is_dark_mode: bool) -> Result<String, String> {
    let source_rgb = color::hex_to_rgb(source_hex)?;
    let source_hsl = color::rgb_to_hsl(source_rgb.r as f64, source_rgb.g as f64, source_rgb.b as f64);

    // Container colors are typically more muted and darker/lighter than the source
    let new_lightness = if is_dark_mode {
        color::clamp(source_hsl.l - 20.0, 0.0, 100.0) // Darker in dark mode
    } else {
        color::clamp(source_hsl.l + 15.0, 0.0, 100.0) // Lighter in light mode
    };

    let new_rgb = color::hsl_to_rgb(source_hsl.h, source_hsl.s, new_lightness);
    Ok(color::rgb_to_hex(new_rgb.r as f64, new_rgb.g as f64, new_rgb.b as f64))
}

/// Adjust the lightness of a color by a given amount
fn adjust_lightness(hexcolor: &str, amount: f64) -> Result<String, String> {
    let rgb = color::hex_to_rgb(hexcolor)?;
    let hsl = color::rgb_to_hsl(rgb.r as f64, rgb.g as f64, rgb.b as f64);
    let new_l = color::clamp(hsl.l + amount, 0.0, 100.0);
    let new_rgb = color::hsl_to_rgb(hsl.h, hsl.s, new_l);
    Ok(color::rgb_to_hex(
        new_rgb.r as f64,
        new_rgb.g as f64,
        new_rgb.b as f64,
    ))
}

/// Adjust both lightness and saturation of a color
fn adjust_lightness_and_saturation(hexcolor: &str, la: f64, sa: f64) -> Result<String, String> {
    let rgb = color::hex_to_rgb(hexcolor)?;
    let hsl = color::rgb_to_hsl(rgb.r as f64, rgb.g as f64, rgb.b as f64);
    let new_l = color::clamp(hsl.l + la, 0.0, 100.0);
    let new_s = color::clamp(hsl.s + sa, 0.0, 100.0);
    let new_rgb = color::hsl_to_rgb(hsl.h, new_s, new_l);
    Ok(color::rgb_to_hex(
        new_rgb.r as f64,
        new_rgb.g as f64,
        new_rgb.b as f64,
    ))
}

/// Process theme - main function to generate theme from JSON and template
pub fn process_theme(
    theme_path: &str,
    template_path: &str,
    output_path: &str,
    mode: &str,
) -> Result<(), String> {
    if crate::log::is_verbose() {
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
    if crate::log::is_verbose() {
        eprintln!("Generating color palette...");
    }
    let palette = generate_palette(&theme, effective_mode == "dark", false)?;
    if crate::log::is_verbose() {
        eprintln!("Color palette generated successfully");
    }

    // Read template
    let template_content = load_template(template_path)?;

    // Replace placeholders
    let result_content = process_template(&template_content, &palette, &effective_mode);

    // Save output
    save_output(&result_content, output_path)?;

    if crate::log::is_verbose() {
        eprintln!(
            "Theme generated successfully! Mode: {}, output: {}",
            effective_mode, output_path
        );
    }
    Ok(())
}
