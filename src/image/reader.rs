//! Image reading and resizing utilities.
//!
//! Replaces Python's `image.py` — uses the `image` crate to decode
//! and downscale wallpaper images for color extraction.
//!
//! The target size of 112x112 matches matugen's default extraction resolution.

use image::imageops::FilterType;
use std::path::Path;

pub use crate::color::Rgb;

/// Default resize dimension for color extraction (matches matugen).
const EXTRACT_SIZE: u32 = 112;

/// Read an image file and return its pixels as RGB tuples.
///
/// The image is downscaled to `EXTRACT_SIZE` x `EXTRACT_SIZE` using
/// the specified filter type for efficient color extraction.
///
/// # Arguments
/// * `path` - Path to the image file (PNG, JPEG, WebP supported)
/// * `filter` - Resize filter type
///
/// # Errors
/// Returns an error if the file cannot be read or decoded.
pub fn read_image(path: &Path, filter: ResizeFilter) -> Result<Vec<Rgb>, String> {
    // Use ImageReader which can auto-detect format from content.
    // This handles symlinks without extensions (e.g. `background` → `foo.jpg`).
    let mut reader =
        image::ImageReader::open(path).map_err(|e| format!("Failed to open image: {}", e))?;

    // Try to detect format from file extension first, then fall back to content sniffing.
    if reader.format().is_none() {
        // Follow symlinks to get the real path for extension-based detection
        let canonical = path
            .canonicalize()
            .map_err(|e| format!("Failed to resolve path: {}", e))?;
        let reader2 = image::ImageReader::open(&canonical)
            .map_err(|e| format!("Failed to open image: {}", e))?;
        // If still no format from extension, try content sniffing
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

    for y in 0..height {
        for x in 0..width {
            let pixel = rgba.get_pixel(x, y);
            let r = pixel[0];
            let g = pixel[1];
            let b = pixel[2];
            // Skip fully transparent pixels
            let a = pixel[3];
            if a > 0 {
                pixels.push((r, g, b));
            }
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
            .flat_map(|_| vec![255, 87, 34, 255]) // #FF5722
            .collect();
        encoder
            .write_image(&pixels, 112, 112, image::ExtendedColorType::Rgba8)
            .unwrap();
        file
    }

    #[test]
    fn test_read_image_solid_color() {
        let file = create_test_png();
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
        let file = create_test_png();
        for filter in [ResizeFilter::Triangle, ResizeFilter::Nearest] {
            let pixels = read_image(file.path(), filter).unwrap();
            assert!(!pixels.is_empty());
        }
    }
}
