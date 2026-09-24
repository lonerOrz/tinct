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
use crate::config::{AnsiConfig, AnsiPalette};
use crate::core::color::Color;

/// Upper bound on CIE LCh chroma; roughly the edge of the sRGB gamut.
const MAX_CHROMA: f32 = 132.0;

/// Light-mode colors compress their wallust anchor by this factor into a band
/// around L\*40, so no slot (blue, red) collapses to near-black on a light
/// background while the per-slot character is preserved.
const LIGHT_L_SCALE: f32 = 0.55;

/// Lightness shift paired with [`LIGHT_L_SCALE`].
const LIGHT_L_SHIFT: f32 = 2.0;

/// Resolved terminal-palette knobs.
///
/// Built from [`AnsiConfig`] via [`AnsiParams::from_config`]: values are
/// clamped to sane ranges and hex strings are parsed once, so the generation
/// hot path never touches strings.
#[derive(Debug, Clone)]
pub struct AnsiParams {
    palette: AnsiPalette,
    source_weight: f32,
    chroma_threshold: f32,
    brightness_delta: f32,
    bright_chroma_multiplier: f32,
    contrast_target: f32,
    background: Option<Color>,
    foreground: Option<Color>,
    /// Per-slot anchor overrides, in [`SLOTS`] order.
    anchors: [Option<Lch>; 6],
}

impl Default for AnsiParams {
    fn default() -> Self {
        Self::from_config(&AnsiConfig::default())
    }
}

impl AnsiParams {
    /// Resolve user configuration into concrete, clamped values.
    pub fn from_config(config: &AnsiConfig) -> Self {
        Self {
            palette: config.palette,
            source_weight: config.source_weight.clamp(0.0, 1.0),
            chroma_threshold: config.chroma_threshold.max(0.0),
            brightness_delta: config.brightness_delta.max(0.0),
            bright_chroma_multiplier: config.bright_chroma_multiplier.max(0.0),
            contrast_target: config.contrast_target.clamp(0.0, 21.0),
            background: config.background.as_deref().and_then(parse_color_hex),
            foreground: config.foreground.as_deref().and_then(parse_color_hex),
            anchors: [
                parse_anchor(&config.anchors.red),
                parse_anchor(&config.anchors.green),
                parse_anchor(&config.anchors.yellow),
                parse_anchor(&config.anchors.blue),
                parse_anchor(&config.anchors.magenta),
                parse_anchor(&config.anchors.cyan),
            ],
        }
    }
}

/// Parse a hex color, warning and falling back on invalid input.
fn parse_color_hex(hex: &str) -> Option<Color> {
    match Color::from_hex(hex) {
        Ok(color) => Some(color),
        Err(_) => {
            ::log::warn!("[ansi] ignoring invalid color '{}'", hex);
            None
        }
    }
}

/// Parse an anchor hex into its CIE LCh representation.
fn parse_anchor(hex: &Option<String>) -> Option<Lch> {
    Some(lch_of(color_to_argb(&parse_color_hex(hex.as_deref()?)?)))
}

/// One chromatic ANSI slot.
#[derive(Clone, Copy)]
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
pub fn ansi_colors(
    scheme: &DynamicScheme,
    source_colors: &[Color],
    params: &AnsiParams,
) -> Vec<(ColorRole, Color)> {
    let is_dark = scheme.is_dark;

    // Wallpaper clusters, in CIE LCh, grey ones dropped.
    let mut remaining: Vec<Lch> = source_colors
        .iter()
        .map(|c| lch_of(color_to_argb(c)))
        .filter(|c| c.chroma > params.chroma_threshold)
        .collect();

    // Contrast is measured against a pinned background when one is configured,
    // otherwise against the scheme's own surface.
    let background = match &params.background {
        Some(bg) => srgb_of(lch_of(color_to_argb(bg))),
        None => srgb_of(lch_of(scheme.surface())),
    };

    let mut colors = Vec::with_capacity(16);

    // Greyscale ladder from the scheme's neutral palette: fixed tones, so it
    // always forms a canonical dark→light ramp that matches the mode. A pinned
    // background/foreground replaces ANSI 0 / ANSI 7.
    let neutral = &scheme.neutral_palette;
    colors.push((
        ColorRole::Black,
        params
            .background
            .unwrap_or_else(|| from_palette(neutral, 10.0)),
    ));
    colors.push((ColorRole::BrightBlack, from_palette(neutral, 30.0)));
    colors.push((
        ColorRole::White,
        params
            .foreground
            .unwrap_or_else(|| from_palette(neutral, 90.0)),
    ));
    colors.push((ColorRole::BrightWhite, from_palette(neutral, 100.0)));

    for (index, slot) in SLOTS.iter().enumerate() {
        let slot = apply_anchor(slot, &params.anchors[index]);
        let (hue, chroma, light) = slot_texture(&slot, &mut remaining, params.source_weight);

        let normal = render(hue, chroma, light, is_dark, false, background, params);
        let bright = render(hue, chroma, light, is_dark, true, background, params);

        colors.push((slot.normal, normal));
        colors.push((slot.bright, bright));
    }

    colors
}

/// Overlay a user-provided anchor color onto a slot's built-in anchor.
fn apply_anchor(slot: &Slot, anchor: &Option<Lch>) -> Slot {
    match anchor {
        Some(a) => Slot {
            anchor_hue: a.hue.into_inner(),
            light: a.l,
            chroma: a.chroma,
            ..*slot
        },
        None => *slot,
    }
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
fn slot_texture(slot: &Slot, remaining: &mut Vec<Lch>, source_weight: f32) -> (f32, f32, f32) {
    // Single pass: matched colors have the same count for hue/chroma/light, so
    // one counter serves all three, and the partial sums accumulate in the same
    // order (and f32 type) as the previous per-series `mean` calls.
    let mut sum_hue = 0.0f32;
    let mut sum_chroma = 0.0f32;
    let mut sum_light = 0.0f32;
    let mut count = 0usize;

    remaining.retain(|c| {
        let hue = c.hue.into_inner();
        if hue >= slot.hue_start && hue < slot.hue_end {
            sum_hue += hue;
            sum_chroma += c.chroma;
            sum_light += c.l;
            count += 1;
            false
        } else {
            true
        }
    });

    if count == 0 {
        (slot.anchor_hue, slot.chroma, slot.light)
    } else {
        let n = count as f32;
        let hue = (sum_hue / n).rem_euclid(360.0);
        let anchor_weight = 1.0 - source_weight;
        let chroma = slot.chroma * anchor_weight + (sum_chroma / n) * source_weight;
        let light = slot.light * anchor_weight + (sum_light / n) * source_weight;
        (hue, chroma.clamp(0.0, MAX_CHROMA), light)
    }
}

/// Turn a slot's hue/chroma/lightness into a final color.
///
/// Lightness is normalized per mode so the whole set stays legible, and the
/// `bright_*` variant is both more extreme in lightness and more saturated
/// (lighter/darker per [`AnsiPalette`]). Every color is gamut mapped
/// (preserving hue) and pushed away from the background until it clears the
/// configured WCAG contrast floor.
fn render(
    hue: f32,
    chroma: f32,
    light: f32,
    is_dark: bool,
    bright: bool,
    background: Srgb<f32>,
    params: &AnsiParams,
) -> Color {
    let base = if is_dark {
        light.clamp(18.0, 88.0)
    } else {
        // Light mode compresses the per-slot character into a band around L*40
        // so no slot collapses to near-black on a light background.
        (light * LIGHT_L_SCALE + LIGHT_L_SHIFT).clamp(30.0, 60.0)
    };

    let (lo, hi) = if is_dark { (18.0, 88.0) } else { (30.0, 64.0) };
    let delta = match params.palette {
        AnsiPalette::Dark => params.brightness_delta,
        AnsiPalette::Light => -params.brightness_delta,
    };
    let lightness = if bright {
        (base + delta).clamp(lo, hi)
    } else {
        base
    };

    let chroma = if bright {
        (chroma * params.bright_chroma_multiplier).min(MAX_CHROMA)
    } else {
        chroma.min(MAX_CHROMA)
    };

    // Fit to sRGB *before* contrast so clipping never distorts the hue.
    let lch = gamut_map(Lch::new(lightness, chroma, LabHue::from_degrees(hue)));
    let lch = ensure_contrast(lch, background, is_dark, params.contrast_target);
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

/// Push a color away from the background until it clears `target`.
fn ensure_contrast(mut lch: Lch, background: Srgb<f32>, is_dark: bool, target: f32) -> Lch {
    if target <= 0.0 {
        return lch;
    }
    let step = if is_dark { 6.0 } else { -6.0 };
    let mut i = 0;
    while srgb_of(lch).relative_contrast(background) < target && i < 12 {
        lch.l = (lch.l + step).clamp(0.0, 100.0);
        i += 1;
    }
    lch
}

// ---- conversion helpers ----

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

    /// The six chromatic slots and their `bright_*` counterparts, in order.
    const CHROMATIC_PAIRS: [(ColorRole, ColorRole); 6] = [
        (ColorRole::Red, ColorRole::BrightRed),
        (ColorRole::Green, ColorRole::BrightGreen),
        (ColorRole::Yellow, ColorRole::BrightYellow),
        (ColorRole::Blue, ColorRole::BrightBlue),
        (ColorRole::Magenta, ColorRole::BrightMagenta),
        (ColorRole::Cyan, ColorRole::BrightCyan),
    ];

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

    /// Test-local wrapper: every test exercises the default knobs unless it
    /// explicitly builds its own [`AnsiParams`].
    fn ansi_colors(scheme: &DynamicScheme, source_colors: &[Color]) -> Vec<(ColorRole, Color)> {
        super::ansi_colors(scheme, source_colors, &AnsiParams::default())
    }

    #[test]
    fn test_ansi_has_all_sixteen_distinct_roles() {
        let colors = ansi_colors(&scheme(true), &[]);
        assert_eq!(colors.len(), 16);
        let roles: HashSet<_> = colors.iter().map(|(role, _)| *role).collect();
        assert_eq!(roles.len(), 16);
    }

    #[test]
    fn test_chromatic_anchors_when_no_clusters() {
        // With no image clusters every chromatic slot falls back to its vivid
        // anchor (in both modes), and the fallback must stay vivid -- never
        // washed out into a grey-ish terminal palette.
        let anchors = [
            (ColorRole::Red, 40.0),
            (ColorRole::Green, 151.0),
            (ColorRole::Yellow, 104.0),
            (ColorRole::Blue, 288.0),
            (ColorRole::Magenta, 350.0),
            (ColorRole::Cyan, 204.0),
        ];
        for is_dark in [true, false] {
            let colors: HashMap<_, _> = ansi_colors(&scheme(is_dark), &[]).into_iter().collect();
            for (role, expected) in anchors {
                let c = lch(&colors[&role]);
                let hue = c.hue.into_inner();
                let diff = (hue - expected).rem_euclid(360.0);
                assert!(
                    diff.min(360.0 - diff) < 15.0,
                    "{role:?} hue {hue:.1} drifted from anchor {expected:.1} (dark={is_dark})"
                );
                if is_dark {
                    assert!(
                        c.chroma > 45.0,
                        "{role:?} chroma {:.1} is too washed out for a monochrome input",
                        c.chroma
                    );
                }
            }
        }
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
    /// with the default (dark) palette always lighter and always a different
    /// color, and with `palette = light` inverted (darker). Chroma is boosted
    /// as much as sRGB allows, but a lighter color physically holds less chroma
    /// for some hues, so vividness is not asserted per-slot.
    #[test]
    fn test_bright_variants_direction_and_distinctness() {
        for is_dark in [true, false] {
            let colors: HashMap<_, _> = ansi_colors(&scheme(is_dark), &[]).into_iter().collect();
            for (normal, bright) in CHROMATIC_PAIRS {
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

        // palette = light inverts the bright direction.
        let params = AnsiParams {
            palette: AnsiPalette::Light,
            ..Default::default()
        };
        let colors: HashMap<_, _> = super::ansi_colors(&scheme(true), &[], &params)
            .into_iter()
            .collect();
        for (normal, bright) in CHROMATIC_PAIRS {
            assert!(
                lch(&colors[&bright]).l < lch(&colors[&normal]).l,
                "{bright:?} must be darker than {normal:?} with palette=light"
            );
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
                    ratio >= AnsiParams::default().contrast_target - 0.05,
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

    fn cyan_chroma(params: &AnsiParams, source: &[Color]) -> f32 {
        let colors: HashMap<_, _> = super::ansi_colors(&scheme(true), source, params)
            .into_iter()
            .collect();
        lch(&colors[&ColorRole::Cyan]).chroma
    }

    #[test]
    fn test_source_weight_controls_blend() {
        let teal = rgb(0, 150, 136);
        let anchor_only = AnsiParams {
            source_weight: 0.0,
            ..Default::default()
        };
        let source_only = AnsiParams {
            source_weight: 1.0,
            ..Default::default()
        };
        assert!(
            cyan_chroma(&anchor_only, &[teal]) > cyan_chroma(&source_only, &[teal]),
            "a higher source weight must pull chroma toward the source"
        );
    }

    #[test]
    fn test_chroma_threshold_drops_candidates() {
        let teal = rgb(0, 150, 136);
        let params = AnsiParams {
            chroma_threshold: 1000.0,
            ..Default::default()
        };
        let colors: HashMap<_, _> = super::ansi_colors(&scheme(true), &[teal], &params)
            .into_iter()
            .collect();
        let hue = lch(&colors[&ColorRole::Cyan]).hue.into_inner();
        assert!(
            (hue - 204.0).abs() < 1.0,
            "with every candidate dropped the cyan slot must fall back to its anchor"
        );
    }

    #[test]
    fn test_background_and_foreground_pins() {
        let background = rgb(0x12, 0x12, 0x12);
        let foreground = rgb(0xEE, 0xEE, 0xEE);
        let params = AnsiParams {
            background: Some(background),
            foreground: Some(foreground),
            ..Default::default()
        };
        let colors: HashMap<_, _> = super::ansi_colors(&scheme(true), &[], &params)
            .into_iter()
            .collect();
        assert_eq!(colors[&ColorRole::Black].hex(), background.hex());
        assert_eq!(colors[&ColorRole::White].hex(), foreground.hex());
    }

    #[test]
    fn test_anchor_override_changes_fallback() {
        let custom = rgb(0xE0, 0x6C, 0x75);
        let params = AnsiParams {
            anchors: [
                Some(lch_of(color_to_argb(&custom))),
                None,
                None,
                None,
                None,
                None,
            ],
            ..Default::default()
        };
        let colors: HashMap<_, _> = super::ansi_colors(&scheme(true), &[], &params)
            .into_iter()
            .collect();
        let hue = lch(&colors[&ColorRole::Red]).hue.into_inner();
        let expected = lch(&custom).hue.into_inner();
        let diff = (hue - expected).rem_euclid(360.0);
        assert!(
            diff.min(360.0 - diff) < 3.0,
            "red slot must adopt the custom anchor hue ({hue:.1} vs {expected:.1})"
        );
    }

    #[test]
    fn test_from_config_resolves_and_clamps() {
        let config = AnsiConfig {
            source_weight: 5.0,
            contrast_target: 100.0,
            background: Some("#123456".to_string()),
            foreground: Some("not-a-color".to_string()),
            ..AnsiConfig::default()
        };
        let params = AnsiParams::from_config(&config);
        assert_eq!(params.source_weight, 1.0);
        assert_eq!(params.contrast_target, 21.0);
        assert!(params.background.is_some());
        assert!(params.foreground.is_none(), "invalid hex must be ignored");
    }
}
