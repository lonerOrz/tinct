//! Color preview module.
//!
//! Provides an artistic terminal palette preview showcasing Material Design 3
//! token roles, ANSI 16 terminal swatches with domino pillars, and a simulated
//! code syntax card.

use crate::core::color::Color;
use crate::core::{Mode, Theme};
use crate::image::SchemeType;
use crate::palette::{AlgorithmParameters, LegacyPaletteGenerator, Palette};
use std::collections::HashMap;

const COL_WIDTH: usize = 38;
const GAP_WIDTH: usize = 4;
const TOTAL_WIDTH: usize = COL_WIDTH * 2 + GAP_WIDTH;
const SWATCH_WIDTH: usize = 4;
const HEX_WIDTH: usize = 7;

/// ANSI terminal escape sequence helpers.
struct Style;

impl Style {
    const RESET: &'static str = "\x1b[0m";
    const BOLD: &'static str = "\x1b[1m";
    const DIM: &'static str = "\x1b[2m";

    #[inline]
    fn fg(c: Color) -> String {
        format!("\x1b[38;2;{};{};{}m", c.r, c.g, c.b)
    }

    #[inline]
    fn bg(c: Color) -> String {
        format!("\x1b[48;2;{};{};{}m", c.r, c.g, c.b)
    }
}

/// Token role descriptor.
struct Role {
    label: &'static str,
    key: &'static str,
}

/// Cell entry in a column.
enum ViewItem {
    Header(&'static str),
    Swatch(Role),
    Blank,
}

const MD3_LEFT: &[ViewItem] = &[
    ViewItem::Header("PRIMARY"),
    ViewItem::Swatch(Role {
        label: "Primary",
        key: "primary",
    }),
    ViewItem::Swatch(Role {
        label: "On Primary",
        key: "on_primary",
    }),
    ViewItem::Swatch(Role {
        label: "Primary Container",
        key: "primary_container",
    }),
    ViewItem::Swatch(Role {
        label: "On Prim. Container",
        key: "on_primary_container",
    }),
    ViewItem::Blank,
    ViewItem::Header("SECONDARY"),
    ViewItem::Swatch(Role {
        label: "Secondary",
        key: "secondary",
    }),
    ViewItem::Swatch(Role {
        label: "On Secondary",
        key: "on_secondary",
    }),
    ViewItem::Swatch(Role {
        label: "Secondary Container",
        key: "secondary_container",
    }),
    ViewItem::Swatch(Role {
        label: "On Sec. Container",
        key: "on_secondary_container",
    }),
    ViewItem::Blank,
    ViewItem::Header("TERTIARY"),
    ViewItem::Swatch(Role {
        label: "Tertiary",
        key: "tertiary",
    }),
    ViewItem::Swatch(Role {
        label: "On Tertiary",
        key: "on_tertiary",
    }),
    ViewItem::Swatch(Role {
        label: "Tertiary Container",
        key: "tertiary_container",
    }),
    ViewItem::Swatch(Role {
        label: "On Ter. Container",
        key: "on_tertiary_container",
    }),
    ViewItem::Blank,
    ViewItem::Header("ERROR"),
    ViewItem::Swatch(Role {
        label: "Error",
        key: "error",
    }),
    ViewItem::Swatch(Role {
        label: "On Error",
        key: "on_error",
    }),
];

const MD3_RIGHT: &[ViewItem] = &[
    ViewItem::Header("SURFACE & BACKGROUND"),
    ViewItem::Swatch(Role {
        label: "Background",
        key: "background",
    }),
    ViewItem::Swatch(Role {
        label: "On Background",
        key: "on_background",
    }),
    ViewItem::Swatch(Role {
        label: "Surface",
        key: "surface",
    }),
    ViewItem::Swatch(Role {
        label: "On Surface",
        key: "on_surface",
    }),
    ViewItem::Swatch(Role {
        label: "Surface Variant",
        key: "surface_variant",
    }),
    ViewItem::Swatch(Role {
        label: "On Surface Variant",
        key: "on_surface_variant",
    }),
    ViewItem::Blank,
    ViewItem::Header("INVERSE"),
    ViewItem::Swatch(Role {
        label: "Inverse Surface",
        key: "inverse_surface",
    }),
    ViewItem::Swatch(Role {
        label: "Inverse On Surface",
        key: "inverse_on_surface",
    }),
    ViewItem::Swatch(Role {
        label: "Inverse Primary",
        key: "inverse_primary",
    }),
    ViewItem::Blank,
    ViewItem::Header("OUTLINE & SHADOW"),
    ViewItem::Swatch(Role {
        label: "Outline",
        key: "outline",
    }),
    ViewItem::Swatch(Role {
        label: "Outline Variant",
        key: "outline_variant",
    }),
    ViewItem::Swatch(Role {
        label: "Shadow",
        key: "shadow",
    }),
    ViewItem::Swatch(Role {
        label: "Scrim",
        key: "scrim",
    }),
    ViewItem::Blank,
    ViewItem::Blank,
    ViewItem::Blank,
];

const ANSI_COLS: &[(&str, &str, &str)] = &[
    ("BLK", "black", "bright_black"),
    ("RED", "red", "bright_red"),
    ("GRN", "green", "bright_green"),
    ("YEL", "yellow", "bright_yellow"),
    ("BLU", "blue", "bright_blue"),
    ("MAG", "magenta", "bright_magenta"),
    ("CYN", "cyan", "bright_cyan"),
    ("WHT", "white", "bright_white"),
];

/// Computes visible width of a string in terminal cells by skipping ANSI escapes.
fn visible_width(s: &str) -> usize {
    let mut width = 0;
    let mut chars = s.chars();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            for next_ch in chars.by_ref() {
                if next_ch == 'm' {
                    break;
                }
            }
        } else {
            width += 1;
        }
    }
    width
}

/// Renderer responsible for organizing and drawing the palette preview dashboard.
struct PaletteView<'a> {
    colors: &'a HashMap<String, Color>,
    mode: Mode,
}

impl<'a> PaletteView<'a> {
    /// Creates a new palette view instance.
    fn new(colors: &'a HashMap<String, Color>, mode: Mode) -> Self {
        Self { colors, mode }
    }

    /// Retrieves a color by key, falling back to black if not found.
    fn color(&self, key: &str) -> Color {
        self.colors
            .get(key)
            .copied()
            .unwrap_or(Color::new(0, 0, 0, 1.0))
    }

    /// Renders the complete preview output to stdout.
    fn render(&self) {
        println!();
        self.render_header();
        self.render_md3_grid();
        self.render_ansi_domino_pillars();
        self.render_syntax_preview_card();
        println!();
    }

    /// Prints the top banner with tinct logo and theme mode.
    fn render_header(&self) {
        let mode_str = match self.mode {
            Mode::Dark => "DARK",
            Mode::Light => "LIGHT",
        };
        let tag = format!("tinct  {}", mode_str);
        let line_len = TOTAL_WIDTH.saturating_sub(tag.len() + 2);
        println!(
            "  {}{}{}  {}{}  {}{}{}",
            Style::BOLD,
            "tinct",
            Style::RESET,
            Style::DIM,
            mode_str,
            Style::DIM,
            "─".repeat(line_len),
            Style::RESET
        );
        println!();
    }

    /// Formats a single column entry item into a styled string.
    fn render_cell(&self, item: Option<&ViewItem>) -> String {
        match item {
            Some(ViewItem::Header(title)) => {
                let dash_len = COL_WIDTH.saturating_sub(title.len() + 1);
                format!(
                    "{}{}{}{}{}",
                    Style::DIM,
                    title,
                    " ",
                    "─".repeat(dash_len),
                    Style::RESET
                )
            }
            Some(ViewItem::Swatch(role)) => {
                let c = self.color(role.key);
                let hex = c.hex().to_uppercase();
                let pad_len =
                    COL_WIDTH.saturating_sub(SWATCH_WIDTH + 1 + role.label.len() + HEX_WIDTH);

                let swatch_box = format!(
                    "{}{}{}",
                    Style::bg(c),
                    " ".repeat(SWATCH_WIDTH),
                    Style::RESET
                );
                let hex_dim = format!("{}{}{}", Style::DIM, hex, Style::RESET);

                format!(
                    "{} {}{}{}",
                    swatch_box,
                    role.label,
                    " ".repeat(pad_len),
                    hex_dim
                )
            }
            Some(ViewItem::Blank) | None => " ".repeat(COL_WIDTH),
        }
    }

    /// Prints the dual-column grid of all Material Design 3 token roles.
    fn render_md3_grid(&self) {
        let rows = MD3_LEFT.len().max(MD3_RIGHT.len());
        let gap = " ".repeat(GAP_WIDTH);
        for i in 0..rows {
            let left = self.render_cell(MD3_LEFT.get(i));
            let right = self.render_cell(MD3_RIGHT.get(i));
            println!("  {}{}{}", left, gap, right);
        }
        println!();
    }

    /// Prints the 8 dual-deck domino color pillars representing ANSI 16 colors.
    fn render_ansi_domino_pillars(&self) {
        let title = "TERMINAL PALETTE ";
        let dash_len = TOTAL_WIDTH.saturating_sub(title.len());
        println!(
            "  {}{}{}{}{}",
            Style::BOLD,
            Style::DIM,
            title,
            "─".repeat(dash_len),
            Style::RESET
        );
        println!();

        print!("  ");
        for (name, _, _) in ANSI_COLS {
            print!("{}{:^8}{}  ", Style::DIM, name, Style::RESET);
        }
        println!();

        print!("  ");
        for (_, norm, _) in ANSI_COLS {
            let c = self.color(norm);
            print!("{}{}{}  ", Style::bg(c), " ".repeat(8), Style::RESET);
        }
        println!();

        print!("  ");
        for (_, _, br) in ANSI_COLS {
            let c = self.color(br);
            print!("{}{}{}  ", Style::bg(c), " ".repeat(8), Style::RESET);
        }
        println!();

        print!("  ");
        for (_, norm, _) in ANSI_COLS {
            let hex = self.color(norm).hex().to_uppercase();
            print!("{}{:^8}{}  ", Style::DIM, hex, Style::RESET);
        }
        println!("\n");
    }

    /// Prints the simulated code editor and shell prompt card.
    fn render_syntax_preview_card(&self) {
        let red = Style::fg(self.color("red"));
        let green = Style::fg(self.color("green"));
        let yellow = Style::fg(self.color("yellow"));
        let blue = Style::fg(self.color("blue"));
        let magenta = Style::fg(self.color("magenta"));
        let cyan = Style::fg(self.color("cyan"));
        let reset = Style::RESET;
        let dim = Style::DIM;

        let inner_width = TOTAL_WIDTH.saturating_sub(2);
        let top_dash_len = inner_width.saturating_sub(10);
        println!("  {}╭─ preview {}╮{}", dim, "─".repeat(top_dash_len), reset);

        let print_card_line = |inner_styled: &str| {
            let v_len = visible_width(inner_styled);
            let pad = inner_width.saturating_sub(v_len);
            println!(
                "  {}│{}{}{}│{}",
                dim,
                inner_styled,
                " ".repeat(pad),
                dim,
                reset
            );
        };

        let dots_line = format!(
            "  {}●{} {}●{} {}●{}  {}~/workspace{} {}main*{}",
            red, reset, yellow, reset, green, reset, cyan, reset, magenta, reset
        );
        print_card_line(&dots_line);
        print_card_line("");

        let prompt_line = format!(
            "  {}${}{} tinct {}{}{} {}{}{}",
            green,
            Style::BOLD,
            reset,
            blue,
            "--build",
            reset,
            yellow,
            "--release",
            reset
        );
        print_card_line(&prompt_line);

        let code1 = format!(
            "  {}{}{} tinct_demo() -> {}{}{} {{",
            magenta, "fn", reset, blue, "Palette", reset
        );
        print_card_line(&code1);

        let code2 = format!(
            "      {}let{} theme = {}{}\"active\"{}; {}// applied successfully{}",
            magenta,
            reset,
            green,
            Style::BOLD,
            reset,
            dim,
            reset
        );
        print_card_line(&code2);

        print_card_line("  }");
        println!("  {}╰{}╯{}", dim, "─".repeat(inner_width), reset);
    }
}

/// Displays a color preview from an already generated palette.
pub fn show_color_preview_from_theme(palette: &Palette, mode: Mode) -> Result<(), String> {
    let colors = palette.to_map();
    PaletteView::new(&colors, mode).render();
    Ok(())
}

/// Loads a theme file and displays its color preview.
pub fn show_color_preview(theme_path: &str, mode: &str) -> Result<(), String> {
    let theme =
        Theme::from_json_file(theme_path, &preview_generator()).map_err(|e| e.to_string())?;
    show_color_preview_for_theme(&theme, parse_mode(mode))
}

/// Parses a JSON theme definition and displays its color preview.
pub fn show_color_preview_from_json(json: &serde_json::Value, mode: &str) -> Result<(), String> {
    let theme = Theme::from_json_value(json, &preview_generator()).map_err(|e| e.to_string())?;
    show_color_preview_for_theme(&theme, parse_mode(mode))
}

/// Default generator for preview mode.
fn preview_generator() -> LegacyPaletteGenerator {
    LegacyPaletteGenerator::new(AlgorithmParameters::default(), SchemeType::TonalSpot)
}

/// Dispatches palette rendering for a theme according to the chosen mode.
fn show_color_preview_for_theme(theme: &Theme, mode: Mode) -> Result<(), String> {
    let palette = match mode {
        Mode::Dark => &theme.dark_palette,
        Mode::Light => &theme.light_palette,
    };
    show_color_preview_from_theme(palette, mode)
}

/// Parses a mode string into a Mode enum.
fn parse_mode(mode: &str) -> Mode {
    if mode.eq_ignore_ascii_case("dark") {
        Mode::Dark
    } else {
        Mode::Light
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mode() {
        assert_eq!(parse_mode("dark"), Mode::Dark);
        assert_eq!(parse_mode("light"), Mode::Light);
        assert_eq!(parse_mode("other"), Mode::Light);
    }
}
