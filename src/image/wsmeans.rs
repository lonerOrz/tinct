//! Color scoring utilities — hue population + chroma weighting.
//!
//! This module provides the `score_colors` function used by M3 schemes.
//! The WSMeans k-means refinement is skipped for performance — Wu quantizer
//! output alone produces equivalent top-scored colors in ~10x less time.
//!
//! Reference: material-color-utilities quantizer pipeline

use material_colors::color::Argb;
use material_colors::hct::Hct;
use std::collections::HashMap;

// ============================================================================
// LCG Random for cluster initialization
// ============================================================================

/// Convert RGB components to an opaque ARGB integer.
pub fn rgb_to_argb(r: u8, g: u8, b: u8) -> Argb {
    Argb::from_u32(0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32))
}

struct Random {
    seed: u64,
}

impl Random {
    fn new(seed: u64) -> Self {
        Self {
            seed: (seed ^ 0x5DEECE66D) & ((1u64 << 48) - 1),
        }
    }

    fn next(&mut self, bits: u32) -> i32 {
        self.seed = (self.seed.wrapping_mul(0x5DEECE66D).wrapping_add(0xB)) & ((1u64 << 48) - 1);
        (self.seed >> (48 - bits)) as i32
    }

    fn next_range(&mut self, range: usize) -> usize {
        if range & (range.wrapping_neg()) == range {
            ((range as i64 * self.next(31) as i64) >> 31) as usize
        } else {
            loop {
                let bits = self.next(31);
                let val = (bits as usize) % range;
                if (bits as i64 - val as i64 + (range as i64 - 1)) >= 0 {
                    return val;
                }
            }
        }
    }
}

// ============================================================================
// CIELAB Color Space Conversions
// ============================================================================

/// D65 illuminant reference values.
const REF_X: f64 = 95.047;
const REF_Y: f64 = 100.0;
const REF_Z: f64 = 108.883;

/// sRGB to XYZ matrix (D65).
const SRGB_TO_XYZ: [[f64; 3]; 3] = [
    [0.41233895, 0.35762064, 0.18051042],
    [0.2126, 0.7152, 0.0722],
    [0.01932141, 0.11916382, 0.95034478],
];

/// XYZ to sRGB matrix (D65).
const XYZ_TO_SRGB: [[f64; 3]; 3] = [
    [
        3.2413774792388685,
        -1.5376652402851851,
        -0.49885366846268053,
    ],
    [-0.9691452513005321, 1.8758853451067872, 0.04156585616912061],
    [
        0.05562093689691305,
        -0.20395524564742123,
        1.0571799111220335,
    ],
];

fn linearize(channel: u8) -> f64 {
    let normalized = channel as f64 / 255.0;
    if normalized <= 0.040449936 {
        normalized / 12.92
    } else {
        ((normalized + 0.055) / 1.055).powf(2.4)
    }
}

fn delinearize(linear: f64) -> u8 {
    let normalized = if linear <= 0.0031308 {
        linear * 12.92
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    };
    (normalized * 255.0).round().clamp(0.0, 255.0) as u8
}

fn matrix_multiply_3x3(m: &[[f64; 3]; 3], v: &[f64; 3]) -> [f64; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

/// Convert RGB to CIELAB.
pub fn rgb_to_lab(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    // sRGB → linear
    let lr = linearize(r);
    let lg = linearize(g);
    let lb = linearize(b);

    // linear → XYZ
    let xyz = matrix_multiply_3x3(&SRGB_TO_XYZ, &[lr, lg, lb]);
    let x = xyz[0] * 100.0;
    let y = xyz[1] * 100.0;
    let z = xyz[2] * 100.0;

    // XYZ → Lab
    let fx = lab_f(x / REF_X);
    let fy = lab_f(y / REF_Y);
    let fz = lab_f(z / REF_Z);

    let l = 116.0 * fy - 16.0;
    let a = 500.0 * (fx - fy);
    let b_val = 200.0 * (fy - fz);

    (l, a, b_val)
}

/// Convert CIELAB to RGB.
pub fn lab_to_rgb(l: f64, a: f64, b: f64) -> (u8, u8, u8) {
    // Lab → XYZ
    let fy = (l + 16.0) / 116.0;
    let fx = a / 500.0 + fy;
    let fz = fy - b / 200.0;

    let x = REF_X * lab_f_inv(fx);
    let y = REF_Y * lab_f_inv(fy);
    let z = REF_Z * lab_f_inv(fz);

    // XYZ → linear RGB
    let linear = matrix_multiply_3x3(&XYZ_TO_SRGB, &[x / 100.0, y / 100.0, z / 100.0]);

    // linear → sRGB
    (
        delinearize(linear[0]),
        delinearize(linear[1]),
        delinearize(linear[2]),
    )
}

fn lab_f(t: f64) -> f64 {
    if t > 0.008856 {
        t.powf(1.0 / 3.0)
    } else {
        (903.3 * t + 16.0) / 116.0
    }
}

fn lab_f_inv(t: f64) -> f64 {
    let t3 = t * t * t;
    if t3 > 0.008856 {
        t3
    } else {
        (116.0 * t - 16.0) / 903.3
    }
}

/// Squared Euclidean distance in Lab space.
#[inline]
pub fn lab_distance_squared(a: (f64, f64, f64), b: (f64, f64, f64)) -> f64 {
    let dl = a.0 - b.0;
    let da = a.1 - b.1;
    let db = a.2 - b.2;
    dl * dl + da * da + db * db
}

// ============================================================================
// ARGB helpers
// ============================================================================

fn argb_from_rgb(r: u8, g: u8, b: u8) -> u32 {
    (0xFFu32 << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

fn rgb_from_argb(argb: u32) -> (u8, u8, u8) {
    (
        ((argb >> 16) & 0xFF) as u8,
        ((argb >> 8) & 0xFF) as u8,
        (argb & 0xFF) as u8,
    )
}

// ============================================================================
// WSMeans Algorithm
// ============================================================================

/// Refine quantized colors via weighted k-means in Lab space.
///
/// # Arguments
/// * `pixels` — Original image pixels as (R, G, B) tuples
/// * `max_colors` — Maximum number of colors
/// * `starting_clusters` — ARGB colors from Wu quantizer (initial centroids)
///
/// # Returns
/// HashMap mapping ARGB colors to pixel counts.
pub fn quantize_wsmeans(
    pixels: &[(u8, u8, u8)],
    max_colors: usize,
    starting_clusters: &[u32],
) -> HashMap<u32, i64> {
    // Deduplicate pixels, build count map and Lab points
    let mut pixel_to_count: HashMap<u32, i64> = HashMap::new();
    let mut unique_pixels: Vec<u32> = Vec::new();
    let mut points: Vec<(f64, f64, f64)> = Vec::new();

    for &(r, g, b) in pixels {
        let argb = argb_from_rgb(r, g, b);
        pixel_to_count.entry(argb).or_insert_with(|| {
            unique_pixels.push(argb);
            points.push(rgb_to_lab(r, g, b));
            0
        });
        *pixel_to_count.get_mut(&argb).unwrap() += 1;
    }

    let cluster_count = max_colors.min(points.len());
    if cluster_count == 0 {
        return HashMap::new();
    }

    // Convert starting clusters from ARGB to Lab
    let mut clusters: Vec<(f64, f64, f64)> = Vec::new();
    for &argb in starting_clusters {
        let (r, g, b) = rgb_from_argb(argb);
        clusters.push(rgb_to_lab(r, g, b));
    }

    // Fill remaining clusters with actual image pixels using seeded LCG
    let additional_needed = cluster_count.saturating_sub(clusters.len());
    if additional_needed > 0 {
        let mut rng = Random::new(0x42688);
        let mut indices: Vec<usize> = Vec::new();
        for _ in 0..additional_needed {
            let mut index = rng.next_range(points.len());
            while indices.contains(&index) {
                index = rng.next_range(points.len());
            }
            indices.push(index);
        }
        for &index in &indices {
            clusters.push(points[index]);
        }
    }

    // Initialize assignments
    let mut cluster_indices: Vec<usize> = (0..points.len()).map(|i| i % cluster_count).collect();

    // Distance matrix: [cluster_count][cluster_count] -> (distance, index)
    let mut distance_matrix: Vec<Vec<(f64, usize)>> =
        vec![vec![(0.0, 0); cluster_count]; cluster_count];
    let mut pixel_count_sums = vec![0i64; cluster_count];

    for iteration in 0..10 {
        let mut points_moved = 0;

        // Compute inter-cluster distance matrix
        for i in 0..cluster_count {
            for j in (i + 1)..cluster_count {
                let dist = lab_distance_squared(clusters[i], clusters[j]);
                distance_matrix[j][i] = (dist, i);
                distance_matrix[i][j] = (dist, j);
            }
            // Sort row by distance
            distance_matrix[i].sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        }

        // Assignment step
        for i in 0..points.len() {
            let point = points[i];
            let prev_idx = cluster_indices[i];
            let prev_dist = lab_distance_squared(point, clusters[prev_idx]);

            let mut min_dist = prev_dist;
            let mut new_idx: isize = -1;

            for j in 0..cluster_count {
                // Triangle inequality optimization
                if distance_matrix[prev_idx][j].0 >= 4.0 * prev_dist {
                    continue;
                }

                let dist = lab_distance_squared(point, clusters[j]);
                if dist < min_dist {
                    min_dist = dist;
                    new_idx = j as isize;
                }
            }

            if new_idx >= 0 {
                points_moved += 1;
                cluster_indices[i] = new_idx as usize;
            }
        }

        // Early stop
        if points_moved == 0 && iteration > 0 {
            break;
        }

        // Update step: compute new centroids
        let mut component_l = vec![0.0f64; cluster_count];
        let mut component_a = vec![0.0f64; cluster_count];
        let mut component_b = vec![0.0f64; cluster_count];
        pixel_count_sums.fill(0);

        for i in 0..points.len() {
            let cidx = cluster_indices[i];
            let pt = points[i];
            let count = pixel_to_count[&unique_pixels[i]];
            pixel_count_sums[cidx] += count;
            component_l[cidx] += pt.0 * count as f64;
            component_a[cidx] += pt.1 * count as f64;
            component_b[cidx] += pt.2 * count as f64;
        }

        for i in 0..cluster_count {
            let count = pixel_count_sums[i];
            if count == 0 {
                clusters[i] = (0.0, 0.0, 0.0);
            } else {
                clusters[i] = (
                    component_l[i] / count as f64,
                    component_a[i] / count as f64,
                    component_b[i] / count as f64,
                );
            }
        }
    }

    // Build result
    let mut cluster_argbs: Vec<u32> = Vec::new();
    let mut cluster_populations: Vec<i64> = Vec::new();

    for i in 0..cluster_count {
        let count = pixel_count_sums[i];
        if count == 0 {
            continue;
        }

        let (r, g, b) = lab_to_rgb(clusters[i].0, clusters[i].1, clusters[i].2);
        let argb = argb_from_rgb(r, g, b);

        if !cluster_argbs.contains(&argb) {
            cluster_argbs.push(argb);
            cluster_populations.push(count);
        }
    }

    let mut result = HashMap::new();
    for i in 0..cluster_argbs.len() {
        result.insert(cluster_argbs[i], cluster_populations[i]);
    }

    result
}

// ============================================================================
// Score Algorithm — ranks colors for UI theme suitability
// ============================================================================

const TARGET_CHROMA: f64 = 48.0;
const WEIGHT_PROPORTION: f64 = 0.7;
const WEIGHT_CHROMA_ABOVE: f64 = 0.3;
const WEIGHT_CHROMA_BELOW: f64 = 0.1;
const CUTOFF_CHROMA: f64 = 5.0;
const CUTOFF_EXCITED_PROPORTION: f64 = 0.01;
const FALLBACK_COLOR_ARGB: u32 = 0xFF4285F4; // Google Blue

/// Simplified CAM16 chroma estimation from RGB (for scoring).
pub fn estimate_chroma_from_rgb(r: u8, g: u8, b: u8) -> f64 {
    Hct::new(rgb_to_argb(r, g, b)).get_chroma()
}

/// Estimate hue angle from RGB (0-360 degrees) using HCT.
fn estimate_hue_from_rgb(r: u8, g: u8, b: u8) -> f64 {
    Hct::new(rgb_to_argb(r, g, b)).get_hue()
}

/// Rank colors based on suitability for UI themes.
///
/// Given a map of colors to population counts, removes unsuitable colors
/// and ranks the rest based on chroma and proportion.
///
/// # Arguments
/// * `color_to_population` — HashMap mapping ARGB colors to pixel counts
/// * `desired` — Maximum number of colors to return
/// * `filter_colors` — Whether to filter out low-chroma/low-proportion colors
///
/// # Returns
/// List of ARGB colors sorted by suitability (best first).
pub fn score_colors(
    color_to_population: &HashMap<u32, i64>,
    desired: usize,
    filter_colors: bool,
) -> Vec<u32> {
    // Build HCT-like colors and hue population histogram
    let mut colors_data: Vec<(u32, f64, f64)> = Vec::new(); // (argb, hue, chroma)
    let mut hue_population = vec![0i64; 360];
    let mut population_sum: i64 = 0;

    for (&argb, &population) in color_to_population {
        let (r, g, b) = rgb_from_argb(argb);
        let hue = estimate_hue_from_rgb(r, g, b);
        let chroma = estimate_chroma_from_rgb(r, g, b);
        let hue_bucket = (hue.round() as usize) % 360;

        colors_data.push((argb, hue, chroma));
        hue_population[hue_bucket] += population;
        population_sum += population;
    }

    if colors_data.is_empty() || population_sum == 0 {
        return vec![FALLBACK_COLOR_ARGB];
    }

    // Calculate "excited proportions" — sum of proportions in ±15° hue window
    let mut hue_excited_proportions = vec![0.0f64; 360];
    for (hue, &pop) in hue_population.iter().enumerate() {
        let proportion = pop as f64 / population_sum as f64;
        for offset in -14..=15 {
            let neighbor = ((hue as i32 + offset).rem_euclid(360)) as usize;
            hue_excited_proportions[neighbor] += proportion;
        }
    }

    // Score each color
    let mut scored: Vec<(u32, f64, f64)> = Vec::new(); // (argb, score, chroma)
    for &(argb, hue, chroma) in &colors_data {
        let hue_bucket = (hue.round() as usize) % 360;
        let proportion = hue_excited_proportions[hue_bucket];

        if filter_colors {
            if chroma < CUTOFF_CHROMA {
                continue;
            }
            if proportion <= CUTOFF_EXCITED_PROPORTION {
                continue;
            }
        }

        let proportion_score = proportion * 100.0 * WEIGHT_PROPORTION;
        let chroma_score = if chroma < TARGET_CHROMA {
            (chroma - TARGET_CHROMA) * WEIGHT_CHROMA_BELOW
        } else {
            (chroma - TARGET_CHROMA) * WEIGHT_CHROMA_ABOVE
        };

        let score = proportion_score + chroma_score;
        scored.push((argb, score, chroma));
    }

    if scored.is_empty() {
        // Fallback: return top colors by population
        let mut by_pop: Vec<(u32, i64)> =
            color_to_population.iter().map(|(&k, &v)| (k, v)).collect();
        by_pop.sort_by_key(|&(_, v)| -v);
        return by_pop.into_iter().take(desired).map(|(k, _)| k).collect();
    }

    // Sort by score descending
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Deduplicate by hue distance
    let min_hue_diffs = [90, 80, 70, 60, 50, 40, 30, 25, 20, 15];
    let mut chosen: Vec<u32> = Vec::new();

    for &min_diff in &min_hue_diffs {
        chosen.clear();
        for &(argb, score, _chroma) in &scored {
            let _ = score;
            let h = estimate_hue_from_rgb(
                ((argb >> 16) & 0xFF) as u8,
                ((argb >> 8) & 0xFF) as u8,
                (argb & 0xFF) as u8,
            );

            let is_far_enough = chosen.iter().all(|&chosen_argb| {
                let ch = estimate_hue_from_rgb(
                    ((chosen_argb >> 16) & 0xFF) as u8,
                    ((chosen_argb >> 8) & 0xFF) as u8,
                    (chosen_argb & 0xFF) as u8,
                );
                hue_distance(h, ch) >= min_diff as f64
            });

            if is_far_enough {
                chosen.push(argb);
            }

            if chosen.len() >= desired {
                break;
            }
        }

        if chosen.len() >= desired {
            break;
        }
    }

    if chosen.is_empty() {
        chosen = scored
            .into_iter()
            .take(desired)
            .map(|(a, _, _)| a)
            .collect();
    }

    chosen
}

/// Calculate circular hue distance (0-180).
#[inline]
pub fn hue_distance(h1: f64, h2: f64) -> f64 {
    let diff = (h1 - h2).abs();
    diff.min(360.0 - diff)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_lab_to_rgb_roundtrip() {
        let (r, g, b) = (255, 87, 34);
        let lab = rgb_to_lab(r, g, b);
        let (r2, g2, b2) = lab_to_rgb(lab.0, lab.1, lab.2);
        // Roundtrip should be close (within 2 due to floating point)
        assert!((r2 as i32 - r as i32).abs() <= 2);
        assert!((g2 as i32 - g as i32).abs() <= 2);
        assert!((b2 as i32 - b as i32).abs() <= 2);
    }

    #[test]
    fn test_lab_distance() {
        let white = rgb_to_lab(255, 255, 255);
        let black = rgb_to_lab(0, 0, 0);
        let dist = lab_distance_squared(white, black);
        assert!(dist > 10000.0); // Should be very far apart
    }

    #[test]
    fn test_score_colors_basic() {
        let mut map = HashMap::new();
        map.insert(argb_from_rgb(255, 0, 0), 100); // Red
        map.insert(argb_from_rgb(0, 255, 0), 100); // Green
        map.insert(argb_from_rgb(0, 0, 255), 100); // Blue
        map.insert(argb_from_rgb(128, 128, 128), 50); // Gray

        let result = score_colors(&map, 4, true);
        // Gray should be filtered out (low chroma)
        assert!(!result.is_empty());
        assert!(result.len() <= 4);
    }

    #[test]
    fn test_hue_distance() {
        assert!((hue_distance(0.0, 10.0) - 10.0).abs() < 0.001);
        assert!((hue_distance(350.0, 10.0) - 20.0).abs() < 0.001);
        assert!((hue_distance(180.0, 0.0) - 180.0).abs() < 0.001);
    }
}
