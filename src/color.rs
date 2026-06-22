/// RGB components as (r, g, b)
pub type Rgb = (u8, u8, u8);

/// HSL components as (h, s, l)
pub type Hsl = (f64, f64, f64);

/// Clamp value between min and max
pub fn clamp<T: PartialOrd + Copy>(n: T, minn: T, maxn: T) -> T {
    if n < minn {
        minn
    } else if n > maxn {
        maxn
    } else {
        n
    }
}

/// Convert RGB to hex string
pub fn rgb_to_hex(r: f64, g: f64, b: f64) -> String {
    let r_byte = (clamp(r, 0.0, 255.0)).round() as u8;
    let g_byte = (clamp(g, 0.0, 255.0)).round() as u8;
    let b_byte = (clamp(b, 0.0, 255.0)).round() as u8;
    format!("#{:02X}{:02X}{:02X}", r_byte, g_byte, b_byte)
}

/// Convert hex string to RGB tuple
pub fn hex_to_rgb(hex: &str) -> Result<Rgb, String> {
    let hex_stripped = hex.trim_start_matches('#');

    if hex_stripped.len() != 6 {
        return Err("Invalid hex color format".to_string());
    }

    let r = u8::from_str_radix(&hex_stripped[0..2], 16)
        .map_err(|_| format!("Invalid hex color: {}", hex))?;
    let g = u8::from_str_radix(&hex_stripped[2..4], 16)
        .map_err(|_| format!("Invalid hex color: {}", hex))?;
    let b = u8::from_str_radix(&hex_stripped[4..6], 16)
        .map_err(|_| format!("Invalid hex color: {}", hex))?;

    Ok((r, g, b))
}

/// Convert RGB to HSL tuple
pub fn rgb_to_hsl(r: f64, g: f64, b: f64) -> Hsl {
    let r = r / 255.0;
    let g = g / 255.0;
    let b = b / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);

    let h = if max == min {
        0.0
    } else if max == r {
        60.0 * (((g - b) / (max - min)) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / (max - min)) + 2.0)
    } else {
        60.0 * (((r - g) / (max - min)) + 4.0)
    };

    let l = (max + min) / 2.0;

    let s = if max == min {
        0.0
    } else if l < 0.5 {
        (max - min) / (2.0 * l)
    } else {
        (max - min) / (2.0 - 2.0 * l)
    };

    (
        clamp(h, 0.0, 360.0),
        clamp(s * 100.0, 0.0, 100.0),
        clamp(l * 100.0, 0.0, 100.0),
    )
}

/// Convert HSL to RGB tuple
pub fn hsl_to_rgb(h: f64, s: f64, l: f64) -> Rgb {
    let h = h / 360.0;
    let s = s / 100.0;
    let l = l / 100.0;

    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = if h < 1.0 / 6.0 {
        (c, x, 0.0)
    } else if h < 2.0 / 6.0 {
        (x, c, 0.0)
    } else if h < 3.0 / 6.0 {
        (0.0, c, x)
    } else if h < 4.0 / 6.0 {
        (0.0, x, c)
    } else if h < 5.0 / 6.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (
        ((r + m) * 255.0).round() as u8,
        ((g + m) * 255.0).round() as u8,
        ((b + m) * 255.0).round() as u8,
    )
}

/// Determine if a color is light or dark based on relative luminance
pub fn is_light_color(hex: &str) -> Result<bool, String> {
    let (r, g, b) = hex_to_rgb(hex)?;
    let luminance = calculate_relative_luminance(r, g, b);
    Ok(luminance > 0.1791)
}

/// Calculate relative luminance for a color
pub fn calculate_relative_luminance(r: u8, g: u8, b: u8) -> f64 {
    let r = r as f64 / 255.0;
    let g = g as f64 / 255.0;
    let b = b as f64 / 255.0;

    let linear_r = if r <= 0.03928 {
        r / 12.92
    } else {
        ((r + 0.055) / 1.055).powf(2.4)
    };
    let linear_g = if g <= 0.03928 {
        g / 12.92
    } else {
        ((g + 0.055) / 1.055).powf(2.4)
    };
    let linear_b = if b <= 0.03928 {
        b / 12.92
    } else {
        ((b + 0.055) / 1.055).powf(2.4)
    };

    0.2126 * linear_r + 0.7152 * linear_g + 0.0722 * linear_b
}

/// Calculate contrast ratio between two colors
pub fn calculate_contrast_ratio(color1: &str, color2: &str) -> Result<f64, String> {
    let (r1, g1, b1) = hex_to_rgb(color1)?;
    let (r2, g2, b2) = hex_to_rgb(color2)?;

    let l1 = calculate_relative_luminance(r1, g1, b1);
    let l2 = calculate_relative_luminance(r2, g2, b2);

    let lighter = l1.max(l2);
    let darker = l1.min(l2);

    Ok((lighter + 0.05) / (darker + 0.05))
}

/// Check if two colors meet WCAG contrast requirements
/// Returns true if contrast ratio meets or exceeds the threshold
pub fn meets_contrast_requirement(
    color1: &str,
    color2: &str,
    threshold: f64,
) -> Result<bool, String> {
    let ratio = calculate_contrast_ratio(color1, color2)?;
    Ok(ratio >= threshold)
}

/// Get contrast rating based on WCAG guidelines
/// Returns: "AAA" (>=7.0), "AA" (>=4.5), "AA Large" (>=3.0), or "Fail"
pub fn get_contrast_rating(color1: &str, color2: &str) -> Result<String, String> {
    let ratio = calculate_contrast_ratio(color1, color2)?;

    if ratio >= 7.0 {
        Ok("AAA".to_string())
    } else if ratio >= 4.5 {
        Ok("AA".to_string())
    } else if ratio >= 3.0 {
        Ok("AA Large".to_string())
    } else {
        Ok("Fail".to_string())
    }
}

/// Generate appropriate text color for a given background
pub fn generate_on_color(base: &str, _is_dark: bool) -> Result<String, String> {
    let light = is_light_color(base)?;

    if light {
        if calculate_contrast_ratio(base, "#000000")? >= 4.5 {
            Ok("#000000".to_string())
        } else {
            Ok("#1c1b1f".to_string())
        }
    } else if calculate_contrast_ratio(base, "#ffffff")? >= 4.5 {
        Ok("#ffffff".to_string())
    } else {
        Ok("#e6e1e5".to_string())
    }
}

/// HCT (Hue-Chroma-Tone) color space implementation for Material Design 3
#[derive(Debug, Clone)]
pub struct Hct {
    pub h: f64, // Hue (0-360)
    pub c: f64, // Chroma (0-100+)
    pub t: f64, // Tone (0-100, equivalent to L* in L*a*b*)
}

impl Hct {
    /// Create an HCT color from hue, chroma, and tone values
    pub fn from_hct(h: f64, c: f64, t: f64) -> Self {
        Self {
            h: clamp(h, 0.0, 360.0),
            c: clamp(c, 0.0, 200.0),
            t: clamp(t, 0.0, 100.0),
        }
    }

    /// Convert HCT to RGB
    pub fn to_rgb(&self) -> Rgb {
        let s = clamp(self.c * 0.8, 0.0, 100.0);
        hsl_to_rgb(self.h, s, self.t)
    }

    /// Convert HCT to HEX
    pub fn to_hex(&self) -> String {
        let (r, g, b) = self.to_rgb();
        rgb_to_hex(r as f64, g as f64, b as f64)
    }
}

/// Generate HCT color from RGB
pub fn rgb_to_hct(r: u8, g: u8, b: u8) -> Hct {
    let (h, s, l) = rgb_to_hsl(r as f64, g as f64, b as f64);
    Hct {
        h,
        c: clamp(s * 1.2, 0.0, 150.0),
        t: l,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_rgb() {
        let (r, g, b) = hex_to_rgb("#ffffff").unwrap();
        assert_eq!(r, 255);
        assert_eq!(g, 255);
        assert_eq!(b, 255);

        let (r, g, b) = hex_to_rgb("#000000").unwrap();
        assert_eq!(r, 0);
        assert_eq!(g, 0);
        assert_eq!(b, 0);
    }

    #[test]
    fn test_rgb_to_hex() {
        let hex = rgb_to_hex(255.0, 255.0, 255.0);
        assert_eq!(hex, "#FFFFFF");

        let hex = rgb_to_hex(0.0, 0.0, 0.0);
        assert_eq!(hex, "#000000");
    }

    #[test]
    fn test_rgb_to_hsl() {
        let (h, s, l) = rgb_to_hsl(255.0, 0.0, 0.0);
        assert!((h - 0.0).abs() < 0.1);
        assert!((s - 100.0).abs() < 0.1);
        assert!((l - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_hsl_to_rgb() {
        let (r, g, b) = hsl_to_rgb(0.0, 100.0, 50.0);
        assert!((r as f64 - 255.0).abs() < 1.0);
        assert!((g as f64 - 0.0).abs() < 1.0);
        assert!((b as f64 - 0.0).abs() < 1.0);
    }

    #[test]
    fn test_contrast_ratio() {
        let ratio = calculate_contrast_ratio("#FFFFFF", "#000000").unwrap();
        assert!((ratio - 21.0).abs() < 1.0);
    }

    #[test]
    fn test_is_light_color() {
        // White should be light
        assert!(is_light_color("#FFFFFF").unwrap());
        // Black should be dark
        assert!(!is_light_color("#000000").unwrap());
        // Test with actual colors
        assert!(is_light_color("#FF5722").unwrap()); // Orange - should be light
        assert!(!is_light_color("#1a1a1a").unwrap()); // Very dark gray - should be dark
    }

    #[test]
    fn test_meets_contrast_requirement() {
        // Black and white should meet AAA requirements
        assert!(meets_contrast_requirement("#000000", "#FFFFFF", 7.0).unwrap());
        // Similar colors should fail
        assert!(!meets_contrast_requirement("#666666", "#999999", 4.5).unwrap());
    }

    #[test]
    fn test_get_contrast_rating() {
        // Black and white should be AAA
        assert_eq!(get_contrast_rating("#000000", "#FFFFFF").unwrap(), "AAA");
        // Medium contrast should be AA
        assert_eq!(get_contrast_rating("#000000", "#767676").unwrap(), "AA");
        // Low contrast should fail
        assert_eq!(get_contrast_rating("#666666", "#999999").unwrap(), "Fail");
    }

    #[test]
    fn test_relative_luminance() {
        // White has luminance of 1.0
        let lum = calculate_relative_luminance(255, 255, 255);
        assert!((lum - 1.0).abs() < 0.001);

        // Black has luminance of 0.0
        let lum = calculate_relative_luminance(0, 0, 0);
        assert!((lum - 0.0).abs() < 0.001);
    }
}
