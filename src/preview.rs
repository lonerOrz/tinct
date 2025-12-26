use crate::theme::{load_theme, select_theme_mode, generate_palette};
use colored::*;

/// Display a color preview showing all available colors in the theme
pub fn show_color_preview(theme_path: &str, mode: &str) -> Result<(), String> {
    println!("{}", "🎨 Color Preview".bold().underline());
    println!();

    // Load the theme
    let theme_all = load_theme(theme_path)?;
    let (theme, effective_mode) = select_theme_mode(&theme_all, mode)?;

    println!("🌙 Theme Mode: {}", effective_mode.bold());
    println!();

    // Generate palette
    let palette = generate_palette(&theme, effective_mode == "dark", false)?;

    // Display all colors one by one with categories
    display_all_colors(&palette);

    Ok(())
}

/// Display all colors in a categorized format
fn display_all_colors(palette: &crate::theme::Palette) {
    // Primary Colors
    println!("{}", "🌟 Primary Colors:".bold());
    display_single_color("Primary", &palette.primary.default);
    display_single_color("On Primary", &palette.on_primary.default);
    display_single_color("Primary Container", &palette.primary_container.default);
    display_single_color("On Primary Container", &palette.on_primary_container.default);
    println!();

    // Primary Fixed Colors
    println!("{}", "🌟 Primary Fixed Colors:".bold());
    display_single_color("Primary Fixed", &palette.primary_fixed.default);
    display_single_color("Primary Fixed Dim", &palette.primary_fixed_dim.default);
    display_single_color("On Primary Fixed", &palette.on_primary_fixed.default);
    display_single_color("On Primary Fixed Variant", &palette.on_primary_fixed_variant.default);
    println!();

    // Secondary Colors
    println!("{}", "🌟 Secondary Colors:".bold());
    display_single_color("Secondary", &palette.secondary.default);
    display_single_color("On Secondary", &palette.on_secondary.default);
    display_single_color("Secondary Container", &palette.secondary_container.default);
    display_single_color("On Secondary Container", &palette.on_secondary_container.default);
    println!();

    // Secondary Fixed Colors
    println!("{}", "🌟 Secondary Fixed Colors:".bold());
    display_single_color("Secondary Fixed", &palette.secondary_fixed.default);
    display_single_color("Secondary Fixed Dim", &palette.secondary_fixed_dim.default);
    display_single_color("On Secondary Fixed", &palette.on_secondary_fixed.default);
    display_single_color("On Secondary Fixed Variant", &palette.on_secondary_fixed_variant.default);
    println!();

    // Tertiary Colors
    println!("{}", "🌟 Tertiary Colors:".bold());
    display_single_color("Tertiary", &palette.tertiary.default);
    display_single_color("On Tertiary", &palette.on_tertiary.default);
    display_single_color("Tertiary Container", &palette.tertiary_container.default);
    display_single_color("On Tertiary Container", &palette.on_tertiary_container.default);
    println!();

    // Tertiary Fixed Colors
    println!("{}", "🌟 Tertiary Fixed Colors:".bold());
    display_single_color("Tertiary Fixed", &palette.tertiary_fixed.default);
    display_single_color("Tertiary Fixed Dim", &palette.tertiary_fixed_dim.default);
    display_single_color("On Tertiary Fixed", &palette.on_tertiary_fixed.default);
    display_single_color("On Tertiary Fixed Variant", &palette.on_tertiary_fixed_variant.default);
    println!();

    // Surface Colors
    println!("{}", "🎨 Surface Colors:".bold());
    display_single_color("Background", &palette.background.default);
    display_single_color("On Background", &palette.on_background.default);
    display_single_color("Surface", &palette.surface.default);
    display_single_color("On Surface", &palette.on_surface.default);
    display_single_color("Surface Variant", &palette.surface_variant.default);
    display_single_color("On Surface Variant", &palette.on_surface_variant.default);
    println!();

    // Surface Container Colors
    println!("{}", "🎨 Surface Container Colors:".bold());
    display_single_color("Surface Container Lowest", &palette.surface_container_lowest.default);
    display_single_color("Surface Container Low", &palette.surface_container_low.default);
    display_single_color("Surface Container", &palette.surface_container.default);
    display_single_color("Surface Container High", &palette.surface_container_high.default);
    display_single_color("Surface Container Highest", &palette.surface_container_highest.default);
    println!();

    // Bright and Dim Surface Colors
    println!("{}", "🎨 Bright & Dim Surface Colors:".bold());
    display_single_color("Surface Dim", &palette.surface_dim.default);
    display_single_color("Surface Bright", &palette.surface_bright.default);
    println!();

    // Inverse Colors
    println!("{}", "🎨 Inverse Colors:".bold());
    display_single_color("Inverse Surface", &palette.inverse_surface.default);
    display_single_color("Inverse On Surface", &palette.inverse_on_surface.default);
    display_single_color("Inverse Primary", &palette.inverse_primary.default);
    println!();

    // Other Colors
    println!("{}", "🎨 Other Colors:".bold());
    display_single_color("Outline", &palette.outline.default);
    display_single_color("Outline Variant", &palette.outline_variant.default);
    display_single_color("Shadow", &palette.shadow.default);
    display_single_color("Scrim", &palette.scrim.default);
    println!();

    // Error Colors (placed last as they are less frequently used)
    println!("{}", "🚨 Error Colors:".bold());
    display_single_color("Error", &palette.error.default);
    display_single_color("On Error", &palette.on_error.default);
    display_single_color("Error Container", &palette.error_container.default);
    display_single_color("On Error Container", &palette.on_error_container.default);
}

/// Display a single color with its name and visual representation
fn display_single_color(name: &str, color: &crate::theme::ColorFormat) {
    let color_block = "        "; // 8 spaces for the color block

    // Apply background color to the block
    let color_block_with_bg = color_block.on_truecolor(color.red, color.green, color.blue);

    // Always use a consistent color for text to ensure readability
    let text_color = format!("{:<30}", name.bold().normal()); // Use normal() to reset to terminal default

    // Print color name with consistent color and visual block
    println!("  {}{}", text_color, color_block_with_bg);
}