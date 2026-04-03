//! Wu color quantizer — Xiaolin Wu's algorithm from Graphics Gems II (1991).
//!
//! Divides image pixels into clusters by recursively cutting an RGB cube
//! based on the weight of pixels in each area. Matches the Python
//! `QuantizerWu` implementation exactly.

use std::collections::HashMap;

/// Number of bits for index (5 bits = 32 levels per channel).
const INDEX_BITS: usize = 5;
/// Side length of the 3D histogram array.
const SIDE_LENGTH: usize = 33; // (1 << INDEX_BITS) + 1
/// Total size of the 3D array.
const TOTAL_SIZE: usize = 35_937; // SIDE_LENGTH^3

/// Direction constants for cutting axis.
const DIR_RED: usize = 0;
const DIR_GREEN: usize = 1;
const DIR_BLUE: usize = 2;

/// Calculate 3D array index from quantized RGB coordinates.
#[inline]
fn get_index(r: usize, g: usize, b: usize) -> usize {
    (r << (INDEX_BITS * 2)) + (r << (INDEX_BITS + 1)) + r + (g << INDEX_BITS) + g + b
}

/// Convert RGB to ARGB integer.
#[inline]
fn argb_from_rgb(r: u8, g: u8, b: u8) -> u32 {
    (0xFFu32 << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

/// Extract RGB from ARGB integer.
#[allow(dead_code)]
#[inline]
fn rgb_from_argb(argb: u32) -> (u8, u8, u8) {
    (
        ((argb >> 16) & 0xFF) as u8,
        ((argb >> 8) & 0xFF) as u8,
        (argb & 0xFF) as u8,
    )
}

/// A box in RGB color space for Wu quantization.
#[derive(Debug, Clone, Copy)]
struct Box {
    r0: usize,
    r1: usize,
    g0: usize,
    g1: usize,
    b0: usize,
    b1: usize,
    vol: usize,
}

impl Box {
    fn new() -> Self {
        Self {
            r0: 0,
            r1: 0,
            g0: 0,
            g1: 0,
            b0: 0,
            b1: 0,
            vol: 0,
        }
    }
}

/// Wu color quantizer.
pub struct QuantizerWu {
    weights: Vec<i64>,
    moments_r: Vec<i64>,
    moments_g: Vec<i64>,
    moments_b: Vec<i64>,
    moments: Vec<f64>,
    cubes: Vec<Box>,
}

impl QuantizerWu {
    pub fn new() -> Self {
        Self {
            weights: vec![0; TOTAL_SIZE],
            moments_r: vec![0; TOTAL_SIZE],
            moments_g: vec![0; TOTAL_SIZE],
            moments_b: vec![0; TOTAL_SIZE],
            moments: vec![0.0; TOTAL_SIZE],
            cubes: Vec::new(),
        }
    }

    /// Quantize pixels to a reduced color palette.
    ///
    /// # Arguments
    /// * `pixels` — List of (R, G, B) tuples
    /// * `max_colors` — Maximum number of colors to return
    ///
    /// # Returns
    /// HashMap mapping ARGB colors to pixel counts.
    pub fn quantize(&mut self, pixels: &[(u8, u8, u8)], max_colors: usize) -> HashMap<u32, i64> {
        self.construct_histogram(pixels);
        self.compute_moments();
        let result_count = self.create_boxes(max_colors);
        self.create_result(result_count)
    }

    fn construct_histogram(&mut self, pixels: &[(u8, u8, u8)]) {
        // Count pixels by color
        let mut count_by_color: HashMap<(u8, u8, u8), i64> = HashMap::new();
        for &(r, g, b) in pixels {
            *count_by_color.entry((r, g, b)).or_insert(0) += 1;
        }

        let bits_to_remove = 8 - INDEX_BITS;
        for (&(red, green, blue), &count) in &count_by_color {
            let i_r = ((red as usize) >> bits_to_remove) + 1;
            let i_g = ((green as usize) >> bits_to_remove) + 1;
            let i_b = ((blue as usize) >> bits_to_remove) + 1;
            let index = get_index(i_r, i_g, i_b);

            self.weights[index] += count;
            self.moments_r[index] += count * (red as i64);
            self.moments_g[index] += count * (green as i64);
            self.moments_b[index] += count * (blue as i64);
            self.moments[index] += count as f64
                * ((red as f64) * (red as f64)
                    + (green as f64) * (green as f64)
                    + (blue as f64) * (blue as f64));
        }
    }

    fn compute_moments(&mut self) {
        for r in 1..SIDE_LENGTH {
            let mut area = vec![0i64; SIDE_LENGTH];
            let mut area_r = vec![0i64; SIDE_LENGTH];
            let mut area_g = vec![0i64; SIDE_LENGTH];
            let mut area_b = vec![0i64; SIDE_LENGTH];
            let mut area2 = vec![0.0f64; SIDE_LENGTH];

            for g in 1..SIDE_LENGTH {
                let mut line = 0i64;
                let mut line_r = 0i64;
                let mut line_g = 0i64;
                let mut line_b = 0i64;
                let mut line2 = 0.0f64;

                for b in 1..SIDE_LENGTH {
                    let index = get_index(r, g, b);
                    line += self.weights[index];
                    line_r += self.moments_r[index];
                    line_g += self.moments_g[index];
                    line_b += self.moments_b[index];
                    line2 += self.moments[index];

                    area[b] += line;
                    area_r[b] += line_r;
                    area_g[b] += line_g;
                    area_b[b] += line_b;
                    area2[b] += line2;

                    let prev_index = get_index(r - 1, g, b);
                    self.weights[index] = self.weights[prev_index] + area[b];
                    self.moments_r[index] = self.moments_r[prev_index] + area_r[b];
                    self.moments_g[index] = self.moments_g[prev_index] + area_g[b];
                    self.moments_b[index] = self.moments_b[prev_index] + area_b[b];
                    self.moments[index] = self.moments[prev_index] + area2[b];
                }
            }
        }
    }

    fn create_boxes(&mut self, max_colors: usize) -> usize {
        self.cubes = (0..max_colors).map(|_| Box::new()).collect();
        let mut volume_variance = vec![0.0f64; max_colors];

        // Initialize first box
        self.cubes[0].r1 = SIDE_LENGTH - 1;
        self.cubes[0].g1 = SIDE_LENGTH - 1;
        self.cubes[0].b1 = SIDE_LENGTH - 1;

        let mut generated_color_count = max_colors;
        let mut next_box = 0usize;
        let mut i = 1usize;

        while i < max_colors {
            if self.cut(next_box, i) {
                volume_variance[next_box] = if self.cubes[next_box].vol > 1 {
                    self.variance(&self.cubes[next_box])
                } else {
                    0.0
                };
                volume_variance[i] = if self.cubes[i].vol > 1 {
                    self.variance(&self.cubes[i])
                } else {
                    0.0
                };
            } else {
                volume_variance[next_box] = 0.0;
                i -= 1;
            }

            // Find box with maximum variance
            next_box = 0;
            let mut temp = volume_variance[0];
            for (j, &v) in volume_variance.iter().enumerate().take(i + 1).skip(1) {
                if v > temp {
                    temp = v;
                    next_box = j;
                }
            }

            if temp <= 0.0 {
                generated_color_count = i + 1;
                break;
            }

            i += 1;
        }

        generated_color_count
    }

    fn create_result(&self, color_count: usize) -> HashMap<u32, i64> {
        let mut result = HashMap::new();
        for i in 0..color_count {
            let cube = &self.cubes[i];
            let weight = self.volume(cube, &self.weights);
            if weight > 0 {
                let r = (self.volume(cube, &self.moments_r) / weight) as u8;
                let g = (self.volume(cube, &self.moments_g) / weight) as u8;
                let b = (self.volume(cube, &self.moments_b) / weight) as u8;
                let color = argb_from_rgb(r, g, b);
                result.insert(color, weight);
            }
        }
        result
    }

    fn variance(&self, cube: &Box) -> f64 {
        let dr = self.volume(cube, &self.moments_r) as f64;
        let dg = self.volume(cube, &self.moments_g) as f64;
        let db = self.volume(cube, &self.moments_b) as f64;

        let xx = self.moments[get_index(cube.r1, cube.g1, cube.b1)]
            - self.moments[get_index(cube.r1, cube.g1, cube.b0)]
            - self.moments[get_index(cube.r1, cube.g0, cube.b1)]
            + self.moments[get_index(cube.r1, cube.g0, cube.b0)]
            - self.moments[get_index(cube.r0, cube.g1, cube.b1)]
            + self.moments[get_index(cube.r0, cube.g1, cube.b0)]
            + self.moments[get_index(cube.r0, cube.g0, cube.b1)]
            - self.moments[get_index(cube.r0, cube.g0, cube.b0)];

        let volume = self.volume(cube, &self.weights) as f64;
        if volume == 0.0 {
            return 0.0;
        }
        xx - (dr * dr + dg * dg + db * db) / volume
    }

    fn cut(&mut self, one_idx: usize, two_idx: usize) -> bool {
        // Clone the boxes we need to avoid borrow conflicts
        let mut one = self.cubes[one_idx];
        let mut two = self.cubes[two_idx];
        let whole_r = self.volume(&one, &self.moments_r);
        let whole_g = self.volume(&one, &self.moments_g);
        let whole_b = self.volume(&one, &self.moments_b);
        let whole_w = self.volume(&one, &self.weights);

        let (max_r_cut, max_r) = self.maximize(
            &one,
            DIR_RED,
            one.r0 + 1,
            one.r1,
            whole_r,
            whole_g,
            whole_b,
            whole_w,
        );
        let (max_g_cut, max_g) = self.maximize(
            &one,
            DIR_GREEN,
            one.g0 + 1,
            one.g1,
            whole_r,
            whole_g,
            whole_b,
            whole_w,
        );
        let (max_b_cut, max_b) = self.maximize(
            &one,
            DIR_BLUE,
            one.b0 + 1,
            one.b1,
            whole_r,
            whole_g,
            whole_b,
            whole_w,
        );

        let (direction, cut_location) = if max_r >= max_g && max_r >= max_b {
            if max_r_cut < 0 {
                return false;
            }
            (DIR_RED, max_r_cut as usize)
        } else if max_g >= max_r && max_g >= max_b {
            (DIR_GREEN, max_g_cut as usize)
        } else {
            (DIR_BLUE, max_b_cut as usize)
        };

        two.r1 = one.r1;
        two.g1 = one.g1;
        two.b1 = one.b1;

        if direction == DIR_RED {
            one.r1 = cut_location;
            two.r0 = one.r1;
            two.g0 = one.g0;
            two.b0 = one.b0;
        } else if direction == DIR_GREEN {
            one.g1 = cut_location;
            two.r0 = one.r0;
            two.g0 = one.g1;
            two.b0 = one.b0;
        } else {
            one.b1 = cut_location;
            two.r0 = one.r0;
            two.g0 = one.g0;
            two.b0 = one.b1;
        }

        one.vol = (one.r1 - one.r0) * (one.g1 - one.g0) * (one.b1 - one.b0);
        two.vol = (two.r1 - two.r0) * (two.g1 - two.g0) * (two.b1 - two.b0);

        // Write back
        self.cubes[one_idx] = one;
        self.cubes[two_idx] = two;

        true
    }

    #[allow(clippy::too_many_arguments)]
    fn maximize(
        &self,
        cube: &Box,
        direction: usize,
        first: usize,
        last: usize,
        whole_r: i64,
        whole_g: i64,
        whole_b: i64,
        whole_w: i64,
    ) -> (i64, f64) {
        let bottom_r = self.bottom(cube, direction, &self.moments_r);
        let bottom_g = self.bottom(cube, direction, &self.moments_g);
        let bottom_b = self.bottom(cube, direction, &self.moments_b);
        let bottom_w = self.bottom(cube, direction, &self.weights);

        let mut max_val = 0.0f64;
        let mut cut: i64 = -1;

        for i in first..last {
            let half_r = bottom_r + self.top(cube, direction, i, &self.moments_r);
            let half_g = bottom_g + self.top(cube, direction, i, &self.moments_g);
            let half_b = bottom_b + self.top(cube, direction, i, &self.moments_b);
            let half_w = bottom_w + self.top(cube, direction, i, &self.weights);

            if half_w == 0 {
                continue;
            }

            let mut temp =
                (half_r * half_r + half_g * half_g + half_b * half_b) as f64 / half_w as f64;

            let half_r = whole_r - half_r;
            let half_g = whole_g - half_g;
            let half_b = whole_b - half_b;
            let half_w = whole_w - half_w;

            if half_w == 0 {
                continue;
            }

            temp += (half_r * half_r + half_g * half_g + half_b * half_b) as f64 / half_w as f64;

            if temp > max_val {
                max_val = temp;
                cut = i as i64;
            }
        }

        (cut, max_val)
    }

    fn volume(&self, cube: &Box, moment: &[i64]) -> i64 {
        moment[get_index(cube.r1, cube.g1, cube.b1)]
            - moment[get_index(cube.r1, cube.g1, cube.b0)]
            - moment[get_index(cube.r1, cube.g0, cube.b1)]
            + moment[get_index(cube.r1, cube.g0, cube.b0)]
            - moment[get_index(cube.r0, cube.g1, cube.b1)]
            + moment[get_index(cube.r0, cube.g1, cube.b0)]
            + moment[get_index(cube.r0, cube.g0, cube.b1)]
            - moment[get_index(cube.r0, cube.g0, cube.b0)]
    }

    fn bottom(&self, cube: &Box, direction: usize, moment: &[i64]) -> i64 {
        if direction == DIR_RED {
            -moment[get_index(cube.r0, cube.g1, cube.b1)]
                + moment[get_index(cube.r0, cube.g1, cube.b0)]
                + moment[get_index(cube.r0, cube.g0, cube.b1)]
                - moment[get_index(cube.r0, cube.g0, cube.b0)]
        } else if direction == DIR_GREEN {
            -moment[get_index(cube.r1, cube.g0, cube.b1)]
                + moment[get_index(cube.r1, cube.g0, cube.b0)]
                + moment[get_index(cube.r0, cube.g0, cube.b1)]
                - moment[get_index(cube.r0, cube.g0, cube.b0)]
        } else {
            -moment[get_index(cube.r1, cube.g1, cube.b0)]
                + moment[get_index(cube.r1, cube.g0, cube.b0)]
                + moment[get_index(cube.r0, cube.g1, cube.b0)]
                - moment[get_index(cube.r0, cube.g0, cube.b0)]
        }
    }

    fn top(&self, cube: &Box, direction: usize, position: usize, moment: &[i64]) -> i64 {
        if direction == DIR_RED {
            moment[get_index(position, cube.g1, cube.b1)]
                - moment[get_index(position, cube.g1, cube.b0)]
                - moment[get_index(position, cube.g0, cube.b1)]
                + moment[get_index(position, cube.g0, cube.b0)]
        } else if direction == DIR_GREEN {
            moment[get_index(cube.r1, position, cube.b1)]
                - moment[get_index(cube.r1, position, cube.b0)]
                - moment[get_index(cube.r0, position, cube.b1)]
                + moment[get_index(cube.r0, position, cube.b0)]
        } else {
            moment[get_index(cube.r1, cube.g1, position)]
                - moment[get_index(cube.r1, cube.g0, position)]
                - moment[get_index(cube.r0, cube.g1, position)]
                + moment[get_index(cube.r0, cube.g0, position)]
        }
    }
}

impl Default for QuantizerWu {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function: quantize RGB pixels using Wu algorithm.
///
/// Returns a HashMap mapping ARGB colors to pixel counts.
pub fn quantize_wu(pixels: &[(u8, u8, u8)], max_colors: usize) -> HashMap<u32, i64> {
    let mut quantizer = QuantizerWu::new();
    quantizer.quantize(pixels, max_colors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantize_wu_single_color() {
        // All pixels the same color
        let pixels: Vec<(u8, u8, u8)> = (0..100).map(|_| (255, 87, 34)).collect();
        let result = quantize_wu(&pixels, 5);
        assert_eq!(result.len(), 1);
        let argb = argb_from_rgb(255, 87, 34);
        assert!(result.contains_key(&argb));
        assert_eq!(result[&argb], 100);
    }

    #[test]
    fn test_quantize_wu_two_colors() {
        let mut pixels: Vec<(u8, u8, u8)> = (0..50).map(|_| (255, 0, 0)).collect();
        pixels.extend((0..50).map(|_| (0, 0, 255)));
        let result = quantize_wu(&pixels, 5);
        assert!(result.len() >= 2);
    }

    #[test]
    fn test_quantize_wu_empty() {
        let pixels: Vec<(u8, u8, u8)> = vec![];
        let result = quantize_wu(&pixels, 5);
        assert!(result.is_empty());
    }
}
