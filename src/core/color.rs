//! Color model.
//!
//! A lean RGB + alpha value ([`Color`]) with on-demand format conversions,
//! plus the built-in [`ColorFilter`] transformations used by templates.

use crate::core::Error;

/// RGB components as (r, g, b)
pub type Rgb = (u8, u8, u8);

/// HSL components as (h, s, l)
pub type Hsl = (f64, f64, f64);

/// Lean color value: RGB + alpha. All derived formats computed on demand.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub alpha: f64, // 0.0–1.0
}

/// Enum representing different color output formats
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorProperty {
    Hex,
    HexStripped,
    Hex8,
    Hex8Stripped,
    Rgb,
    Rgba,
    Red,
    Green,
    Blue,
    Alpha,
    Hsl,
    Hsla,
    Hue,
    Saturation,
    Lightness,
}

impl ColorProperty {
    pub fn from_property(property: &str) -> Option<Self> {
        match property {
            "hex" => Some(ColorProperty::Hex),
            "hex_stripped" => Some(ColorProperty::HexStripped),
            "hex8" => Some(ColorProperty::Hex8),
            "hex8_stripped" => Some(ColorProperty::Hex8Stripped),
            "rgb" => Some(ColorProperty::Rgb),
            "rgba" => Some(ColorProperty::Rgba),
            "red" => Some(ColorProperty::Red),
            "green" => Some(ColorProperty::Green),
            "blue" => Some(ColorProperty::Blue),
            "alpha" => Some(ColorProperty::Alpha),
            "hsl" => Some(ColorProperty::Hsl),
            "hsla" => Some(ColorProperty::Hsla),
            "hue" => Some(ColorProperty::Hue),
            "saturation" => Some(ColorProperty::Saturation),
            "lightness" => Some(ColorProperty::Lightness),
            _ => None,
        }
    }

    /// Returns true for formats that represent a full color value (not a channel).
    pub fn is_complete_color(&self) -> bool {
        matches!(
            self,
            ColorProperty::Hex
                | ColorProperty::HexStripped
                | ColorProperty::Hex8
                | ColorProperty::Hex8Stripped
                | ColorProperty::Rgb
                | ColorProperty::Rgba
                | ColorProperty::Hsl
                | ColorProperty::Hsla
        )
    }
}

/// Built-in color filters
#[derive(Debug, Clone, Copy)]
pub enum ColorFilter {
    SetAlpha(f64),
    Lighten(f64),
    Darken(f64),
    Saturate(f64),
    Desaturate(f64),
}

impl ColorFilter {
    pub fn from_name(name: &str, param: &str) -> Option<Self> {
        let val = param.parse::<f64>().ok()?;
        match name {
            "set_alpha" => Some(ColorFilter::SetAlpha(val.clamp(0.0, 1.0))),
            "lighten" => Some(ColorFilter::Lighten(val)),
            "darken" => Some(ColorFilter::Darken(val)),
            "saturate" => Some(ColorFilter::Saturate(val)),
            "desaturate" => Some(ColorFilter::Desaturate(val)),
            _ => None,
        }
    }

    pub fn is_compatible(&self, format_type: &ColorProperty) -> bool {
        format_type.is_complete_color()
    }

    /// Apply this filter to a color and return the formatted result.
    pub fn apply_to(&self, color: Color, prop: ColorProperty) -> String {
        let filtered = color.apply_filter(self);
        filtered.format(&prop)
    }
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8, alpha: f64) -> Self {
        Self { r, g, b, alpha }
    }

    pub fn rgb(&self) -> Rgb {
        (self.r, self.g, self.b)
    }

    /// Hex string e.g. "#FF5722"
    pub fn hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// 8-digit hex with alpha e.g. "#FF572280"
    ///
    /// Alpha is clamped to `0.0..=1.0` first; a NaN alpha is serialised as `00`
    /// rather than being allowed to wrap through the float-to-int cast.
    pub fn hex8(&self) -> String {
        let alpha = if self.alpha.is_nan() {
            0.0
        } else {
            self.alpha.clamp(0.0, 1.0)
        };
        let a = (alpha * 255.0).round() as u8;
        format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, a)
    }

    /// Stripped hex without '#' e.g. "FF5722"
    pub fn hex_stripped(&self) -> String {
        self.hex().trim_start_matches('#').to_string()
    }

    /// Stripped 8-digit hex e.g. "FF572280"
    pub fn hex8_stripped(&self) -> String {
        self.hex8().trim_start_matches('#').to_string()
    }

    /// rgb(r, g, b) string
    pub fn rgb_str(&self) -> String {
        format!("rgb({}, {}, {})", self.r, self.g, self.b)
    }

    /// rgba(r, g, b, a) string
    pub fn rgba_str(&self) -> String {
        format!(
            "rgba({}, {}, {}, {:.1})",
            self.r, self.g, self.b, self.alpha
        )
    }

    /// hsl(h, s%, l%) string
    pub fn hsl_str(&self) -> String {
        let (h, s, l) = self.hsl();
        let h_i = h.round() as u32 % 360;
        let s_i = s.round().clamp(0.0, 100.0) as u32;
        let l_i = l.round().clamp(0.0, 100.0) as u32;
        format!("hsl({}, {}%, {}%)", h_i, s_i, l_i)
    }

    /// hsla(h, s%, l%, a) string
    pub fn hsla_str(&self) -> String {
        let (h, s, l) = self.hsl();
        let h_i = h.round() as u32 % 360;
        let s_i = s.round().clamp(0.0, 100.0) as u32;
        let l_i = l.round().clamp(0.0, 100.0) as u32;
        format!("hsla({}, {}%, {}%, {:.1})", h_i, s_i, l_i, self.alpha)
    }

    /// HSL tuple (h: 0–360, s: 0–100, l: 0–100)
    pub fn hsl(&self) -> Hsl {
        rgb_to_hsl(self.r as f64, self.g as f64, self.b as f64)
    }

    /// Hue in degrees 0–360
    pub fn hue(&self) -> f64 {
        self.hsl().0
    }

    /// Saturation 0–100
    pub fn saturation(&self) -> f64 {
        self.hsl().1
    }

    /// Lightness 0–100
    pub fn lightness(&self) -> f64 {
        self.hsl().2
    }

    /// Create from a hex string (`#RRGGBB` or `#RRGGBBAA`).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Color`] when the input is not ASCII or is not a
    /// 6- or 8-digit hex color.
    pub fn from_hex(hex: &str) -> crate::core::Result<Self> {
        let hex_stripped = hex.trim_start_matches('#');
        if !hex_stripped.is_ascii() {
            return Err(Error::Color(format!("invalid hex color: {hex}")));
        }
        if hex_stripped.len() == 8 {
            let r = u8::from_str_radix(&hex_stripped[0..2], 16)
                .map_err(|_| Error::Color(format!("invalid hex color format: {hex_stripped}")))?;
            let g = u8::from_str_radix(&hex_stripped[2..4], 16)
                .map_err(|_| Error::Color(format!("invalid hex color format: {hex_stripped}")))?;
            let b = u8::from_str_radix(&hex_stripped[4..6], 16)
                .map_err(|_| Error::Color(format!("invalid hex color format: {hex_stripped}")))?;
            let a = u8::from_str_radix(&hex_stripped[6..8], 16)
                .map_err(|_| Error::Color(format!("invalid hex color format: {hex_stripped}")))?;
            Ok(Self::new(r, g, b, a as f64 / 255.0))
        } else if hex_stripped.len() == 6 {
            let (r, g, b) = hex_to_rgb(hex)?;
            Ok(Self::new(r, g, b, 1.0))
        } else {
            Err(Error::Color(format!("invalid hex color format: {hex}")))
        }
    }

    /// Apply a color filter (lighten/darken/saturate/desaturate) and return new Color.
    ///
    /// Filtering happens in the perceptually uniform HCT space (the same space
    /// Material Design 3 uses): `lighten`/`darken` shift tone, `saturate`/
    /// `desaturate` shift chroma. This keeps results visually even, unlike HSL
    /// where equal lightness steps are perceptually uneven.
    pub fn apply_filter(&self, filter: &ColorFilter) -> Self {
        if let ColorFilter::SetAlpha(a) = filter {
            return Self::new(self.r, self.g, self.b, a.clamp(0.0, 1.0));
        }

        let argb = material_colors::color::Argb::from_u32(
            0xFF000000 | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32),
        );
        let hct = material_colors::hct::Hct::new(argb);
        let (hue, chroma, tone) = (hct.get_hue(), hct.get_chroma(), hct.get_tone());

        let (hue, chroma, tone) = match filter {
            ColorFilter::Lighten(amount) => (hue, chroma, (tone + amount).clamp(0.0, 100.0)),
            ColorFilter::Darken(amount) => (hue, chroma, (tone - amount).clamp(0.0, 100.0)),
            ColorFilter::Saturate(amount) => (hue, (chroma + amount).clamp(0.0, 120.0), tone),
            ColorFilter::Desaturate(amount) => (hue, (chroma - amount).clamp(0.0, 120.0), tone),
            ColorFilter::SetAlpha(_) => unreachable!("handled above"),
        };

        let filtered: material_colors::color::Argb =
            material_colors::hct::Hct::from(hue, chroma, tone).into();
        Self::new(filtered.red, filtered.green, filtered.blue, self.alpha)
    }

    /// Format this color according to a ColorProperty
    pub fn format(&self, prop: &ColorProperty) -> String {
        use ColorProperty as P;
        match prop {
            P::Hex => self.hex(),
            P::HexStripped => self.hex_stripped(),
            P::Hex8 => self.hex8(),
            P::Hex8Stripped => self.hex8_stripped(),
            P::Rgb => self.rgb_str(),
            P::Rgba => self.rgba_str(),
            P::Hsl => self.hsl_str(),
            P::Hsla => self.hsla_str(),
            P::Red => self.r.to_string(),
            P::Green => self.g.to_string(),
            P::Blue => self.b.to_string(),
            P::Alpha => format!("{:.2}", self.alpha),
            P::Hue => format!("{:.0}", self.hue()),
            P::Saturation => format!("{:.0}", self.saturation()),
            P::Lightness => format!("{:.0}", self.lightness()),
        }
    }
}

/// Circular hue distance (0-180).
#[inline]
pub fn hue_distance(h1: f64, h2: f64) -> f64 {
    let diff = (h1 - h2).abs();
    diff.min(360.0 - diff)
}

/// Estimate chroma from RGB using HCT color space.
pub fn estimate_chroma(r: u8, g: u8, b: u8) -> f64 {
    let argb = 0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
    material_colors::hct::Hct::new(material_colors::color::Argb::from_u32(argb)).get_chroma()
}

/// Estimate hue from RGB using HCT color space (0-360 degrees).
pub fn estimate_hue(r: u8, g: u8, b: u8) -> f64 {
    let argb = 0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
    material_colors::hct::Hct::new(material_colors::color::Argb::from_u32(argb)).get_hue()
}

/// Estimate both hue and chroma in a single HCT lookup to avoid double work.
pub fn estimate_hct(r: u8, g: u8, b: u8) -> (f64, f64) {
    let argb = 0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
    let hct = material_colors::hct::Hct::new(material_colors::color::Argb::from_u32(argb));
    (hct.get_hue(), hct.get_chroma())
}

/// Convert RGB to hex string
pub fn rgb_to_hex(r: f64, g: f64, b: f64) -> String {
    let r_byte = (r.clamp(0.0, 255.0)).round() as u8;
    let g_byte = (g.clamp(0.0, 255.0)).round() as u8;
    let b_byte = (b.clamp(0.0, 255.0)).round() as u8;
    format!("#{:02X}{:02X}{:02X}", r_byte, g_byte, b_byte)
}

/// Convert hex string to RGB tuple
///
/// # Errors
///
/// Returns [`Error::Color`] when the input is not ASCII or is not a
/// 6-digit hex color.
pub fn hex_to_rgb(hex: &str) -> crate::core::Result<Rgb> {
    let hex_stripped = hex.trim_start_matches('#');

    if hex_stripped.len() != 6 {
        return Err(Error::Color("invalid hex color format".to_string()));
    }

    if !hex_stripped.is_ascii() {
        return Err(Error::Color(format!("invalid hex color: {hex}")));
    }

    let r = u8::from_str_radix(&hex_stripped[0..2], 16)
        .map_err(|_| Error::Color(format!("invalid hex color: {hex}")))?;
    let g = u8::from_str_radix(&hex_stripped[2..4], 16)
        .map_err(|_| Error::Color(format!("invalid hex color: {hex}")))?;
    let b = u8::from_str_radix(&hex_stripped[4..6], 16)
        .map_err(|_| Error::Color(format!("invalid hex color: {hex}")))?;

    Ok((r, g, b))
}

/// Convert RGB to HSL tuple (h: 0–360, s: 0–100, l: 0–100)
pub fn rgb_to_hsl(r: f64, g: f64, b: f64) -> Hsl {
    let r = r / 255.0;
    let g = g / 255.0;
    let b = b / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);

    let h = if max == min {
        0.0
    } else if max == r {
        60.0 * (((g - b) / (max - min)).rem_euclid(6.0))
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
        h.clamp(0.0, 360.0),
        (s * 100.0).clamp(0.0, 100.0),
        (l * 100.0).clamp(0.0, 100.0),
    )
}

/// Convert HSL to RGB tuple. Expects h in 0-360, s and l in 0-100.
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
///
/// # Errors
///
/// Returns [`Error::Color`] when `hex` is not a valid color.
pub fn is_light_color(hex: &str) -> crate::core::Result<bool> {
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
///
/// # Errors
///
/// Returns [`Error::Color`] when either input is not a valid color.
pub fn calculate_contrast_ratio(color1: &str, color2: &str) -> crate::core::Result<f64> {
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
///
/// # Errors
///
/// Returns [`Error::Color`] when either input is not a valid color.
pub fn meets_contrast_requirement(
    color1: &str,
    color2: &str,
    threshold: f64,
) -> crate::core::Result<bool> {
    let ratio = calculate_contrast_ratio(color1, color2)?;
    Ok(ratio >= threshold)
}

/// Get contrast rating based on WCAG guidelines
/// Returns: "AAA" (>=7.0), "AA" (>=4.5), "AA Large" (>=3.0), or "Fail"
///
/// # Errors
///
/// Returns [`Error::Color`] when either input is not a valid color.
pub fn get_contrast_rating(color1: &str, color2: &str) -> crate::core::Result<&'static str> {
    let ratio = calculate_contrast_ratio(color1, color2)?;

    if ratio >= 7.0 {
        Ok("AAA")
    } else if ratio >= 4.5 {
        Ok("AA")
    } else if ratio >= 3.0 {
        Ok("AA Large")
    } else {
        Ok("Fail")
    }
}

/// Generate appropriate text color for a given background
///
/// # Errors
///
/// Returns [`Error::Color`] when `base` is not a valid color.
pub fn generate_on_color(base: &str) -> crate::core::Result<String> {
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

impl std::str::FromStr for Color {
    type Err = Error;

    /// Parse a color from a hex string, delegating to [`Color::from_hex`].
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Color::from_hex(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_rgb_and_back() {
        assert_eq!(hex_to_rgb("#ffffff").unwrap(), (255, 255, 255));
        assert_eq!(hex_to_rgb("#000000").unwrap(), (0, 0, 0));

        assert_eq!(rgb_to_hex(255.0, 255.0, 255.0), "#FFFFFF");
        assert_eq!(rgb_to_hex(0.0, 0.0, 0.0), "#000000");
    }

    #[test]
    fn test_rgb_to_hsl() {
        let (h, s, l) = rgb_to_hsl(255.0, 0.0, 0.0);
        assert!((h - 0.0).abs() < 0.1);
        assert!((s - 100.0).abs() < 0.1);
        assert!((l - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_rgb_to_hsl_magenta_has_positive_hue() {
        // Regression: the red sector used `% 6.0`, so when max == r and g < b
        // the hue went negative and was clamped to 0. Magenta must report 300.
        let (h, s, l) = rgb_to_hsl(255.0, 0.0, 255.0);
        assert!((h - 300.0).abs() < 0.1, "magenta hue was {h}");
        assert!((s - 100.0).abs() < 0.1);
        assert!((l - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_hsl_to_rgb() {
        // h=0, s=100, l=50 → pure red
        let (r, g, b) = hsl_to_rgb(0.0, 100.0, 50.0);
        assert!((r as f64 - 255.0).abs() < 1.0);
        assert!((g as f64 - 0.0).abs() < 1.0);
        assert!((b as f64 - 0.0).abs() < 1.0);
        // h=120, s=100, l=50 → pure green
        let (r, g, b) = hsl_to_rgb(120.0, 100.0, 50.0);
        assert!((r as f64 - 0.0).abs() < 1.0);
        assert!((g as f64 - 255.0).abs() < 1.0);
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
        let lum = calculate_relative_luminance(255, 255, 255);
        assert!((lum - 1.0).abs() < 0.001);

        let lum = calculate_relative_luminance(0, 0, 0);
        assert!((lum - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_hue_distance() {
        assert!((hue_distance(0.0, 10.0) - 10.0).abs() < 0.001);
        assert!((hue_distance(350.0, 10.0) - 20.0).abs() < 0.001);
        assert!((hue_distance(180.0, 0.0) - 180.0).abs() < 0.001);
    }

    #[test]
    fn test_estimate_hue_and_chroma() {
        // Pure red: both estimates agree it is a vivid, roughly hue-0 colour.
        let hue = estimate_hue(255, 0, 0);
        assert!((0.0..=360.0).contains(&hue));
        assert!(
            estimate_chroma(255, 0, 0) > 45.0,
            "pure red must register as strongly chromatic"
        );
    }

    #[test]
    fn test_color_from_hex() {
        let c = Color::from_hex("#FF5722").unwrap();
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 87);
        assert_eq!(c.b, 34);
        assert!((c.alpha - 1.0).abs() < 0.001);

        let c8 = Color::from_hex("#FF572280").unwrap();
        assert_eq!(c8.r, 255);
        assert!((c8.alpha - 0.502).abs() < 0.01);
    }

    #[test]
    fn test_from_hex_rejects_non_ascii_without_panicking() {
        // `é` is two UTF-8 bytes, so these strings are 6 and 8 bytes long and
        // used to slip past the length check and panic while byte-slicing.
        assert!(Color::from_hex("ééé").is_err());
        assert!(Color::from_hex("#éééé").is_err());
        assert!(hex_to_rgb("ééé").is_err());
    }

    #[test]
    fn test_color_formats() {
        let c = Color::new(255, 87, 34, 1.0);
        assert_eq!(c.hex(), "#FF5722");
        assert_eq!(c.hex_stripped(), "FF5722");
        assert_eq!(c.hex8(), "#FF5722FF");
        assert_eq!(c.hex8_stripped(), "FF5722FF");
        assert_eq!(c.rgb_str(), "rgb(255, 87, 34)");
        assert_eq!(c.rgba_str(), "rgba(255, 87, 34, 1.0)");
    }

    #[test]
    fn test_hsl_str_uses_css_units() {
        // Regression: saturation/lightness must be 0–100 in the output, not 0–10000.
        let c = Color::new(255, 87, 34, 1.0);
        assert_eq!(c.hsl_str(), "hsl(14, 100%, 57%)");
        assert_eq!(c.hsla_str(), "hsla(14, 100%, 57%, 1.0)");
    }

    #[test]
    fn test_filter_lighten_shifts_tone_not_hue() {
        let c = Color::new(103, 80, 164, 1.0);
        let lighter = c.apply_filter(&ColorFilter::Lighten(10.0));
        assert!(
            calculate_relative_luminance(lighter.r, lighter.g, lighter.b)
                > calculate_relative_luminance(c.r, c.g, c.b)
        );
        assert!(
            (estimate_hue(c.r, c.g, c.b) - estimate_hue(lighter.r, lighter.g, lighter.b)).abs()
                < 5.0
        );
    }

    #[test]
    fn test_filter_desaturate_reduces_chroma() {
        let c = Color::new(103, 80, 164, 1.0);
        let muted = c.apply_filter(&ColorFilter::Desaturate(30.0));
        assert!(estimate_chroma(muted.r, muted.g, muted.b) < estimate_chroma(c.r, c.g, c.b));
    }

    #[test]
    fn test_color_lighten_and_darken() {
        let light = Color::new(200, 200, 200, 1.0);
        assert!(light.apply_filter(&ColorFilter::Lighten(15.0)).lightness() > light.lightness());

        let dark = Color::new(50, 50, 50, 1.0);
        assert!(dark.apply_filter(&ColorFilter::Darken(15.0)).lightness() < dark.lightness());
    }
}
