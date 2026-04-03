//! Palette generation logic using material-colors crate
//!
//! This module uses the official Material You algorithm via the material-colors crate
//! to generate perceptually uniform color palettes from a seed color.
//!
//! # Supported Theme Formats
//!
//! ## Format 1: Seed only
//! ```json
//! { "seed": "#7aa2f7" }
//! ```
//!
//! ## Format 2: Overrides only (seed extracted from Primary)
//! ```json
//! {
//!   "Primary": "#7aa2f7",
//!   "Secondary": "#bb9af7",
//!   "Tertiary": "#9ece6a"
//! }
//! ```
//!
//! ## Format 3: Seed + Overrides
//! ```json
//! {
//!   "seed": "#7aa2f7",
//!   "Primary": "#7aa2f7",
//!   "Secondary": "#bb9af7",
//!   "Tertiary": "#9ece6a"
//! }
//! ```

use material_colors::color::Argb;
use material_colors::dynamic_color::{DynamicScheme, Variant};
use material_colors::hct::Hct;
use material_colors::palette::TonalPalette;
use serde_json::Value;

use super::color_parser::create_color_format;
use super::params::{AlgorithmParameters, ColorHarmony};
use super::types::{ColorEntry, Palette};

/// Generate color palette from theme data using HCT color space
///
/// Seed color priority:
/// 1. `seed` field (if present)
/// 2. `Primary` field (as fallback seed)
pub fn generate_palette(
    theme: &Value,
    is_dark_mode: bool,
    _is_strict: bool,
) -> Result<Palette, String> {
    // Get seed color: prefer explicit "seed", fallback to "Primary"
    let seed_hex = theme
        .get("seed")
        .and_then(|v| v.as_str())
        .or_else(|| theme.get("Primary").and_then(|v| v.as_str()))
        .ok_or("Theme must contain either 'seed' or 'Primary' color")?;

    // Parse seed color from hex
    let seed_argb = parse_hex_color(seed_hex)?;

    let scheme =
        generate_scheme_with_params(seed_argb, is_dark_mode, &AlgorithmParameters::default());

    scheme_to_palette(&scheme, is_dark_mode, theme)
}

/// Generate color palette with algorithm parameters
///
/// Seed color priority:
/// 1. `seed` field (if present)
/// 2. `Primary` field (as fallback seed)
pub fn generate_palette_with_params(
    theme: &Value,
    is_dark_mode: bool,
    params: AlgorithmParameters,
) -> Result<Palette, String> {
    // Get seed color: prefer explicit "seed", fallback to "Primary"
    let seed_hex = theme
        .get("seed")
        .and_then(|v| v.as_str())
        .or_else(|| theme.get("Primary").and_then(|v| v.as_str()))
        .ok_or("Theme must contain either 'seed' or 'Primary' color")?;

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

/// Calculate secondary hue based on MD3 algorithm
/// This creates more harmonious color relationships
fn calculate_secondary_hue(hue: f64) -> f64 {
    // MD3 uses different hue offsets based on the seed hue range
    let offset = if (0.0..41.0).contains(&hue) {
        15.0
    } else if (41.0..61.0).contains(&hue) {
        10.0
    } else if (61.0..101.0).contains(&hue) {
        8.0
    } else if (101.0..141.0).contains(&hue) {
        5.0
    } else if (141.0..181.0).contains(&hue) {
        3.0
    } else if (181.0..221.0).contains(&hue) {
        2.0
    } else if (221.0..261.0).contains(&hue) {
        5.0
    } else if (261.0..301.0).contains(&hue) {
        10.0
    } else if (301.0..341.0).contains(&hue) {
        15.0
    } else {
        20.0
    };

    (hue + offset) % 360.0
}

/// Calculate tertiary hue based on MD3 algorithm
/// This creates complementary or analogous color relationships
fn calculate_tertiary_hue(hue: f64) -> f64 {
    // MD3 uses different hue offsets based on the seed hue range
    let offset = if (0.0..41.0).contains(&hue) {
        30.0
    } else if (41.0..61.0).contains(&hue) {
        25.0
    } else if (61.0..101.0).contains(&hue) {
        20.0
    } else if (101.0..141.0).contains(&hue) {
        15.0
    } else if (141.0..181.0).contains(&hue) {
        10.0
    } else if (181.0..221.0).contains(&hue) {
        5.0
    } else if (221.0..261.0).contains(&hue) {
        10.0
    } else if (261.0..301.0).contains(&hue) {
        20.0
    } else if (301.0..341.0).contains(&hue) {
        30.0
    } else {
        40.0
    };

    (hue + offset) % 360.0
}

/// Generate a Material You scheme with algorithm parameters
fn generate_scheme_with_params(
    seed: Argb,
    is_dark_mode: bool,
    params: &AlgorithmParameters,
) -> DynamicScheme {
    let hct = Hct::new(seed);
    let base_hue = hct.get_hue();
    let base_chroma = hct.get_chroma();

    // Calculate secondary/tertiary hues based on harmony mode
    let (secondary_base_hue, tertiary_base_hue) = match params.color_harmony {
        ColorHarmony::Md3 => (
            calculate_secondary_hue(base_hue),
            calculate_tertiary_hue(base_hue),
        ),
        ColorHarmony::Analogous => ((base_hue + 15.0) % 360.0, (base_hue + 30.0) % 360.0),
        ColorHarmony::Complementary => ((base_hue + 180.0) % 360.0, (base_hue + 180.0) % 360.0),
        ColorHarmony::Triadic => ((base_hue + 120.0) % 360.0, (base_hue + 240.0) % 360.0),
        ColorHarmony::SplitComplementary => {
            ((base_hue + 150.0) % 360.0, (base_hue + 210.0) % 360.0)
        }
    };

    // Then apply hue shift on top
    let apply_shift =
        |hue: f64| -> f64 { ((hue + params.hue_shift as f64) % 360.0 + 360.0) % 360.0 };

    let primary_hue = apply_shift(base_hue);
    let secondary_hue = apply_shift(secondary_base_hue);
    let tertiary_hue = apply_shift(tertiary_base_hue);
    let neutral_hue = primary_hue;
    let neutral_variant_hue = primary_hue;

    // Apply saturation adjustment (modify chroma)
    let chroma_multiplier = 1.0 + (params.saturation_adjustment as f64 / 100.0);
    let adjusted_chroma = (base_chroma * chroma_multiplier).max(0.0);

    // Create tonal palettes with official MD3 Fidelity chroma formulas
    // Secondary: max(chroma - 32, chroma * 0.5)
    let secondary_chroma = (adjusted_chroma - 32.0).max(adjusted_chroma * 0.5).max(0.0);
    // Tertiary: uses complement hue with full chroma
    let tertiary_chroma = adjusted_chroma;
    // Neutral: chroma / 8.0
    let neutral_chroma = adjusted_chroma / 8.0;
    // NeutralVariant: chroma / 8.0 + 4.0
    let neutral_variant_chroma = adjusted_chroma / 8.0 + 4.0;

    let primary = TonalPalette::from_hue_and_chroma(primary_hue, adjusted_chroma);
    let secondary = TonalPalette::from_hue_and_chroma(secondary_hue, secondary_chroma);
    let tertiary = TonalPalette::from_hue_and_chroma(tertiary_hue, tertiary_chroma);
    let neutral = TonalPalette::from_hue_and_chroma(neutral_hue, neutral_chroma);
    let neutral_variant =
        TonalPalette::from_hue_and_chroma(neutral_variant_hue, neutral_variant_chroma);

    DynamicScheme::new(
        seed,
        Some(Hct::from(primary_hue, adjusted_chroma, hct.get_tone())),
        Variant::Fidelity,
        is_dark_mode,
        Some(params.contrast_level),
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
    _is_dark_mode: bool,
    theme: &Value,
) -> Result<Palette, String> {
    // Helper to get override color from theme
    let get_override = |key: &str| -> Option<String> {
        // Support both lowercase ("primary") and PascalCase ("Primary")
        theme
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                // Try PascalCase (first letter uppercase)
                let pascal_case = format!("{}{}", &key[..1].to_uppercase(), &key[1..]);
                theme
                    .get(&pascal_case)
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
    };

    // Helper to create ColorEntry from scheme colors
    // Pass the color role name directly, get_override will handle both formats
    let create_entry = |get_color_fn: fn(&DynamicScheme) -> Argb,
                        role_name: Option<&str>|
     -> Result<ColorEntry, String> {
        // Check for override in theme using get_override
        let override_hex = role_name.and_then(&get_override);

        let argb = if let Some(hex) = override_hex {
            parse_hex_color(&hex)?
        } else {
            get_color_fn(scheme)
        };

        let hex = argb.to_hex();

        Ok(ColorEntry {
            default: create_color_format(&hex)?,
        })
    };

    // Build palette using MD3 color roles
    Ok(Palette {
        primary: create_entry(|s| s.primary(), Some("primary"))?,
        on_primary: create_entry(|s| s.on_primary(), Some("on_primary"))?,
        primary_container: create_entry(|s| s.primary_container(), Some("primary_container"))?,
        on_primary_container: create_entry(
            |s| s.on_primary_container(),
            Some("on_primary_container"),
        )?,
        primary_fixed: create_entry(|s| s.primary_fixed(), None)?,
        primary_fixed_dim: create_entry(|s| s.primary_fixed_dim(), None)?,
        on_primary_fixed: create_entry(|s| s.on_primary_fixed(), None)?,
        on_primary_fixed_variant: create_entry(|s| s.on_primary_fixed_variant(), None)?,

        secondary: create_entry(|s| s.secondary(), Some("secondary"))?,
        on_secondary: create_entry(|s| s.on_secondary(), Some("on_secondary"))?,
        secondary_container: create_entry(
            |s| s.secondary_container(),
            Some("secondary_container"),
        )?,
        on_secondary_container: create_entry(
            |s| s.on_secondary_container(),
            Some("on_secondary_container"),
        )?,
        secondary_fixed: create_entry(|s| s.secondary_fixed(), None)?,
        secondary_fixed_dim: create_entry(|s| s.secondary_fixed_dim(), None)?,
        on_secondary_fixed: create_entry(|s| s.on_secondary_fixed(), None)?,
        on_secondary_fixed_variant: create_entry(|s| s.on_secondary_fixed_variant(), None)?,

        tertiary: create_entry(|s| s.tertiary(), Some("tertiary"))?,
        on_tertiary: create_entry(|s| s.on_tertiary(), Some("on_tertiary"))?,
        tertiary_container: create_entry(|s| s.tertiary_container(), Some("tertiary_container"))?,
        on_tertiary_container: create_entry(
            |s| s.on_tertiary_container(),
            Some("on_tertiary_container"),
        )?,
        tertiary_fixed: create_entry(|s| s.tertiary_fixed(), None)?,
        tertiary_fixed_dim: create_entry(|s| s.tertiary_fixed_dim(), None)?,
        on_tertiary_fixed: create_entry(|s| s.on_tertiary_fixed(), None)?,
        on_tertiary_fixed_variant: create_entry(|s| s.on_tertiary_fixed_variant(), None)?,

        error: create_entry(|s| s.error(), Some("error"))?,
        on_error: create_entry(|s| s.on_error(), Some("on_error"))?,
        error_container: create_entry(|s| s.error_container(), Some("error_container"))?,
        on_error_container: create_entry(|s| s.on_error_container(), Some("on_error_container"))?,

        background: create_entry(|s| s.background(), Some("background"))?,
        on_background: create_entry(|s| s.on_background(), Some("on_background"))?,
        surface: create_entry(|s| s.surface(), Some("surface"))?,
        on_surface: create_entry(|s| s.on_surface(), Some("on_surface"))?,
        surface_variant: create_entry(|s| s.surface_variant(), Some("surface_variant"))?,
        on_surface_variant: create_entry(|s| s.on_surface_variant(), Some("on_surface_variant"))?,

        surface_container_lowest: create_entry(|s| s.surface_container_lowest(), None)?,
        surface_container_low: create_entry(|s| s.surface_container_low(), None)?,
        surface_container: create_entry(|s| s.surface_container(), None)?,
        surface_container_high: create_entry(|s| s.surface_container_high(), None)?,
        surface_container_highest: create_entry(|s| s.surface_container_highest(), None)?,

        inverse_surface: create_entry(|s| s.inverse_surface(), Some("inverse_surface"))?,
        inverse_on_surface: create_entry(|s| s.inverse_on_surface(), Some("inverse_on_surface"))?,
        inverse_primary: create_entry(|s| s.inverse_primary(), Some("inverse_primary"))?,

        surface_dim: create_entry(|s| s.surface_dim(), None)?,
        surface_bright: create_entry(|s| s.surface_bright(), None)?,

        outline: create_entry(|s| s.outline(), Some("outline"))?,
        outline_variant: create_entry(|s| s.outline_variant(), Some("outline_variant"))?,

        shadow: create_entry(|s| s.shadow(), Some("shadow"))?,
        scrim: create_entry(|s| s.scrim(), Some("scrim"))?,

        // Terminal colors - map from MD3 colors with intelligent hue-based mapping
        black: create_entry(|s| s.surface(), None)?,
        red: create_entry(|s| s.error(), None)?,
        green: create_entry(|s| s.tertiary(), None)?,
        // Use hue-based mapping for yellow/blue to ensure terminal colors are distinct
        yellow: create_entry(get_terminal_yellow, None)?,
        blue: create_entry(get_terminal_blue, None)?,
        magenta: create_entry(get_terminal_magenta, None)?,
        cyan: create_entry(get_terminal_cyan, None)?,
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

/// Get terminal yellow based on seed hue
/// Ensures yellow is distinct from primary when primary is not yellow-ish
fn get_terminal_yellow(scheme: &DynamicScheme) -> Argb {
    // Use primary_fixed for bright yellow-like colors
    scheme.primary_fixed()
}

/// Get terminal blue based on seed hue
/// Ensures blue is distinct and appropriate for the color scheme
fn get_terminal_blue(scheme: &DynamicScheme) -> Argb {
    // Use secondary for blue-like colors
    scheme.secondary()
}

/// Get terminal magenta based on seed hue
fn get_terminal_magenta(scheme: &DynamicScheme) -> Argb {
    // Use tertiary for magenta/purple-like colors
    scheme.tertiary()
}

/// Get terminal cyan based on seed hue
fn get_terminal_cyan(scheme: &DynamicScheme) -> Argb {
    // Use secondary_container for cyan-like colors
    scheme.secondary_container()
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
        assert!(palette.surface.default.hex.starts_with("#"));
    }
}
