//! Color preview functionality
//!
//! Displays Material Design 3 color palettes in the terminal with actual color blocks.

use crate::color::Color;
use crate::core::Mode;
use crate::palette::{AlgorithmParameters, LegacyPaletteGenerator, Palette};
use crate::theme::JsonThemeLoader;
use colored::*;
use std::collections::HashMap;

/// Display a color preview from an already-built palette.
pub fn show_color_preview_from_theme(palette: &Palette, mode: Mode) -> Result<(), String> {
    let colors = palette.to_map();
    println!(
        "{}",
        "🎨 Material Design 3 Color Preview".bold().underline()
    );
    println!("🌙 Theme Mode: {}", mode.to_string().bold());
    println!();

    display_md3_cards_grid(&colors)
}

pub fn show_color_preview(theme_path: &str, mode: &str) -> Result<(), String> {
    let palette_gen = LegacyPaletteGenerator::new(AlgorithmParameters::default());
    let theme_loader = JsonThemeLoader::new(palette_gen);
    let theme = theme_loader.load(theme_path).map_err(|e| e.to_string())?;

    let mode = parse_mode(mode);
    let palette = if mode == Mode::Dark {
        &theme.dark_palette
    } else {
        &theme.light_palette
    };

    show_color_preview_from_theme(palette, mode)
}

pub fn show_color_preview_from_json(json: &serde_json::Value, mode: &str) -> Result<(), String> {
    let palette_gen = LegacyPaletteGenerator::new(AlgorithmParameters::default());
    let theme_loader = JsonThemeLoader::new(palette_gen);
    let theme = theme_loader.load_value(json).map_err(|e| e.to_string())?;

    let mode = parse_mode(mode);
    let palette = if mode == Mode::Dark {
        &theme.dark_palette
    } else {
        &theme.light_palette
    };

    show_color_preview_from_theme(palette, mode)
}

fn parse_mode(mode: &str) -> Mode {
    if mode == "dark" {
        Mode::Dark
    } else {
        Mode::Light
    }
}

const REQUIRED_COLOR_KEYS: &[&str] = &[
    "primary",
    "on_primary",
    "primary_container",
    "on_primary_container",
    "secondary",
    "on_secondary",
    "secondary_container",
    "on_secondary_container",
    "tertiary",
    "on_tertiary",
    "tertiary_container",
    "on_tertiary_container",
    "error",
    "on_error",
    "error_container",
    "on_error_container",
    "primary_fixed",
    "primary_fixed_dim",
    "on_primary_fixed",
    "on_primary_fixed_variant",
    "secondary_fixed",
    "secondary_fixed_dim",
    "on_secondary_fixed",
    "on_secondary_fixed_variant",
    "tertiary_fixed",
    "tertiary_fixed_dim",
    "on_tertiary_fixed",
    "on_tertiary_fixed_variant",
    "surface_dim",
    "surface",
    "surface_bright",
    "surface_variant",
    "on_surface_variant",
    "surface_container_lowest",
    "surface_container_low",
    "surface_container",
    "surface_container_high",
    "surface_container_highest",
    "background",
    "on_background",
    "outline",
    "outline_variant",
    "inverse_surface",
    "inverse_on_surface",
    "inverse_primary",
    "shadow",
    "scrim",
];

/// Display colors in a card grid layout with true color blocks
fn display_md3_cards_grid(colors: &HashMap<String, Color>) -> Result<(), String> {
    let missing: Vec<&str> = REQUIRED_COLOR_KEYS
        .iter()
        .filter(|k| !colors.contains_key(**k))
        .copied()
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "Color preview requires {} color roles, missing: {:?}",
            REQUIRED_COLOR_KEYS.len(),
            missing
        ));
    }
    // Define color cards based on the MD3 documentation structure
    let cards: Vec<Vec<(&str, &Color)>> = vec![
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
        vec![
            ("Error", colors.get("error").unwrap()),
            ("On Error", colors.get("on_error").unwrap()),
            ("Error Container", colors.get("error_container").unwrap()),
            (
                "On Error Container",
                colors.get("on_error_container").unwrap(),
            ),
        ],
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
        vec![
            ("Surface Dim", colors.get("surface_dim").unwrap()),
            ("Surface", colors.get("surface").unwrap()),
            ("Surface Bright", colors.get("surface_bright").unwrap()),
        ],
        vec![
            ("Surface Variant", colors.get("surface_variant").unwrap()),
            (
                "On Surface Variant",
                colors.get("on_surface_variant").unwrap(),
            ),
        ],
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
        vec![
            ("Background", colors.get("background").unwrap()),
            ("On Background", colors.get("on_background").unwrap()),
        ],
        vec![
            ("Outline", colors.get("outline").unwrap()),
            ("Outline Variant", colors.get("outline_variant").unwrap()),
        ],
        vec![
            ("Inverse Surface", colors.get("inverse_surface").unwrap()),
            (
                "Inverse On Surface",
                colors.get("inverse_on_surface").unwrap(),
            ),
            ("Inverse Primary", colors.get("inverse_primary").unwrap()),
        ],
        vec![
            ("Shadow", colors.get("shadow").unwrap()),
            ("Scrim", colors.get("scrim").unwrap()),
        ],
    ];

    const CARDS_PER_ROW: usize = 3;

    for chunk in cards.chunks(CARDS_PER_ROW) {
        let max_colors = chunk.iter().map(|card| card.len()).max().unwrap_or(0);

        for color_idx in 0..max_colors {
            for line_num in 0..3 {
                for (idx, card) in chunk.iter().enumerate() {
                    if color_idx < card.len() {
                        let (label, color) = &card[color_idx];
                        let block_width = 24;

                        let display_content = if line_num == 1 {
                            let text_len = label.len();
                            if text_len > block_width {
                                let chars: Vec<char> = label.chars().collect();
                                let mut truncated = String::new();
                                for i in 0..(block_width - 3) {
                                    if i < chars.len() {
                                        truncated.push(chars[i]);
                                    }
                                }
                                truncated.push_str("...");
                                truncated
                            } else {
                                label.to_string()
                            }
                        } else {
                            " ".repeat(block_width)
                        };

                        let total_padding = block_width - display_content.len();
                        let left_padding = total_padding / 2;
                        let right_padding = total_padding - left_padding;
                        let centered = format!(
                            "{}{}{}",
                            " ".repeat(left_padding),
                            display_content,
                            " ".repeat(right_padding)
                        );

                        let color_block = centered.on_truecolor(color.r, color.g, color.b);
                        let luminance =
                            crate::color::calculate_relative_luminance(color.r, color.g, color.b);
                        let text_color = if luminance > 0.1791 {
                            color_block.black()
                        } else {
                            color_block.white()
                        };

                        print!(" {} ", text_color);
                    } else {
                        print!("{:>26} ", "");
                    }

                    if idx < chunk.len() - 1 {
                        print!("  ");
                    }
                }
                println!();
            }
        }
        println!();
    }

    println!("{}", "📊 Terminal Color Palette".bold().underline());
    println!();
    print_terminal_palette(colors);
    Ok(())
}

fn print_terminal_palette(colors: &HashMap<String, Color>) {
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

    let mid = terminal_colors.len() / 2;
    for i in 0..mid {
        let (_, key1) = &terminal_colors[i];
        let (_, key2) = &terminal_colors[i + mid];

        if let Some(color1) = colors.get(*key1) {
            let luminance1 =
                crate::color::calculate_relative_luminance(color1.r, color1.g, color1.b);
            let block1 = format!(" {:<24} ", key1);
            let color_block1 = if luminance1 > 0.1791 {
                block1.black().on_truecolor(color1.r, color1.g, color1.b)
            } else {
                block1.white().on_truecolor(color1.r, color1.g, color1.b)
            };
            print!("{}", color_block1);
        }

        print!("  ");

        if let Some(color2) = colors.get(*key2) {
            let luminance2 =
                crate::color::calculate_relative_luminance(color2.r, color2.g, color2.b);
            let block2 = format!(" {:<24} ", key2);
            let color_block2 = if luminance2 > 0.1791 {
                block2.black().on_truecolor(color2.r, color2.g, color2.b)
            } else {
                block2.white().on_truecolor(color2.r, color2.g, color2.b)
            };
            print!("{}", color_block2);
        }

        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mode() {
        assert_eq!(parse_mode("dark"), Mode::Dark);
        assert_eq!(parse_mode("light"), Mode::Light);
        assert_eq!(parse_mode("anything"), Mode::Light);
    }

    #[test]
    fn test_color_map_access() {
        let mut colors = HashMap::new();
        colors.insert("primary".to_string(), Color::new(255, 0, 0, 1.0));
        assert!(colors.contains_key("primary"));
        assert!(!colors.contains_key("secondary"));
        assert_eq!(colors.get("primary").unwrap().r, 255);
    }
}
