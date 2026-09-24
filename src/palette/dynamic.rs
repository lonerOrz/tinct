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
use crate::core::Mode;
use crate::core::color::Color;
use crate::image::SchemeType;
use std::collections::HashMap;

/// Extract seed hex from theme: prefer "seed", fallback to "Primary".
pub fn extract_seed_hex(theme: &Value) -> Option<&str> {
    theme
        .get("seed")
        .and_then(|v| v.as_str())
        .or_else(|| theme.get("Primary").and_then(|v| v.as_str()))
}

/// Collect every hex color defined in a theme value, recursively.
///
/// A `-t <theme>` file is just a map of colors, so this lets the terminal
/// palette be generated from the theme's *own* colors (their hue, chroma and
/// lightness) instead of only its seed — a soft, Catppuccin-like theme then
/// yields a soft terminal palette. Strings are accepted only when they start
/// with `#` and parse as a 6- or 8-digit hex color; duplicates are dropped and
/// the result is capped to keep pathological inputs cheap.
pub fn collect_theme_colors(theme: &Value) -> Vec<Color> {
    const MAX_COLORS: usize = 64;
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    collect_colors_into(theme, &mut out, &mut seen, MAX_COLORS);
    out
}

fn collect_colors_into(
    value: &Value,
    out: &mut Vec<Color>,
    seen: &mut std::collections::HashSet<String>,
    cap: usize,
) {
    if out.len() >= cap {
        return;
    }
    match value {
        Value::String(text) => {
            let trimmed = text.trim();
            if trimmed.starts_with('#')
                && let Ok(argb) = parse_hex_color(trimmed)
                && seen.insert(trimmed.to_ascii_lowercase())
            {
                out.push(argb_to_color(argb));
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_colors_into(item, out, seen, cap);
            }
        }
        Value::Object(map) => {
            for item in map.values() {
                collect_colors_into(item, out, seen, cap);
            }
        }
        _ => {}
    }
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
    build_palette(
        theme,
        is_dark_mode,
        scheme_type,
        params,
        &[],
        &ansi::AnsiParams::default(),
    )
}

/// Generate a palette, threading the source image's color clusters into the
/// terminal (ANSI) mapping. See [`super::ansi`].
pub fn build_palette(
    theme: &Value,
    is_dark_mode: bool,
    scheme_type: SchemeType,
    params: AlgorithmParameters,
    source_colors: &[Color],
    ansi_params: &ansi::AnsiParams,
) -> Result<Palette, String> {
    let seed_hex =
        extract_seed_hex(theme).ok_or("Theme must contain either 'seed' or 'Primary' color")?;

    let seed_argb = parse_hex_color(seed_hex)?;
    let scheme = generate_scheme(seed_argb, is_dark_mode, scheme_type, &params);
    scheme_to_palette(&scheme, theme, source_colors, ansi_params)
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
    let has_tone = params.seed_tone.is_some();
    let has_floor = params.chroma_floor > 0.0;
    if params.hue_shift == 0 && params.saturation_adjustment == 0 && !has_tone && !has_floor {
        return seed;
    }

    let hct = Hct::new(seed);
    let hue = ((hct.get_hue() + params.hue_shift as f64) % 360.0 + 360.0) % 360.0;
    let chroma = (hct.get_chroma() * (1.0 + params.saturation_adjustment as f64 / 100.0))
        .clamp(0.0, 120.0)
        .max(params.chroma_floor);
    let tone = params.seed_tone.unwrap_or_else(|| hct.get_tone());

    let adjusted: Argb = Hct::from(hue, chroma, tone).into();
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
fn scheme_to_palette(
    scheme: &DynamicScheme,
    theme: &Value,
    source_colors: &[Color],
    ansi_params: &ansi::AnsiParams,
) -> Result<Palette, String> {
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

    // Terminal colors get their own dedicated, wallust-inspired mapping, using
    // the real image clusters where they match an ANSI hue slot.
    for (role, color) in ansi::ansi_colors(scheme, source_colors, ansi_params) {
        colors.insert(role, color);
    }

    Ok(Palette::new(colors))
}

/// Stateful palette generator: seed adjustments plus the MD3 scheme variant.
///
/// A thin facade over [`generate_palette_with_params`] so callers can hold a
/// configured generator instead of threading parameters through every call.
/// The historical name is kept to avoid churning the public API.
pub struct LegacyPaletteGenerator {
    params: AlgorithmParameters,
    scheme_type: SchemeType,
    /// Optional scheme variant used only in light mode. Falls back to
    /// `scheme_type` when unset.
    light_scheme_type: Option<SchemeType>,
    /// Terminal (ANSI) mapping knobs.
    ansi: ansi::AnsiParams,
    /// Representative wallpaper clusters, used for ANSI hue snapping.
    source_colors: Vec<Color>,
}

impl LegacyPaletteGenerator {
    pub fn new(params: AlgorithmParameters, scheme_type: SchemeType) -> Self {
        Self {
            params,
            scheme_type,
            light_scheme_type: None,
            ansi: ansi::AnsiParams::default(),
            source_colors: Vec::new(),
        }
    }

    /// Generator with no seed adjustments and the default Tonal Spot scheme.
    pub fn with_defaults() -> Self {
        Self {
            params: AlgorithmParameters::default(),
            scheme_type: SchemeType::TonalSpot,
            light_scheme_type: None,
            ansi: ansi::AnsiParams::default(),
            source_colors: Vec::new(),
        }
    }

    /// Attach the source image's extracted clusters (empty for seed/theme
    /// sources). Only the terminal palette uses them.
    pub fn with_source_colors(mut self, source_colors: Vec<Color>) -> Self {
        self.source_colors = source_colors;
        self
    }

    /// Configure the terminal (ANSI) mapping.
    pub fn with_ansi(mut self, ansi: ansi::AnsiParams) -> Self {
        self.ansi = ansi;
        self
    }

    /// Use a different MD3 variant in light mode.
    pub fn with_light_scheme(mut self, scheme_type: SchemeType) -> Self {
        self.light_scheme_type = Some(scheme_type);
        self
    }

    /// The scheme variant this generator produces in dark mode.
    pub fn scheme_type(&self) -> SchemeType {
        self.scheme_type
    }

    pub fn generate(&self, theme: &Value, mode: Mode) -> crate::core::Result<Palette> {
        let scheme_type = if mode.is_light() {
            self.light_scheme_type.unwrap_or(self.scheme_type)
        } else {
            self.scheme_type
        };
        build_palette(
            theme,
            mode.is_dark(),
            scheme_type,
            self.params,
            &self.source_colors,
            &self.ansi,
        )
        .map_err(crate::core::Error::Palette)
    }
}

impl Default for LegacyPaletteGenerator {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::test_support::ANSI_ROLE_NAMES;
    use material_colors::dynamic_color::Variant;
    use serde_json::json;

    #[test]
    fn test_generate_palette_shapes_and_overrides() {
        let palette = generate_palette(&json!({ "seed": "#FF5722" }), false).unwrap();
        for role in ["primary", "secondary", "tertiary", "surface"] {
            assert!(palette.get(role).is_some(), "missing role {role}");
        }
        assert_ne!(
            palette.get("primary").unwrap().hex(),
            String::from("#000000")
        );

        let dark = generate_palette(&json!({ "seed": "#2196F3" }), true).unwrap();
        assert!(dark.get("surface").is_some());

        let overridden =
            generate_palette(&json!({ "seed": "#FF5722", "error": "#F44336" }), false).unwrap();
        assert_eq!(overridden.get("error").unwrap().hex(), "#F44336");
    }

    #[test]
    fn test_collect_theme_colors_skips_non_colors_and_dedupes() {
        let theme = json!({
            "seed": "#6750A4",
            "primary": "#6750a4",       // duplicate of seed (case-insensitive)
            "secondary": "#BCD2E8",
            "nested": { "list": ["#112233", "not-a-color", "#112233"] },
            "name": "abcdef",           // 6 hex chars but no '#': must be ignored
            "count": 42,
            "flag": true,
        });
        let colors = collect_theme_colors(&theme);
        assert_eq!(colors.len(), 3);
    }

    #[test]
    fn test_collect_theme_colors_on_empty_theme() {
        assert!(collect_theme_colors(&json!({})).is_empty());
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
        use crate::core::color::{estimate_chroma, estimate_hct, hue_distance};

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

    /// Different scheme types must actually change the palette, through both
    /// the parameterised entry point and the facade.
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

        let facade = |scheme| {
            LegacyPaletteGenerator::new(AlgorithmParameters::default(), scheme)
                .generate(&theme, Mode::Dark)
                .unwrap()
        };
        assert_ne!(
            facade(SchemeType::TonalSpot).get("primary").unwrap().hex(),
            facade(SchemeType::Monochrome).get("primary").unwrap().hex()
        );
    }

    // ---- LegacyPaletteGenerator facade (formerly palette/adapter.rs) ----

    #[test]
    fn test_legacy_palette_generator_construction() {
        let custom = LegacyPaletteGenerator::new(
            AlgorithmParameters {
                saturation_adjustment: 10,
                hue_shift: 15,
                ..Default::default()
            },
            SchemeType::Content,
        );
        assert_eq!(custom.params.saturation_adjustment, 10);
        assert_eq!(custom.scheme_type(), SchemeType::Content);

        for generator in [
            LegacyPaletteGenerator::with_defaults(),
            LegacyPaletteGenerator::default(),
        ] {
            assert_eq!(generator.params.saturation_adjustment, 0);
            assert_eq!(generator.params.hue_shift, 0);
            assert_eq!(generator.scheme_type(), SchemeType::TonalSpot);
        }
    }

    #[test]
    fn test_legacy_palette_generator_generate_modes_and_params() {
        let cases = [
            (AlgorithmParameters::default(), Mode::Dark, "#FF5722"),
            (AlgorithmParameters::default(), Mode::Light, "#2196F3"),
            (
                AlgorithmParameters {
                    hue_shift: 180,
                    ..Default::default()
                },
                Mode::Dark,
                "#FF0000",
            ),
            (
                AlgorithmParameters {
                    saturation_adjustment: 50,
                    ..Default::default()
                },
                Mode::Dark,
                "#FF5722",
            ),
        ];

        for (params, mode, seed) in cases {
            let theme = json!({ "seed": seed });
            let generator = LegacyPaletteGenerator::new(params, SchemeType::TonalSpot);
            let palette = generator
                .generate(&theme, mode)
                .unwrap_or_else(|e| panic!("{seed} {mode:?}: {e}"));

            for role in ["primary", "secondary", "tertiary", "surface", "error"] {
                let color = palette
                    .get(role)
                    .unwrap_or_else(|| panic!("missing {role} for {seed}"));
                assert!(color.hex().starts_with('#'), "bad hex for {role}");
            }
        }
    }

    #[test]
    fn test_legacy_palette_generator_generate_all_color_roles() {
        let generator = LegacyPaletteGenerator::with_defaults();
        let theme = json!({ "seed": "#6200EE" });

        let result = generator.generate(&theme, Mode::Dark);
        assert!(result.is_ok());

        let palette = result.unwrap();
        let map = palette.to_map();

        let expected_roles: &[&str] = &[
            "primary",
            "on_primary",
            "primary_container",
            "on_primary_container",
            "secondary",
            "on_secondary",
            "secondary_container",
            "on_secondary_container",
            "tertiary",
            "on_tertiary",
            "tertiary_container",
            "on_tertiary_container",
            "error",
            "on_error",
            "error_container",
            "on_error_container",
            "background",
            "on_background",
            "surface",
            "on_surface",
            "surface_variant",
            "on_surface_variant",
            "outline",
            "outline_variant",
            "shadow",
            "scrim",
            "inverse_surface",
            "inverse_on_surface",
            "inverse_primary",
            "surface_dim",
            "surface_bright",
            "surface_container_lowest",
            "surface_container_low",
            "surface_container",
            "surface_container_high",
            "surface_container_highest",
        ];

        for role in expected_roles {
            assert!(map.contains_key(*role), "Missing color role: {}", role);
            let color = map.get(*role).unwrap();
            assert!(!color.hex().is_empty(), "Empty hex for role: {}", role);
        }

        for role in ANSI_ROLE_NAMES {
            assert!(map.contains_key(role), "Missing ANSI role: {}", role);
            assert!(!map.get(role).unwrap().hex().is_empty());
        }
    }

    #[test]
    fn test_legacy_palette_generator_generate_with_overrides() {
        let generator = LegacyPaletteGenerator::with_defaults();
        let theme = json!({ "seed": "#FF5722", "error": "#FF0000", "surface": "#121212" });

        let result = generator.generate(&theme, Mode::Dark);
        assert!(result.is_ok());

        let palette = result.unwrap();
        let map = palette.to_map();
        let error = map.get("error").unwrap();
        assert_eq!(error.hex(), "#FF0000");

        let surface = map.get("surface").unwrap();
        assert_eq!(surface.hex(), "#121212");
    }
}
