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

#[derive(Debug, Clone)]
pub struct ColorEntry {
    pub default: ColorFormat,
    pub dark: ColorFormat,
    pub light: ColorFormat,
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

impl Default for AlgorithmParameters {
    fn default() -> Self {
        Self {
            contrast_threshold: 0.15,
            saturation_adjustment: 0,
            lightness_adjustment: 0,
            hue_shift: 0,
            min_contrast_ratio: 4.5,
        }
    }
}

/// Helper struct to store single-mode palette data
#[derive(Debug, Clone)]
struct SingleModePalette {
    primary: ColorFormat,
    on_primary: ColorFormat,
    primary_container: ColorFormat,
    on_primary_container: ColorFormat,
    primary_fixed: ColorFormat,
    primary_fixed_dim: ColorFormat,
    on_primary_fixed: ColorFormat,
    on_primary_fixed_variant: ColorFormat,

    secondary: ColorFormat,
    on_secondary: ColorFormat,
    secondary_container: ColorFormat,
    on_secondary_container: ColorFormat,
    secondary_fixed: ColorFormat,
    secondary_fixed_dim: ColorFormat,
    on_secondary_fixed: ColorFormat,
    on_secondary_fixed_variant: ColorFormat,

    tertiary: ColorFormat,
    on_tertiary: ColorFormat,
    tertiary_container: ColorFormat,
    on_tertiary_container: ColorFormat,
    tertiary_fixed: ColorFormat,
    tertiary_fixed_dim: ColorFormat,
    on_tertiary_fixed: ColorFormat,
    on_tertiary_fixed_variant: ColorFormat,

    error: ColorFormat,
    on_error: ColorFormat,
    error_container: ColorFormat,
    on_error_container: ColorFormat,

    background: ColorFormat,
    on_background: ColorFormat,
    surface: ColorFormat,
    on_surface: ColorFormat,
    surface_variant: ColorFormat,
    on_surface_variant: ColorFormat,

    surface_container_lowest: ColorFormat,
    surface_container_low: ColorFormat,
    surface_container: ColorFormat,
    surface_container_high: ColorFormat,
    surface_container_highest: ColorFormat,

    inverse_surface: ColorFormat,
    inverse_on_surface: ColorFormat,
    inverse_primary: ColorFormat,

    surface_dim: ColorFormat,
    surface_bright: ColorFormat,

    outline: ColorFormat,
    outline_variant: ColorFormat,

    shadow: ColorFormat,
    scrim: ColorFormat,

    // Terminal colors
    black: ColorFormat,
    red: ColorFormat,
    green: ColorFormat,
    yellow: ColorFormat,
    blue: ColorFormat,
    magenta: ColorFormat,
    cyan: ColorFormat,
    white: ColorFormat,
    bright_black: ColorFormat,
    bright_red: ColorFormat,
    bright_green: ColorFormat,
    bright_yellow: ColorFormat,
    bright_blue: ColorFormat,
    bright_magenta: ColorFormat,
    bright_cyan: ColorFormat,
    bright_white: ColorFormat,
}

/// Generate single-mode palette from theme data
/// This is an internal helper function that generates colors for one mode (dark or light)
fn generate_single_mode_palette(
    theme: &Value,
    is_dark_mode: bool,
    params: &AlgorithmParameters,
) -> Result<SingleModePalette, String> {
    if crate::log::is_verbose() {
        eprintln!("Generating single-mode palette (is_dark_mode={})...", is_dark_mode);
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
    primary_hct = apply_algorithm_params(primary_hct, params);

    // Convert hex to HCT for secondary and tertiary
    let secondary_rgb = color::hex_to_rgb(secondary_hex)?;
    let mut secondary_hct = color::rgb_to_hct(secondary_rgb.r, secondary_rgb.g, secondary_rgb.b);
    // Apply algorithm parameters to secondary
    secondary_hct = apply_algorithm_params(secondary_hct, params);

    let tertiary_rgb = color::hex_to_rgb(tertiary_hex)?;
    let mut tertiary_hct = color::rgb_to_hct(tertiary_rgb.r, tertiary_rgb.g, tertiary_rgb.b);
    // Apply algorithm parameters to tertiary
    tertiary_hct = apply_algorithm_params(tertiary_hct, params);

    let error_rgb = color::hex_to_rgb(error_hex)?;
    let mut error_hct = color::rgb_to_hct(error_rgb.r, error_rgb.g, error_rgb.b);
    // Apply algorithm parameters to error
    error_hct = apply_algorithm_params(error_hct, params);

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

    // Build and return single-mode palette
    Ok(SingleModePalette {
        primary: primary.clone(),
        on_primary: on_primary.clone(),
        primary_container: primary_container.clone(),
        on_primary_container: on_primary_container.clone(),
        primary_fixed: primary_fixed.clone(),
        primary_fixed_dim: primary_fixed_dim.clone(),
        on_primary_fixed: on_primary_fixed.clone(),
        on_primary_fixed_variant: on_primary_fixed_variant.clone(),

        secondary: secondary.clone(),
        on_secondary: on_secondary.clone(),
        secondary_container: secondary_container.clone(),
        on_secondary_container: on_secondary_container.clone(),
        secondary_fixed: secondary_fixed.clone(),
        secondary_fixed_dim: secondary_fixed_dim.clone(),
        on_secondary_fixed: on_secondary_fixed.clone(),
        on_secondary_fixed_variant: on_secondary_fixed_variant.clone(),

        tertiary: tertiary.clone(),
        on_tertiary: on_tertiary.clone(),
        tertiary_container: tertiary_container.clone(),
        on_tertiary_container: on_tertiary_container.clone(),
        tertiary_fixed: tertiary_fixed.clone(),
        tertiary_fixed_dim: tertiary_fixed_dim.clone(),
        on_tertiary_fixed: on_tertiary_fixed.clone(),
        on_tertiary_fixed_variant: on_tertiary_fixed_variant.clone(),

        error: error.clone(),
        on_error: on_error.clone(),
        error_container: error_container.clone(),
        on_error_container: on_error_container.clone(),

        background: background.clone(),
        on_background: on_background.clone(),
        surface: surface.clone(),
        on_surface: on_surface.clone(),
        surface_variant: surface_variant.clone(),
        on_surface_variant: on_surface_variant.clone(),

        surface_container_lowest,
        surface_container_low,
        surface_container,
        surface_container_high,
        surface_container_highest,

        inverse_surface: inverse_surface.clone(),
        inverse_on_surface,
        inverse_primary,

        surface_dim,
        surface_bright,

        outline,
        outline_variant,

        shadow,
        scrim,

        // Terminal colors
        black: surface.clone(),
        red: error.clone(),
        green: tertiary.clone(),
        yellow: primary.clone(),
        blue: secondary.clone(),
        magenta: primary_container.clone(),
        cyan: secondary_container.clone(),
        white: on_surface.clone(),
        bright_black: surface_variant.clone(),
        bright_red: error_container.clone(),
        bright_green: tertiary_container.clone(),
        bright_yellow: primary_fixed.clone(),
        bright_blue: secondary_fixed.clone(),
        bright_magenta: primary_fixed_dim.clone(),
        bright_cyan: secondary_fixed_dim.clone(),
        bright_white: inverse_surface.clone(),
    })
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

    // Generate palettes for both dark and light modes
    let dark_palette = generate_single_mode_palette(theme, true, &params)?;
    let light_palette = generate_single_mode_palette(theme, false, &params)?;

    // Build the final palette with both modes
    let palette = Palette {
        primary: ColorEntry {
            default: if is_dark_mode { dark_palette.primary.clone() } else { light_palette.primary.clone() },
            dark: dark_palette.primary.clone(),
            light: light_palette.primary.clone(),
        },
        on_primary: ColorEntry {
            default: if is_dark_mode { dark_palette.on_primary.clone() } else { light_palette.on_primary.clone() },
            dark: dark_palette.on_primary.clone(),
            light: light_palette.on_primary.clone(),
        },
        primary_container: ColorEntry {
            default: if is_dark_mode { dark_palette.primary_container.clone() } else { light_palette.primary_container.clone() },
            dark: dark_palette.primary_container.clone(),
            light: light_palette.primary_container.clone(),
        },
        on_primary_container: ColorEntry {
            default: if is_dark_mode { dark_palette.on_primary_container.clone() } else { light_palette.on_primary_container.clone() },
            dark: dark_palette.on_primary_container.clone(),
            light: light_palette.on_primary_container.clone(),
        },
        primary_fixed: ColorEntry {
            default: if is_dark_mode { dark_palette.primary_fixed.clone() } else { light_palette.primary_fixed.clone() },
            dark: dark_palette.primary_fixed.clone(),
            light: light_palette.primary_fixed.clone(),
        },
        primary_fixed_dim: ColorEntry {
            default: if is_dark_mode { dark_palette.primary_fixed_dim.clone() } else { light_palette.primary_fixed_dim.clone() },
            dark: dark_palette.primary_fixed_dim.clone(),
            light: light_palette.primary_fixed_dim.clone(),
        },
        on_primary_fixed: ColorEntry {
            default: if is_dark_mode { dark_palette.on_primary_fixed.clone() } else { light_palette.on_primary_fixed.clone() },
            dark: dark_palette.on_primary_fixed.clone(),
            light: light_palette.on_primary_fixed.clone(),
        },
        on_primary_fixed_variant: ColorEntry {
            default: if is_dark_mode { dark_palette.on_primary_fixed_variant.clone() } else { light_palette.on_primary_fixed_variant.clone() },
            dark: dark_palette.on_primary_fixed_variant.clone(),
            light: light_palette.on_primary_fixed_variant.clone(),
        },

        secondary: ColorEntry {
            default: if is_dark_mode { dark_palette.secondary.clone() } else { light_palette.secondary.clone() },
            dark: dark_palette.secondary.clone(),
            light: light_palette.secondary.clone(),
        },
        on_secondary: ColorEntry {
            default: if is_dark_mode { dark_palette.on_secondary.clone() } else { light_palette.on_secondary.clone() },
            dark: dark_palette.on_secondary.clone(),
            light: light_palette.on_secondary.clone(),
        },
        secondary_container: ColorEntry {
            default: if is_dark_mode { dark_palette.secondary_container.clone() } else { light_palette.secondary_container.clone() },
            dark: dark_palette.secondary_container.clone(),
            light: light_palette.secondary_container.clone(),
        },
        on_secondary_container: ColorEntry {
            default: if is_dark_mode { dark_palette.on_secondary_container.clone() } else { light_palette.on_secondary_container.clone() },
            dark: dark_palette.on_secondary_container.clone(),
            light: light_palette.on_secondary_container.clone(),
        },
        secondary_fixed: ColorEntry {
            default: if is_dark_mode { dark_palette.secondary_fixed.clone() } else { light_palette.secondary_fixed.clone() },
            dark: dark_palette.secondary_fixed.clone(),
            light: light_palette.secondary_fixed.clone(),
        },
        secondary_fixed_dim: ColorEntry {
            default: if is_dark_mode { dark_palette.secondary_fixed_dim.clone() } else { light_palette.secondary_fixed_dim.clone() },
            dark: dark_palette.secondary_fixed_dim.clone(),
            light: light_palette.secondary_fixed_dim.clone(),
        },
        on_secondary_fixed: ColorEntry {
            default: if is_dark_mode { dark_palette.on_secondary_fixed.clone() } else { light_palette.on_secondary_fixed.clone() },
            dark: dark_palette.on_secondary_fixed.clone(),
            light: light_palette.on_secondary_fixed.clone(),
        },
        on_secondary_fixed_variant: ColorEntry {
            default: if is_dark_mode { dark_palette.on_secondary_fixed_variant.clone() } else { light_palette.on_secondary_fixed_variant.clone() },
            dark: dark_palette.on_secondary_fixed_variant.clone(),
            light: light_palette.on_secondary_fixed_variant.clone(),
        },

        tertiary: ColorEntry {
            default: if is_dark_mode { dark_palette.tertiary.clone() } else { light_palette.tertiary.clone() },
            dark: dark_palette.tertiary.clone(),
            light: light_palette.tertiary.clone(),
        },
        on_tertiary: ColorEntry {
            default: if is_dark_mode { dark_palette.on_tertiary.clone() } else { light_palette.on_tertiary.clone() },
            dark: dark_palette.on_tertiary.clone(),
            light: light_palette.on_tertiary.clone(),
        },
        tertiary_container: ColorEntry {
            default: if is_dark_mode { dark_palette.tertiary_container.clone() } else { light_palette.tertiary_container.clone() },
            dark: dark_palette.tertiary_container.clone(),
            light: light_palette.tertiary_container.clone(),
        },
        on_tertiary_container: ColorEntry {
            default: if is_dark_mode { dark_palette.on_tertiary_container.clone() } else { light_palette.on_tertiary_container.clone() },
            dark: dark_palette.on_tertiary_container.clone(),
            light: light_palette.on_tertiary_container.clone(),
        },
        tertiary_fixed: ColorEntry {
            default: if is_dark_mode { dark_palette.tertiary_fixed.clone() } else { light_palette.tertiary_fixed.clone() },
            dark: dark_palette.tertiary_fixed.clone(),
            light: light_palette.tertiary_fixed.clone(),
        },
        tertiary_fixed_dim: ColorEntry {
            default: if is_dark_mode { dark_palette.tertiary_fixed_dim.clone() } else { light_palette.tertiary_fixed_dim.clone() },
            dark: dark_palette.tertiary_fixed_dim.clone(),
            light: light_palette.tertiary_fixed_dim.clone(),
        },
        on_tertiary_fixed: ColorEntry {
            default: if is_dark_mode { dark_palette.on_tertiary_fixed.clone() } else { light_palette.on_tertiary_fixed.clone() },
            dark: dark_palette.on_tertiary_fixed.clone(),
            light: light_palette.on_tertiary_fixed.clone(),
        },
        on_tertiary_fixed_variant: ColorEntry {
            default: if is_dark_mode { dark_palette.on_tertiary_fixed_variant.clone() } else { light_palette.on_tertiary_fixed_variant.clone() },
            dark: dark_palette.on_tertiary_fixed_variant.clone(),
            light: light_palette.on_tertiary_fixed_variant.clone(),
        },

        error: ColorEntry {
            default: if is_dark_mode { dark_palette.error.clone() } else { light_palette.error.clone() },
            dark: dark_palette.error.clone(),
            light: light_palette.error.clone(),
        },
        on_error: ColorEntry {
            default: if is_dark_mode { dark_palette.on_error.clone() } else { light_palette.on_error.clone() },
            dark: dark_palette.on_error.clone(),
            light: light_palette.on_error.clone(),
        },
        error_container: ColorEntry {
            default: if is_dark_mode { dark_palette.error_container.clone() } else { light_palette.error_container.clone() },
            dark: dark_palette.error_container.clone(),
            light: light_palette.error_container.clone(),
        },
        on_error_container: ColorEntry {
            default: if is_dark_mode { dark_palette.on_error_container.clone() } else { light_palette.on_error_container.clone() },
            dark: dark_palette.on_error_container.clone(),
            light: light_palette.on_error_container.clone(),
        },
        background: ColorEntry {
            default: if is_dark_mode { dark_palette.background.clone() } else { light_palette.background.clone() },
            dark: dark_palette.background.clone(),
            light: light_palette.background.clone(),
        },
        on_background: ColorEntry {
            default: if is_dark_mode { dark_palette.on_background.clone() } else { light_palette.on_background.clone() },
            dark: dark_palette.on_background.clone(),
            light: light_palette.on_background.clone(),
        },
        surface: ColorEntry {
            default: if is_dark_mode { dark_palette.surface.clone() } else { light_palette.surface.clone() },
            dark: dark_palette.surface.clone(),
            light: light_palette.surface.clone(),
        },
        on_surface: ColorEntry {
            default: if is_dark_mode { dark_palette.on_surface.clone() } else { light_palette.on_surface.clone() },
            dark: dark_palette.on_surface.clone(),
            light: light_palette.on_surface.clone(),
        },
        surface_variant: ColorEntry {
            default: if is_dark_mode { dark_palette.surface_variant.clone() } else { light_palette.surface_variant.clone() },
            dark: dark_palette.surface_variant.clone(),
            light: light_palette.surface_variant.clone(),
        },
        on_surface_variant: ColorEntry {
            default: if is_dark_mode { dark_palette.on_surface_variant.clone() } else { light_palette.on_surface_variant.clone() },
            dark: dark_palette.on_surface_variant.clone(),
            light: light_palette.on_surface_variant.clone(),
        },
        surface_container_lowest: ColorEntry {
            default: if is_dark_mode { dark_palette.surface_container_lowest.clone() } else { light_palette.surface_container_lowest.clone() },
            dark: dark_palette.surface_container_lowest.clone(),
            light: light_palette.surface_container_lowest.clone(),
        },
        surface_container_low: ColorEntry {
            default: if is_dark_mode { dark_palette.surface_container_low.clone() } else { light_palette.surface_container_low.clone() },
            dark: dark_palette.surface_container_low.clone(),
            light: light_palette.surface_container_low.clone(),
        },
        surface_container: ColorEntry {
            default: if is_dark_mode { dark_palette.surface_container.clone() } else { light_palette.surface_container.clone() },
            dark: dark_palette.surface_container.clone(),
            light: light_palette.surface_container.clone(),
        },
        surface_container_high: ColorEntry {
            default: if is_dark_mode { dark_palette.surface_container_high.clone() } else { light_palette.surface_container_high.clone() },
            dark: dark_palette.surface_container_high.clone(),
            light: light_palette.surface_container_high.clone(),
        },
        surface_container_highest: ColorEntry {
            default: if is_dark_mode { dark_palette.surface_container_highest.clone() } else { light_palette.surface_container_highest.clone() },
            dark: dark_palette.surface_container_highest.clone(),
            light: light_palette.surface_container_highest.clone(),
        },
        inverse_surface: ColorEntry {
            default: if is_dark_mode { dark_palette.inverse_surface.clone() } else { light_palette.inverse_surface.clone() },
            dark: dark_palette.inverse_surface.clone(),
            light: light_palette.inverse_surface.clone(),
        },
        inverse_on_surface: ColorEntry {
            default: if is_dark_mode { dark_palette.inverse_on_surface.clone() } else { light_palette.inverse_on_surface.clone() },
            dark: dark_palette.inverse_on_surface.clone(),
            light: light_palette.inverse_on_surface.clone(),
        },
        inverse_primary: ColorEntry {
            default: if is_dark_mode { dark_palette.inverse_primary.clone() } else { light_palette.inverse_primary.clone() },
            dark: dark_palette.inverse_primary.clone(),
            light: light_palette.inverse_primary.clone(),
        },
        surface_dim: ColorEntry {
            default: if is_dark_mode { dark_palette.surface_dim.clone() } else { light_palette.surface_dim.clone() },
            dark: dark_palette.surface_dim.clone(),
            light: light_palette.surface_dim.clone(),
        },
        surface_bright: ColorEntry {
            default: if is_dark_mode { dark_palette.surface_bright.clone() } else { light_palette.surface_bright.clone() },
            dark: dark_palette.surface_bright.clone(),
            light: light_palette.surface_bright.clone(),
        },
        outline: ColorEntry {
            default: if is_dark_mode { dark_palette.outline.clone() } else { light_palette.outline.clone() },
            dark: dark_palette.outline.clone(),
            light: light_palette.outline.clone(),
        },
        outline_variant: ColorEntry {
            default: if is_dark_mode { dark_palette.outline_variant.clone() } else { light_palette.outline_variant.clone() },
            dark: dark_palette.outline_variant.clone(),
            light: light_palette.outline_variant.clone(),
        },
        shadow: ColorEntry {
            default: if is_dark_mode { dark_palette.shadow.clone() } else { light_palette.shadow.clone() },
            dark: dark_palette.shadow.clone(),
            light: light_palette.shadow.clone(),
        },
        scrim: ColorEntry {
            default: if is_dark_mode { dark_palette.scrim.clone() } else { light_palette.scrim.clone() },
            dark: dark_palette.scrim.clone(),
            light: light_palette.scrim.clone(),
        },

        // Terminal colors
        black: ColorEntry {
            default: if is_dark_mode { dark_palette.black.clone() } else { light_palette.black.clone() },
            dark: dark_palette.black.clone(),
            light: light_palette.black.clone(),
        },
        red: ColorEntry {
            default: if is_dark_mode { dark_palette.red.clone() } else { light_palette.red.clone() },
            dark: dark_palette.red.clone(),
            light: light_palette.red.clone(),
        },
        green: ColorEntry {
            default: if is_dark_mode { dark_palette.green.clone() } else { light_palette.green.clone() },
            dark: dark_palette.green.clone(),
            light: light_palette.green.clone(),
        },
        yellow: ColorEntry {
            default: if is_dark_mode { dark_palette.yellow.clone() } else { light_palette.yellow.clone() },
            dark: dark_palette.yellow.clone(),
            light: light_palette.yellow.clone(),
        },
        blue: ColorEntry {
            default: if is_dark_mode { dark_palette.blue.clone() } else { light_palette.blue.clone() },
            dark: dark_palette.blue.clone(),
            light: light_palette.blue.clone(),
        },
        magenta: ColorEntry {
            default: if is_dark_mode { dark_palette.magenta.clone() } else { light_palette.magenta.clone() },
            dark: dark_palette.magenta.clone(),
            light: light_palette.magenta.clone(),
        },
        cyan: ColorEntry {
            default: if is_dark_mode { dark_palette.cyan.clone() } else { light_palette.cyan.clone() },
            dark: dark_palette.cyan.clone(),
            light: light_palette.cyan.clone(),
        },
        white: ColorEntry {
            default: if is_dark_mode { dark_palette.white.clone() } else { light_palette.white.clone() },
            dark: dark_palette.white.clone(),
            light: light_palette.white.clone(),
        },
        bright_black: ColorEntry {
            default: if is_dark_mode { dark_palette.bright_black.clone() } else { light_palette.bright_black.clone() },
            dark: dark_palette.bright_black.clone(),
            light: light_palette.bright_black.clone(),
        },
        bright_red: ColorEntry {
            default: if is_dark_mode { dark_palette.bright_red.clone() } else { light_palette.bright_red.clone() },
            dark: dark_palette.bright_red.clone(),
            light: light_palette.bright_red.clone(),
        },
        bright_green: ColorEntry {
            default: if is_dark_mode { dark_palette.bright_green.clone() } else { light_palette.bright_green.clone() },
            dark: dark_palette.bright_green.clone(),
            light: light_palette.bright_green.clone(),
        },
        bright_yellow: ColorEntry {
            default: if is_dark_mode { dark_palette.bright_yellow.clone() } else { light_palette.bright_yellow.clone() },
            dark: dark_palette.bright_yellow.clone(),
            light: light_palette.bright_yellow.clone(),
        },
        bright_blue: ColorEntry {
            default: if is_dark_mode { dark_palette.bright_blue.clone() } else { light_palette.bright_blue.clone() },
            dark: dark_palette.bright_blue.clone(),
            light: light_palette.bright_blue.clone(),
        },
        bright_magenta: ColorEntry {
            default: if is_dark_mode { dark_palette.bright_magenta.clone() } else { light_palette.bright_magenta.clone() },
            dark: dark_palette.bright_magenta.clone(),
            light: light_palette.bright_magenta.clone(),
        },
        bright_cyan: ColorEntry {
            default: if is_dark_mode { dark_palette.bright_cyan.clone() } else { light_palette.bright_cyan.clone() },
            dark: dark_palette.bright_cyan.clone(),
            light: light_palette.bright_cyan.clone(),
        },
        bright_white: ColorEntry {
            default: if is_dark_mode { dark_palette.bright_white.clone() } else { light_palette.bright_white.clone() },
            dark: dark_palette.bright_white.clone(),
            light: light_palette.bright_white.clone(),
        },
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

    #[test]
    fn test_process_theme_workflow_with_algorithm_params() {
        let theme = json!({
            "primary": "#FF5722",
            "secondary": "#607D8B",
            "tertiary": "#9C27B0",
            "error": "#F44336",
            "surface": "#FAFAFA",
            "on_surface": "#212121"
        });

        let params = AlgorithmParameters {
            contrast_threshold: 0.15,
            saturation_adjustment: 10,
            lightness_adjustment: 5,
            hue_shift: 15,
            min_contrast_ratio: 4.5,
        };

        let palette = generate_palette_with_params(&theme, false, params).unwrap();
        assert!(!palette.primary.default.hex.is_empty());
    }
}