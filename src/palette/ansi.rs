//! Terminal (ANSI) color mapping.
//!
//! A terminal theme does not care about MD3 role names — it needs sixteen
//! stable, recognizable colors. We therefore derive them from fixed HCT hue
//! anchors (so `red` stays red, `green` stays green, …) while taking the tone
//! from the generated scheme's mode so every color stays readable against the
//! terminal background.
//!
//! `bright_*` variants are made prominent two ways at once: a lighter tone
//! (brighter on both dark and light backgrounds, matching the traditional
//! "bold/bright" meaning) **and** a higher chroma, so they read as vivid
//! emphasised versions rather than darker, muddier ones.

use material_colors::color::Argb;
use material_colors::dynamic_color::DynamicScheme;
use material_colors::palette::TonalPalette;

use super::types::ColorRole;
use crate::core::color::Color;

/// HCT hue anchors (degrees) for the five non-error chromatic ANSI slots.
/// The sixth (red) reuses the scheme's dedicated error palette.
const CHROMATIC_ANCHORS: [(f64, ColorRole, ColorRole); 5] = [
    (70.0, ColorRole::Yellow, ColorRole::BrightYellow),
    (140.0, ColorRole::Green, ColorRole::BrightGreen),
    (200.0, ColorRole::Cyan, ColorRole::BrightCyan),
    (260.0, ColorRole::Blue, ColorRole::BrightBlue),
    (330.0, ColorRole::Magenta, ColorRole::BrightMagenta),
];

/// Chroma for the normal chromatic slots — restrained, so the bright variants
/// can clearly out-saturate them.
const NORMAL_CHROMA: f64 = 40.0;

/// Chroma for the bright chromatic slots — pushed high (gamut-mapped down when
/// a hue cannot sustain it) to make `bright_*` the vivid version.
const BRIGHT_CHROMA: f64 = 68.0;

/// Build the sixteen ANSI terminal colors from a generated scheme.
///
/// The returned roles are empty of duplicates (callers insert them into a map).
pub fn ansi_colors(scheme: &DynamicScheme) -> Vec<(ColorRole, Color)> {
    let mut colors = Vec::with_capacity(16);

    // Greyscale ladder taken straight from the scheme's neutral palette. These
    // are fixed tones so they always form a canonical dark→light ramp.
    let neutral = &scheme.neutral_palette;
    colors.push((ColorRole::Black, from_palette(neutral, 10.0)));
    colors.push((ColorRole::BrightBlack, from_palette(neutral, 30.0)));
    colors.push((ColorRole::White, from_palette(neutral, 90.0)));
    colors.push((ColorRole::BrightWhite, from_palette(neutral, 100.0)));

    // Reds reuse the scheme's error palette (hue ~25°) for the normal variant so
    // red stays semantically tied to the theme's error color; the bright variant
    // keeps that hue and cranks the chroma.
    let error = &scheme.error_palette;
    colors.push((ColorRole::Red, from_palette(error, tone(scheme, false))));
    colors.push((
        ColorRole::BrightRed,
        from_palette(
            &TonalPalette::from_hue_and_chroma(error.hue(), BRIGHT_CHROMA),
            tone(scheme, true),
        ),
    ));

    // Fixed-hue chromatic anchors.
    for (hue, normal, bright) in CHROMATIC_ANCHORS {
        colors.push((
            normal,
            from_palette(
                &TonalPalette::from_hue_and_chroma(hue, NORMAL_CHROMA),
                tone(scheme, false),
            ),
        ));
        colors.push((
            bright,
            from_palette(
                &TonalPalette::from_hue_and_chroma(hue, BRIGHT_CHROMA),
                tone(scheme, true),
            ),
        ));
    }

    colors
}

/// Tone to use for a chromatic slot.
///
/// `bright` always sits at a lighter tone than the normal slot, in both modes,
/// so it keeps the traditional "bright/bold" meaning. Normal slots in light
/// mode sit lower (more contrast) so the set as a whole stays readable.
fn tone(scheme: &DynamicScheme, bright: bool) -> f64 {
    match (scheme.is_dark, bright) {
        (true, false) => 70.0,
        (true, true) => 80.0,
        (false, false) => 45.0,
        (false, true) => 60.0,
    }
}

/// Resolve a palette tone to a [`Color`].
fn from_palette(palette: &TonalPalette, tone: f64) -> Color {
    let argb: Argb = palette.get_hct(tone).into();
    Color::new(argb.red, argb.green, argb.blue, argb.alpha as f64 / 255.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use material_colors::dynamic_color::Variant;
    use material_colors::hct::Hct;
    use std::collections::{HashMap, HashSet};

    fn scheme(is_dark: bool) -> DynamicScheme {
        DynamicScheme::by_variant(
            Argb::from_u32(0xFF_6750A4),
            &Variant::TonalSpot,
            is_dark,
            Some(0.0),
        )
    }

    fn argb_of(color: &Color) -> Argb {
        let (r, g, b) = color.rgb();
        Argb::from_u32(((r as u32) << 16) | ((g as u32) << 8) | b as u32)
    }

    #[test]
    fn test_ansi_has_all_sixteen_distinct_roles() {
        let colors = ansi_colors(&scheme(true));
        assert_eq!(colors.len(), 16);
        let roles: HashSet<_> = colors.iter().map(|(role, _)| *role).collect();
        assert_eq!(roles.len(), 16);
    }

    #[test]
    fn test_chromatic_anchors_keep_expected_hues() {
        let colors: HashMap<_, _> = ansi_colors(&scheme(false)).into_iter().collect();
        let hue = |role| Hct::new(argb_of(&colors[&role])).get_hue();
        assert!((hue(ColorRole::Yellow) - 70.0).abs() < 20.0);
        assert!((hue(ColorRole::Green) - 140.0).abs() < 20.0);
        assert!((hue(ColorRole::Blue) - 260.0).abs() < 20.0);
        assert!((hue(ColorRole::Magenta) - 330.0).abs() < 20.0);
    }

    #[test]
    fn test_bright_variants_are_lighter_and_more_vivid() {
        for is_dark in [true, false] {
            let colors: HashMap<_, _> = ansi_colors(&scheme(is_dark)).into_iter().collect();
            let hct = |role| Hct::new(argb_of(&colors[&role]));

            // Bright always sits at a lighter tone (traditional bold/bright meaning).
            assert!(
                hct(ColorRole::BrightGreen).get_tone() > hct(ColorRole::Green).get_tone(),
                "bright green should be lighter (dark={is_dark})"
            );
            assert!(
                hct(ColorRole::BrightRed).get_tone() > hct(ColorRole::Red).get_tone(),
                "bright red should be lighter (dark={is_dark})"
            );

            // ...and clearly more saturated, so it never looks darker/muddier.
            assert!(
                hct(ColorRole::BrightGreen).get_chroma() > hct(ColorRole::Green).get_chroma(),
                "bright green should be more vivid (dark={is_dark})"
            );
            assert!(
                hct(ColorRole::BrightMagenta).get_chroma() > hct(ColorRole::Magenta).get_chroma(),
                "bright magenta should be more vivid (dark={is_dark})"
            );

            // Bright and normal are never the same color.
            assert_ne!(
                colors[&ColorRole::BrightBlue].hex(),
                colors[&ColorRole::Blue].hex()
            );
        }
    }

    #[test]
    fn test_greyscale_ladder_is_ordered() {
        let colors: HashMap<_, _> = ansi_colors(&scheme(true)).into_iter().collect();
        let tone = |role| Hct::new(argb_of(&colors[&role])).get_tone();
        assert!(tone(ColorRole::Black) < tone(ColorRole::BrightBlack));
        assert!(tone(ColorRole::BrightBlack) < tone(ColorRole::White));
        assert!(tone(ColorRole::White) < tone(ColorRole::BrightWhite));
    }
}
