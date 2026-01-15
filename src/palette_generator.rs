use crate::color;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ColorFormat {
    pub hex: String,
    pub hex_stripped: String,
    pub hex8: String,          // 8-digit hex with alpha (#rrggbbaa)
    pub hex8_stripped: String, // 8-digit hex without # prefix (rrggbbaa)
    pub rgb: String,
    pub rgba: String,
    pub hsl: String,
    pub hsla: String,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: f64, // Changed from u8 (0-255) to f64 (0.0-1.0) for consistency
    pub hue: f64,
    pub saturation: f64,
    pub lightness: f64,
    // Store the original HSL values as they appeared in the source (for consistent formatting)
    pub original_hue: Option<u32>,
    pub original_saturation: Option<u32>,
    pub original_lightness: Option<u32>,
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

    // Terminal colors
    pub black: ColorEntry,
    pub red: ColorEntry,
    pub green: ColorEntry,
    pub yellow: ColorEntry,
    pub blue: ColorEntry,
    pub magenta: ColorEntry,
    pub cyan: ColorEntry,
    pub white: ColorEntry,
    pub bright_black: ColorEntry,
    pub bright_red: ColorEntry,
    pub bright_green: ColorEntry,
    pub bright_yellow: ColorEntry,
    pub bright_blue: ColorEntry,
    pub bright_magenta: ColorEntry,
    pub bright_cyan: ColorEntry,
    pub bright_white: ColorEntry,
}

/// Create a color format from a hex string
fn create_color_format(hex: &str) -> Result<ColorFormat, String> {
    let hex_stripped = hex.trim_start_matches('#');
    let (rgb, alpha) = if hex_stripped.len() == 8 {
        // Handle 8-digit hex (RGBA)
        let r = u8::from_str_radix(&hex_stripped[0..2], 16)
            .map_err(|_| format!("Invalid hex color format: {}", hex_stripped))?;
        let g = u8::from_str_radix(&hex_stripped[2..4], 16)
            .map_err(|_| format!("Invalid hex color format: {}", hex_stripped))?;
        let b = u8::from_str_radix(&hex_stripped[4..6], 16)
            .map_err(|_| format!("Invalid hex color format: {}", hex_stripped))?;
        let a = u8::from_str_radix(&hex_stripped[6..8], 16)
            .map_err(|_| format!("Invalid hex color format: {}", hex_stripped))?;
        (color::Rgb { r, g, b }, a as f64 / 255.0) // Convert to 0.0-1.0 range
    } else {
        // Handle 6-digit hex (RGB) or call the existing function
        let rgb = color::hex_to_rgb(hex)?;
        (rgb, 1.0) // Default to fully opaque (1.0)
    };

    let hsl = color::rgb_to_hsl(rgb.r as f64, rgb.g as f64, rgb.b as f64);

    // Create 8-digit hex formats with uppercase letters to match wallust behavior
    let alpha_byte = (alpha * 255.0).round() as u8;
    let hex8 = format!("#{:02X}{:02X}{:02X}{:02X}", rgb.r, rgb.g, rgb.b, alpha_byte);
    let hex8_stripped = format!("{:02X}{:02X}{:02X}{:02X}", rgb.r, rgb.g, rgb.b, alpha_byte);

    // Round the HSL values to integers for consistent formatting
    let h_int = hsl.h.round() as u32;
    let s_int = hsl.s.round() as u32;
    let l_int = hsl.l.round() as u32;

    // Create uppercase hex formats to match wallust behavior
    let hex_upper = format!("#{:02X}{:02X}{:02X}", rgb.r, rgb.g, rgb.b);
    let hex_stripped_upper = format!("{:02X}{:02X}{:02X}", rgb.r, rgb.g, rgb.b);

    Ok(ColorFormat {
        hex: hex_upper,
        hex_stripped: hex_stripped_upper,
        hex8,
        hex8_stripped,
        rgb: format!("rgb({}, {}, {})", rgb.r, rgb.g, rgb.b),
        rgba: format!("rgba({}, {}, {}, {:.1})", rgb.r, rgb.g, rgb.b, alpha),
        hsl: format!(
            "hsl({}, {}%, {}%)",
            h_int % 360,
            s_int.min(100),
            l_int.min(100)
        ),
        hsla: format!(
            "hsla({}, {}%, {}%, {:.1})",
            h_int % 360,
            s_int.min(100),
            l_int.min(100),
            alpha
        ),
        red: rgb.r,
        green: rgb.g,
        blue: rgb.b,
        alpha, // Now stored as f64 in 0.0-1.0 range
        hue: hsl.h,
        saturation: hsl.s,
        lightness: hsl.l,
        // Store original HSL values for consistent formatting
        original_hue: Some(h_int),
        original_saturation: Some(s_int),
        original_lightness: Some(l_int),
    })
}

/// Generate color palette from theme data using HCT (Hue-Chroma-Tone) color space
pub fn generate_palette(
    theme: &Value,
    is_dark_mode: bool,
    _is_strict: bool,
) -> Result<Palette, String> {
    // Use default algorithm parameters for now
    let alg_params = AlgorithmParameters {
        contrast_threshold: 0.15,
        saturation_adjustment: 0,
        lightness_adjustment: 0,
        hue_shift: 0,
        min_contrast_ratio: 4.5,
    };

    generate_palette_with_params(theme, is_dark_mode, alg_params)
}

/// Parameters for controlling the color generation algorithm
#[derive(Debug, Clone)]
pub struct AlgorithmParameters {
    /// Color contrast threshold (0.0-1.0)
    pub contrast_threshold: f64,
    /// Saturation adjustment (-100 to 100)
    pub saturation_adjustment: i8,
    /// Lightness adjustment (-100 to 100)
    pub lightness_adjustment: i8,
    /// Hue shift (-180 to 180)
    pub hue_shift: i16,
    /// Minimum contrast ratio for readability
    pub min_contrast_ratio: f64,
}

/// Generate color palette from theme data using HCT (Hue-Chroma-Tone) color space with algorithm parameters
pub fn generate_palette_with_params(
    theme: &Value,
    is_dark_mode: bool,
    params: AlgorithmParameters,
) -> Result<Palette, String> {
    if crate::log::is_verbose() {
        eprintln!("Generating color palette with algorithm parameters...");
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
    let mut primary_hct = color::rgb_to_hct(primary_rgb.r, primary_rgb.g, primary_rgb.b);
    // Apply algorithm parameters to primary
    primary_hct = apply_algorithm_params(primary_hct, &params);

    // Convert hex to HCT for secondary and tertiary
    let secondary_rgb = color::hex_to_rgb(secondary_hex)?;
    let mut secondary_hct = color::rgb_to_hct(secondary_rgb.r, secondary_rgb.g, secondary_rgb.b);
    // Apply algorithm parameters to secondary
    secondary_hct = apply_algorithm_params(secondary_hct, &params);

    let tertiary_rgb = color::hex_to_rgb(tertiary_hex)?;
    let mut tertiary_hct = color::rgb_to_hct(tertiary_rgb.r, tertiary_rgb.g, tertiary_rgb.b);
    // Apply algorithm parameters to tertiary
    tertiary_hct = apply_algorithm_params(tertiary_hct, &params);

    let error_rgb = color::hex_to_rgb(error_hex)?;
    let mut error_hct = color::rgb_to_hct(error_rgb.r, error_rgb.g, error_rgb.b);
    // Apply algorithm parameters to error
    error_hct = apply_algorithm_params(error_hct, &params);

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
        primary_hct.c * 0.4,                    // Much less chroma
        if is_dark_mode { 30.0 } else { 90.0 }, // Lower tone for container
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
        if is_dark_mode { 20.0 } else { 95.0 },
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
        if is_dark_mode { 25.0 } else { 95.0 },
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
                .unwrap_or_else(|| create_color_format("#e0e0e0"))? // Light text on dark surface
        } else {
            theme
                .get("on_surface")
                .and_then(|v| v.as_str())
                .or_else(|| theme.get("mOnSurface").and_then(|v| v.as_str()))
                .map(create_color_format)
                .unwrap_or_else(|| create_color_format("#1f1f1f"))? // Dark text on light surface
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
            create_color_format("#e0e0e0")? // Light text on dark surface
        } else {
            create_color_format("#1f1f1f")? // Dark text on light surface
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
            (surface_hct.h + 15.0) % 360.0, // Slight hue shift from actual surface
            5.0,
            if is_dark_mode { 10.0 } else { 94.0 },
        );
        let surface_variant = create_color_format(&surface_variant_hct.to_hex())?;
        let on_surface_variant = if is_dark_mode {
            create_color_format("#c4c4c4")?
        } else {
            create_color_format("#49454f")?
        };
        (surface_variant, on_surface_variant)
    };

    // Helper function to calculate container tone based on base tone and theme
    fn container_tone(base_tone: f64, level: u8, is_dark: bool) -> f64 {
        let step = if is_dark { 2.0 } else { 4.0 };
        let tone = if is_dark {
            base_tone + step * level as f64
        } else {
            base_tone - step * level as f64
        };
        color::clamp(tone, 4.0, 100.0)
    }

    // Surface container colors (different tones for hierarchy)
    // Using surface_hct as base instead of primary_hct for better consistency
    let surface_container_lowest_hct = color::Hct::from_hct(
        surface_hct.h,
        5.0,
        container_tone(surface_hct.t, 0, is_dark_mode), // lowest level
    );
    let surface_container_low_hct = color::Hct::from_hct(
        surface_hct.h,
        5.0,
        container_tone(surface_hct.t, 1, is_dark_mode), // low level
    );
    let surface_container_hct = color::Hct::from_hct(
        surface_hct.h,
        5.0,
        container_tone(surface_hct.t, 2, is_dark_mode), // medium level
    );
    let surface_container_high_hct = color::Hct::from_hct(
        surface_hct.h,
        5.0,
        container_tone(surface_hct.t, 3, is_dark_mode), // high level
    );
    let surface_container_highest_hct = color::Hct::from_hct(
        surface_hct.h,
        5.0,
        container_tone(surface_hct.t, 4, is_dark_mode), // highest level
    );

    let surface_container_lowest = create_color_format(&surface_container_lowest_hct.to_hex())?;
    let surface_container_low = create_color_format(&surface_container_low_hct.to_hex())?;
    let surface_container = create_color_format(&surface_container_hct.to_hex())?;
    let surface_container_high = create_color_format(&surface_container_high_hct.to_hex())?;
    let surface_container_highest = create_color_format(&surface_container_highest_hct.to_hex())?;

    // Fixed accent colors (maintain consistent appearance across themes)
    // According to MD3 spec, fixed colors should maintain readability in both themes
    // but should still vary based on the source color to preserve visual identity

    // Primary fixed colors - preserve source color tone information while ensuring readability
    let min_chroma = 12.0;
    let base_chroma = if primary_hct.c > min_chroma {
        primary_hct.c
    } else {
        min_chroma
    };
    let primary_fixed_tone = color::clamp(primary_hct.t * 0.8 + 18.0, 20.0, 90.0); // Clamp tone to readable range
    let primary_fixed_hct = color::Hct::from_hct(
        primary_hct.h,
        base_chroma * 0.9,  // Moderate reduction in chroma
        primary_fixed_tone, // Preserve original tone information with adjustment for readability
    );
    let primary_fixed_dim_tone = color::clamp(primary_hct.t * 0.7 + 25.0, 20.0, 90.0); // Clamp tone to readable range
    let primary_fixed_dim_hct = color::Hct::from_hct(
        primary_hct.h,
        if primary_hct.c > 8.0 {
            primary_hct.c * 0.7
        } else {
            8.0
        }, // Maintain minimum chroma
        primary_fixed_dim_tone, // Preserve original tone information with adjustment
    );
    let primary_fixed = create_color_format(&primary_fixed_hct.to_hex())?;
    let primary_fixed_dim = create_color_format(&primary_fixed_dim_hct.to_hex())?;

    // Generate appropriate text colors for fixed colors with more flexibility
    let on_primary_fixed = {
        // Allow for more flexible contrast ratios and potentially gray text colors
        let fixed_color_hex = primary_fixed_hct.to_hex();
        let on_color_hex = color::generate_on_color(&fixed_color_hex, is_dark_mode)?;
        create_color_format(&on_color_hex)?
    };
    let on_primary_fixed_variant = {
        // Generate variant with slight hue shift for distinction while maintaining harmony
        let shifted_hue = (primary_hct.h + 20.0) % 360.0; // Small hue shift for distinction
        let variant_tone = if primary_hct.t > 60.0 { 45.0 } else { 65.0 }; // Adjust based on source brightness
        let base_hct = color::Hct::from_hct(
            shifted_hue,
            if primary_hct.c > 8.0 {
                primary_hct.c * 0.6
            } else {
                8.0
            }, // Maintain minimum chroma
            variant_tone,
        );
        create_color_format(&base_hct.to_hex())?
    };

    // Secondary fixed colors - preserve source color tone information while ensuring readability
    let secondary_base_chroma = if secondary_hct.c > min_chroma {
        secondary_hct.c
    } else {
        min_chroma
    };
    let secondary_fixed_tone = color::clamp(secondary_hct.t * 0.8 + 18.0, 20.0, 90.0); // Clamp tone to readable range
    let secondary_fixed_hct = color::Hct::from_hct(
        secondary_hct.h,
        secondary_base_chroma * 0.9, // Moderate reduction in chroma
        secondary_fixed_tone, // Preserve original tone information with adjustment for readability
    );
    let secondary_fixed_dim_tone = color::clamp(secondary_hct.t * 0.7 + 25.0, 20.0, 90.0); // Clamp tone to readable range
    let secondary_fixed_dim_hct = color::Hct::from_hct(
        secondary_hct.h,
        if secondary_hct.c > 8.0 {
            secondary_hct.c * 0.7
        } else {
            8.0
        }, // Maintain minimum chroma
        secondary_fixed_dim_tone, // Preserve original tone information with adjustment
    );
    let secondary_fixed = create_color_format(&secondary_fixed_hct.to_hex())?;
    let secondary_fixed_dim = create_color_format(&secondary_fixed_dim_hct.to_hex())?;

    let on_secondary_fixed = {
        // Allow for more flexible contrast ratios and potentially gray text colors
        let fixed_color_hex = secondary_fixed_hct.to_hex();
        let on_color_hex = color::generate_on_color(&fixed_color_hex, is_dark_mode)?;
        create_color_format(&on_color_hex)?
    };
    let on_secondary_fixed_variant = {
        // Generate variant with slight hue shift for distinction while maintaining harmony
        let shifted_hue = (secondary_hct.h + 20.0) % 360.0; // Small hue shift for distinction
        let variant_tone = if secondary_hct.t > 60.0 { 45.0 } else { 65.0 }; // Adjust based on source brightness
        let base_hct = color::Hct::from_hct(
            shifted_hue,
            if secondary_hct.c > 8.0 {
                secondary_hct.c * 0.6
            } else {
                8.0
            }, // Maintain minimum chroma
            variant_tone,
        );
        create_color_format(&base_hct.to_hex())?
    };

    // Tertiary fixed colors - preserve source color tone information while ensuring readability
    let tertiary_base_chroma = if tertiary_hct.c > min_chroma {
        tertiary_hct.c
    } else {
        min_chroma
    };
    let tertiary_fixed_tone = color::clamp(tertiary_hct.t * 0.8 + 18.0, 20.0, 90.0); // Clamp tone to readable range
    let tertiary_fixed_hct = color::Hct::from_hct(
        tertiary_hct.h,
        tertiary_base_chroma * 0.9, // Moderate reduction in chroma
        tertiary_fixed_tone, // Preserve original tone information with adjustment for readability
    );
    let tertiary_fixed_dim_tone = color::clamp(tertiary_hct.t * 0.7 + 25.0, 20.0, 90.0); // Clamp tone to readable range
    let tertiary_fixed_dim_hct = color::Hct::from_hct(
        tertiary_hct.h,
        if tertiary_hct.c > 8.0 {
            tertiary_hct.c * 0.7
        } else {
            8.0
        }, // Maintain minimum chroma
        tertiary_fixed_dim_tone, // Preserve original tone information with adjustment
    );
    let tertiary_fixed = create_color_format(&tertiary_fixed_hct.to_hex())?;
    let tertiary_fixed_dim = create_color_format(&tertiary_fixed_dim_hct.to_hex())?;

    let on_tertiary_fixed = {
        // Allow for more flexible contrast ratios and potentially gray text colors
        let fixed_color_hex = tertiary_fixed_hct.to_hex();
        let on_color_hex = color::generate_on_color(&fixed_color_hex, is_dark_mode)?;
        create_color_format(&on_color_hex)?
    };
    let on_tertiary_fixed_variant = {
        // Generate variant with slight hue shift for distinction while maintaining harmony
        let shifted_hue = (tertiary_hct.h + 20.0) % 360.0; // Small hue shift for distinction
        let variant_tone = if tertiary_hct.t > 60.0 { 45.0 } else { 65.0 }; // Adjust based on source brightness
        let base_hct = color::Hct::from_hct(
            shifted_hue,
            if tertiary_hct.c > 8.0 {
                tertiary_hct.c * 0.6
            } else {
                8.0
            }, // Maintain minimum chroma
            variant_tone,
        );
        create_color_format(&base_hct.to_hex())?
    };

    // Inverse colors - based on surface color but inverted
    let inverse_surface_hct = color::Hct::from_hct(
        surface_hct.h,
        surface_hct.c,
        if is_dark_mode { 90.0 } else { 20.0 }, // Opposite tone of surface
    );
    let inverse_surface = create_color_format(&inverse_surface_hct.to_hex())?;

    // Generate appropriate text color for inverse surface
    let inverse_on_surface = {
        let inv_surf_hex = inverse_surface_hct.to_hex();
        let on_color_hex = color::generate_on_color(&inv_surf_hex, is_dark_mode)?;
        create_color_format(&on_color_hex)?
    };

    // Inverse primary is based on the primary color but inverted in tone
    let inverse_primary_hct = color::Hct::from_hct(
        primary_hct.h,                          // Same hue as primary
        primary_hct.c,                          // Same chroma as primary
        if is_dark_mode { 40.0 } else { 80.0 }, // Different tone for contrast
    );
    let inverse_primary = create_color_format(&inverse_primary_hct.to_hex())?;

    // Bright and dim surface colors
    let surface_dim_hct = color::Hct::from_hct(
        surface_hct.h,
        surface_hct.c,
        if is_dark_mode { 6.0 } else { 87.0 },
    );
    let surface_bright_hct = color::Hct::from_hct(
        surface_hct.h,
        surface_hct.c,
        if is_dark_mode { 24.0 } else { 100.0 },
    );
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
            .unwrap_or_else(|| create_color_format("#410002"))? // Dark text on light error
    } else {
        theme
            .get("on_error")
            .and_then(|v| v.as_str())
            .or_else(|| theme.get("mOnError").and_then(|v| v.as_str()))
            .map(create_color_format)
            .unwrap_or_else(|| create_color_format("#ffffff"))? // Light text on dark error
    };

    let error_container_hct =
        color::Hct::from_hct(error_hct.h, 30.0, if is_dark_mode { 30.0 } else { 95.0 });
    let error_container = create_color_format(&error_container_hct.to_hex())?;
    let on_error_container = if is_dark_mode {
        create_color_format("#ffdad6")? // Light text on dark error container
    } else {
        create_color_format("#410002")? // Dark text on light error container
    };

    // Outline colors - try to use mOutline if available
    let outline = theme
        .get("outline")
        .and_then(|v| v.as_str())
        .or_else(|| theme.get("mOutline").and_then(|v| v.as_str()))
        .map(create_color_format)
        .unwrap_or_else(|| {
            let outline_hct =
                color::Hct::from_hct(surface_hct.h, 10.0, if is_dark_mode { 60.0 } else { 50.0 });
            create_color_format(&outline_hct.to_hex())
        })?;

    let outline_variant = {
        let outline_variant_hct =
            color::Hct::from_hct(surface_hct.h, 5.0, if is_dark_mode { 30.0 } else { 80.0 });
        create_color_format(&outline_variant_hct.to_hex())?
    };

    // Other colors
    let shadow = theme
        .get("shadow")
        .and_then(|v| v.as_str())
        .or_else(|| theme.get("mShadow").and_then(|v| v.as_str()))
        .map(create_color_format)
        .unwrap_or_else(|| create_color_format("#000000"))?; // Use mShadow if available, otherwise black

    // Scrim color - should adapt to theme mode for better visual integration
    // Using semi-transparent black for better overlay effect
    let scrim_hex = if is_dark_mode {
        "#00000080" // 50% opacity in dark mode
    } else {
        "#1111114D" // 30% opacity in light mode
    };
    let scrim = create_color_format(scrim_hex)?;

    let palette = Palette {
        primary: ColorEntry {
            default: primary.clone(),
        },
        on_primary: ColorEntry {
            default: on_primary.clone(),
        },
        primary_container: ColorEntry {
            default: primary_container.clone(),
        },
        on_primary_container: ColorEntry {
            default: on_primary_container.clone(),
        },
        primary_fixed: ColorEntry {
            default: primary_fixed.clone(),
        },
        primary_fixed_dim: ColorEntry {
            default: primary_fixed_dim.clone(),
        },
        on_primary_fixed: ColorEntry {
            default: on_primary_fixed.clone(),
        },
        on_primary_fixed_variant: ColorEntry {
            default: on_primary_fixed_variant.clone(),
        },
        secondary: ColorEntry {
            default: secondary.clone(),
        },
        on_secondary: ColorEntry {
            default: on_secondary.clone(),
        },
        secondary_container: ColorEntry {
            default: secondary_container.clone(),
        },
        on_secondary_container: ColorEntry {
            default: on_secondary_container.clone(),
        },
        secondary_fixed: ColorEntry {
            default: secondary_fixed.clone(),
        },
        secondary_fixed_dim: ColorEntry {
            default: secondary_fixed_dim.clone(),
        },
        on_secondary_fixed: ColorEntry {
            default: on_secondary_fixed.clone(),
        },
        on_secondary_fixed_variant: ColorEntry {
            default: on_secondary_fixed_variant.clone(),
        },
        tertiary: ColorEntry {
            default: tertiary.clone(),
        },
        on_tertiary: ColorEntry {
            default: on_tertiary.clone(),
        },
        tertiary_container: ColorEntry {
            default: tertiary_container.clone(),
        },
        on_tertiary_container: ColorEntry {
            default: on_tertiary_container.clone(),
        },
        tertiary_fixed: ColorEntry {
            default: tertiary_fixed.clone(),
        },
        tertiary_fixed_dim: ColorEntry {
            default: tertiary_fixed_dim.clone(),
        },
        on_tertiary_fixed: ColorEntry {
            default: on_tertiary_fixed.clone(),
        },
        on_tertiary_fixed_variant: ColorEntry {
            default: on_tertiary_fixed_variant.clone(),
        },
        error: ColorEntry {
            default: error.clone(),
        },
        on_error: ColorEntry {
            default: on_error.clone(),
        },
        error_container: ColorEntry {
            default: error_container.clone(),
        },
        on_error_container: ColorEntry {
            default: on_error_container.clone(),
        },
        background: ColorEntry {
            default: background.clone(),
        },
        on_background: ColorEntry {
            default: on_background.clone(),
        },
        surface: ColorEntry {
            default: surface.clone(),
        },
        on_surface: ColorEntry {
            default: on_surface.clone(),
        },
        surface_variant: ColorEntry {
            default: surface_variant.clone(),
        },
        on_surface_variant: ColorEntry {
            default: on_surface_variant.clone(),
        },
        surface_container_lowest: ColorEntry {
            default: surface_container_lowest.clone(),
        },
        surface_container_low: ColorEntry {
            default: surface_container_low.clone(),
        },
        surface_container: ColorEntry {
            default: surface_container.clone(),
        },
        surface_container_high: ColorEntry {
            default: surface_container_high.clone(),
        },
        surface_container_highest: ColorEntry {
            default: surface_container_highest.clone(),
        },
        inverse_surface: ColorEntry {
            default: inverse_surface.clone(),
        },
        inverse_on_surface: ColorEntry {
            default: inverse_on_surface.clone(),
        },
        inverse_primary: ColorEntry {
            default: inverse_primary.clone(),
        },
        surface_dim: ColorEntry {
            default: surface_dim.clone(),
        },
        surface_bright: ColorEntry {
            default: surface_bright.clone(),
        },
        outline: ColorEntry {
            default: outline.clone(),
        },
        outline_variant: ColorEntry {
            default: outline_variant.clone(),
        },
        shadow: ColorEntry {
            default: shadow.clone(),
        },
        scrim: ColorEntry {
            default: scrim.clone(),
        },

        // Terminal colors - mapping MD3 colors to terminal equivalents
        black: ColorEntry {
            default: surface.clone(),
        }, // Use surface as black
        red: ColorEntry {
            default: error.clone(),
        }, // Use error as red
        green: ColorEntry {
            default: tertiary.clone(),
        }, // Use tertiary as green
        yellow: ColorEntry {
            default: primary.clone(),
        }, // Use primary as yellow
        blue: ColorEntry {
            default: secondary.clone(),
        }, // Use secondary as blue
        magenta: ColorEntry {
            default: primary_container.clone(),
        }, // Use primary container as magenta
        cyan: ColorEntry {
            default: secondary_container.clone(),
        }, // Use secondary container as cyan
        white: ColorEntry {
            default: on_surface.clone(),
        }, // Use on_surface as white
        bright_black: ColorEntry {
            default: surface_variant.clone(),
        }, // Use surface variant as bright black
        bright_red: ColorEntry {
            default: error_container.clone(),
        }, // Use error container as bright red
        bright_green: ColorEntry {
            default: tertiary_container.clone(),
        }, // Use tertiary container as bright green
        bright_yellow: ColorEntry {
            default: primary_fixed.clone(),
        }, // Use primary fixed as bright yellow
        bright_blue: ColorEntry {
            default: secondary_fixed.clone(),
        }, // Use secondary fixed as bright blue
        bright_magenta: ColorEntry {
            default: primary_fixed_dim.clone(),
        }, // Use primary fixed dim as bright magenta
        bright_cyan: ColorEntry {
            default: secondary_fixed_dim.clone(),
        }, // Use secondary fixed dim as bright cyan
        bright_white: ColorEntry {
            default: inverse_surface.clone(),
        }, // Use inverse surface as bright white
    };

    if crate::log::is_verbose() {
        eprintln!("Color palette generated successfully");
    }
    Ok(palette)
}

/// Apply algorithm parameters to an HCT color
fn apply_algorithm_params(mut hct: color::Hct, params: &AlgorithmParameters) -> color::Hct {
    // Apply hue shift
    hct.h = (hct.h + params.hue_shift as f64) % 360.0;
    if hct.h < 0.0 {
        hct.h += 360.0;
    }

    // Apply saturation adjustment
    hct.c = color::clamp(hct.c + params.saturation_adjustment as f64, 0.0, 200.0);

    // Apply lightness adjustment
    hct.t = color::clamp(hct.t + params.lightness_adjustment as f64, 0.0, 100.0);

    hct
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_create_color_format() {
        let color_format = create_color_format("#FF5722").unwrap();
        assert_eq!(color_format.hex, "#FF5722");
        assert_eq!(color_format.hex_stripped, "FF5722");
        assert_eq!(color_format.rgb, "rgb(255, 87, 34)");
        assert_eq!(color_format.red, 255);
        assert_eq!(color_format.green, 87);
        assert_eq!(color_format.blue, 34);
    }

    #[test]
    fn test_generate_palette() {
        let theme = json!({
            "primary": "#FF5722",
            "secondary": "#607D8B",
            "tertiary": "#9C27B0",
            "error": "#F44336",
            "surface": "#FAFAFA",
            "on_surface": "#212121"
        });

        let palette = generate_palette(&theme, false, false).unwrap();

        // Test that primary color exists and has expected structure
        assert!(!palette.primary.default.hex.is_empty());
        assert!(!palette.primary.default.rgb.is_empty());
        assert!(!palette.primary.default.hsl.is_empty());

        // Test that other colors were generated
        assert!(!palette.secondary.default.hex.is_empty());
        assert!(!palette.tertiary.default.hex.is_empty());
        assert!(!palette.error.default.hex.is_empty());
        assert!(!palette.surface.default.hex.is_empty());
        assert!(!palette.on_surface.default.hex.is_empty());
    }
}
