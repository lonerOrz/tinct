use crate::palette_generator::ColorFormat;

/// Enum representing different color output formats
#[derive(Debug, Clone, PartialEq)]
pub enum ColorFormatType {
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

impl ColorFormatType {
    pub fn from_property(property: &str) -> Option<Self> {
        match property {
            "hex" => Some(ColorFormatType::Hex),
            "hex_stripped" => Some(ColorFormatType::HexStripped),
            "hex8" => Some(ColorFormatType::Hex8),
            "hex8_stripped" => Some(ColorFormatType::Hex8Stripped),
            "rgb" => Some(ColorFormatType::Rgb),
            "rgba" => Some(ColorFormatType::Rgba),
            "red" => Some(ColorFormatType::Red),
            "green" => Some(ColorFormatType::Green),
            "blue" => Some(ColorFormatType::Blue),
            "alpha" => Some(ColorFormatType::Alpha),
            "hsl" => Some(ColorFormatType::Hsl),
            "hsla" => Some(ColorFormatType::Hsla),
            "hue" => Some(ColorFormatType::Hue),
            "saturation" => Some(ColorFormatType::Saturation),
            "lightness" => Some(ColorFormatType::Lightness),
            _ => None,
        }
    }

    /// Check if this format represents a complete color (vs a single channel)
    pub fn is_complete_color(&self) -> bool {
        matches!(self,
            ColorFormatType::Hex |
            ColorFormatType::HexStripped |
            ColorFormatType::Hex8 |
            ColorFormatType::Hex8Stripped |
            ColorFormatType::Rgb |
            ColorFormatType::Rgba |
            ColorFormatType::Hsl |
            ColorFormatType::Hsla
        )
    }

    /// Check if this format represents a color channel (single component)
    pub fn is_channel(&self) -> bool {
        matches!(self,
            ColorFormatType::Red |
            ColorFormatType::Green |
            ColorFormatType::Blue |
            ColorFormatType::Alpha |
            ColorFormatType::Hue |
            ColorFormatType::Saturation |
            ColorFormatType::Lightness
        )
    }
}

/// Context for filter application
pub struct FilterContext {
    pub original_value: String,
    pub format_type: ColorFormatType,
    pub color_format: ColorFormat,
}

/// Trait for defining filters
pub trait Filter {
    fn apply(&self, ctx: &FilterContext, param: Option<&str>) -> String;

    /// Check if this filter is compatible with the given format type
    fn is_compatible(&self, _format_type: &ColorFormatType) -> bool {
        // By default, all filters are compatible with all formats
        true
    }
}

/// Registry for managing filters
pub struct FilterRegistry {
    filters: std::collections::HashMap<String, Box<dyn Filter>>,
}

impl Default for FilterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FilterRegistry {
    pub fn new() -> Self {
        let mut registry = FilterRegistry {
            filters: std::collections::HashMap::new(),
        };

        // Register default filters
        registry.register("set_alpha", Box::new(SetAlphaFilter));
        registry.register("lighten", Box::new(LightenFilter));
        registry.register("darken", Box::new(DarkenFilter));
        registry.register("saturate", Box::new(SaturateFilter));
        registry.register("desaturate", Box::new(DesaturateFilter));

        registry
    }

    pub fn register(&mut self, name: &str, filter: Box<dyn Filter>) {
        self.filters.insert(name.to_string(), filter);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Filter> {
        self.filters.get(name).map(|boxed| boxed.as_ref())
    }

    pub fn apply_filter(
        &self,
        value: &str,
        name: &str,
        param: Option<&str>,
        color_format: &ColorFormat,
        format_type: ColorFormatType,
    ) -> String {
        if let Some(filter) = self.get(name) {
            if filter.is_compatible(&format_type) {
                let ctx = FilterContext {
                    original_value: value.to_string(),
                    format_type,
                    color_format: color_format.clone(),
                };
                filter.apply(&ctx, param)
            } else {
                // Return original value if filter is not compatible with format
                value.to_string()
            }
        } else {
            // Return original value if filter not found
            value.to_string()
        }
    }
}

/// SetAlpha filter implementation
pub struct SetAlphaFilter;
impl Filter for SetAlphaFilter {
    fn apply(&self, ctx: &FilterContext, param: Option<&str>) -> String {
        if let Some(alpha_param) = param {
            // Parse the alpha parameter (should be a float like 0.5)
            let alpha_val = if let Ok(f) = alpha_param.parse::<f64>() {
                // Clamp the value to 0.0-1.0 range
                f.clamp(0.0, 1.0)
            } else {
                // If parsing fails, return the original value
                return ctx.original_value.clone();
            };

            // Modify the alpha value based on the original format
            match ctx.format_type {
                ColorFormatType::Rgba => {
                    // Original was rgba, modify the alpha
                    if ctx.original_value.starts_with("rgba(") {
                        if let Some(rgb_part) = ctx
                            .original_value
                            .strip_prefix("rgba(")
                            .unwrap_or(&ctx.original_value)
                            .strip_suffix(')')
                        {
                            let parts: Vec<&str> = rgb_part.split(',').map(|s| s.trim()).collect();
                            if parts.len() >= 4 {
                                return format!(
                                    "rgba({}, {}, {}, {:.1})",
                                    parts[0].trim(),
                                    parts[1].trim(),
                                    parts[2].trim(),
                                    alpha_val
                                );
                            }
                        }
                    }
                    // Fallback to rgba with the current color
                    format!(
                        "rgba({}, {}, {}, {:.1})",
                        ctx.color_format.red,
                        ctx.color_format.green,
                        ctx.color_format.blue,
                        alpha_val
                    )
                }
                ColorFormatType::Hsla => {
                    // Original was hsla, modify the alpha
                    // Use the original HSL values stored in the color format to maintain consistency
                    let h = ctx.color_format.original_hue.unwrap_or(ctx.color_format.hue.round() as u32);
                    let s = ctx.color_format.original_saturation.unwrap_or(ctx.color_format.saturation.round() as u32);
                    let l = ctx.color_format.original_lightness.unwrap_or(ctx.color_format.lightness.round() as u32);

                    format!(
                        "hsla({}, {}%, {}%, {:.1})",
                        h, s, l, alpha_val
                    )
                }
                ColorFormatType::Rgb => {
                    // Original was rgb, convert to rgba with new alpha
                    format!(
                        "rgba({}, {}, {}, {:.1})",
                        ctx.color_format.red,
                        ctx.color_format.green,
                        ctx.color_format.blue,
                        alpha_val
                    )
                }
                ColorFormatType::Hsl => {
                    // Original was hsl, convert to hsla with new alpha
                    // Use the original HSL values stored in the color format to maintain consistency
                    let h = ctx.color_format.original_hue.unwrap_or(ctx.color_format.hue.round() as u32);
                    let s = ctx.color_format.original_saturation.unwrap_or(ctx.color_format.saturation.round() as u32);
                    let l = ctx.color_format.original_lightness.unwrap_or(ctx.color_format.lightness.round() as u32);

                    format!(
                        "hsla({}, {}%, {}%, {:.1})",
                        h, s, l, alpha_val
                    )
                }
                ColorFormatType::Hex => {
                    // For hex format, convert to 8-digit hex with alpha (hex8 format)
                    let alpha_byte = (alpha_val * 255.0).round() as u8;
                    format!("#{:02x}{:02x}{:02x}{:02x}",
                           ctx.color_format.red,
                           ctx.color_format.green,
                           ctx.color_format.blue,
                           alpha_byte)
                }
                ColorFormatType::HexStripped => {
                    // For hex_stripped format, convert to 8-digit hex without # prefix
                    let alpha_byte = (alpha_val * 255.0).round() as u8;
                    format!("{:02x}{:02x}{:02x}{:02x}",
                           ctx.color_format.red,
                           ctx.color_format.green,
                           ctx.color_format.blue,
                           alpha_byte)
                }
                ColorFormatType::Hex8 => {
                    // For hex8 format, modify the alpha part
                    let alpha_byte = (alpha_val * 255.0).round() as u8;
                    format!("#{:02x}{:02x}{:02x}{:02x}",
                           ctx.color_format.red,
                           ctx.color_format.green,
                           ctx.color_format.blue,
                           alpha_byte)
                }
                ColorFormatType::Hex8Stripped => {
                    // For hex8_stripped format, modify the alpha part
                    let alpha_byte = (alpha_val * 255.0).round() as u8;
                    format!("{:02x}{:02x}{:02x}{:02x}",
                           ctx.color_format.red,
                           ctx.color_format.green,
                           ctx.color_format.blue,
                           alpha_byte)
                }
                _ => {
                    // For other formats, return rgba
                    format!(
                        "rgba({}, {}, {}, {:.1})",
                        ctx.color_format.red,
                        ctx.color_format.green,
                        ctx.color_format.blue,
                        alpha_val
                    )
                }
            }
        } else {
            ctx.original_value.clone()
        }
    }

    fn is_compatible(&self, format_type: &ColorFormatType) -> bool {
        // set_alpha should only be applied to complete color formats, not individual channels
        format_type.is_complete_color()
    }
}

/// Lighten filter implementation
pub struct LightenFilter;
impl Filter for LightenFilter {
    fn apply(&self, ctx: &FilterContext, param: Option<&str>) -> String {
        if let Some(amount) = param {
            if let Ok(percent) = amount.parse::<f64>() {
                // Calculate lighter version of the color
                let hsl = crate::color::rgb_to_hsl(
                    ctx.color_format.red as f64,
                    ctx.color_format.green as f64,
                    ctx.color_format.blue as f64,
                );

                let new_lightness = crate::color::clamp(hsl.l + percent, 0.0, 100.0);
                let new_rgb = crate::color::hsl_to_rgb(hsl.h, hsl.s, new_lightness);

                match ctx.format_type {
                    ColorFormatType::Rgb => {
                        format!("rgb({}, {}, {})", new_rgb.r, new_rgb.g, new_rgb.b)
                    }
                    ColorFormatType::Rgba => {
                        format!("rgba({}, {}, {}, {:.1})", new_rgb.r, new_rgb.g, new_rgb.b, ctx.color_format.alpha)
                    } // Keep alpha value but update RGB
                    ColorFormatType::Hsl => {
                        // Use original hue and saturation values for consistency, but updated lightness
                        let h = ctx.color_format.original_hue.unwrap_or(hsl.h.round() as u32);
                        let s = ctx.color_format.original_saturation.unwrap_or(hsl.s.round() as u32);
                        let l = new_lightness.round() as u32;

                        format!("hsl({}, {}%, {}%)", h, s, l)
                    }
                    ColorFormatType::Hsla => format!(
                        "hsla({}, {}%, {}%, {:.1})",
                        ctx.color_format.original_hue.unwrap_or(hsl.h.round() as u32),  // Use original hue for consistency
                        ctx.color_format.original_saturation.unwrap_or(hsl.s.round() as u32),  // Use original saturation for consistency
                        new_lightness.round() as u32,  // Updated lightness
                        ctx.color_format.alpha
                    ),
                    ColorFormatType::Hex => crate::color::rgb_to_hex(
                        new_rgb.r as f64,
                        new_rgb.g as f64,
                        new_rgb.b as f64,
                    ),
                    ColorFormatType::HexStripped => {
                        // Return hex without the '#' prefix
                        let hex = crate::color::rgb_to_hex(
                            new_rgb.r as f64,
                            new_rgb.g as f64,
                            new_rgb.b as f64,
                        );
                        hex.strip_prefix('#').unwrap_or(&hex).to_string()
                    },
                    ColorFormatType::Hex8 => {
                        // Convert to hex8 format with alpha preserved
                        let alpha_byte = (ctx.color_format.alpha * 255.0).round() as u8;
                        format!("#{:02x}{:02x}{:02x}{:02x}",
                               new_rgb.r, new_rgb.g, new_rgb.b, alpha_byte)
                    },
                    ColorFormatType::Hex8Stripped => {
                        // Convert to hex8 format without # prefix
                        let alpha_byte = (ctx.color_format.alpha * 255.0).round() as u8;
                        format!("{:02x}{:02x}{:02x}{:02x}",
                               new_rgb.r, new_rgb.g, new_rgb.b, alpha_byte)
                    },
                    _ => ctx.original_value.clone(),
                }
            } else {
                ctx.original_value.clone()
            }
        } else {
            ctx.original_value.clone()
        }
    }

    fn is_compatible(&self, format_type: &ColorFormatType) -> bool {
        // lighten should only be applied to complete color formats, not individual channels
        format_type.is_complete_color()
    }
}

/// Darken filter implementation
pub struct DarkenFilter;
impl Filter for DarkenFilter {
    fn apply(&self, ctx: &FilterContext, param: Option<&str>) -> String {
        if let Some(amount) = param {
            if let Ok(percent) = amount.parse::<f64>() {
                // Calculate darker version of the color
                let hsl = crate::color::rgb_to_hsl(
                    ctx.color_format.red as f64,
                    ctx.color_format.green as f64,
                    ctx.color_format.blue as f64,
                );

                let new_lightness = crate::color::clamp(hsl.l - percent, 0.0, 100.0);
                let new_rgb = crate::color::hsl_to_rgb(hsl.h, hsl.s, new_lightness);

                match ctx.format_type {
                    ColorFormatType::Rgb => {
                        format!("rgb({}, {}, {})", new_rgb.r, new_rgb.g, new_rgb.b)
                    }
                    ColorFormatType::Rgba => {
                        format!("rgba({}, {}, {}, {:.1})", new_rgb.r, new_rgb.g, new_rgb.b, ctx.color_format.alpha)
                    } // Keep alpha value but update RGB
                    ColorFormatType::Hsl => {
                        // Use original hue and saturation values for consistency, but updated lightness
                        let h = ctx.color_format.original_hue.unwrap_or(hsl.h.round() as u32);
                        let s = ctx.color_format.original_saturation.unwrap_or(hsl.s.round() as u32);
                        let l = new_lightness.round() as u32;

                        format!("hsl({}, {}%, {}%)", h, s, l)
                    }
                    ColorFormatType::Hsla => format!(
                        "hsla({}, {}%, {}%, {:.1})",
                        ctx.color_format.original_hue.unwrap_or(hsl.h.round() as u32),  // Use original hue for consistency
                        ctx.color_format.original_saturation.unwrap_or(hsl.s.round() as u32),  // Use original saturation for consistency
                        new_lightness.round() as u32,  // Updated lightness
                        ctx.color_format.alpha
                    ),
                    ColorFormatType::Hex => crate::color::rgb_to_hex(
                        new_rgb.r as f64,
                        new_rgb.g as f64,
                        new_rgb.b as f64,
                    ),
                    ColorFormatType::HexStripped => {
                        // Return hex without the '#' prefix
                        let hex = crate::color::rgb_to_hex(
                            new_rgb.r as f64,
                            new_rgb.g as f64,
                            new_rgb.b as f64,
                        );
                        hex.strip_prefix('#').unwrap_or(&hex).to_string()
                    },
                    ColorFormatType::Hex8 => {
                        // Convert to hex8 format with alpha preserved
                        let alpha_byte = (ctx.color_format.alpha * 255.0).round() as u8;
                        format!("#{:02x}{:02x}{:02x}{:02x}",
                               new_rgb.r, new_rgb.g, new_rgb.b, alpha_byte)
                    },
                    ColorFormatType::Hex8Stripped => {
                        // Convert to hex8 format without # prefix
                        let alpha_byte = (ctx.color_format.alpha * 255.0).round() as u8;
                        format!("{:02x}{:02x}{:02x}{:02x}",
                               new_rgb.r, new_rgb.g, new_rgb.b, alpha_byte)
                    },
                    _ => ctx.original_value.clone(),
                }
            } else {
                ctx.original_value.clone()
            }
        } else {
            ctx.original_value.clone()
        }
    }

    fn is_compatible(&self, format_type: &ColorFormatType) -> bool {
        // darken should only be applied to complete color formats, not individual channels
        format_type.is_complete_color()
    }
}

/// Saturate filter implementation
pub struct SaturateFilter;
impl Filter for SaturateFilter {
    fn apply(&self, ctx: &FilterContext, param: Option<&str>) -> String {
        if let Some(amount) = param {
            if let Ok(percent) = amount.parse::<f64>() {
                // Calculate more saturated version of the color
                let hsl = crate::color::rgb_to_hsl(
                    ctx.color_format.red as f64,
                    ctx.color_format.green as f64,
                    ctx.color_format.blue as f64,
                );

                let new_saturation = crate::color::clamp(hsl.s + percent, 0.0, 100.0);
                let new_rgb = crate::color::hsl_to_rgb(hsl.h, new_saturation, hsl.l);

                match ctx.format_type {
                    ColorFormatType::Rgb => {
                        format!("rgb({}, {}, {})", new_rgb.r, new_rgb.g, new_rgb.b)
                    }
                    ColorFormatType::Rgba => {
                        format!("rgba({}, {}, {}, {:.1})", new_rgb.r, new_rgb.g, new_rgb.b, ctx.color_format.alpha)
                    } // Keep alpha value but update RGB
                    ColorFormatType::Hsl => {
                        // Use original hue and lightness values for consistency, but updated saturation
                        let h = ctx.color_format.original_hue.unwrap_or(hsl.h.round() as u32);
                        let s = new_saturation.round() as u32;
                        let l = ctx.color_format.original_lightness.unwrap_or(hsl.l.round() as u32);

                        format!("hsl({}, {}%, {}%)", h, s, l)
                    }
                    ColorFormatType::Hsla => format!(
                        "hsla({}, {}%, {}%, {:.1})",
                        ctx.color_format.original_hue.unwrap_or(hsl.h.round() as u32),  // Use original hue for consistency
                        new_saturation.round() as u32,  // Updated saturation
                        ctx.color_format.original_lightness.unwrap_or(hsl.l.round() as u32),  // Use original lightness for consistency
                        ctx.color_format.alpha
                    ),
                    ColorFormatType::Hex => crate::color::rgb_to_hex(
                        new_rgb.r as f64,
                        new_rgb.g as f64,
                        new_rgb.b as f64,
                    ),
                    ColorFormatType::HexStripped => {
                        // Return hex without the '#' prefix
                        let hex = crate::color::rgb_to_hex(
                            new_rgb.r as f64,
                            new_rgb.g as f64,
                            new_rgb.b as f64,
                        );
                        hex.strip_prefix('#').unwrap_or(&hex).to_string()
                    },
                    ColorFormatType::Hex8 => {
                        // Convert to hex8 format with alpha preserved
                        let alpha_byte = (ctx.color_format.alpha * 255.0).round() as u8;
                        format!("#{:02x}{:02x}{:02x}{:02x}",
                               new_rgb.r, new_rgb.g, new_rgb.b, alpha_byte)
                    },
                    ColorFormatType::Hex8Stripped => {
                        // Convert to hex8 format without # prefix
                        let alpha_byte = (ctx.color_format.alpha * 255.0).round() as u8;
                        format!("{:02x}{:02x}{:02x}{:02x}",
                               new_rgb.r, new_rgb.g, new_rgb.b, alpha_byte)
                    },
                    _ => ctx.original_value.clone(),
                }
            } else {
                ctx.original_value.clone()
            }
        } else {
            ctx.original_value.clone()
        }
    }

    fn is_compatible(&self, format_type: &ColorFormatType) -> bool {
        // saturate should only be applied to complete color formats, not individual channels
        format_type.is_complete_color()
    }
}

/// Desaturate filter implementation
pub struct DesaturateFilter;
impl Filter for DesaturateFilter {
    fn apply(&self, ctx: &FilterContext, param: Option<&str>) -> String {
        if let Some(amount) = param {
            if let Ok(percent) = amount.parse::<f64>() {
                // Calculate less saturated version of the color
                let hsl = crate::color::rgb_to_hsl(
                    ctx.color_format.red as f64,
                    ctx.color_format.green as f64,
                    ctx.color_format.blue as f64,
                );

                let new_saturation = crate::color::clamp(hsl.s - percent, 0.0, 100.0);
                let new_rgb = crate::color::hsl_to_rgb(hsl.h, new_saturation, hsl.l);

                match ctx.format_type {
                    ColorFormatType::Rgb => {
                        format!("rgb({}, {}, {})", new_rgb.r, new_rgb.g, new_rgb.b)
                    }
                    ColorFormatType::Rgba => {
                        format!("rgba({}, {}, {}, {:.1})", new_rgb.r, new_rgb.g, new_rgb.b, ctx.color_format.alpha)
                    } // Keep alpha value but update RGB
                    ColorFormatType::Hsl => {
                        // Use original hue and lightness values for consistency, but updated saturation
                        let h = ctx.color_format.original_hue.unwrap_or(hsl.h.round() as u32);
                        let s = new_saturation.round() as u32;
                        let l = ctx.color_format.original_lightness.unwrap_or(hsl.l.round() as u32);

                        format!("hsl({}, {}%, {}%)", h, s, l)
                    }
                    ColorFormatType::Hsla => format!(
                        "hsla({}, {}%, {}%, {:.1})",
                        ctx.color_format.original_hue.unwrap_or(hsl.h.round() as u32),  // Use original hue for consistency
                        new_saturation.round() as u32,  // Updated saturation
                        ctx.color_format.original_lightness.unwrap_or(hsl.l.round() as u32),  // Use original lightness for consistency
                        ctx.color_format.alpha
                    ),
                    ColorFormatType::Hex => crate::color::rgb_to_hex(
                        new_rgb.r as f64,
                        new_rgb.g as f64,
                        new_rgb.b as f64,
                    ),
                    ColorFormatType::HexStripped => {
                        // Return hex without the '#' prefix
                        let hex = crate::color::rgb_to_hex(
                            new_rgb.r as f64,
                            new_rgb.g as f64,
                            new_rgb.b as f64,
                        );
                        hex.strip_prefix('#').unwrap_or(&hex).to_string()
                    },
                    ColorFormatType::Hex8 => {
                        // Convert to hex8 format with alpha preserved
                        let alpha_byte = (ctx.color_format.alpha * 255.0).round() as u8;
                        format!("#{:02x}{:02x}{:02x}{:02x}",
                               new_rgb.r, new_rgb.g, new_rgb.b, alpha_byte)
                    },
                    ColorFormatType::Hex8Stripped => {
                        // Convert to hex8 format without # prefix
                        let alpha_byte = (ctx.color_format.alpha * 255.0).round() as u8;
                        format!("{:02x}{:02x}{:02x}{:02x}",
                               new_rgb.r, new_rgb.g, new_rgb.b, alpha_byte)
                    },
                    _ => ctx.original_value.clone(),
                }
            } else {
                ctx.original_value.clone()
            }
        } else {
            ctx.original_value.clone()
        }
    }

    fn is_compatible(&self, format_type: &ColorFormatType) -> bool {
        // desaturate should only be applied to complete color formats, not individual channels
        format_type.is_complete_color()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette_generator::ColorFormat;

    #[test]
    fn test_set_alpha_filter() {
        let filter = SetAlphaFilter;
        let color_format = ColorFormat {
            hex: "#FF5722".to_string(),
            hex_stripped: "FF5722".to_string(),
            hex8: "#FF5722FF".to_string(),  // Assuming full opacity
            hex8_stripped: "FF5722FF".to_string(), // Assuming full opacity
            rgb: "rgb(255, 87, 34)".to_string(),
            rgba: "rgba(255, 87, 34, 255)".to_string(),
            hsl: "hsl(14, 100%, 57%)".to_string(),
            hsla: "hsla(14, 100%, 57%, 1.0)".to_string(),
            red: 255,
            green: 87,
            blue: 34,
            alpha: 1.0,
            hue: 14.0,
            saturation: 100.0,
            lightness: 57.0,
            original_hue: Some(14),
            original_saturation: Some(100),
            original_lightness: Some(57),
        };

        // Test with rgba value
        let ctx = FilterContext {
            original_value: "rgba(255, 87, 34, 255)".to_string(),
            format_type: ColorFormatType::Rgba,
            color_format: color_format.clone(),
        };
        let result = filter.apply(&ctx, Some("0.5"));
        assert!(result.contains("rgba(255, 87, 34, 0.")); // Check for approximate match due to float precision

        // Test with rgb value
        let ctx = FilterContext {
            original_value: "rgb(255, 87, 34)".to_string(),
            format_type: ColorFormatType::Rgb,
            color_format: color_format.clone(),
        };
        let result = filter.apply(&ctx, Some("0.5"));
        assert!(result.contains("rgba(255, 87, 34, 0.")); // Check for approximate match due to float precision

        // Test with hex value
        let ctx = FilterContext {
            original_value: "#FF5722".to_string(),
            format_type: ColorFormatType::Hex,
            color_format: color_format.clone(),
        };
        let result = filter.apply(&ctx, Some("0.3"));
        assert!(result.contains("#ff5722")); // Hex8 format with alpha

        // Test with hex_stripped value
        let ctx = FilterContext {
            original_value: "FF5722".to_string(),
            format_type: ColorFormatType::HexStripped,
            color_format: color_format.clone(),
        };
        let result = filter.apply(&ctx, Some("0.7"));
        assert!(result.contains("ff5722")); // Hex8 stripped format with alpha
    }

    #[test]
    fn test_invalid_alpha_values() {
        let filter = SetAlphaFilter;
        let color_format = ColorFormat {
            hex: "#FF5722".to_string(),
            hex_stripped: "FF5722".to_string(),
            hex8: "#FF5722FF".to_string(),
            hex8_stripped: "FF5722FF".to_string(),
            rgb: "rgb(255, 87, 34)".to_string(),
            rgba: "rgba(255, 87, 34, 255)".to_string(),
            hsl: "hsl(14, 100%, 57%)".to_string(),
            hsla: "hsla(14, 100%, 57%, 1.0)".to_string(),
            red: 255,
            green: 87,
            blue: 34,
            alpha: 1.0,
            hue: 14.0,
            saturation: 100.0,
            lightness: 57.0,
            original_hue: Some(14),
            original_saturation: Some(100),
            original_lightness: Some(57),
        };

        // Test with invalid alpha value (should return original value)
        let ctx = FilterContext {
            original_value: "rgba(255, 87, 34, 255)".to_string(),
            format_type: ColorFormatType::Rgba,
            color_format: color_format.clone(),
        };
        let result = filter.apply(&ctx, Some("invalid"));
        assert_eq!(result, "rgba(255, 87, 34, 255)"); // Should return original value

        // Test with out-of-range alpha value (should clamp to valid range)
        let ctx = FilterContext {
            original_value: "rgb(255, 87, 34)".to_string(),
            format_type: ColorFormatType::Rgb,
            color_format: color_format.clone(),
        };
        let result = filter.apply(&ctx, Some("1.5")); // Out of range (> 1.0)
        assert!(result.contains("rgba(255, 87, 34, 1.0")); // Should clamp to 1.0

        let ctx = FilterContext {
            original_value: "rgb(255, 87, 34)".to_string(),
            format_type: ColorFormatType::Rgb,
            color_format: color_format.clone(),
        };
        let result = filter.apply(&ctx, Some("-0.5")); // Out of range (< 0.0)
        assert!(result.contains("rgba(255, 87, 34, 0.0")); // Should clamp to 0.0
    }

    #[test]
    fn test_channel_formats_not_filtered() {
        let registry = FilterRegistry::new();
        let color_format = ColorFormat {
            hex: "#FF5722".to_string(),
            hex_stripped: "FF5722".to_string(),
            hex8: "#FF5722FF".to_string(),
            hex8_stripped: "FF5722FF".to_string(),
            rgb: "rgb(255, 87, 34)".to_string(),
            rgba: "rgba(255, 87, 34, 255)".to_string(),
            hsl: "hsl(14, 100%, 57%)".to_string(),
            hsla: "hsla(14, 100%, 57%, 1.0)".to_string(),
            red: 255,
            green: 87,
            blue: 34,
            alpha: 1.0,
            hue: 14.0,
            saturation: 100.0,
            lightness: 57.0,
            original_hue: Some(14),
            original_saturation: Some(100),
            original_lightness: Some(57),
        };

        // Test that channel formats (like red) are not affected by color filters
        let result = registry.apply_filter("255", "set_alpha", Some("0.5"), &color_format, ColorFormatType::Red);
        assert_eq!(result, "255"); // Should return original value since Red is a channel, not a color format

        let result = registry.apply_filter("14", "lighten", Some("10"), &color_format, ColorFormatType::Hue);
        assert_eq!(result, "14"); // Should return original value since Hue is a channel, not a color format
    }

    #[test]
    fn test_registry() {
        let registry = FilterRegistry::new();

        // Test that default filters are registered
        assert!(registry.get("set_alpha").is_some());
        assert!(registry.get("lighten").is_some());
        assert!(registry.get("darken").is_some());
        assert!(registry.get("saturate").is_some());
        assert!(registry.get("desaturate").is_some());

        // Test applying a filter
        let color_format = ColorFormat {
            hex: "#FF5722".to_string(),
            hex_stripped: "FF5722".to_string(),
            hex8: "#FF5722FF".to_string(),  // Assuming full opacity
            hex8_stripped: "FF5722FF".to_string(), // Assuming full opacity
            rgb: "rgb(255, 87, 34)".to_string(),
            rgba: "rgba(255, 87, 34, 255)".to_string(),
            hsl: "hsl(14, 100%, 57%)".to_string(),
            hsla: "hsla(14, 100%, 57%, 1.0)".to_string(),
            red: 255,
            green: 87,
            blue: 34,
            alpha: 1.0,
            hue: 14.0,
            saturation: 100.0,
            lightness: 57.0,
            original_hue: Some(14),
            original_saturation: Some(100),
            original_lightness: Some(57),
        };

        let result = registry.apply_filter(
            "rgba(255, 87, 34, 255)",
            "set_alpha",
            Some("0.5"),
            &color_format,
            ColorFormatType::Rgba,
        );
        assert!(result.contains("rgba(255, 87, 34, 0.")); // Check for approximate match due to float precision
    }
}
