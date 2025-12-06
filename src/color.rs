// Color format representation
#[derive(Debug, Clone)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone)]
pub struct Hsl {
    pub h: f64,
    pub s: f64,
    pub l: f64,
}

/// Clamp a value between min and max
pub fn clamp(n: f64, minn: f64, maxn: f64) -> f64 {
    n.max(minn).min(maxn)
}

/// Convert HEX color to RGB values
pub fn hex_to_rgb(hex_color: &str) -> Result<Rgb, String> {
    let hex = hex_color.trim_start_matches('#');
    if hex.len() != 6 {
        return Err(format!(
            "Invalid hex color format: {}. Expected 6 characters.",
            hex
        ));
    }

    let r = u8::from_str_radix(&hex[0..2], 16)
        .map_err(|_| format!("Invalid hex color format: {}", hex))?;
    let g = u8::from_str_radix(&hex[2..4], 16)
        .map_err(|_| format!("Invalid hex color format: {}", hex))?;
    let b = u8::from_str_radix(&hex[4..6], 16)
        .map_err(|_| format!("Invalid hex color format: {}", hex))?;

    Ok(Rgb { r, g, b })
}

/// Convert RGB values to HEX color
pub fn rgb_to_hex(r: f64, g: f64, b: f64) -> String {
    let r = clamp(r.round(), 0.0, 255.0) as u8;
    let g = clamp(g.round(), 0.0, 255.0) as u8;
    let b = clamp(b.round(), 0.0, 255.0) as u8;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

/// Convert RGB values to HSL
pub fn rgb_to_hsl(r: f64, g: f64, b: f64) -> Hsl {
    let r = r / 255.0;
    let g = g / 255.0;
    let b = b / 255.0;
    let mx = r.max(g).max(b);
    let mn = r.min(g).min(b);
    let l = (mx + mn) / 2.0;

    if (mx - mn).abs() < f64::EPSILON {
        return Hsl {
            h: 0.0,
            s: 0.0,
            l: l * 100.0,
        };
    }

    let d = mx - mn;
    let s = if l > 0.5 {
        d / (2.0 - mx - mn)
    } else {
        d / (mx + mn)
    };

    let h = if (mx - r).abs() < f64::EPSILON {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if (mx - g).abs() < f64::EPSILON {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    } / 6.0;

    Hsl {
        h: h * 360.0,
        s: s * 100.0,
        l: l * 100.0,
    }
}

/// Convert HSL values to RGB
pub fn hsl_to_rgb(h: f64, s: f64, l: f64) -> Rgb {
    let h = h / 360.0;
    let s = s / 100.0;
    let l = l / 100.0;

    if s.abs() < f64::EPSILON {
        let val = (l * 255.0).round() as u8;
        return Rgb {
            r: val,
            g: val,
            b: val,
        };
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;

    fn hue2rgb(p: f64, q: f64, t: f64) -> f64 {
        let mut t = t;
        if t < 0.0 {
            t += 1.0;
        }
        if t > 1.0 {
            t -= 1.0;
        }
        if t < 1.0 / 6.0 {
            return p + (q - p) * 6.0 * t;
        }
        if t < 0.5 {
            return q;
        }
        if t < 2.0 / 3.0 {
            return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
        }
        p
    }

    let r = hue2rgb(p, q, h + 1.0 / 3.0);
    let g = hue2rgb(p, q, h);
    let b = hue2rgb(p, q, h - 1.0 / 3.0);

    Rgb {
        r: (r * 255.0).round() as u8,
        g: (g * 255.0).round() as u8,
        b: (b * 255.0).round() as u8,
    }
}

/// Calculate relative luminance of a color
pub fn get_luminance(hexcolor: &str) -> Result<f64, String> {
    let rgb = hex_to_rgb(hexcolor)?;

    fn convert(v: u8) -> f64 {
        let v = v as f64 / 255.0;
        if v <= 0.03928 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }

    let r = convert(rgb.r);
    let g = convert(rgb.g);
    let b = convert(rgb.b);
    Ok(0.2126 * r + 0.7152 * g + 0.0722 * b)
}

/// Calculate contrast ratio between two colors
pub fn get_contrast_ratio(a: &str, b: &str) -> Result<f64, String> {
    let l1 = get_luminance(a)?;
    let l2 = get_luminance(b)?;

    let bright = l1.max(l2);
    let dark = l1.min(l2);
    Ok((bright + 0.05) / (dark + 0.05))
}

/// Check if a color is light or dark based on luminance
pub fn is_light_color(hexcolor: &str) -> Result<bool, String> {
    Ok(get_luminance(hexcolor)? > 0.5)
}

/// Adjust the lightness of a color by a given amount
pub fn adjust_lightness(hexcolor: &str, amount: f64) -> Result<String, String> {
    let rgb = hex_to_rgb(hexcolor)?;
    let hsl = rgb_to_hsl(rgb.r as f64, rgb.g as f64, rgb.b as f64);
    let new_l = clamp(hsl.l + amount, 0.0, 100.0);
    let new_rgb = hsl_to_rgb(hsl.h, hsl.s, new_l);
    Ok(rgb_to_hex(
        new_rgb.r as f64,
        new_rgb.g as f64,
        new_rgb.b as f64,
    ))
}

/// Adjust the saturation of a color by a given amount
pub fn adjust_saturation(hexcolor: &str, amount: f64) -> Result<String, String> {
    let rgb = hex_to_rgb(hexcolor)?;
    let hsl = rgb_to_hsl(rgb.r as f64, rgb.g as f64, rgb.b as f64);
    let new_s = clamp(hsl.s + amount, 0.0, 100.0);
    let new_rgb = hsl_to_rgb(hsl.h, new_s, hsl.l);
    Ok(rgb_to_hex(
        new_rgb.r as f64,
        new_rgb.g as f64,
        new_rgb.b as f64,
    ))
}

/// Adjust both lightness and saturation of a color
pub fn adjust_lightness_and_saturation(hexcolor: &str, la: f64, sa: f64) -> Result<String, String> {
    let rgb = hex_to_rgb(hexcolor)?;
    let hsl = rgb_to_hsl(rgb.r as f64, rgb.g as f64, rgb.b as f64);
    let new_l = clamp(hsl.l + la, 0.0, 100.0);
    let new_s = clamp(hsl.s + sa, 0.0, 100.0);
    let new_rgb = hsl_to_rgb(hsl.h, new_s, new_l);
    Ok(rgb_to_hex(
        new_rgb.r as f64,
        new_rgb.g as f64,
        new_rgb.b as f64,
    ))
}

/// Generate appropriate text color for a given background
pub fn generate_on_color(base: &str, _is_dark: bool) -> Result<String, String> {
    // The _is_dark parameter determines the overall theme preference,
    // but we still want to ensure good contrast regardless of the theme preference
    let light = is_light_color(base)?;

    if light {
        // If the base color is light, we want dark text for contrast
        if get_contrast_ratio(base, "#000000")? >= 4.5 {
            Ok("#000000".to_string())
        } else {
            Ok("#1c1b1f".to_string())
        }
    } else {
        // If the base color is dark, we want light text for contrast
        if get_contrast_ratio(base, "#ffffff")? >= 4.5 {
            Ok("#ffffff".to_string())
        } else {
            Ok("#e6e1e5".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_rgb() {
        let rgb = hex_to_rgb("#ffffff").unwrap();
        assert_eq!(rgb.r, 255);
        assert_eq!(rgb.g, 255);
        assert_eq!(rgb.b, 255);

        let rgb = hex_to_rgb("#000000").unwrap();
        assert_eq!(rgb.r, 0);
        assert_eq!(rgb.g, 0);
        assert_eq!(rgb.b, 0);

        let rgb = hex_to_rgb("#ff0000").unwrap();
        assert_eq!(rgb.r, 255);
        assert_eq!(rgb.g, 0);
        assert_eq!(rgb.b, 0);
    }

    #[test]
    fn test_hex_to_rgb_no_hash() {
        let rgb = hex_to_rgb("ffffff").unwrap();
        assert_eq!(rgb.r, 255);
        assert_eq!(rgb.g, 255);
        assert_eq!(rgb.b, 255);
    }

    #[test]
    fn test_rgb_to_hex() {
        assert_eq!(rgb_to_hex(255.0, 255.0, 255.0), "#ffffff");
        assert_eq!(rgb_to_hex(0.0, 0.0, 0.0), "#000000");
        assert_eq!(rgb_to_hex(255.0, 0.0, 0.0), "#ff0000");
    }

    #[test]
    fn test_rgb_to_hsl() {
        let hsl = rgb_to_hsl(255.0, 0.0, 0.0); // Red
        assert_eq!(hsl.h as u32, 0); // Hue should be around 0 for red
        assert!((hsl.s - 100.0).abs() < 1.0); // Should be fully saturated
        assert!((hsl.l - 50.0).abs() < 1.0); // Should be mid-lightness

        let hsl = rgb_to_hsl(0.0, 255.0, 0.0); // Green
        assert_eq!(hsl.h as u32, 120); // Hue should be around 120 for green
    }

    #[test]
    fn test_hsl_to_rgb() {
        let rgb = hsl_to_rgb(0.0, 100.0, 50.0); // Red
        assert_eq!(rgb.r, 255);
        assert_eq!(rgb.g, 0);
        assert_eq!(rgb.b, 0);

        let rgb = hsl_to_rgb(120.0, 100.0, 50.0); // Green
        assert_eq!(rgb.r, 0);
        assert_eq!(rgb.g, 255);
        assert_eq!(rgb.b, 0);
    }

    #[test]
    fn test_adjust_lightness() {
        let result = adjust_lightness("#ff0000", -20.0).unwrap();
        assert!(result != "#ff0000"); // Should be different

        // Test clamping
        let result = adjust_lightness("#808080", 100.0).unwrap(); // Very light gray
        let rgb = hex_to_rgb(&result).unwrap();
        let hsl = rgb_to_hsl(rgb.r as f64, rgb.g as f64, rgb.b as f64);
        assert!(hsl.l >= 99.9); // Should be close to max lightness
    }

    #[test]
    fn test_adjust_saturation() {
        let result = adjust_saturation("#804040", 20.0).unwrap(); // Start with a less saturated color
        assert!(result != "#804040"); // Should be different
    }

    #[test]
    fn test_get_luminance() {
        let lum_white = get_luminance("#ffffff").unwrap();
        assert!(lum_white > 0.9); // White should have high luminance

        let lum_black = get_luminance("#000000").unwrap();
        assert!(lum_black < 0.1); // Black should have low luminance
    }

    #[test]
    fn test_get_contrast_ratio() {
        let ratio = get_contrast_ratio("#ffffff", "#000000").unwrap();
        assert!(ratio > 20.0); // Black and white should have high contrast

        let ratio = get_contrast_ratio("#ffffff", "#ffffff").unwrap();
        assert!((ratio - 1.0).abs() < 0.1); // Same colors should have ratio of 1
    }

    #[test]
    fn test_is_light_color() {
        assert_eq!(is_light_color("#ffffff").unwrap(), true);
        assert_eq!(is_light_color("#000000").unwrap(), false);
        assert_eq!(is_light_color("#888888").unwrap(), false); // Mid gray is considered dark
    }

    #[test]
    fn test_generate_on_color() {
        // Test with light background - should return dark text color
        let color = generate_on_color("#ffffff", true).unwrap();
        assert!(color == "#000000" || color == "#1c1b1f"); // Either pure black or dark gray

        // Test with dark background - should return light text color
        let color = generate_on_color("#000000", true).unwrap();
        assert!(color == "#ffffff" || color == "#e6e1e5"); // Either pure white or light gray
    }

    #[test]
    fn test_adjust_lightness_and_saturation() {
        let result = adjust_lightness_and_saturation("#ff8080", -10.0, 10.0).unwrap();
        assert!(result != "#ff8080"); // Should be different
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(5.0, 0.0, 10.0), 5.0);
        assert_eq!(clamp(-1.0, 0.0, 10.0), 0.0);
        assert_eq!(clamp(15.0, 0.0, 10.0), 10.0);
    }
}
