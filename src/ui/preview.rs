//! Color preview functionality
//!
//! Displays Material Design 3 color palettes in the terminal with actual color blocks.

use crate::core::color::{Color, calculate_relative_luminance};
use crate::core::{Mode, Theme};
use crate::image::SchemeType;
use crate::palette::{AlgorithmParameters, LegacyPaletteGenerator, Palette};
use colored::*;
use std::collections::HashMap;
use std::fmt::Write as _;

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
    let theme =
        Theme::from_json_file(theme_path, &preview_generator()).map_err(|e| e.to_string())?;
    show_color_preview_for_theme(&theme, parse_mode(mode))
}

pub fn show_color_preview_from_json(json: &serde_json::Value, mode: &str) -> Result<(), String> {
    let theme = Theme::from_json_value(json, &preview_generator()).map_err(|e| e.to_string())?;
    show_color_preview_for_theme(&theme, parse_mode(mode))
}

/// Generator used by the standalone preview entry points.
fn preview_generator() -> LegacyPaletteGenerator {
    LegacyPaletteGenerator::new(AlgorithmParameters::default(), SchemeType::TonalSpot)
}

fn show_color_preview_for_theme(theme: &Theme, mode: Mode) -> Result<(), String> {
    let palette = match mode {
        Mode::Dark => &theme.dark_palette,
        Mode::Light => &theme.light_palette,
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
            (
                "Primary",
                colors
                    .get("primary")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "On Primary",
                colors
                    .get("on_primary")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "Primary Container",
                colors
                    .get("primary_container")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "On Primary Container",
                colors
                    .get("on_primary_container")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
        ],
        vec![
            (
                "Secondary",
                colors
                    .get("secondary")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "On Secondary",
                colors
                    .get("on_secondary")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "Secondary Container",
                colors
                    .get("secondary_container")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "On Secondary Container",
                colors
                    .get("on_secondary_container")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
        ],
        vec![
            (
                "Tertiary",
                colors
                    .get("tertiary")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "On Tertiary",
                colors
                    .get("on_tertiary")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "Tertiary Container",
                colors
                    .get("tertiary_container")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "On Tertiary Container",
                colors
                    .get("on_tertiary_container")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
        ],
        vec![
            (
                "Error",
                colors
                    .get("error")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "On Error",
                colors
                    .get("on_error")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "Error Container",
                colors
                    .get("error_container")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "On Error Container",
                colors
                    .get("on_error_container")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
        ],
        vec![
            (
                "Primary Fixed",
                colors
                    .get("primary_fixed")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "Primary Fixed Dim",
                colors
                    .get("primary_fixed_dim")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "On Primary Fixed",
                colors
                    .get("on_primary_fixed")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "On Primary Fixed Var",
                colors
                    .get("on_primary_fixed_variant")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
        ],
        vec![
            (
                "Secondary Fixed",
                colors
                    .get("secondary_fixed")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "Secondary Fixed Dim",
                colors
                    .get("secondary_fixed_dim")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "On Secondary Fixed",
                colors
                    .get("on_secondary_fixed")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "On Secondary Fixed Var",
                colors
                    .get("on_secondary_fixed_variant")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
        ],
        vec![
            (
                "Tertiary Fixed",
                colors
                    .get("tertiary_fixed")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "Tertiary Fixed Dim",
                colors
                    .get("tertiary_fixed_dim")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "On Tertiary Fixed",
                colors
                    .get("on_tertiary_fixed")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "On Tertiary Fixed Var",
                colors
                    .get("on_tertiary_fixed_variant")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
        ],
        vec![
            (
                "Surface Dim",
                colors
                    .get("surface_dim")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "Surface",
                colors
                    .get("surface")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "Surface Bright",
                colors
                    .get("surface_bright")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
        ],
        vec![
            (
                "Surface Variant",
                colors
                    .get("surface_variant")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "On Surface Variant",
                colors
                    .get("on_surface_variant")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
        ],
        vec![
            (
                "Container Lowest",
                colors
                    .get("surface_container_lowest")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "Container Low",
                colors
                    .get("surface_container_low")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "Container",
                colors
                    .get("surface_container")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "Container High",
                colors
                    .get("surface_container_high")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "Container Highest",
                colors
                    .get("surface_container_highest")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
        ],
        vec![
            (
                "Background",
                colors
                    .get("background")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "On Background",
                colors
                    .get("on_background")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
        ],
        vec![
            (
                "Outline",
                colors
                    .get("outline")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "Outline Variant",
                colors
                    .get("outline_variant")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
        ],
        vec![
            (
                "Inverse Surface",
                colors
                    .get("inverse_surface")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "Inverse On Surface",
                colors
                    .get("inverse_on_surface")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "Inverse Primary",
                colors
                    .get("inverse_primary")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
        ],
        vec![
            (
                "Shadow",
                colors
                    .get("shadow")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
            (
                "Scrim",
                colors
                    .get("scrim")
                    .expect("BUG: preview card keys must be listed in REQUIRED_COLOR_KEYS"),
            ),
        ],
    ];

    const CARDS_PER_ROW: usize = 3;

    let mut line = String::new();

    for chunk in cards.chunks(CARDS_PER_ROW) {
        let max_colors = chunk.iter().map(|card| card.len()).max().unwrap_or(0);

        for color_idx in 0..max_colors {
            for line_num in 0..3 {
                line.clear();
                for (idx, card) in chunk.iter().enumerate() {
                    if color_idx < card.len() {
                        let (label, color) = &card[color_idx];
                        let block_width = 24;

                        let display_content = if line_num == 1 {
                            if label.len() > block_width {
                                let mut truncated: String =
                                    label.chars().take(block_width - 3).collect();
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
                        let luminance = calculate_relative_luminance(color.r, color.g, color.b);
                        let text_color = if luminance > 0.1791 {
                            color_block.black()
                        } else {
                            color_block.white()
                        };

                        write!(line, " {} ", text_color).expect("writing to a String cannot fail");
                    } else {
                        write!(line, "{:>26} ", "").expect("writing to a String cannot fail");
                    }

                    if idx < chunk.len() - 1 {
                        line.push_str("  ");
                    }
                }
                println!("{}", line);
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
            let luminance1 = calculate_relative_luminance(color1.r, color1.g, color1.b);
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
            let luminance2 = calculate_relative_luminance(color2.r, color2.g, color2.b);
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
}
