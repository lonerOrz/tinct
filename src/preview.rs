//! Color preview functionality
//!
//! Displays Material Design 3 color palettes in the terminal with actual color blocks.

use crate::core::{Mode, ThemeLoader};
use crate::palette::{AlgorithmParameters, LegacyPaletteGenerator};
use crate::theme::JsonThemeLoader;
use colored::*;
use std::sync::Arc;

/// Display a color preview showing all available colors in the theme as a matrix
pub fn show_color_preview(theme_path: &str, mode: &str) -> Result<(), String> {
    // Create palette generator
    let palette_gen = Arc::new(LegacyPaletteGenerator::new(AlgorithmParameters::default()));

    // Load theme
    let theme_loader = JsonThemeLoader::new(palette_gen.clone());
    let theme = theme_loader.load(theme_path).map_err(|e| e.to_string())?;

    // Determine mode and get colors
    let mode = if mode == "dark" {
        Mode::Dark
    } else {
        Mode::Light
    };
    let colors = match mode {
        Mode::Dark => &theme.dark_colors,
        Mode::Light => &theme.light_colors,
    };

    println!(
        "{}",
        "🎨 Material Design 3 Color Preview".bold().underline()
    );
    println!("🌙 Theme Mode: {}", mode.to_string().bold());
    println!();

    // Display colors in MD3 style with actual color blocks
    display_md3_cards_grid(colors);

    Ok(())
}

/// Display a color preview from a JSON value (for seed-based preview)
pub fn show_color_preview_from_json(json: &serde_json::Value, mode: &str) -> Result<(), String> {
    // Create palette generator
    let palette_gen = Arc::new(LegacyPaletteGenerator::new(AlgorithmParameters::default()));

    // Load theme from JSON value
    let theme_loader = JsonThemeLoader::new(palette_gen.clone());
    let theme = theme_loader.load_value(json).map_err(|e| e.to_string())?;

    // Determine mode and get colors
    let mode = if mode == "dark" {
        Mode::Dark
    } else {
        Mode::Light
    };
    let colors = match mode {
        Mode::Dark => &theme.dark_colors,
        Mode::Light => &theme.light_colors,
    };

    println!(
        "{}",
        "🎨 Material Design 3 Color Preview".bold().underline()
    );
    println!("🌙 Theme Mode: {}", mode.to_string().bold());
    println!();

    // Display colors in MD3 style with actual color blocks
    display_md3_cards_grid(colors);

    Ok(())
}

/// Display colors in a card grid layout with true color blocks
fn display_md3_cards_grid(colors: &std::collections::HashMap<String, crate::core::ColorFormat>) {
    // Define color cards based on the MD3 documentation structure
    let cards: Vec<Vec<(&str, &crate::core::ColorFormat)>> = vec![
        // Primary card
        vec![
            ("Primary", colors.get("primary").unwrap()),
            ("On Primary", colors.get("on_primary").unwrap()),
            (
                "Primary Container",
                colors.get("primary_container").unwrap(),
            ),
            (
                "On Primary Container",
                colors.get("on_primary_container").unwrap(),
            ),
        ],
        // Secondary card
        vec![
            ("Secondary", colors.get("secondary").unwrap()),
            ("On Secondary", colors.get("on_secondary").unwrap()),
            (
                "Secondary Container",
                colors.get("secondary_container").unwrap(),
            ),
            (
                "On Secondary Container",
                colors.get("on_secondary_container").unwrap(),
            ),
        ],
        // Tertiary card
        vec![
            ("Tertiary", colors.get("tertiary").unwrap()),
            ("On Tertiary", colors.get("on_tertiary").unwrap()),
            (
                "Tertiary Container",
                colors.get("tertiary_container").unwrap(),
            ),
            (
                "On Tertiary Container",
                colors.get("on_tertiary_container").unwrap(),
            ),
        ],
        // Error card
        vec![
            ("Error", colors.get("error").unwrap()),
            ("On Error", colors.get("on_error").unwrap()),
            ("Error Container", colors.get("error_container").unwrap()),
            (
                "On Error Container",
                colors.get("on_error_container").unwrap(),
            ),
        ],
        // Fixed Accent Cards
        vec![
            ("Primary Fixed", colors.get("primary_fixed").unwrap()),
            (
                "Primary Fixed Dim",
                colors.get("primary_fixed_dim").unwrap(),
            ),
            ("On Primary Fixed", colors.get("on_primary_fixed").unwrap()),
            (
                "On Primary Fixed Var",
                colors.get("on_primary_fixed_variant").unwrap(),
            ),
        ],
        vec![
            ("Secondary Fixed", colors.get("secondary_fixed").unwrap()),
            (
                "Secondary Fixed Dim",
                colors.get("secondary_fixed_dim").unwrap(),
            ),
            (
                "On Secondary Fixed",
                colors.get("on_secondary_fixed").unwrap(),
            ),
            (
                "On Secondary Fixed Var",
                colors.get("on_secondary_fixed_variant").unwrap(),
            ),
        ],
        vec![
            ("Tertiary Fixed", colors.get("tertiary_fixed").unwrap()),
            (
                "Tertiary Fixed Dim",
                colors.get("tertiary_fixed_dim").unwrap(),
            ),
            (
                "On Tertiary Fixed",
                colors.get("on_tertiary_fixed").unwrap(),
            ),
            (
                "On Tertiary Fixed Var",
                colors.get("on_tertiary_fixed_variant").unwrap(),
            ),
        ],
        // Surface card
        vec![
            ("Surface Dim", colors.get("surface_dim").unwrap()),
            ("Surface", colors.get("surface").unwrap()),
            ("Surface Bright", colors.get("surface_bright").unwrap()),
        ],
        // Surface Variant card
        vec![
            ("Surface Variant", colors.get("surface_variant").unwrap()),
            (
                "On Surface Variant",
                colors.get("on_surface_variant").unwrap(),
            ),
        ],
        // Surface Containers card
        vec![
            (
                "Container Lowest",
                colors.get("surface_container_lowest").unwrap(),
            ),
            (
                "Container Low",
                colors.get("surface_container_low").unwrap(),
            ),
            ("Container", colors.get("surface_container").unwrap()),
            (
                "Container High",
                colors.get("surface_container_high").unwrap(),
            ),
            (
                "Container Highest",
                colors.get("surface_container_highest").unwrap(),
            ),
        ],
        // Background card
        vec![
            ("Background", colors.get("background").unwrap()),
            ("On Background", colors.get("on_background").unwrap()),
        ],
        // Outline card
        vec![
            ("Outline", colors.get("outline").unwrap()),
            ("Outline Variant", colors.get("outline_variant").unwrap()),
        ],
        // Inverse card
        vec![
            ("Inverse Surface", colors.get("inverse_surface").unwrap()),
            (
                "Inverse On Surface",
                colors.get("inverse_on_surface").unwrap(),
            ),
            ("Inverse Primary", colors.get("inverse_primary").unwrap()),
        ],
        // Special card
        vec![
            ("Shadow", colors.get("shadow").unwrap()),
            ("Scrim", colors.get("scrim").unwrap()),
        ],
    ];

    // Print cards in groups of 3 per row
    const CARDS_PER_ROW: usize = 3;

    for chunk in cards.chunks(CARDS_PER_ROW) {
        // Find the max number of colors in any card in this row
        let max_colors = chunk.iter().map(|card| card.len()).max().unwrap_or(0);

        // Print each color row across all cards in the chunk
        for color_idx in 0..max_colors {
            // For each color, we'll print 3 lines to simulate height
            for line_num in 0..3 {
                // Print 3 lines to make blocks taller
                for (idx, card) in chunk.iter().enumerate() {
                    if color_idx < card.len() {
                        let (label, color) = &card[color_idx];

                        // Create a color block with centered text
                        let block_width = 24;

                        // For the middle line, show the text; for others, show empty color blocks
                        let display_content = if line_num == 1 {
                            // Middle line shows text
                            let text_len = label.len();

                            // If text is too long, handle it by truncating with ellipsis
                            let display_text = if text_len > block_width {
                                // Truncate text and add ellipsis
                                let mut truncated = String::new();
                                let chars: Vec<char> = label.chars().collect();
                                for i in 0..(block_width - 3) {
                                    if i < chars.len() {
                                        truncated.push(chars[i]);
                                    }
                                }
                                truncated.push_str("...");
                                truncated
                            } else {
                                label.to_string()
                            };

                            // Center the text in the block
                            let total_padding = block_width - display_text.len();
                            let left_padding = total_padding / 2;
                            let right_padding = total_padding - left_padding;

                            format!(
                                "{}{}{}",
                                " ".repeat(left_padding),
                                display_text,
                                " ".repeat(right_padding)
                            )
                        } else {
                            // For other lines, just show empty color block
                            " ".repeat(block_width)
                        };

                        // Apply the background color to the content
                        let color_block =
                            display_content.on_truecolor(color.red, color.green, color.blue);

                        // Choose text color based on contrast
                        let text_color = if (0.299 * color.red as f64
                            + 0.587 * color.green as f64
                            + 0.114 * color.blue as f64)
                            > 128.0
                        {
                            // Dark text for light backgrounds
                            color_block.black()
                        } else {
                            // Light text for dark backgrounds
                            color_block.white()
                        };

                        print!(" {} ", text_color);
                    } else {
                        // Empty space if no color at this index
                        print!("{:>26} ", "");
                    }

                    // Add horizontal spacing between cards
                    if idx < chunk.len() - 1 {
                        print!("  ");
                    }
                }
                println!();
            }
        }

        println!();
    }

    // Display terminal color palette with actual colors
    println!("{}", "📊 Terminal Color Palette".bold().underline());
    println!();
    print_terminal_palette(colors);
}

/// Print terminal color palette with actual color blocks
fn print_terminal_palette(colors: &std::collections::HashMap<String, crate::core::ColorFormat>) {
    let terminal_colors = vec![
        ("Black", "black"),
        ("Red", "red"),
        ("Green", "green"),
        ("Yellow", "yellow"),
        ("Blue", "blue"),
        ("Magenta", "magenta"),
        ("Cyan", "cyan"),
        ("White", "white"),
        ("Bright Black", "bright_black"),
        ("Bright Red", "bright_red"),
        ("Bright Green", "bright_green"),
        ("Bright Yellow", "bright_yellow"),
        ("Bright Blue", "bright_blue"),
        ("Bright Magenta", "bright_magenta"),
        ("Bright Cyan", "bright_cyan"),
        ("Bright White", "bright_white"),
    ];

    // Print in two columns
    let mid = terminal_colors.len() / 2;
    for i in 0..mid {
        let (_, key1) = &terminal_colors[i];
        let (_, key2) = &terminal_colors[i + mid];

        if let Some(color1) = colors.get(*key1) {
            let r1 = color1.red;
            let g1 = color1.green;
            let b1 = color1.blue;
            let luminance1 = 0.299 * r1 as f64 + 0.587 * g1 as f64 + 0.114 * b1 as f64;

            let block1 = format!(" {:<24} ", key1);
            let color_block1 = if luminance1 > 128.0 {
                block1.black().on_truecolor(r1, g1, b1)
            } else {
                block1.white().on_truecolor(r1, g1, b1)
            };

            print!("{}", color_block1);
        }

        print!("  ");

        if let Some(color2) = colors.get(*key2) {
            let r2 = color2.red;
            let g2 = color2.green;
            let b2 = color2.blue;
            let luminance2 = 0.299 * r2 as f64 + 0.587 * g2 as f64 + 0.114 * b2 as f64;

            let block2 = format!(" {:<24} ", key2);
            let color_block2 = if luminance2 > 128.0 {
                block2.black().on_truecolor(r2, g2, b2)
            } else {
                block2.white().on_truecolor(r2, g2, b2)
            };

            print!("{}", color_block2);
        }

        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_color_format(r: u8, g: u8, b: u8) -> crate::core::ColorFormat {
        crate::core::ColorFormat {
            hex: format!("#{:02X}{:02X}{:02X}", r, g, b),
            hex_stripped: format!("{:02X}{:02X}{:02X}", r, g, b),
            hex8: format!("#{:02X}{:02X}{:02X}FF", r, g, b),
            hex8_stripped: format!("{:02X}{:02X}{:02X}FF", r, g, b),
            rgb: format!("rgb({}, {}, {})", r, g, b),
            rgba: format!("rgba({}, {}, {}, 1.0)", r, g, b),
            hsl: "hsl(0, 0%, 0%)".to_string(),
            hsla: "hsla(0, 0%, 0%, 1.0)".to_string(),
            red: r,
            green: g,
            blue: b,
            alpha: 1.0,
            hue: 0.0,
            saturation: 0.0,
            lightness: 0.0,
        }
    }

    #[test]
    fn test_create_test_color_format() {
        let color = create_test_color_format(255, 128, 64);
        assert_eq!(color.red, 255);
        assert_eq!(color.green, 128);
        assert_eq!(color.blue, 64);
        assert!(color.hex.starts_with("#"));
    }

    #[test]
    fn test_luminance_calculation_white() {
        let color = create_test_color_format(255, 255, 255);
        let luminance =
            0.299 * color.red as f64 + 0.587 * color.green as f64 + 0.114 * color.blue as f64;
        assert!(luminance > 128.0); // White should be bright
    }

    #[test]
    fn test_luminance_calculation_black() {
        let color = create_test_color_format(0, 0, 0);
        let luminance =
            0.299 * color.red as f64 + 0.587 * color.green as f64 + 0.114 * color.blue as f64;
        assert!(luminance < 128.0); // Black should be dark
    }

    #[test]
    fn test_luminance_calculation_gray() {
        let color = create_test_color_format(128, 128, 128);
        let luminance =
            0.299 * color.red as f64 + 0.587 * color.green as f64 + 0.114 * color.blue as f64;
        assert!((luminance - 128.0).abs() < 1.0); // Gray should be around 128
    }

    #[test]
    fn test_mode_parsing() {
        // Test mode parsing logic
        let mode_dark = if "dark" == "dark" {
            Mode::Dark
        } else {
            Mode::Light
        };
        assert_eq!(mode_dark, Mode::Dark);

        let mode_light = if "light" == "dark" {
            Mode::Dark
        } else {
            Mode::Light
        };
        assert_eq!(mode_light, Mode::Light);
    }

    #[test]
    fn test_color_map_access() {
        let mut colors = HashMap::new();
        colors.insert("primary".to_string(), create_test_color_format(255, 0, 0));

        assert!(colors.contains_key("primary"));
        assert!(!colors.contains_key("secondary"));

        let primary = colors.get("primary");
        assert!(primary.is_some());
        assert_eq!(primary.unwrap().red, 255);
    }
}
