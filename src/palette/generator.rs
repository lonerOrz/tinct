//! Palette generation logic using material-colors crate
//!
//! This module uses the official Material You algorithm via the material-colors crate
//! to generate perceptually uniform color palettes from a seed color.

use material_colors::color::Argb;
use material_colors::dynamic_color::{DynamicScheme, Variant};
use material_colors::hct::Hct;
use material_colors::palette::TonalPalette;
use serde_json::Value;

use super::params::{AlgorithmParameters, ColorHarmony};
use super::types::{ColorRole, Palette};
use crate::color::Color;
use std::collections::HashMap;

/// Extract seed hex from theme: prefer "seed", fallback to "Primary".
pub fn extract_seed_hex(theme: &Value) -> Option<&str> {
    theme
        .get("seed")
        .and_then(|v| v.as_str())
        .or_else(|| theme.get("Primary").and_then(|v| v.as_str()))
}

/// Generate color palette from theme data using HCT color space
pub fn generate_palette(theme: &Value, is_dark_mode: bool) -> Result<Palette, String> {
    let seed_hex =
        extract_seed_hex(theme).ok_or("Theme must contain either 'seed' or 'Primary' color")?;

    let seed_argb = parse_hex_color(seed_hex)?;
    let scheme =
        generate_scheme_with_params(seed_argb, is_dark_mode, &AlgorithmParameters::default());
    scheme_to_palette(&scheme, theme)
}

/// Generate color palette with algorithm parameters
pub fn generate_palette_with_params(
    theme: &Value,
    is_dark_mode: bool,
    params: AlgorithmParameters,
) -> Result<Palette, String> {
    let seed_hex =
        extract_seed_hex(theme).ok_or("Theme must contain either 'seed' or 'Primary' color")?;

    let seed_argb = parse_hex_color(seed_hex)?;
    let scheme = generate_scheme_with_params(seed_argb, is_dark_mode, &params);
    scheme_to_palette(&scheme, theme)
}

fn parse_hex_color(hex: &str) -> Result<Argb, String> {
    let hex = hex.trim_start_matches('#');
    u32::from_str_radix(hex, 16)
        .map(Argb::from_u32)
        .map_err(|e| format!("Invalid hex color '{}': {}", hex, e))
}

/// Map an Argb to a Color.
fn argb_to_color(argb: Argb) -> Color {
    Color::new(argb.red, argb.green, argb.blue, argb.alpha as f64 / 255.0)
}

/// Convert snake_case key to PascalCase for theme lookup.
fn pascal_case(key: &str) -> Option<String> {
    let mut chars = key.chars();
    let first = chars.next()?;
    Some(format!("{}{}", first.to_uppercase(), chars.as_str()))
}

/// Calculate secondary hue based on MD3 algorithm
fn calculate_secondary_hue(hue: f64) -> f64 {
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
fn calculate_tertiary_hue(hue: f64) -> f64 {
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

    let apply_shift =
        |hue: f64| -> f64 { ((hue + params.hue_shift as f64) % 360.0 + 360.0) % 360.0 };

    let primary_hue = apply_shift(base_hue);
    let secondary_hue = apply_shift(secondary_base_hue);
    let tertiary_hue = apply_shift(tertiary_base_hue);
    let neutral_hue = primary_hue;
    let neutral_variant_hue = primary_hue;

    let chroma_multiplier = 1.0 + (params.saturation_adjustment as f64 / 100.0);
    let adjusted_chroma = (base_chroma * chroma_multiplier).max(0.0);

    let secondary_chroma = (adjusted_chroma - 32.0).max(adjusted_chroma * 0.5).max(0.0);
    let tertiary_chroma = adjusted_chroma;
    let neutral_chroma = adjusted_chroma / 8.0;
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
fn scheme_to_palette(scheme: &DynamicScheme, theme: &Value) -> Result<Palette, String> {
    let get_override = |key: &str| -> Option<&str> {
        theme
            .get(key)
            .and_then(|v| v.as_str())
            .or_else(|| pascal_case(key).and_then(|k| theme.get(k).and_then(|v| v.as_str())))
    };

    // Helper: resolve a color from scheme or override
    let resolve = |get_color_fn: fn(&DynamicScheme) -> Argb,
                   role_opt: Option<&str>|
     -> Result<Color, String> {
        if let Some(hex) = role_opt.and_then(&get_override) {
            let argb = parse_hex_color(hex)?;
            Ok(argb_to_color(argb))
        } else {
            Ok(argb_to_color(get_color_fn(scheme)))
        }
    };

    let mut colors: HashMap<ColorRole, Color> = HashMap::new();
    let insert =
        |colors: &mut HashMap<ColorRole, Color>, role: ColorRole, result: Result<Color, String>| {
            colors.insert(role, result.expect("palette color generation failed"));
        };

    insert(
        &mut colors,
        ColorRole::Primary,
        resolve(|s| s.primary(), Some("primary")),
    );
    insert(
        &mut colors,
        ColorRole::OnPrimary,
        resolve(|s| s.on_primary(), Some("on_primary")),
    );
    insert(
        &mut colors,
        ColorRole::PrimaryContainer,
        resolve(|s| s.primary_container(), Some("primary_container")),
    );
    insert(
        &mut colors,
        ColorRole::OnPrimaryContainer,
        resolve(|s| s.on_primary_container(), Some("on_primary_container")),
    );
    insert(
        &mut colors,
        ColorRole::PrimaryFixed,
        resolve(|s| s.primary_fixed(), None),
    );
    insert(
        &mut colors,
        ColorRole::PrimaryFixedDim,
        resolve(|s| s.primary_fixed_dim(), None),
    );
    insert(
        &mut colors,
        ColorRole::OnPrimaryFixed,
        resolve(|s| s.on_primary_fixed(), None),
    );
    insert(
        &mut colors,
        ColorRole::OnPrimaryFixedVariant,
        resolve(|s| s.on_primary_fixed_variant(), None),
    );

    insert(
        &mut colors,
        ColorRole::Secondary,
        resolve(|s| s.secondary(), Some("secondary")),
    );
    insert(
        &mut colors,
        ColorRole::OnSecondary,
        resolve(|s| s.on_secondary(), Some("on_secondary")),
    );
    insert(
        &mut colors,
        ColorRole::SecondaryContainer,
        resolve(|s| s.secondary_container(), Some("secondary_container")),
    );
    insert(
        &mut colors,
        ColorRole::OnSecondaryContainer,
        resolve(
            |s| s.on_secondary_container(),
            Some("on_secondary_container"),
        ),
    );
    insert(
        &mut colors,
        ColorRole::SecondaryFixed,
        resolve(|s| s.secondary_fixed(), None),
    );
    insert(
        &mut colors,
        ColorRole::SecondaryFixedDim,
        resolve(|s| s.secondary_fixed_dim(), None),
    );
    insert(
        &mut colors,
        ColorRole::OnSecondaryFixed,
        resolve(|s| s.on_secondary_fixed(), None),
    );
    insert(
        &mut colors,
        ColorRole::OnSecondaryFixedVariant,
        resolve(|s| s.on_secondary_fixed_variant(), None),
    );

    insert(
        &mut colors,
        ColorRole::Tertiary,
        resolve(|s| s.tertiary(), Some("tertiary")),
    );
    insert(
        &mut colors,
        ColorRole::OnTertiary,
        resolve(|s| s.on_tertiary(), Some("on_tertiary")),
    );
    insert(
        &mut colors,
        ColorRole::TertiaryContainer,
        resolve(|s| s.tertiary_container(), Some("tertiary_container")),
    );
    insert(
        &mut colors,
        ColorRole::OnTertiaryContainer,
        resolve(|s| s.on_tertiary_container(), Some("on_tertiary_container")),
    );
    insert(
        &mut colors,
        ColorRole::TertiaryFixed,
        resolve(|s| s.tertiary_fixed(), None),
    );
    insert(
        &mut colors,
        ColorRole::TertiaryFixedDim,
        resolve(|s| s.tertiary_fixed_dim(), None),
    );
    insert(
        &mut colors,
        ColorRole::OnTertiaryFixed,
        resolve(|s| s.on_tertiary_fixed(), None),
    );
    insert(
        &mut colors,
        ColorRole::OnTertiaryFixedVariant,
        resolve(|s| s.on_tertiary_fixed_variant(), None),
    );

    insert(
        &mut colors,
        ColorRole::Error,
        resolve(|s| s.error(), Some("error")),
    );
    insert(
        &mut colors,
        ColorRole::OnError,
        resolve(|s| s.on_error(), Some("on_error")),
    );
    insert(
        &mut colors,
        ColorRole::ErrorContainer,
        resolve(|s| s.error_container(), Some("error_container")),
    );
    insert(
        &mut colors,
        ColorRole::OnErrorContainer,
        resolve(|s| s.on_error_container(), Some("on_error_container")),
    );

    insert(
        &mut colors,
        ColorRole::Background,
        resolve(|s| s.background(), Some("background")),
    );
    insert(
        &mut colors,
        ColorRole::OnBackground,
        resolve(|s| s.on_background(), Some("on_background")),
    );
    insert(
        &mut colors,
        ColorRole::Surface,
        resolve(|s| s.surface(), Some("surface")),
    );
    insert(
        &mut colors,
        ColorRole::OnSurface,
        resolve(|s| s.on_surface(), Some("on_surface")),
    );
    insert(
        &mut colors,
        ColorRole::SurfaceVariant,
        resolve(|s| s.surface_variant(), Some("surface_variant")),
    );
    insert(
        &mut colors,
        ColorRole::OnSurfaceVariant,
        resolve(|s| s.on_surface_variant(), Some("on_surface_variant")),
    );

    insert(
        &mut colors,
        ColorRole::SurfaceContainerLowest,
        resolve(|s| s.surface_container_lowest(), None),
    );
    insert(
        &mut colors,
        ColorRole::SurfaceContainerLow,
        resolve(|s| s.surface_container_low(), None),
    );
    insert(
        &mut colors,
        ColorRole::SurfaceContainer,
        resolve(|s| s.surface_container(), None),
    );
    insert(
        &mut colors,
        ColorRole::SurfaceContainerHigh,
        resolve(|s| s.surface_container_high(), None),
    );
    insert(
        &mut colors,
        ColorRole::SurfaceContainerHighest,
        resolve(|s| s.surface_container_highest(), None),
    );

    insert(
        &mut colors,
        ColorRole::InverseSurface,
        resolve(|s| s.inverse_surface(), Some("inverse_surface")),
    );
    insert(
        &mut colors,
        ColorRole::InverseOnSurface,
        resolve(|s| s.inverse_on_surface(), Some("inverse_on_surface")),
    );
    insert(
        &mut colors,
        ColorRole::InversePrimary,
        resolve(|s| s.inverse_primary(), Some("inverse_primary")),
    );

    insert(
        &mut colors,
        ColorRole::SurfaceDim,
        resolve(|s| s.surface_dim(), None),
    );
    insert(
        &mut colors,
        ColorRole::SurfaceBright,
        resolve(|s| s.surface_bright(), None),
    );
    insert(
        &mut colors,
        ColorRole::SurfaceTint,
        resolve(|s| s.surface_tint(), None),
    );

    insert(
        &mut colors,
        ColorRole::Outline,
        resolve(|s| s.outline(), Some("outline")),
    );
    insert(
        &mut colors,
        ColorRole::OutlineVariant,
        resolve(|s| s.outline_variant(), Some("outline_variant")),
    );

    insert(
        &mut colors,
        ColorRole::Shadow,
        resolve(|s| s.shadow(), Some("shadow")),
    );
    insert(
        &mut colors,
        ColorRole::Scrim,
        resolve(|s| s.scrim(), Some("scrim")),
    );

    // Terminal colors
    insert(
        &mut colors,
        ColorRole::Black,
        resolve(|s| s.surface(), None),
    );
    insert(&mut colors, ColorRole::Red, resolve(|s| s.error(), None));
    insert(
        &mut colors,
        ColorRole::Green,
        resolve(|s| s.tertiary(), None),
    );
    insert(
        &mut colors,
        ColorRole::Yellow,
        resolve(|s| s.primary_fixed(), None),
    );
    insert(
        &mut colors,
        ColorRole::Blue,
        resolve(|s| s.secondary(), None),
    );
    insert(
        &mut colors,
        ColorRole::Magenta,
        resolve(|s| s.tertiary(), None),
    );
    insert(
        &mut colors,
        ColorRole::Cyan,
        resolve(|s| s.secondary_container(), None),
    );
    insert(
        &mut colors,
        ColorRole::White,
        resolve(|s| s.on_surface(), None),
    );
    insert(
        &mut colors,
        ColorRole::BrightBlack,
        resolve(|s| s.surface_variant(), None),
    );
    insert(
        &mut colors,
        ColorRole::BrightRed,
        resolve(|s| s.error_container(), None),
    );
    insert(
        &mut colors,
        ColorRole::BrightGreen,
        resolve(|s| s.tertiary_container(), None),
    );
    insert(
        &mut colors,
        ColorRole::BrightYellow,
        resolve(|s| s.primary_fixed(), None),
    );
    insert(
        &mut colors,
        ColorRole::BrightBlue,
        resolve(|s| s.secondary_fixed(), None),
    );
    insert(
        &mut colors,
        ColorRole::BrightMagenta,
        resolve(|s| s.primary_fixed_dim(), None),
    );
    insert(
        &mut colors,
        ColorRole::BrightCyan,
        resolve(|s| s.secondary_fixed_dim(), None),
    );
    insert(
        &mut colors,
        ColorRole::BrightWhite,
        resolve(|s| s.inverse_surface(), None),
    );

    Ok(Palette::new(colors))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_generate_palette_from_seed() {
        let theme = json!({ "seed": "#FF5722" });
        let palette = generate_palette(&theme, false).unwrap();

        assert!(palette.get("primary").is_some());
        let primary = palette.get("primary").unwrap();
        assert_ne!(primary.hex(), String::from("#000000"));
        assert!(palette.get("secondary").is_some());
        assert!(palette.get("tertiary").is_some());
    }

    #[test]
    fn test_generate_palette_with_override() {
        let theme = json!({ "seed": "#FF5722", "error": "#F44336" });
        let palette = generate_palette(&theme, false).unwrap();
        let error = palette.get("error").unwrap();
        assert_eq!(error.hex(), "#F44336");
    }

    #[test]
    fn test_generate_palette_dark_mode() {
        let theme = json!({ "seed": "#2196F3" });
        let palette = generate_palette(&theme, true).unwrap();
        assert!(palette.get("surface").is_some());
    }
}
