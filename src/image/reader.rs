//! Image reading and resizing utilities.
//!
//! Replaces Python's `image.py` — uses the `image` crate to decode
//! and downscale wallpaper images for color extraction.
//!
//! The target size of 112x112 matches matugen's default extraction resolution.

use image::imageops::FilterType;
use std::path::Path;

pub use crate::core::color::Rgb;

/// Default resize dimension for color extraction (matches matugen).
const EXTRACT_SIZE: u32 = 112;

/// Read an image file and return its pixels as RGB tuples.
///
/// The image is downscaled to `EXTRACT_SIZE` × `EXTRACT_SIZE` using the
/// specified filter type for efficient color extraction.
///
/// # Errors
/// Returns an error if the file cannot be read or decoded.
pub fn read_image(path: &Path, filter: ResizeFilter) -> Result<Vec<Rgb>, String> {
    let mut reader =
        image::ImageReader::open(path).map_err(|e| format!("Failed to open image: {}", e))?;

    if reader.format().is_none() {
        let canonical = path
            .canonicalize()
            .map_err(|e| format!("Failed to resolve path: {}", e))?;
        let reader2 = image::ImageReader::open(&canonical)
            .map_err(|e| format!("Failed to open image: {}", e))?;
        reader = if reader2.format().is_none() {
            reader2
                .with_guessed_format()
                .map_err(|e| format!("Failed to guess image format: {}", e))?
        } else {
            reader2
        };
    }

    let filter_type = match filter {
        ResizeFilter::Triangle => FilterType::Triangle,
        ResizeFilter::Nearest => FilterType::Nearest,
    };

    let img = reader
        .decode()
        .map_err(|e| format!("Failed to decode image: {}", e))?;
    let resized = img.resize_exact(EXTRACT_SIZE, EXTRACT_SIZE, filter_type);
    let rgba = resized.to_rgba8();

    let (width, height) = rgba.dimensions();
    let mut pixels = Vec::with_capacity((width * height) as usize);

    for pixel in rgba.pixels() {
        if pixel[3] > 0 {
            pixels.push((pixel[0], pixel[1], pixel[2]));
        }
    }

    if pixels.is_empty() {
        return Err("Image contains no opaque pixels".to_string());
    }

    Ok(pixels)
}

/// Resize filter types matching matugen's options.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ResizeFilter {
    /// Bilinear interpolation — default for M3 schemes.
    #[default]
    Triangle,
    /// Nearest neighbor — preserves distinct color regions for k-means.
    Nearest,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::test_support::{TEST_IMAGE_SIZE, solid_png};

    #[test]
    fn test_read_image_solid_color() {
        let file = solid_png([255, 87, 34], TEST_IMAGE_SIZE);
        let pixels = read_image(file.path(), ResizeFilter::Triangle).unwrap();
        assert!(!pixels.is_empty());
        // All pixels should be approximately #FF5722
        let (r, g, b) = pixels[0];
        assert!(r > 200);
        assert!(g > 50 && g < 120);
        assert!(b < 60);
    }

    #[test]
    fn test_read_image_filter_types() {
        let file = solid_png([255, 87, 34], TEST_IMAGE_SIZE);
        for filter in [ResizeFilter::Triangle, ResizeFilter::Nearest] {
            let pixels = read_image(file.path(), filter).unwrap();
            assert!(!pixels.is_empty());
        }
    }
}
