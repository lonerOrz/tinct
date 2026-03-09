//! Palette generation logic using material-colors crate
//!
//! This module uses the official Material You algorithm via the material-colors crate
//! to generate perceptually uniform color palettes from a seed color.

use material_colors::color::Argb;
use material_colors::dynamic_color::{DynamicScheme, Variant};
use material_colors::hct::Hct;
use material_colors::palette::TonalPalette;
use serde_json::Value;

use super::color_parser::create_color_format;
use super::params::AlgorithmParameters;
use super::types::{ColorEntry, Palette};

/// Generate color palette from theme data using HCT color space
pub fn generate_palette(
    theme: &Value,
    is_dark_mode: bool,
    _is_strict: bool,
) -> Result<Palette, String> {
    // Try to get seed color from new format first
    let seed_hex = theme
        .get("seed")
        .and_then(|v| v.as_str())
        // Fallback to old format: extract from dark/light mode
        .or_else(|| {
            // Try old format with dark/light modes
            let mode_key = if is_dark_mode { "dark" } else { "light" };
            theme.get(mode_key)
                .and_then(|m| m.get("mPrimary").or_else(|| m.get("primary")))
                .and_then(|v| v.as_str())
        })
        // Fallback to top-level primary
        .or_else(|| theme.get("primary").and_then(|v| v.as_str()))
        .or_else(|| theme.get("mPrimary").and_then(|v| v.as_str()))
        .ok_or("Seed/primary color not found in theme. Expected 'seed', 'primary', or 'dark/light' object with 'mPrimary'/'primary'")?;

    // Parse seed color from hex
    let seed_argb = parse_hex_color(seed_hex)?;

    // Generate scheme using material-colors
    let scheme = generate_scheme(seed_argb, is_dark_mode);

    // Convert scheme to our Palette format
    scheme_to_palette(&scheme, is_dark_mode, theme)
}

/// Generate color palette with algorithm parameters
pub fn generate_palette_with_params(
    theme: &Value,
    is_dark_mode: bool,
    params: AlgorithmParameters,
) -> Result<Palette, String> {
    // Get seed color
    let seed_hex = theme
        .get("seed")
        .and_then(|v| v.as_str())
        .or_else(|| theme.get("primary").and_then(|v| v.as_str()))
        .or_else(|| theme.get("mPrimary").and_then(|v| v.as_str()))
        .ok_or("Seed/primary color not found in theme")?;

    // Parse seed color
    let seed_argb = parse_hex_color(seed_hex)?;

    // Generate scheme with algorithm parameters
    let scheme = generate_scheme_with_params(seed_argb, is_dark_mode, &params);

    // Convert to palette
    scheme_to_palette(&scheme, is_dark_mode, theme)
}

/// Parse hex color string to Argb
fn parse_hex_color(hex: &str) -> Result<Argb, String> {
    let hex = hex.trim_start_matches('#');
    u32::from_str_radix(hex, 16)
        .map(Argb::from_u32)
        .map_err(|e| format!("Invalid hex color '{}': {}", hex, e))
}

/// Generate a Material You scheme from a seed color
fn generate_scheme(seed: Argb, is_dark_mode: bool) -> DynamicScheme {
    // Create HCT from seed
    let hct = Hct::new(seed);

    // Create tonal palettes from the seed color's hue and chroma
    // Using CorePalette-like approach with customizable chroma
    let primary = TonalPalette::from_hue_and_chroma(hct.get_hue(), hct.get_chroma());
    let secondary = TonalPalette::from_hue_and_chroma(hct.get_hue(), hct.get_chroma() * 0.6); // Less chroma
    let tertiary =
        TonalPalette::from_hue_and_chroma((hct.get_hue() + 60.0) % 360.0, hct.get_chroma() * 0.8); // +60 hue
    let neutral = TonalPalette::from_hue_and_chroma(hct.get_hue(), hct.get_chroma() * 0.4); // Low chroma
    let neutral_variant = TonalPalette::from_hue_and_chroma(hct.get_hue(), hct.get_chroma() * 0.5);

    // Create dynamic scheme with all required parameters
    DynamicScheme::new(
        seed,
        Some(hct),
        Variant::Fidelity,
        is_dark_mode,
        None,
        primary,
        secondary,
        tertiary,
        neutral,
        neutral_variant,
        None,
    )
}

/// Generate a Material You scheme with algorithm parameters
fn generate_scheme_with_params(
    seed: Argb,
    is_dark_mode: bool,
    params: &AlgorithmParameters,
) -> DynamicScheme {
    // Create HCT from seed
    let hct = Hct::new(seed);

    // Apply hue shift
    let shifted_hue = (hct.get_hue() + params.hue_shift as f64) % 360.0;
    let shifted_hue = if shifted_hue < 0.0 {
        shifted_hue + 360.0
    } else {
        shifted_hue
    };

    // Apply saturation adjustment (modify chroma)
    let chroma_multiplier = 1.0 + (params.saturation_adjustment as f64 / 100.0);
    let adjusted_chroma = (hct.get_chroma() * chroma_multiplier).max(0.0);

    // Create tonal palettes with adjusted hue and chroma
    let primary = TonalPalette::from_hue_and_chroma(shifted_hue, adjusted_chroma);
    let secondary = TonalPalette::from_hue_and_chroma(shifted_hue, adjusted_chroma * 0.6);
    let tertiary =
        TonalPalette::from_hue_and_chroma((shifted_hue + 60.0) % 360.0, adjusted_chroma * 0.8);
    let neutral = TonalPalette::from_hue_and_chroma(shifted_hue, adjusted_chroma * 0.4);
    let neutral_variant = TonalPalette::from_hue_and_chroma(shifted_hue, adjusted_chroma * 0.5);

    // Create dynamic scheme
    DynamicScheme::new(
        seed,
        Some(Hct::from(shifted_hue, adjusted_chroma, hct.get_tone())),
        Variant::Fidelity,
        is_dark_mode,
        None,
        primary,
        secondary,
        tertiary,
        neutral,
        neutral_variant,
        None,
    )
}

/// Convert material-colors scheme to our Palette format
fn scheme_to_palette(
    scheme: &DynamicScheme,
    is_dark_mode: bool,
    theme: &Value,
) -> Result<Palette, String> {
    // Helper to get override color from theme (supports both old and new formats)
    let get_override = |key: &str| -> Option<String> {
        // Try new format first (top-level override)
        if let Some(hex) = theme.get(key).and_then(|v| v.as_str()) {
            return Some(hex.to_string());
        }

        // Try old format (dark/light mode specific)
        let mode_key = if is_dark_mode { "dark" } else { "light" };
        theme
            .get(mode_key)
            .and_then(|m| {
                m.get(key)
                    .or_else(|| m.get(format!("m{}", key[..1].to_uppercase() + &key[1..])))
            })
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    };

    // Helper to create ColorEntry from scheme colors
    let create_entry = |get_color_fn: fn(&DynamicScheme) -> Argb,
                        override_key: Option<&str>|
     -> Result<ColorEntry, String> {
        // Check for override in theme
        let override_hex = override_key.and_then(&get_override);

        let argb = if let Some(hex) = override_hex {
            parse_hex_color(&hex)?
        } else {
            get_color_fn(scheme)
        };

        let hex = argb.to_hex();

        Ok(ColorEntry {
            default: create_color_format(&hex)?,
            dark: create_color_format(&hex)?,
            light: create_color_format(&hex)?,
        })
    };

    // Build palette using MD3 color roles
    Ok(Palette {
        primary: create_entry(
            |s| s.primary(),
            theme
                .get("primary")
                .and_then(|v| v.as_str())
                .map(|_| "primary"),
        )?,
        on_primary: create_entry(
            |s| s.on_primary(),
            theme
                .get("on_primary")
                .and_then(|v| v.as_str())
                .map(|_| "on_primary"),
        )?,
        primary_container: create_entry(
            |s| s.primary_container(),
            theme
                .get("primary_container")
                .and_then(|v| v.as_str())
                .map(|_| "primary_container"),
        )?,
        on_primary_container: create_entry(
            |s| s.on_primary_container(),
            theme
                .get("on_primary_container")
                .and_then(|v| v.as_str())
                .map(|_| "on_primary_container"),
        )?,
        primary_fixed: create_entry(|s| s.primary_fixed(), None)?,
        primary_fixed_dim: create_entry(|s| s.primary_fixed_dim(), None)?,
        on_primary_fixed: create_entry(|s| s.on_primary_fixed(), None)?,
        on_primary_fixed_variant: create_entry(|s| s.on_primary_fixed_variant(), None)?,

        secondary: create_entry(
            |s| s.secondary(),
            theme
                .get("secondary")
                .and_then(|v| v.as_str())
                .map(|_| "secondary"),
        )?,
        on_secondary: create_entry(
            |s| s.on_secondary(),
            theme
                .get("on_secondary")
                .and_then(|v| v.as_str())
                .map(|_| "on_secondary"),
        )?,
        secondary_container: create_entry(
            |s| s.secondary_container(),
            theme
                .get("secondary_container")
                .and_then(|v| v.as_str())
                .map(|_| "secondary_container"),
        )?,
        on_secondary_container: create_entry(
            |s| s.on_secondary_container(),
            theme
                .get("on_secondary_container")
                .and_then(|v| v.as_str())
                .map(|_| "on_secondary_container"),
        )?,
        secondary_fixed: create_entry(|s| s.secondary_fixed(), None)?,
        secondary_fixed_dim: create_entry(|s| s.secondary_fixed_dim(), None)?,
        on_secondary_fixed: create_entry(|s| s.on_secondary_fixed(), None)?,
        on_secondary_fixed_variant: create_entry(|s| s.on_secondary_fixed_variant(), None)?,

        tertiary: create_entry(
            |s| s.tertiary(),
            theme
                .get("tertiary")
                .and_then(|v| v.as_str())
                .map(|_| "tertiary"),
        )?,
        on_tertiary: create_entry(
            |s| s.on_tertiary(),
            theme
                .get("on_tertiary")
                .and_then(|v| v.as_str())
                .map(|_| "on_tertiary"),
        )?,
        tertiary_container: create_entry(
            |s| s.tertiary_container(),
            theme
                .get("tertiary_container")
                .and_then(|v| v.as_str())
                .map(|_| "tertiary_container"),
        )?,
        on_tertiary_container: create_entry(
            |s| s.on_tertiary_container(),
            theme
                .get("on_tertiary_container")
                .and_then(|v| v.as_str())
                .map(|_| "on_tertiary_container"),
        )?,
        tertiary_fixed: create_entry(|s| s.tertiary_fixed(), None)?,
        tertiary_fixed_dim: create_entry(|s| s.tertiary_fixed_dim(), None)?,
        on_tertiary_fixed: create_entry(|s| s.on_tertiary_fixed(), None)?,
        on_tertiary_fixed_variant: create_entry(|s| s.on_tertiary_fixed_variant(), None)?,

        error: create_entry(
            |s| s.error(),
            theme.get("error").and_then(|v| v.as_str()).map(|_| "error"),
        )?,
        on_error: create_entry(
            |s| s.on_error(),
            theme
                .get("on_error")
                .and_then(|v| v.as_str())
                .map(|_| "on_error"),
        )?,
        error_container: create_entry(
            |s| s.error_container(),
            theme
                .get("error_container")
                .and_then(|v| v.as_str())
                .map(|_| "error_container"),
        )?,
        on_error_container: create_entry(
            |s| s.on_error_container(),
            theme
                .get("on_error_container")
                .and_then(|v| v.as_str())
                .map(|_| "on_error_container"),
        )?,

        background: create_entry(
            |s| s.background(),
            theme
                .get("background")
                .and_then(|v| v.as_str())
                .map(|_| "background"),
        )?,
        on_background: create_entry(
            |s| s.on_background(),
            theme
                .get("on_background")
                .and_then(|v| v.as_str())
                .map(|_| "on_background"),
        )?,
        surface: create_entry(
            |s| s.surface(),
            theme
                .get("surface")
                .and_then(|v| v.as_str())
                .map(|_| "surface"),
        )?,
        on_surface: create_entry(
            |s| s.on_surface(),
            theme
                .get("on_surface")
                .and_then(|v| v.as_str())
                .map(|_| "on_surface"),
        )?,
        surface_variant: create_entry(
            |s| s.surface_variant(),
            theme
                .get("surface_variant")
                .and_then(|v| v.as_str())
                .map(|_| "surface_variant"),
        )?,
        on_surface_variant: create_entry(
            |s| s.on_surface_variant(),
            theme
                .get("on_surface_variant")
                .and_then(|v| v.as_str())
                .map(|_| "on_surface_variant"),
        )?,

        surface_container_lowest: create_entry(|s| s.surface_container_lowest(), None)?,
        surface_container_low: create_entry(|s| s.surface_container_low(), None)?,
        surface_container: create_entry(|s| s.surface_container(), None)?,
        surface_container_high: create_entry(|s| s.surface_container_high(), None)?,
        surface_container_highest: create_entry(|s| s.surface_container_highest(), None)?,

        inverse_surface: create_entry(
            |s| s.inverse_surface(),
            theme
                .get("inverse_surface")
                .and_then(|v| v.as_str())
                .map(|_| "inverse_surface"),
        )?,
        inverse_on_surface: create_entry(
            |s| s.inverse_on_surface(),
            theme
                .get("inverse_on_surface")
                .and_then(|v| v.as_str())
                .map(|_| "inverse_on_surface"),
        )?,
        inverse_primary: create_entry(
            |s| s.inverse_primary(),
            theme
                .get("inverse_primary")
                .and_then(|v| v.as_str())
                .map(|_| "inverse_primary"),
        )?,

        surface_dim: create_entry(|s| s.surface_dim(), None)?,
        surface_bright: create_entry(|s| s.surface_bright(), None)?,

        outline: create_entry(
            |s| s.outline(),
            theme
                .get("outline")
                .and_then(|v| v.as_str())
                .map(|_| "outline"),
        )?,
        outline_variant: create_entry(
            |s| s.outline_variant(),
            theme
                .get("outline_variant")
                .and_then(|v| v.as_str())
                .map(|_| "outline_variant"),
        )?,

        shadow: create_entry(
            |s| s.shadow(),
            theme
                .get("shadow")
                .and_then(|v| v.as_str())
                .map(|_| "shadow"),
        )?,
        scrim: create_entry(
            |s| s.scrim(),
            theme.get("scrim").and_then(|v| v.as_str()).map(|_| "scrim"),
        )?,

        // Terminal colors - map from MD3 colors
        black: create_entry(|s| s.surface(), None)?,
        red: create_entry(|s| s.error(), None)?,
        green: create_entry(|s| s.tertiary(), None)?,
        yellow: create_entry(|s| s.primary(), None)?,
        blue: create_entry(|s| s.secondary(), None)?,
        magenta: create_entry(|s| s.primary_container(), None)?,
        cyan: create_entry(|s| s.secondary_container(), None)?,
        white: create_entry(|s| s.on_surface(), None)?,
        bright_black: create_entry(|s| s.surface_variant(), None)?,
        bright_red: create_entry(|s| s.error_container(), None)?,
        bright_green: create_entry(|s| s.tertiary_container(), None)?,
        bright_yellow: create_entry(|s| s.primary_fixed(), None)?,
        bright_blue: create_entry(|s| s.secondary_fixed(), None)?,
        bright_magenta: create_entry(|s| s.primary_fixed_dim(), None)?,
        bright_cyan: create_entry(|s| s.secondary_fixed_dim(), None)?,
        bright_white: create_entry(|s| s.inverse_surface(), None)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_generate_palette_from_seed() {
        let theme = json!({
            "seed": "#FF5722"
        });

        let palette = generate_palette(&theme, false, false).unwrap();

        // Verify primary color was generated
        assert!(!palette.primary.default.hex.is_empty());
        assert!(palette.primary.default.hex.starts_with("#"));

        // Verify other colors exist
        assert!(!palette.secondary.default.hex.is_empty());
        assert!(!palette.tertiary.default.hex.is_empty());
    }

    #[test]
    fn test_generate_palette_with_override() {
        let theme = json!({
            "seed": "#FF5722",
            "error": "#F44336"
        });

        let palette = generate_palette(&theme, false, false).unwrap();

        // Error color should be the override
        assert_eq!(palette.error.default.hex, "#F44336");
    }

    #[test]
    fn test_generate_palette_dark_mode() {
        let theme = json!({
            "seed": "#2196F3"
        });

        let palette = generate_palette(&theme, true, false).unwrap();

        // Dark mode should have dark surface
        assert!(palette.surface.dark.hex.starts_with("#"));
    }
}
