//! K-means clustering in Lab space with 5 scoring modes.
//!
//! Ported from Python's `palette.py` — provides k-means clustering
//! and scoring functions for non-MD3 scheme types:
//! - `chroma`: vibrant, chroma-prioritized
//! - `count`: area-dominant, picks by pixel count
//! - `dysfunctional`: picks 2nd most dominant color family
//! - `muted`: like count but without chroma filtering
//! - `population`: matugen-like, representative colors

use std::collections::HashMap;

use super::wsmeans::{hue_distance, lab_distance_squared, lab_to_rgb, rgb_to_lab};

/// RGB pixel tuple.
pub type Rgb = (u8, u8, u8);

/// Downsample pixels for faster processing.
pub fn downsample_pixels(pixels: &[Rgb], factor: usize) -> Vec<Rgb> {
    if factor <= 1 {
        return pixels.to_vec();
    }
    let step = factor * factor;
    pixels.iter().step_by(step).copied().collect()
}

/// Perform K-means clustering on colors in Lab color space.
///
/// Returns list of `(centroid_rgb, representative_rgb, cluster_size)` tuples,
/// sorted by cluster size (largest first).
pub fn kmeans_cluster(pixels: &[Rgb], k: usize, iterations: usize) -> Vec<(Rgb, Rgb, i64)> {
    if pixels.is_empty() {
        return Vec::new();
    }

    let actual_k = k.min(pixels.len());
    if actual_k == 0 {
        return Vec::new();
    }

    // Pre-compute Lab values and deduplicate pixels for performance
    let mut pixel_counts: HashMap<Rgb, i64> = HashMap::new();
    for &p in pixels {
        *pixel_counts.entry(p).or_insert(0) += 1;
    }

    let unique_pixels: Vec<(Rgb, i64)> = pixel_counts.into_iter().collect();
    let colors_lab: Vec<(f64, f64, f64)> = unique_pixels
        .iter()
        .map(|&(rgb, _)| rgb_to_lab(rgb.0, rgb.1, rgb.2))
        .collect();
    let weights: Vec<i64> = unique_pixels.iter().map(|&(_, c)| c).collect();
    let n = colors_lab.len();

    // Deterministic initialization: pick evenly spaced colors from sorted list
    let mut sorted_indices: Vec<usize> = (0..n).collect();
    sorted_indices.sort_by(|&a, &b| colors_lab[a].0.partial_cmp(&colors_lab[b].0).unwrap());

    let step = n / actual_k;
    let mut centroids: Vec<(f64, f64, f64)> = Vec::with_capacity(actual_k);
    for i in 0..actual_k {
        centroids.push(colors_lab[sorted_indices[i * step]]);
    }

    let mut assignments = vec![0usize; n];
    let mut counts = vec![0i64; actual_k];

    for _ in 0..iterations {
        // Assign colors to nearest centroid
        for (idx, &color) in colors_lab.iter().enumerate() {
            let mut min_dist = f64::MAX;
            let mut min_cluster = 0usize;
            for (i, &centroid) in centroids.iter().enumerate() {
                let dist = lab_distance_squared(color, centroid);
                if dist < min_dist {
                    min_dist = dist;
                    min_cluster = i;
                }
            }
            assignments[idx] = min_cluster;
        }

        // Update centroids (weighted mean in Lab space)
        let mut new_centroids = vec![(0.0, 0.0, 0.0); actual_k];
        counts.fill(0);

        for (idx, &color) in colors_lab.iter().enumerate() {
            let c = assignments[idx];
            let w = weights[idx];
            new_centroids[c].0 += color.0 * w as f64;
            new_centroids[c].1 += color.1 * w as f64;
            new_centroids[c].2 += color.2 * w as f64;
            counts[c] += w;
        }

        for i in 0..actual_k {
            if counts[i] > 0 {
                new_centroids[i].0 /= counts[i] as f64;
                new_centroids[i].1 /= counts[i] as f64;
                new_centroids[i].2 /= counts[i] as f64;
            } else {
                new_centroids[i] = centroids[i];
            }
        }

        centroids = new_centroids;
    }

    // Build results
    let mut results = Vec::new();
    for i in 0..actual_k {
        if counts[i] > 0 {
            let centroid_rgb = lab_to_rgb(centroids[i].0, centroids[i].1, centroids[i].2);
            // Find representative (closest actual pixel to centroid)
            let mut best_rep = unique_pixels[0].0;
            let mut best_dist = f64::MAX;
            for (idx, &color) in colors_lab.iter().enumerate() {
                if assignments[idx] == i {
                    let dist = lab_distance_squared(color, centroids[i]);
                    if dist < best_dist {
                        best_dist = dist;
                        best_rep = unique_pixels[idx].0;
                    }
                }
            }
            results.push((centroid_rgb, best_rep, counts[i]));
        }
    }

    // Sort by cluster size (most common first)
    results.sort_by(|a, b| b.2.cmp(&a.2));
    results
}

// ============================================================================
// Scoring Functions
// ============================================================================

/// Map hue to perceptual color family.
fn hue_to_family(hue: f64) -> usize {
    if !(30.0..330.0).contains(&hue) {
        0 // RED
    } else if hue < 60.0 {
        1 // ORANGE
    } else if hue < 105.0 {
        2 // YELLOW
    } else if hue < 190.0 {
        3 // GREEN
    } else if hue < 270.0 {
        4 // BLUE
    } else {
        5 // PURPLE
    }
}

/// Get the center hue for a family index.
fn family_center_hue(family: usize) -> f64 {
    [0.0, 45.0, 82.5, 147.5, 230.0, 300.0][family]
}

/// Circular hue difference (0-180).
fn circular_hue_diff(h1: f64, h2: f64) -> f64 {
    let diff = (h1 - h2).abs();
    diff.min(360.0 - diff)
}

/// Estimate chroma from RGB.
pub fn estimate_chroma(r: u8, g: u8, b: u8) -> f64 {
    let (l, a, bv) = rgb_to_lab(r, g, b);
    let _ = l;
    (a * a + bv * bv).sqrt()
}

/// Estimate hue from RGB.
fn estimate_hue(r: u8, g: u8, b: u8) -> f64 {
    let (l, a, bv) = rgb_to_lab(r, g, b);
    let _ = l;
    let hue_rad = bv.atan2(a);
    let mut hue_deg = hue_rad.to_degrees();
    if hue_deg < 0.0 {
        hue_deg += 360.0;
    }
    hue_deg
}

/// Score colors prioritizing chroma (vibrancy) over area coverage.
///
/// Uses count^0.3 weighting so saturated colors win even with small area.
pub fn score_colors_chroma(colors_with_counts: &[(Rgb, i64)]) -> Vec<(Rgb, f64)> {
    let mut result: Vec<(Rgb, f64)> = Vec::new();

    for &(rgb, count) in colors_with_counts {
        let (r, g, b) = rgb;
        let chroma = estimate_chroma(r, g, b);
        let hue = estimate_hue(r, g, b);

        // Tone estimation from Lab L
        let (l, _, _) = rgb_to_lab(r, g, b);
        let tone = l;

        // Tone penalty
        let tone_penalty = if tone < 20.0 {
            (20.0 - tone) * 2.0
        } else if tone > 80.0 {
            (tone - 80.0) * 1.5
        } else if tone < 40.0 {
            (40.0 - tone) * 0.5
        } else if tone > 60.0 {
            (tone - 60.0) * 0.3
        } else {
            0.0
        };

        // Hue penalty — slight penalty for yellow-green hues
        let hue_penalty = if (80.0..110.0).contains(&hue) {
            5.0
        } else {
            0.0
        };

        let score = (chroma - tone_penalty - hue_penalty) * (count as f64).powf(0.3);
        result.push((rgb, score));
    }

    result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    result
}

/// Score colors prioritizing pixel count (area coverage) by hue family.
pub fn score_colors_count(colors_with_counts: &[(Rgb, i64)]) -> Vec<(Rgb, f64)> {
    const MIN_CHROMA: f64 = 10.0;

    // Group by hue family
    let mut hue_families: HashMap<usize, Vec<(Rgb, f64, f64, i64)>> = HashMap::new();

    for &(rgb, count) in colors_with_counts {
        let (r, g, b) = rgb;
        let chroma = estimate_chroma(r, g, b);
        if chroma >= MIN_CHROMA {
            let hue = estimate_hue(r, g, b);
            let family = hue_to_family(hue);
            hue_families
                .entry(family)
                .or_default()
                .push((rgb, hue, chroma, count));
        }
    }

    // If no colorful colors found, fall back to all colors by count
    if hue_families.is_empty() {
        let mut result: Vec<(Rgb, f64)> = colors_with_counts
            .iter()
            .map(|&(rgb, count)| (rgb, count as f64))
            .collect();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        return result;
    }

    // Calculate total count per hue family
    let mut family_totals: Vec<(usize, i64)> = hue_families
        .iter()
        .map(|(&family, colors)| {
            let total: i64 = colors.iter().map(|c| c.3).sum();
            (family, total)
        })
        .collect();
    family_totals.sort_by(|a, b| b.1.cmp(&a.1));

    // Build result: colors from dominant families first
    let mut result_colors: Vec<(Rgb, f64)> = Vec::new();
    for (family, _) in &family_totals {
        let mut family_colors = hue_families[family].clone();
        family_colors.sort_by(|a, b| {
            // Sort by count descending, chroma as tiebreaker
            b.3.cmp(&a.3).then(b.2.partial_cmp(&a.2).unwrap())
        });
        for (rgb, hue, chroma, count) in family_colors {
            let _ = hue;
            let family_rank = family_totals
                .iter()
                .position(|(f, _)| *f == *family)
                .unwrap();
            let score = (family_totals.len() - family_rank) as f64 * 1_000_000.0
                + count as f64 * 1000.0
                + chroma;
            result_colors.push((rgb, score));
        }
    }

    result_colors.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    result_colors
}

/// Score colors prioritizing the 2nd most dominant hue family.
pub fn score_colors_dysfunctional(colors_with_counts: &[(Rgb, i64)]) -> Vec<(Rgb, f64)> {
    const MIN_CHROMA: f64 = 10.0;
    const MIN_HUE_DISTANCE: f64 = 45.0;
    const MIN_COUNT_RATIO: f64 = 0.02;

    // Group by hue family
    let mut hue_families: HashMap<usize, Vec<(Rgb, f64, f64, i64)>> = HashMap::new();

    for &(rgb, count) in colors_with_counts {
        let (r, g, b) = rgb;
        let chroma = estimate_chroma(r, g, b);
        if chroma >= MIN_CHROMA {
            let hue = estimate_hue(r, g, b);
            let family = hue_to_family(hue);
            hue_families
                .entry(family)
                .or_default()
                .push((rgb, hue, chroma, count));
        }
    }

    if hue_families.is_empty() {
        let mut result: Vec<(Rgb, f64)> = colors_with_counts
            .iter()
            .map(|&(rgb, count)| (rgb, count as f64))
            .collect();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        return result;
    }

    let mut family_totals: Vec<(usize, i64)> = hue_families
        .iter()
        .map(|(&family, colors)| {
            let total: i64 = colors.iter().map(|c| c.3).sum();
            (family, total)
        })
        .collect();
    family_totals.sort_by(|a, b| b.1.cmp(&a.1));

    let dominant_family = family_totals[0].0;
    let _dominant_count = family_totals[0].1;
    let dominant_center = family_center_hue(dominant_family);
    let total_colorful_pixels: i64 = family_totals.iter().map(|(_, c)| c).sum();
    let min_count = (total_colorful_pixels as f64 * MIN_COUNT_RATIO) as i64;

    // Find distant families
    let mut distant_families: Vec<(usize, i64, f64, f64)> = Vec::new();
    let mut close_families = vec![dominant_family];

    for (family, count) in &family_totals[1..] {
        let family_center = family_center_hue(*family);
        let hue_diff = circular_hue_diff(dominant_center, family_center);
        if hue_diff >= MIN_HUE_DISTANCE && *count >= min_count {
            let max_chroma = hue_families[family]
                .iter()
                .map(|c| c.2)
                .fold(0.0f64, f64::max);
            distant_families.push((*family, *count, hue_diff, max_chroma));
        } else {
            close_families.push(*family);
        }
    }

    // Sort distant families by hue_distance * max_chroma
    distant_families.sort_by(|a, b| {
        let score_a = a.2 * a.3;
        let score_b = b.2 * b.3;
        score_b.partial_cmp(&score_a).unwrap()
    });

    let mut result_colors: Vec<(Rgb, f64)> = Vec::new();

    // Distant families first
    for (rank, (family, _, _, _)) in distant_families.iter().enumerate() {
        let mut family_colors = hue_families[family].clone();
        family_colors.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap().then(b.3.cmp(&a.3)));
        for (rgb, _hue, chroma, count) in family_colors {
            let score = (distant_families.len() - rank) as f64 * 1_000_000.0
                + chroma * 1000.0
                + count as f64;
            result_colors.push((rgb, score));
        }
    }

    // Close families at lower priority
    for family in &close_families {
        let mut family_colors = hue_families[family].clone();
        family_colors.sort_by(|a, b| b.3.cmp(&a.3).then(b.2.partial_cmp(&a.2).unwrap()));
        for (rgb, _hue, chroma, count) in family_colors {
            let score = count as f64 * 1000.0 + chroma;
            result_colors.push((rgb, score));
        }
    }

    result_colors.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    result_colors
}

/// Score colors for muted mode — pure pixel count without chroma filtering.
pub fn score_colors_muted(colors_with_counts: &[(Rgb, i64)]) -> Vec<(Rgb, f64)> {
    let mut result: Vec<(Rgb, f64)> = colors_with_counts
        .iter()
        .map(|&(rgb, count)| (rgb, count as f64))
        .collect();
    result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    result
}

/// Score colors using Material Design's Score algorithm.
#[allow(dead_code)]
pub fn score_colors_population(
    colors_with_counts: &[(Rgb, i64)],
    _total_pixels: usize,
) -> Vec<(Rgb, f64)> {
    const TARGET_CHROMA: f64 = 48.0;
    const WEIGHT_PROPORTION: f64 = 0.7;
    const WEIGHT_CHROMA_ABOVE: f64 = 0.3;
    const WEIGHT_CHROMA_BELOW: f64 = 0.1;
    const CUTOFF_CHROMA: f64 = 5.0;
    const CUTOFF_EXCITED_PROPORTION: f64 = 0.01;

    // Build per-hue population histogram
    let mut hue_population = vec![0i64; 360];
    let mut population_sum: i64 = 0;

    let mut colors_hct: Vec<(Rgb, f64, f64, i64)> = Vec::new(); // (rgb, hue, chroma, count)

    for &(rgb, count) in colors_with_counts {
        let (r, g, b) = rgb;
        let chroma = estimate_chroma(r, g, b);
        let hue = estimate_hue(r, g, b);
        let hue_bucket = (hue.round() as usize) % 360;
        hue_population[hue_bucket] += count;
        population_sum += count;
        colors_hct.push((rgb, hue, chroma, count));
    }

    if colors_hct.is_empty() || population_sum == 0 {
        let mut result: Vec<(Rgb, f64)> = colors_with_counts
            .iter()
            .map(|&(rgb, count)| (rgb, count as f64))
            .collect();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        return result;
    }

    // Calculate "excited proportions"
    let mut hue_excited_proportions = vec![0.0f64; 360];
    for (hue, &pop) in hue_population.iter().enumerate().take(360) {
        let proportion = pop as f64 / population_sum as f64;
        for offset in -14..=15 {
            let neighbor = ((hue as i32 + offset).rem_euclid(360)) as usize;
            hue_excited_proportions[neighbor] += proportion;
        }
    }

    // Score each color
    let mut scored: Vec<(Rgb, f64, f64, f64)> = Vec::new(); // (rgb, hue, chroma, score)
    for &(rgb, hue, chroma, count) in &colors_hct {
        let _ = count;
        let hue_bucket = (hue.round() as usize) % 360;
        let proportion = hue_excited_proportions[hue_bucket];

        if chroma < CUTOFF_CHROMA {
            continue;
        }
        if proportion <= CUTOFF_EXCITED_PROPORTION {
            continue;
        }

        let proportion_score = proportion * 100.0 * WEIGHT_PROPORTION;
        let chroma_score = if chroma < TARGET_CHROMA {
            (chroma - TARGET_CHROMA) * WEIGHT_CHROMA_BELOW
        } else {
            (chroma - TARGET_CHROMA) * WEIGHT_CHROMA_ABOVE
        };

        let score = proportion_score + chroma_score;
        scored.push((rgb, hue, chroma, score));
    }

    if scored.is_empty() {
        let mut result: Vec<(Rgb, f64)> = colors_with_counts
            .iter()
            .map(|&(rgb, count)| (rgb, count as f64))
            .collect();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        return result;
    }

    scored.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap());

    // Deduplicate by hue distance
    let mut chosen: Vec<(Rgb, f64)> = Vec::new();

    for min_hue_diff in (15..=90).rev() {
        chosen.clear();
        for &(rgb, hue, chroma, score) in &scored {
            let _ = chroma;
            let is_far_enough = chosen.iter().all(|(c_rgb, _)| {
                let ch = estimate_hue(c_rgb.0, c_rgb.1, c_rgb.2);
                hue_distance(hue, ch) >= min_hue_diff as f64
            });

            if is_far_enough {
                chosen.push((rgb, score));
            }

            if chosen.len() >= 4 {
                break;
            }
        }

        if chosen.len() >= 4 {
            break;
        }
    }

    if chosen.is_empty() {
        chosen = scored
            .iter()
            .take(4)
            .map(|&(rgb, _, _, score)| (rgb, score))
            .collect();
    }

    chosen
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kmeans_single_color() {
        let pixels: Vec<Rgb> = (0..100).map(|_| (255, 0, 0)).collect();
        let result = kmeans_cluster(&pixels, 3, 10);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].2, 100);
    }

    #[test]
    fn test_kmeans_two_colors() {
        let mut pixels: Vec<Rgb> = (0..50).map(|_| (255, 0, 0)).collect();
        pixels.extend((0..50).map(|_| (0, 0, 255)));
        let result = kmeans_cluster(&pixels, 5, 10);
        assert!(result.len() >= 2);
    }

    #[test]
    fn test_downsample() {
        let pixels: Vec<Rgb> = (0..100).map(|i| (i as u8, 0, 0)).collect();
        let sampled = downsample_pixels(&pixels, 4);
        assert!(sampled.len() < pixels.len());
    }

    #[test]
    fn test_score_chroma_prefers_vibrant() {
        let colors = vec![
            ((255, 0, 0), 10),     // Vibrant red
            ((128, 128, 128), 50), // Gray (high count, low chroma)
        ];
        let scored = score_colors_chroma(&colors);
        // Vibrant red should score higher despite lower count
        assert_eq!(scored[0].0, (255, 0, 0));
    }

    #[test]
    fn test_score_count_prefers_dominant() {
        let colors = vec![
            ((255, 0, 0), 10),
            ((0, 0, 255), 100), // More pixels
        ];
        let scored = score_colors_count(&colors);
        assert_eq!(scored[0].0, (0, 0, 255));
    }

    #[test]
    fn test_score_muted_accepts_gray() {
        let colors = vec![((128, 128, 128), 50), ((255, 0, 0), 10)];
        let scored = score_colors_muted(&colors);
        // Gray should be first (highest count)
        assert_eq!(scored[0].0, (128, 128, 128));
    }
}
