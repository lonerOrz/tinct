//! Image-based color extraction.
//!
//! Unified entry point for extracting source colors from wallpaper images.
//! Routes to the appropriate algorithm based on `SchemeType`.
//!
//! # M3 Schemes (Wu + WSMeans + Score)
//! - `tonal-spot`, `content`, `fruit-salad`, `rainbow`, `monochrome`
//!   → Use Wu quantizer → WSMeans refinement → Score algorithm
//!   → Returns the top-scored color as the source color
//!
//! # Non-M3 Schemes (K-means + custom scoring)
//! - `vibrant` → K-means with chroma scoring
//! - `faithful` → K-means with count scoring
//! - `dysfunctional` → K-means with dysfunctional scoring
//! - `muted` → K-means with muted scoring

use material_colors::color::Argb;
use material_colors::dynamic_color::Variant;
use std::collections::HashMap;
use std::path::Path;

use super::kmeans;
use super::quantizer;
use super::reader::{self, ResizeFilter};
use super::wsmeans;
use crate::core::color::{Color, Rgb};

/// Supported scheme types for color extraction.
#[derive(
    Debug, Clone, Copy, PartialEq, clap::ValueEnum, serde::Serialize, serde::Deserialize, Default,
)]
#[clap(rename_all = "kebab-case")]
pub enum SchemeType {
    // M3 schemes (Wu + Score)
    #[default]
    TonalSpot,
    Content,
    FruitSalad,
    Rainbow,
    Monochrome,
    // Non-M3 schemes (K-means)
    Vibrant,
    Faithful,
    Dysfunctional,
    Muted,
}

impl SchemeType {
    /// Check if this is an M3 scheme (uses Wu + Score pipeline).
    pub fn is_m3_scheme(&self) -> bool {
        matches!(
            self,
            Self::TonalSpot | Self::Content | Self::FruitSalad | Self::Rainbow | Self::Monochrome
        )
    }

    /// Map this scheme type to the official MD3 `Variant` used for palette
    /// generation.
    ///
    /// This is independent of the extraction pipeline (see [`Self::is_m3_scheme`]):
    /// it only chooses which Material You algorithm builds the palette. The
    /// non-M3 extraction modes (`vibrant`, `faithful`, `dysfunctional`, `muted`)
    /// have no exact variant, so they map onto the closest official one.
    pub fn to_variant(self) -> Variant {
        match self {
            Self::TonalSpot => Variant::TonalSpot,
            Self::Content => Variant::Content,
            Self::FruitSalad => Variant::FruitSalad,
            Self::Rainbow => Variant::Rainbow,
            Self::Monochrome => Variant::Monochrome,
            Self::Vibrant => Variant::Vibrant,
            // Fidelity keeps the source color; Neutral is the muted scheme;
            // Expressive is the closest fit for the clashing/dysfunctional mode.
            Self::Faithful => Variant::Fidelity,
            Self::Muted => Variant::Neutral,
            Self::Dysfunctional => Variant::Expressive,
        }
    }

    /// Get the appropriate resize filter for this scheme type.
    /// M3 schemes use Triangle (bilinear), others use Nearest.
    pub fn resize_filter(&self) -> ResizeFilter {
        if self.is_m3_scheme() {
            ResizeFilter::Triangle
        } else {
            ResizeFilter::Nearest
        }
    }

    /// Parse scheme type from string (case-insensitive, hyphen-tolerant).
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().replace('-', "").as_str() {
            "tonalspot" => Some(Self::TonalSpot),
            "content" => Some(Self::Content),
            "fruitsalad" => Some(Self::FruitSalad),
            "rainbow" => Some(Self::Rainbow),
            "monochrome" => Some(Self::Monochrome),
            "vibrant" => Some(Self::Vibrant),
            "faithful" => Some(Self::Faithful),
            "dysfunctional" => Some(Self::Dysfunctional),
            "muted" => Some(Self::Muted),
            _ => None,
        }
    }
}

impl std::fmt::Display for SchemeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TonalSpot => write!(f, "tonal-spot"),
            Self::Content => write!(f, "content"),
            Self::FruitSalad => write!(f, "fruit-salad"),
            Self::Rainbow => write!(f, "rainbow"),
            Self::Monochrome => write!(f, "monochrome"),
            Self::Vibrant => write!(f, "vibrant"),
            Self::Faithful => write!(f, "faithful"),
            Self::Dysfunctional => write!(f, "dysfunctional"),
            Self::Muted => write!(f, "muted"),
        }
    }
}

/// The result of extracting colors from an image: the seed `source` color plus
/// the representative color clusters found in the wallpaper.
///
/// The clusters are used by the terminal (ANSI) palette builder to snap each
/// chromatic slot to a hue actually present in the image.
#[derive(Debug, Clone)]
pub struct ExtractedPalette {
    /// The dominant/source color, used to seed MD3 palette generation.
    pub source: Argb,
    /// Representative image clusters (most prominent first, already filtered).
    pub colors: Vec<Color>,
}

/// Extract the source color from an image file.
///
/// # Arguments
/// * `path` — Path to the image file
/// * `scheme_type` — Determines the extraction algorithm
///
/// # Returns
/// The extracted source color as an ARGB value, ready for palette generation.
pub fn extract_source_color(path: &Path, scheme_type: SchemeType) -> Result<Argb, String> {
    Ok(extract_source_palette(path, scheme_type)?.source)
}

/// Extract the source color **and** the representative color clusters.
///
/// See [`extract_source_color`]; this richer variant additionally exposes the
/// image's color clusters for terminal-palette hue snapping.
pub fn extract_source_palette(
    path: &Path,
    scheme_type: SchemeType,
) -> Result<ExtractedPalette, String> {
    let filter = scheme_type.resize_filter();
    let pixels = reader::read_image(path, filter)?;

    if pixels.is_empty() {
        return Err("Image contains no opaque pixels".to_string());
    }

    let (argb, colors) = if scheme_type.is_m3_scheme() {
        extract_m3_source_color(&pixels)?
    } else {
        extract_kmeans_source_color(&pixels, scheme_type)?
    };

    Ok(ExtractedPalette {
        source: Argb::from_u32(argb),
        colors,
    })
}

/// Convert a color-count map into at most `cap` [`Color`] clusters, most
/// prominent first.
fn clusters_from_counts(counts: &HashMap<u32, i64>, cap: usize) -> Vec<Color> {
    let mut entries: Vec<(u32, i64)> = counts.iter().map(|(&c, &n)| (c, n)).collect();
    // Population descending, then ARGB ascending: a total order, so the result
    // never depends on randomized `HashMap` iteration.
    entries.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    entries
        .into_iter()
        .take(cap)
        .map(|(argb, _)| {
            Color::new(
                ((argb >> 16) & 0xFF) as u8,
                ((argb >> 8) & 0xFF) as u8,
                (argb & 0xFF) as u8,
                1.0,
            )
        })
        .collect()
}

/// Convert scored `(rgb, weight)` pairs into at most `cap` [`Color`] clusters.
fn clusters_from_scored(scored: &[(Rgb, f64)], cap: usize) -> Vec<Color> {
    scored
        .iter()
        .take(cap)
        .map(|((r, g, b), _)| Color::new(*r, *g, *b, 1.0))
        .collect()
}

/// Extract source color using Wu + WSMeans + Score pipeline (M3 schemes).
///
/// Matches the Python `extract_source_color` in theming/lib/quantizer.py exactly.
fn extract_m3_source_color(pixels: &[Rgb]) -> Result<(u32, Vec<Color>), String> {
    // Step 1: Wu quantization (128 colors)
    let wu_result = quantizer::quantize_wu(pixels, 128);

    if wu_result.is_empty() {
        return Err("Wu quantizer produced no colors".to_string());
    }

    // Step 2: WSMeans refinement in Lab space.
    //
    // WSMeans is deterministic *given* the order of the starting clusters, so
    // the order must not come from `HashMap` iteration (which is randomized per
    // process). Order by population (then ARGB) so the same image always yields
    // the same palette.
    let mut starting: Vec<(u32, i64)> = wu_result.iter().map(|(&c, &n)| (c, n)).collect();
    starting.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let starting_clusters: Vec<u32> = starting.into_iter().map(|(c, _)| c).collect();
    let wsmeans_result = wsmeans::quantize_wsmeans(pixels, 128, &starting_clusters);

    let color_to_count = if wsmeans_result.is_empty() {
        // Fall back to Wu result if WSMeans fails
        wu_result
    } else {
        wsmeans_result
    };

    // Step 3: Filter low-chroma colors (like Python)
    const MIN_CHROMA: f64 = 5.0;
    let mut filtered: HashMap<u32, i64> = HashMap::new();
    for (&argb, &count) in &color_to_count {
        let r = ((argb >> 16) & 0xFF) as u8;
        let g = ((argb >> 8) & 0xFF) as u8;
        let b = (argb & 0xFF) as u8;
        let chroma = crate::core::color::estimate_chroma(r, g, b);
        if chroma >= MIN_CHROMA {
            filtered.insert(argb, count);
        }
    }

    let filtered = if filtered.is_empty() {
        color_to_count
    } else {
        filtered
    };

    // Step 4: Score and pick the best color
    let clusters = clusters_from_counts(&filtered, 32);
    let scored = wsmeans::score_colors(&filtered, 4, true);
    let best = scored
        .first()
        .copied()
        .ok_or_else(|| "No colors scored".to_string())?;

    Ok((best, clusters))
}

/// Extract source color using K-means + custom scoring (non-M3 schemes).
fn extract_kmeans_source_color(
    pixels: &[Rgb],
    scheme_type: SchemeType,
) -> Result<(u32, Vec<Color>), String> {
    // Downsample for performance
    let sampled = kmeans::downsample_pixels(pixels, 4);

    // Determine cluster count and scoring method
    let (cluster_count, scoring) = match scheme_type {
        SchemeType::Vibrant => (20, ScoringMode::Chroma),
        SchemeType::Faithful => (48, ScoringMode::Count),
        SchemeType::Dysfunctional => (48, ScoringMode::Dysfunctional),
        SchemeType::Muted => (24, ScoringMode::Muted),
        _ => return Err("Non-M3 scheme type not supported for k-means extraction".to_string()),
    };

    // For vibrant mode, pre-filter to colorful pixels
    let mut filtered_pixels = sampled.to_vec();
    if matches!(scheme_type, SchemeType::Vibrant) {
        filtered_pixels.retain(|&(r, g, b)| crate::core::color::estimate_chroma(r, g, b) >= 5.0);
    }

    if filtered_pixels.is_empty() {
        let colors = kmeans::kmeans_cluster(&sampled, cluster_count, 10);
        if colors.is_empty() {
            return Err("K-means produced no clusters".to_string());
        }
        let scored = kmeans::score_colors_count(
            &colors
                .iter()
                .map(|(_, rep, count)| (*rep, *count))
                .collect::<Vec<_>>(),
        );
        if scored.is_empty() {
            return Err("No colors scored".to_string());
        }
        let clusters = clusters_from_scored(&scored, 32);
        let (r, g, b) = scored[0].0;
        return Ok((
            (0xFFu32 << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),
            clusters,
        ));
    }

    // K-means clustering
    let clusters = kmeans::kmeans_cluster(&filtered_pixels, cluster_count, 10);

    if clusters.is_empty() {
        return Err("K-means produced no clusters".to_string());
    }

    // Score based on mode
    let scored = match scoring {
        ScoringMode::Chroma => {
            // Use centroid colors (averaged, smoother)
            let colors: Vec<(Rgb, i64)> = clusters
                .iter()
                .map(|(centroid, _, count)| (*centroid, *count))
                .collect();
            kmeans::score_colors_chroma(&colors)
        }
        ScoringMode::Count => {
            // Use representative colors by area dominance
            let colors: Vec<(Rgb, i64)> = clusters
                .iter()
                .map(|(_, rep, count)| (*rep, *count))
                .collect();
            kmeans::score_colors_count(&colors)
        }
        ScoringMode::Dysfunctional => {
            let colors: Vec<(Rgb, i64)> = clusters
                .iter()
                .map(|(_, rep, count)| (*rep, *count))
                .collect();
            kmeans::score_colors_dysfunctional(&colors)
        }
        ScoringMode::Muted => {
            let colors: Vec<(Rgb, i64)> = clusters
                .iter()
                .map(|(_, rep, count)| (*rep, *count))
                .collect();
            kmeans::score_colors_muted(&colors)
        }
    };

    if scored.is_empty() {
        return Err("No colors scored after k-means".to_string());
    }

    let clusters = clusters_from_scored(&scored, 32);
    let (r, g, b) = scored[0].0;
    Ok((
        (0xFFu32 << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),
        clusters,
    ))
}

/// Scoring mode for K-means extraction.
#[derive(Debug, Clone, Copy)]
enum ScoringMode {
    Chroma,
    Count,
    Dysfunctional,
    Muted,
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::ImageEncoder;
    use tempfile::NamedTempFile;

    fn create_test_png() -> NamedTempFile {
        let mut file = tempfile::Builder::new()
            .prefix("test_")
            .suffix(".png")
            .tempfile()
            .unwrap();
        let encoder = image::codecs::png::PngEncoder::new(&mut file);
        let pixels: Vec<u8> = (0..112 * 112)
            .flat_map(|_| vec![103, 80, 164, 255]) // #6750A4 (Material purple)
            .collect();
        encoder
            .write_image(&pixels, 112, 112, image::ExtendedColorType::Rgba8)
            .unwrap();
        file
    }

    #[test]
    fn test_extract_source_color_tonal_spot() {
        let file = create_test_png();
        let argb = extract_source_color(file.path(), SchemeType::TonalSpot).unwrap();
        // Should extract something close to #6750A4
        let r = argb.red;
        let g = argb.green;
        let b = argb.blue;
        assert!(r > 80 && r < 130);
        assert!(g > 50 && g < 110);
        assert!(b > 130 && b < 190);
    }

    #[test]
    fn test_extract_source_color_vibrant() {
        let file = create_test_png();
        let argb = extract_source_color(file.path(), SchemeType::Vibrant).unwrap();
        let r = argb.red;
        let g = argb.green;
        let b = argb.blue;
        // Should extract something reasonable
        assert!(r > 0 || g > 0 || b > 0);
    }

    #[test]
    fn test_extract_source_palette_returns_clusters() {
        let file = create_test_png();
        let palette = extract_source_palette(file.path(), SchemeType::TonalSpot).unwrap();
        assert!(
            !palette.colors.is_empty(),
            "expected representative clusters"
        );
        // The source color must match the thin wrapper.
        let direct = extract_source_color(file.path(), SchemeType::TonalSpot).unwrap();
        assert_eq!(palette.source, direct);
    }

    #[test]
    fn test_scheme_type_parse() {
        assert_eq!(SchemeType::parse("tonal-spot"), Some(SchemeType::TonalSpot));
        assert_eq!(SchemeType::parse("TonalSpot"), Some(SchemeType::TonalSpot));
        assert_eq!(
            SchemeType::parse("fruit-salad"),
            Some(SchemeType::FruitSalad)
        );
        assert_eq!(SchemeType::parse("invalid"), None);
    }

    #[test]
    fn test_scheme_type_to_variant() {
        assert!(SchemeType::TonalSpot.to_variant() == Variant::TonalSpot);
        assert!(SchemeType::Content.to_variant() == Variant::Content);
        assert!(SchemeType::Monochrome.to_variant() == Variant::Monochrome);
        assert!(SchemeType::Faithful.to_variant() == Variant::Fidelity);
        assert!(SchemeType::Muted.to_variant() == Variant::Neutral);
    }
}
