//! Color filter system for template rendering
//!
//! Filters transform color values during template rendering.
//! Supports syntax: `{{colors.primary.default.hex|lighten:10}}`

use crate::color::Color;
pub use crate::color::{ColorFilter, ColorProperty};

/// Context for filter application
pub struct FilterContext {
    pub color: Color,
    pub format_type: ColorProperty,
}

impl FilterContext {
    pub fn new(color: Color, format_type: ColorProperty) -> Self {
        Self { color, format_type }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_color() -> Color {
        Color::new(255, 87, 34, 1.0)
    }

    fn apply(name: &str, param: &str, color: &Color, fmt: ColorProperty) -> String {
        let Some(filter) = ColorFilter::from_name(name, param) else {
            return color.format(&fmt);
        };
        if !filter.is_compatible(&fmt) {
            return color.format(&fmt);
        }
        filter.apply_to(*color, fmt)
    }

    #[test]
    fn test_unknown_filter_returns_none() {
        assert!(ColorFilter::from_name("unknown_filter", "").is_none());
    }

    #[test]
    fn test_set_alpha_filter() {
        let color = create_test_color();
        let result = apply("set_alpha", "0.5", &color, ColorProperty::Rgba);
        assert!(result.contains("0.5"));
    }

    #[test]
    fn test_lighten_filter() {
        let color = create_test_color();
        let result = apply("lighten", "10", &color, ColorProperty::Rgb);
        assert!(result.starts_with("rgb("));
    }

    #[test]
    fn test_darken_filter() {
        let color = create_test_color();
        let result = apply("darken", "10", &color, ColorProperty::Rgb);
        assert!(result.starts_with("rgb("));
    }

    #[test]
    fn test_saturate_filter() {
        let color = create_test_color();
        let result = apply("saturate", "10", &color, ColorProperty::Rgb);
        assert!(result.starts_with("rgb("));
    }

    #[test]
    fn test_desaturate_filter() {
        let color = create_test_color();
        let result = apply("desaturate", "10", &color, ColorProperty::Rgb);
        assert!(result.starts_with("rgb("));
    }

    #[test]
    fn test_set_alpha_hex_to_hex8() {
        let color = create_test_color();
        // set_alpha with hex format: alpha is applied to color but hex is still 6-digit
        let result = apply("set_alpha", "0.5", &color, ColorProperty::Hex);
        assert_eq!(result, "#FF5722");
        // Use Hex8 property to get alpha in output
        let result8 = apply("set_alpha", "0.5", &color, ColorProperty::Hex8);
        assert_eq!(result8, "#FF572280");
    }
}
