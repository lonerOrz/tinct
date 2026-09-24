//! Shared fixtures for the image module's tests.

use image::ImageEncoder;
use tempfile::NamedTempFile;

/// Edge length (px) of the solid-colour fixtures.
pub const TEST_IMAGE_SIZE: u32 = 112;

/// Write a `size`×`size` PNG filled with a single opaque RGB colour.
pub fn solid_png(rgb: [u8; 3], size: u32) -> NamedTempFile {
    let mut file = tempfile::Builder::new()
        .prefix("test_")
        .suffix(".png")
        .tempfile()
        .unwrap();
    let encoder = image::codecs::png::PngEncoder::new(&mut file);
    let pixels = [rgb[0], rgb[1], rgb[2], 255u8].repeat((size * size) as usize);
    encoder
        .write_image(&pixels, size, size, image::ExtendedColorType::Rgba8)
        .unwrap();
    file
}
