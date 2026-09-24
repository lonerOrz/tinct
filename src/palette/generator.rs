//! Palette generation using the official Material You algorithm.
//!
//! Generation is delegated entirely to the `material-colors` crate. We never
//! reimplement MD3's hue/chroma tables: we select the official `Variant` that
//! matches the requested [`SchemeType`], optionally nudge the seed (hue shift /
//! chroma scaling / contrast level), and let [`DynamicScheme::by_variant`]
//! derive the complete, spec-compliant scheme.

use material_colors::color::Argb;
use material_colors::dynamic_color::DynamicScheme;
use material_colors::hct::Hct;
use serde_json::Value;

use super::ansi;
use super::params::AlgorithmParameters;
use super::types::{ColorRole, Palette};
use crate::color::Color;
use crate::image::SchemeType;
use std::collections::HashMap;

/// Extract seed hex from theme: prefer "seed", fallback to "Primary".
pub fn extract_seed_hex(theme: &Value) -> Option<&str> {
    theme
        .get("seed")
        .and_then(|v| v.as_str())
        .or_else(|| theme.get("Primary").and_then(|v| v.as_str()))
}

/// Generate a palette using the default Tonal Spot scheme and no adjustments.
pub fn generate_palette(theme: &Value, is_dark_mode: bool) -> Result<Palette, String> {
    generate_palette_with_params(
        theme,
        is_dark_mode,
        SchemeType::TonalSpot,
        AlgorithmParameters::default(),
    )
}

/// Generate a palette with an explicit scheme type and seed adjustments.
pub fn generate_palette_with_params(
    theme: &Value,
    is_dark_mode: bool,
    scheme_type: SchemeType,
    params: AlgorithmParameters,
) -> Result<Palette, String> {
    let seed_hex =
        extract_seed_hex(theme).ok_or("Theme must contain either 'seed' or 'Primary' color")?;

    let seed_argb = parse_hex_color(seed_hex)?;
    let scheme = generate_scheme(seed_argb, is_dark_mode, scheme_type, &params);
    scheme_to_palette(&scheme, theme)
}

/// Build the official MD3 scheme for a seed.
///
/// This is the single source of truth for generation: every role is produced by
/// `material-colors`, so secondary/tertiary/neutral relationships match Material
/// You exactly. `AlgorithmParameters` only influence the seed itself.
pub fn generate_scheme(
    seed: Argb,
    is_dark_mode: bool,
    scheme_type: SchemeType,
    params: &AlgorithmParameters,
) -> DynamicScheme {
    let params = params.sanitized();
    let adjusted_seed = adjust_seed(seed, &params);
    DynamicScheme::by_variant(
        adjusted_seed,
        &scheme_type.to_variant(),
        is_dark_mode,
        Some(params.contrast_level),
    )
}

/// Apply seed-level adjustments in HCT space, preserving tone.
///
/// Returns the seed untouched when the parameters are the defaults, so the
/// default path is bit-for-bit the official algorithm.
fn adjust_seed(seed: Argb, params: &AlgorithmParameters) -> Argb {
    if params.hue_shift == 0 && params.saturation_adjustment == 0 {
        return seed;
    }

    let hct = Hct::new(seed);
    let hue = ((hct.get_hue() + params.hue_shift as f64) % 360.0 + 360.0) % 360.0;
    let chroma =
        (hct.get_chroma() * (1.0 + params.saturation_adjustment as f64 / 100.0)).clamp(0.0, 120.0);

    let adjusted: Argb = Hct::from(hue, chroma, hct.get_tone()).into();
    adjusted
}

fn parse_hex_color(hex: &str) -> Result<Argb, String> {
    let hex = hex.trim_start_matches('#');
    let argb = if hex.len() == 6 {
        // 6-digit RGB: treat as opaque (alpha = 255)
        u32::from_str_radix(&format!("FF{}", hex), 16)
            .map_err(|e| format!("Invalid hex color '{}': {}", hex, e))?
    } else if hex.len() == 8 {
        u32::from_str_radix(hex, 16).map_err(|e| format!("Invalid hex color '{}': {}", hex, e))?
    } else {
        return Err(format!(
            "Invalid hex color '{}': expected 6 or 8 digits, got {}",
            hex,
            hex.len()
        ));
    };
    Ok(Argb::from_u32(argb))
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

/// Convert a material-colors scheme to our Palette format.
///
/// Every MD3 role comes straight from the scheme; theme entries may override
/// individual roles by snake_case or PascalCase key. Terminal roles are always
/// derived by [`ansi::ansi_colors`].
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
    let mut insert = |role: ColorRole, result: Result<Color, String>| -> Result<(), String> {
        colors.insert(role, result?);
        Ok(())
    };

    insert(
        ColorRole::Primary,
        resolve(|s| s.primary(), Some("primary")),
    )?;
    insert(
        ColorRole::OnPrimary,
        resolve(|s| s.on_primary(), Some("on_primary")),
    )?;
    insert(
        ColorRole::PrimaryContainer,
        resolve(|s| s.primary_container(), Some("primary_container")),
    )?;
    insert(
        ColorRole::OnPrimaryContainer,
        resolve(|s| s.on_primary_container(), Some("on_primary_container")),
    )?;
    insert(
        ColorRole::PrimaryFixed,
        resolve(|s| s.primary_fixed(), None),
    )?;
    insert(
        ColorRole::PrimaryFixedDim,
        resolve(|s| s.primary_fixed_dim(), None),
    )?;
    insert(
        ColorRole::OnPrimaryFixed,
        resolve(|s| s.on_primary_fixed(), None),
    )?;
    insert(
        ColorRole::OnPrimaryFixedVariant,
        resolve(|s| s.on_primary_fixed_variant(), None),
    )?;

    insert(
        ColorRole::Secondary,
        resolve(|s| s.secondary(), Some("secondary")),
    )?;
    insert(
        ColorRole::OnSecondary,
        resolve(|s| s.on_secondary(), Some("on_secondary")),
    )?;
    insert(
        ColorRole::SecondaryContainer,
        resolve(|s| s.secondary_container(), Some("secondary_container")),
    )?;
    insert(
        ColorRole::OnSecondaryContainer,
        resolve(
            |s| s.on_secondary_container(),
            Some("on_secondary_container"),
        ),
    )?;
    insert(
        ColorRole::SecondaryFixed,
        resolve(|s| s.secondary_fixed(), None),
    )?;
    insert(
        ColorRole::SecondaryFixedDim,
        resolve(|s| s.secondary_fixed_dim(), None),
    )?;
    insert(
        ColorRole::OnSecondaryFixed,
        resolve(|s| s.on_secondary_fixed(), None),
    )?;
    insert(
        ColorRole::OnSecondaryFixedVariant,
        resolve(|s| s.on_secondary_fixed_variant(), None),
    )?;

    insert(
        ColorRole::Tertiary,
        resolve(|s| s.tertiary(), Some("tertiary")),
    )?;
    insert(
        ColorRole::OnTertiary,
        resolve(|s| s.on_tertiary(), Some("on_tertiary")),
    )?;
    insert(
        ColorRole::TertiaryContainer,
        resolve(|s| s.tertiary_container(), Some("tertiary_container")),
    )?;
    insert(
        ColorRole::OnTertiaryContainer,
        resolve(|s| s.on_tertiary_container(), Some("on_tertiary_container")),
    )?;
    insert(
        ColorRole::TertiaryFixed,
        resolve(|s| s.tertiary_fixed(), None),
    )?;
    insert(
        ColorRole::TertiaryFixedDim,
        resolve(|s| s.tertiary_fixed_dim(), None),
    )?;
    insert(
        ColorRole::OnTertiaryFixed,
        resolve(|s| s.on_tertiary_fixed(), None),
    )?;
    insert(
        ColorRole::OnTertiaryFixedVariant,
        resolve(|s| s.on_tertiary_fixed_variant(), None),
    )?;

    insert(ColorRole::Error, resolve(|s| s.error(), Some("error")))?;
    insert(
        ColorRole::OnError,
        resolve(|s| s.on_error(), Some("on_error")),
    )?;
    insert(
        ColorRole::ErrorContainer,
        resolve(|s| s.error_container(), Some("error_container")),
    )?;
    insert(
        ColorRole::OnErrorContainer,
        resolve(|s| s.on_error_container(), Some("on_error_container")),
    )?;

    insert(
        ColorRole::Background,
        resolve(|s| s.background(), Some("background")),
    )?;
    insert(
        ColorRole::OnBackground,
        resolve(|s| s.on_background(), Some("on_background")),
    )?;
    insert(
        ColorRole::Surface,
        resolve(|s| s.surface(), Some("surface")),
    )?;
    insert(
        ColorRole::OnSurface,
        resolve(|s| s.on_surface(), Some("on_surface")),
    )?;
    insert(
        ColorRole::SurfaceVariant,
        resolve(|s| s.surface_variant(), Some("surface_variant")),
    )?;
    insert(
        ColorRole::OnSurfaceVariant,
        resolve(|s| s.on_surface_variant(), Some("on_surface_variant")),
    )?;

    insert(
        ColorRole::SurfaceContainerLowest,
        resolve(|s| s.surface_container_lowest(), None),
    )?;
    insert(
        ColorRole::SurfaceContainerLow,
        resolve(|s| s.surface_container_low(), None),
    )?;
    insert(
        ColorRole::SurfaceContainer,
        resolve(|s| s.surface_container(), None),
    )?;
    insert(
        ColorRole::SurfaceContainerHigh,
        resolve(|s| s.surface_container_high(), None),
    )?;
    insert(
        ColorRole::SurfaceContainerHighest,
        resolve(|s| s.surface_container_highest(), None),
    )?;

    insert(
        ColorRole::InverseSurface,
        resolve(|s| s.inverse_surface(), Some("inverse_surface")),
    )?;
    insert(
        ColorRole::InverseOnSurface,
        resolve(|s| s.inverse_on_surface(), Some("inverse_on_surface")),
    )?;
    insert(
        ColorRole::InversePrimary,
        resolve(|s| s.inverse_primary(), Some("inverse_primary")),
    )?;

    insert(ColorRole::SurfaceDim, resolve(|s| s.surface_dim(), None))?;
    insert(
        ColorRole::SurfaceBright,
        resolve(|s| s.surface_bright(), None),
    )?;
    insert(ColorRole::SurfaceTint, resolve(|s| s.surface_tint(), None))?;

    insert(
        ColorRole::Outline,
        resolve(|s| s.outline(), Some("outline")),
    )?;
    insert(
        ColorRole::OutlineVariant,
        resolve(|s| s.outline_variant(), Some("outline_variant")),
    )?;

    insert(ColorRole::Shadow, resolve(|s| s.shadow(), Some("shadow")))?;
    insert(ColorRole::Scrim, resolve(|s| s.scrim(), Some("scrim")))?;

    // Terminal colors get their own dedicated, fixed-hue mapping.
    for (role, color) in ansi::ansi_colors(scheme) {
        colors.insert(role, color);
    }

    Ok(Palette::new(colors))
}

#[cfg(test)]
mod tests {
    use super::*;
    use material_colors::dynamic_color::Variant;
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

    /// With default parameters our generation must be exactly the official
    /// `DynamicScheme::by_variant` output — this is the guarantee that the
    /// palette is real Material You.
    #[test]
    fn test_default_scheme_matches_official_algorithm() {
        let seed = parse_hex_color("#FF5722").unwrap();
        let mine = generate_scheme(
            seed,
            false,
            SchemeType::TonalSpot,
            &AlgorithmParameters::default(),
        );
        let official = DynamicScheme::by_variant(seed, &Variant::TonalSpot, false, Some(0.0));

        assert_eq!(mine.primary(), official.primary());
        assert_eq!(mine.secondary(), official.secondary());
        assert_eq!(mine.tertiary(), official.tertiary());
        assert_eq!(mine.surface(), official.surface());
        assert_eq!(mine.error(), official.error());
    }

    /// MD3 guarantees secondary is desaturated and tertiary sits ~60° from the
    /// seed. Asserting chroma/hue relationships guards against regressions to a
    /// hand-rolled algorithm.
    #[test]
    fn test_tonal_spot_relationships() {
        use crate::color::{estimate_chroma, estimate_hct, hue_distance};

        let theme = json!({ "seed": "#FF5722" });
        let palette = generate_palette(&theme, false).unwrap();

        let primary = palette.get("primary").unwrap();
        let tertiary = palette.get("tertiary").unwrap();
        let secondary = palette.get("secondary").unwrap();

        let (primary_hue, _) = estimate_hct(primary.r, primary.g, primary.b);
        let (tertiary_hue, _) = estimate_hct(tertiary.r, tertiary.g, tertiary.b);
        assert!(
            (hue_distance(primary_hue, tertiary_hue) - 60.0).abs() < 25.0,
            "tertiary should be ~60° from primary (got {:.1}°)",
            hue_distance(primary_hue, tertiary_hue)
        );

        let secondary_chroma = estimate_chroma(secondary.r, secondary.g, secondary.b);
        assert!(
            secondary_chroma < 40.0,
            "secondary should be desaturated (got chroma {:.1})",
            secondary_chroma
        );
    }

    /// Different scheme types must actually change the palette.
    #[test]
    fn test_scheme_type_changes_palette() {
        let theme = json!({ "seed": "#FF5722" });
        let spot =
            generate_palette_with_params(&theme, false, SchemeType::TonalSpot, Default::default())
                .unwrap();
        let mono =
            generate_palette_with_params(&theme, false, SchemeType::Monochrome, Default::default())
                .unwrap();

        assert_ne!(
            spot.get("primary").unwrap().hex(),
            mono.get("primary").unwrap().hex()
        );
    }
}
