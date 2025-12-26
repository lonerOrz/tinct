use crate::theme::{generate_palette, load_theme, select_theme_mode};
use colored::*;

/// Display a color preview showing all available colors in the theme as a matrix
pub fn show_color_preview(theme_path: &str, mode: &str) -> Result<(), String> {
    // Load the theme
    let theme_all = load_theme(theme_path)?;
    let (theme, effective_mode) = select_theme_mode(&theme_all, mode)?;

    // Generate palette
    let palette = generate_palette(&theme, effective_mode == "dark", false)?;

    println!(
        "{}",
        "🎨 Material Design 3 Color Preview".bold().underline()
    );
    println!("🌙 Theme Mode: {}", effective_mode.bold());
    println!();

    // Display colors in MD3 style similar to the official documentation
    display_md3_cards_grid(&palette);

    Ok(())
}

/// Display colors in a card grid layout similar to the MD3 official documentation
fn display_md3_cards_grid(palette: &crate::theme::Palette) {
    // Define color cards based on the MD3 documentation structure
    let cards = vec![
        // Primary card
        vec![
            ("Primary", &palette.primary.default),
            ("On Primary", &palette.on_primary.default),
            ("Primary Container", &palette.primary_container.default),
            (
                "On Primary Container",
                &palette.on_primary_container.default,
            ),
        ],
        // Secondary card
        vec![
            ("Secondary", &palette.secondary.default),
            ("On Secondary", &palette.on_secondary.default),
            ("Secondary Container", &palette.secondary_container.default),
            (
                "On Secondary Container",
                &palette.on_secondary_container.default,
            ),
        ],
        // Tertiary card
        vec![
            ("Tertiary", &palette.tertiary.default),
            ("On Tertiary", &palette.on_tertiary.default),
            ("Tertiary Container", &palette.tertiary_container.default),
            (
                "On Tertiary Container",
                &palette.on_tertiary_container.default,
            ),
        ],
        // Error card
        vec![
            ("Error", &palette.error.default),
            ("On Error", &palette.on_error.default),
            ("Error Container", &palette.error_container.default),
            ("On Error Container", &palette.on_error_container.default),
        ],
        // Surface card
        vec![
            ("Surface Dim", &palette.surface_dim.default),
            ("Surface", &palette.surface.default),
            ("Surface Bright", &palette.surface_bright.default),
        ],
        // Surface Variant card
        vec![
            ("Surface Variant", &palette.surface_variant.default),
            ("On Surface Variant", &palette.on_surface_variant.default),
        ],
        // Surface Containers card (vertical layout)
        vec![
            (
                "Container Lowest",
                &palette.surface_container_lowest.default,
            ),
            ("Container Low", &palette.surface_container_low.default),
            ("Container", &palette.surface_container.default),
            ("Container High", &palette.surface_container_high.default),
            (
                "Container Highest",
                &palette.surface_container_highest.default,
            ),
        ],
        // Background card
        vec![
            ("Background", &palette.background.default),
            ("On Background", &palette.on_background.default),
        ],
        // Outline card
        vec![
            ("Outline", &palette.outline.default),
            ("Outline Variant", &palette.outline_variant.default),
        ],
        // Inverse card
        vec![
            ("Inverse Surface", &palette.inverse_surface.default),
            ("Inverse On Surface", &palette.inverse_on_surface.default),
            ("Inverse Primary", &palette.inverse_primary.default),
        ],
        // Special card
        vec![
            ("Shadow", &palette.shadow.default),
            ("Scrim", &palette.scrim.default),
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
                        let block_width = 24; // Increased width to accommodate longer text

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
                        print!("{:>26} ", ""); // 24 + 2 for spacing
                    }

                    // Add horizontal spacing between cards
                    if idx < chunk.len() - 1 {
                        print!("  "); // 2 spaces between cards
                    }
                }
                println!(); // New line after each row of blocks
            }
        }

        println!(); // Extra spacing between rows of cards
    }
}
