//! Terminal (ANSI) color mapping.
//!
//! A terminal does not care about MD3 role names — it needs sixteen stable,
//! recognizable colors. MD3 has no concept of ANSI 16, so the desktop/UI palette
//! stays pure Material You while the terminal palette borrows the logic that
//! makes [wallust](https://codeberg.org/explosion-mental/wallust) (MIT) look so
//! good:
//!
//! 1. **Work in CIE LCh, not RGB/HSL.** Lightness (`L`) is perceptually uniform,
//!    so we can normalize a color's tone without it turning grey or muddy.
//! 2. **Snap real colors to ANSI hue slots.** Each of the six chromatic slots
//!    owns a hue range (`lchansi`). If the wallpaper or `-t` theme contains a
//!    color in that range we adopt *its* hue and blend its chroma/lightness
//!    toward the slot anchor — the terminal reflects the source instead of a
//!    fixed rainbow. Each candidate color is consumed once.
//! 3. **Fall back to the full anchor.** When nothing matches, the slot uses
//!    wallust's saturated anchor at its canonical hue, so every hue family stays
//!    recognizable and vivid even for a monochrome wallpaper.
//! 4. **Deliberate lightness ladder.** Normal slots sit at a mode-appropriate
//!    `L*`; `bright_*` variants are both lighter *and* more saturated, never the
//!    darker/muddier version. Every chromatic slot is then pushed away from the
//!    background until it meets a WCAG contrast floor.
//!
//! The slot hue ranges and the `(anchor + 2·average) / 3` weighting are adapted
//! from wallust's `lchansi` colorspace (MIT © explosion-mental).

use material_colors::color::Argb;
use material_colors::dynamic_color::DynamicScheme;
use palette::color_difference::Wcag21RelativeContrast;
use palette::convert::FromColorUnclamped;
use palette::{Clamp, IntoColor, LabHue, Lch, Srgb};

use super::types::ColorRole;
use crate::core::color::Color;

/// Near-grey clusters are useless for hue snapping, so they are ignored.
/// Clusters at or below this chroma are effectively neutral (MD3 neutrals carry
/// 4–8 chroma, and UI backgrounds/surfaces sit around 10–12). They are ignored
/// for hue snapping so a theme's background never contaminates a chromatic slot.
const MIN_CLUSTER_CHROMA: f32 = 12.0;

/// Minimum contrast ratio between a chromatic slot and the terminal background.
/// WCAG AA for large text (3:1) — enough to stay readable without washing out.
const CONTRAST_TARGET: f32 = 3.0;

/// Upper bound on CIE LCh chroma; roughly the edge of the sRGB gamut.
const MAX_CHROMA: f32 = 132.0;

/// One chromatic ANSI slot.
struct Slot {
    normal: ColorRole,
    bright: ColorRole,
    /// Hue range owned by this slot, in CIE LCh degrees: `[hue_start, hue_end)`.
    hue_start: f32,
    hue_end: f32,
    /// Canonical hue used when no candidate color matches this slot.
    anchor_hue: f32,
    /// Wallust anchors (`light_def` / `chroma_def`): the vivid, legible target
    /// that matched source colors are blended toward.
    light: f32,
    chroma: f32,
}

/// ANSI order: red, green, yellow, blue, magenta, cyan.
///
/// Ranges partition the CIE LCh hue wheel so every slot stays semantically
/// correct (red = error, green = success, yellow = warning, …). The anchors are
/// vivid representatives of each family — the colours wallust converges on —
/// and are used verbatim when the input has nothing in the slot's range.
const SLOTS: [Slot; 6] = [
    Slot {
        normal: ColorRole::Red,
        bright: ColorRole::BrightRed,
        hue_start: 0.0,
        hue_end: 60.0,
        anchor_hue: 40.0,
        light: 52.0,
        chroma: 181.0,
    },
    Slot {
        normal: ColorRole::Green,
        bright: ColorRole::BrightGreen,
        hue_start: 121.0,
        hue_end: 180.0,
        anchor_hue: 151.0,
        light: 55.0,
        chroma: 128.0,
    },
    Slot {
        normal: ColorRole::Yellow,
        bright: ColorRole::BrightYellow,
        hue_start: 61.0,
        hue_end: 120.0,
        anchor_hue: 104.0,
        light: 80.0,
        chroma: 128.0,
    },
    Slot {
        normal: ColorRole::Blue,
        bright: ColorRole::BrightBlue,
        hue_start: 211.0,
        hue_end: 300.0,
        anchor_hue: 288.0,
        light: 52.0,
        chroma: 181.0,
    },
    Slot {
        normal: ColorRole::Magenta,
        bright: ColorRole::BrightMagenta,
        hue_start: 301.0,
        hue_end: 360.0,
        anchor_hue: 350.0,
        light: 55.0,
        chroma: 128.0,
    },
    Slot {
        normal: ColorRole::Cyan,
        bright: ColorRole::BrightCyan,
        hue_start: 181.0,
        hue_end: 210.0,
        anchor_hue: 204.0,
        light: 84.0,
        chroma: 128.0,
    },
];

/// Build the sixteen ANSI terminal colors from a generated scheme.
///
/// `source_colors` are the representative clusters extracted from the wallpaper
/// (empty for seed/theme sources, in which case every chromatic slot is
/// synthesized from the seed). The returned roles are unique.
pub fn ansi_colors(scheme: &DynamicScheme, source_colors: &[Color]) -> Vec<(ColorRole, Color)> {
    let is_dark = scheme.is_dark;

    // Wallpaper clusters, in CIE LCh, grey ones dropped.
    let mut remaining: Vec<Lch> = source_colors
        .iter()
        .map(|c| lch_of(color_to_argb(c)))
        .filter(|c| c.chroma > MIN_CLUSTER_CHROMA)
        .collect();

    let background = srgb_of(lch_of(scheme.surface()));

    let mut colors = Vec::with_capacity(16);

    // Greyscale ladder from the scheme's neutral palette: fixed tones, so it
    // always forms a canonical dark→light ramp that matches the mode.
    let neutral = &scheme.neutral_palette;
    colors.push((ColorRole::Black, from_palette(neutral, 10.0)));
    colors.push((ColorRole::BrightBlack, from_palette(neutral, 30.0)));
    colors.push((ColorRole::White, from_palette(neutral, 90.0)));
    colors.push((ColorRole::BrightWhite, from_palette(neutral, 100.0)));

    for slot in &SLOTS {
        let (hue, chroma, light) = slot_texture(slot, &mut remaining);

        let normal = render(hue, chroma, light, is_dark, false, background);
        let bright = render(hue, chroma, light, is_dark, true, background);

        colors.push((slot.normal, normal));
        colors.push((slot.bright, bright));
    }

    colors
}

/// Resolve a slot's hue, chroma and lightness from the remaining candidates.
///
/// Matching is per-slot: every candidate whose hue falls in the slot's range is
/// consumed (removed from `remaining`) so no source color is reused. Matched
/// colors set the hue, while their chroma/lightness are blended toward the slot
/// anchor with wallust's `(anchor + 2·average) / 3` weighting — the source tints
/// the palette but the slot keeps the vividness and legibility the anchor
/// encodes. With no match the anchor itself is used, which is what keeps a
/// monochrome input from collapsing into a washed-out palette.
fn slot_texture(slot: &Slot, remaining: &mut Vec<Lch>) -> (f32, f32, f32) {
    let mut hues = Vec::new();
    let mut chromas = Vec::new();
    let mut lights = Vec::new();

    remaining.retain(|c| {
        let hue = c.hue.into_inner();
        if hue >= slot.hue_start && hue < slot.hue_end {
            hues.push(hue);
            chromas.push(c.chroma);
            lights.push(c.l);
            false
        } else {
            true
        }
    });

    if hues.is_empty() {
        (slot.anchor_hue, slot.chroma, slot.light)
    } else {
        let hue = mean(&hues).rem_euclid(360.0);
        let chroma = (slot.chroma + 2.0 * mean(&chromas)) / 3.0;
        let light = (slot.light + 2.0 * mean(&lights)) / 3.0;
        (hue, chroma.clamp(0.0, MAX_CHROMA), light)
    }
}

/// Light-mode colors compress their wallust anchor by this factor into a band
/// around L\*40, so no slot (blue, red) collapses to near-black on a light
/// background while the per-slot character is preserved.
const LIGHT_L_SCALE: f32 = 0.55;

/// Lightness shift paired with [`LIGHT_L_SCALE`].
const LIGHT_L_SHIFT: f32 = 2.0;

/// How much lighter the `bright_*` variant is than its normal counterpart.
/// (Bright variants also receive a chroma boost; see [`render`].)
const BRIGHT_LIFT: f32 = 8.0;

/// Turn a slot's hue/chroma/lightness into a final color.
///
/// Lightness is normalized per mode so the whole set stays legible, and the
/// `bright_*` variant is both lighter and more saturated. Every color is gamut
/// mapped (preserving hue) and pushed away from the background until it clears
/// the WCAG contrast floor.
fn render(
    hue: f32,
    chroma: f32,
    light: f32,
    is_dark: bool,
    bright: bool,
    background: Srgb<f32>,
) -> Color {
    let base = if is_dark {
        light.clamp(18.0, 88.0)
    } else {
        // Light mode compresses the per-slot character into a band around L*40
        // so no slot collapses to near-black on a light background.
        (light * LIGHT_L_SCALE + LIGHT_L_SHIFT).clamp(30.0, 60.0)
    };
    let lightness = if bright {
        (base + BRIGHT_LIFT).min(if is_dark { 88.0 } else { 64.0 })
    } else {
        base
    };

    let chroma = if bright {
        (chroma * 1.2).min(MAX_CHROMA)
    } else {
        chroma.min(MAX_CHROMA)
    };

    // Fit to sRGB *before* contrast so clipping never distorts the hue.
    let lch = gamut_map(Lch::new(lightness, chroma, LabHue::from_degrees(hue)));
    let lch = ensure_contrast(lch, background, is_dark);
    // The contrast step changes lightness, which can leave the gamut again.
    lch_to_color(gamut_map(lch))
}

/// Reduce chroma until the color fits inside sRGB, preserving hue and lightness.
///
/// Clamping each RGB channel independently (what `Clamp` does) can swing the hue
/// dramatically for saturated colors; reducing chroma is the perceptually sane
/// way to bring a color back into gamut.
fn gamut_map(lch: Lch) -> Lch {
    if in_gamut(lch) {
        return lch;
    }

    let (mut lo, mut hi) = (0.0_f32, lch.chroma);
    for _ in 0..16 {
        let mid = (lo + hi) / 2.0;
        if in_gamut(Lch::new(lch.l, mid, lch.hue)) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Lch::new(lch.l, lo, lch.hue)
}

fn in_gamut(lch: Lch) -> bool {
    let srgb: Srgb<f32> = Srgb::from_color_unclamped(lch);
    let fits = |c: f32| (-1.0e-4..=1.0 + 1.0e-4).contains(&c);
    fits(srgb.red) && fits(srgb.green) && fits(srgb.blue)
}

/// Push a color away from the background until it clears [`CONTRAST_TARGET`].
fn ensure_contrast(mut lch: Lch, background: Srgb<f32>, is_dark: bool) -> Lch {
    let step = if is_dark { 6.0 } else { -6.0 };
    let mut i = 0;
    while srgb_of(lch).relative_contrast(background) < CONTRAST_TARGET && i < 12 {
        lch.l = (lch.l + step).clamp(0.0, 100.0);
        i += 1;
    }
    lch
}

// ---- conversion helpers ----

fn mean(values: &[f32]) -> f32 {
    values.iter().sum::<f32>() / values.len().max(1) as f32
}

fn lch_of(argb: Argb) -> Lch {
    let srgb = Srgb::new(argb.red, argb.green, argb.blue).into_format::<f32>();
    srgb.into_color()
}

fn srgb_of(lch: Lch) -> Srgb<f32> {
    let srgb: Srgb<f32> = lch.into_color();
    srgb.clamp()
}

fn lch_to_color(lch: Lch) -> Color {
    let (r, g, b) = srgb_of(lch).into_format::<u8>().into_components();
    Color::new(r, g, b, 1.0)
}

fn color_to_argb(color: &Color) -> Argb {
    let (r, g, b) = color.rgb();
    Argb::from_u32(((r as u32) << 16) | ((g as u32) << 8) | b as u32)
}

/// Resolve a palette tone to a [`Color`].
fn from_palette(palette: &material_colors::palette::TonalPalette, tone: f64) -> Color {
    let argb: Argb = palette.get_hct(tone).into();
    Color::new(argb.red, argb.green, argb.blue, argb.alpha as f64 / 255.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use material_colors::dynamic_color::Variant;
    use std::collections::{HashMap, HashSet};

    fn scheme(is_dark: bool) -> DynamicScheme {
        DynamicScheme::by_variant(
            Argb::from_u32(0xFF_6750A4),
            &Variant::TonalSpot,
            is_dark,
            Some(0.0),
        )
    }

    fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color::new(r, g, b, 1.0)
    }

    fn lch(color: &Color) -> Lch {
        lch_of(color_to_argb(color))
    }

    #[test]
    fn test_ansi_has_all_sixteen_distinct_roles() {
        let colors = ansi_colors(&scheme(true), &[]);
        assert_eq!(colors.len(), 16);
        let roles: HashSet<_> = colors.iter().map(|(role, _)| *role).collect();
        assert_eq!(roles.len(), 16);
    }

    #[test]
    fn test_chromatic_anchors_keep_expected_hues() {
        // With no image clusters each slot falls back to its vivid anchor.
        let colors: HashMap<_, _> = ansi_colors(&scheme(false), &[]).into_iter().collect();
        let hue = |role| lch(&colors[&role]).hue.into_inner();
        let near = |a: f32, b: f32| (a - b).rem_euclid(360.0).min((b - a).rem_euclid(360.0)) < 15.0;

        assert!(near(hue(ColorRole::Red), 40.0));
        assert!(near(hue(ColorRole::Green), 151.0));
        assert!(near(hue(ColorRole::Yellow), 104.0));
        assert!(near(hue(ColorRole::Blue), 288.0));
        assert!(near(hue(ColorRole::Magenta), 350.0));
        assert!(near(hue(ColorRole::Cyan), 204.0));
    }

    #[test]
    fn test_clusters_snap_to_matching_slot() {
        // A wallpaper that is mostly teal: the cyan slot must adopt it instead of
        // the canonical 195° anchor, while red (absent) still falls back.
        let teal = rgb(0, 150, 136); // #009688
        let colors: HashMap<_, _> = ansi_colors(&scheme(true), &[teal]).into_iter().collect();

        let cyan = lch(&colors[&ColorRole::Cyan]).hue.into_inner();
        let teal_hue = lch(&teal).hue.into_inner();
        assert!(
            (cyan - teal_hue)
                .rem_euclid(360.0)
                .min((teal_hue - cyan).rem_euclid(360.0))
                < 12.0,
            "cyan slot should follow the real cluster (cyan {cyan:.1} vs teal {teal_hue:.1})"
        );

        let red = lch(&colors[&ColorRole::Red]).hue.into_inner();
        assert!((red - 40.0).abs() < 15.0, "absent red falls back to 40°");
    }

    /// `bright_*` must read as a distinct, livelier version of the normal slot:
    /// always lighter, always a different color. (Chroma is boosted as much as
    /// sRGB allows, but a lighter color physically holds less chroma for some
    /// hues, so vividness is not asserted per-slot.)
    #[test]
    fn test_bright_variants_are_lighter_and_distinct() {
        for is_dark in [true, false] {
            let colors: HashMap<_, _> = ansi_colors(&scheme(is_dark), &[]).into_iter().collect();

            for (normal, bright) in [
                (ColorRole::Red, ColorRole::BrightRed),
                (ColorRole::Green, ColorRole::BrightGreen),
                (ColorRole::Yellow, ColorRole::BrightYellow),
                (ColorRole::Blue, ColorRole::BrightBlue),
                (ColorRole::Magenta, ColorRole::BrightMagenta),
                (ColorRole::Cyan, ColorRole::BrightCyan),
            ] {
                let normal_l = lch(&colors[&normal]).l;
                let bright_l = lch(&colors[&bright]).l;
                assert!(
                    bright_l > normal_l,
                    "{bright:?} must be lighter than {normal:?} (dark={is_dark}): \
                     {bright_l:.1} vs {normal_l:.1}"
                );
                assert_ne!(
                    colors[&bright].hex(),
                    colors[&normal].hex(),
                    "{bright:?} must differ from {normal:?} (dark={is_dark})"
                );
            }
        }
    }

    #[test]
    fn test_chromatic_slots_clear_contrast_floor() {
        for is_dark in [true, false] {
            let s = scheme(is_dark);
            let background = srgb_of(lch_of(s.surface()));
            let colors: HashMap<_, _> = ansi_colors(&s, &[]).into_iter().collect();

            for role in [
                ColorRole::Red,
                ColorRole::Green,
                ColorRole::Yellow,
                ColorRole::Blue,
                ColorRole::Magenta,
                ColorRole::Cyan,
                ColorRole::BrightRed,
                ColorRole::BrightGreen,
                ColorRole::BrightYellow,
                ColorRole::BrightBlue,
                ColorRole::BrightMagenta,
                ColorRole::BrightCyan,
            ] {
                let ratio = srgb_of(lch(&colors[&role])).relative_contrast(background);
                assert!(
                    ratio >= CONTRAST_TARGET - 0.05,
                    "{role:?} contrast {ratio:.2} below floor (dark={is_dark})"
                );
            }
        }
    }

    #[test]
    fn test_greyscale_ladder_is_ordered() {
        let colors: HashMap<_, _> = ansi_colors(&scheme(true), &[]).into_iter().collect();
        let tone = |role| lch(&colors[&role]).l;
        assert!(tone(ColorRole::Black) < tone(ColorRole::BrightBlack));
        assert!(tone(ColorRole::BrightBlack) < tone(ColorRole::White));
        assert!(tone(ColorRole::White) < tone(ColorRole::BrightWhite));
    }

    /// A matched source keeps its hue but is pulled toward the vivid anchor, so
    /// the terminal palette stays recognizable instead of mirroring a washed-out
    /// UI palette.
    #[test]
    fn test_matched_slots_keep_source_hue_but_gain_saturation() {
        // Catppuccin Mocha chromatic colors.
        let mocha = [
            rgb(0xF3, 0x8B, 0xA8), // red
            rgb(0xA6, 0xE3, 0xA1), // green
            rgb(0xF9, 0xE2, 0xAF), // yellow
            rgb(0x89, 0xB4, 0xFA), // blue
            rgb(0xF5, 0xC2, 0xE7), // pink → magenta
            rgb(0x94, 0xE2, 0xD5), // teal → cyan
        ];
        let colors: HashMap<_, _> = ansi_colors(&scheme(true), &mocha).into_iter().collect();

        for (role, source) in [
            (ColorRole::Red, mocha[0]),
            (ColorRole::Green, mocha[1]),
            (ColorRole::Yellow, mocha[2]),
            (ColorRole::Blue, mocha[3]),
            (ColorRole::Magenta, mocha[4]),
            (ColorRole::Cyan, mocha[5]),
        ] {
            let out = lch(&colors[&role]);
            let src = lch(&source);
            let dh = (out.hue.into_inner() - src.hue.into_inner()).rem_euclid(360.0);
            assert!(
                dh.min(360.0 - dh) < 15.0,
                "{role:?} hue drifted {dh:.1}° from the source"
            );
            assert!(
                out.chroma >= src.chroma,
                "{role:?} chroma {:.1} should gain saturation over source {:.1}",
                out.chroma,
                src.chroma
            );
        }
    }

    /// A monochrome input must still produce a full, vivid 16-color set (the
    /// wallust anchors), never a washed-out grey-ish palette.
    #[test]
    fn test_monochrome_input_still_yields_vivid_anchors() {
        let colors: HashMap<_, _> = ansi_colors(&scheme(true), &[]).into_iter().collect();
        for role in [
            ColorRole::Red,
            ColorRole::Green,
            ColorRole::Yellow,
            ColorRole::Blue,
            ColorRole::Magenta,
            ColorRole::Cyan,
        ] {
            let c = lch(&colors[&role]);
            assert!(
                c.chroma > 45.0,
                "{role:?} chroma {:.1} is too washed out for a monochrome input",
                c.chroma
            );
        }
    }
}
