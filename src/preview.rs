use crate::theme::{load_theme, select_theme_mode, generate_palette};
use colored::*;

/// Display a color preview showing all available colors in the theme
pub fn show_color_preview(theme_path: &str, mode: &str) -> Result<(), String> {
    println!("{}", "Color Preview".bold().underline());
    println!();

    // Load the theme
    let theme_all = load_theme(theme_path)?;
    let (theme, effective_mode) = select_theme_mode(&theme_all, mode)?;
    
    println!("{}: {}", "Theme Mode".bold(), effective_mode);
    println!();

    // Generate palette
    let palette = generate_palette(&theme, effective_mode == "dark", false)?;

    // Display all colors in the palette
    display_color_entry("Primary", &palette.primary.default);
    display_color_entry("On Primary", &palette.on_primary.default);
    display_color_entry("Primary Container", &palette.primary_container.default);
    display_color_entry("On Primary Container", &palette.on_primary_container.default);
    display_color_entry("Secondary", &palette.secondary.default);
    display_color_entry("On Secondary", &palette.on_secondary.default);
    display_color_entry("Secondary Container", &palette.secondary_container.default);
    display_color_entry("On Secondary Container", &palette.on_secondary_container.default);
    display_color_entry("Tertiary", &palette.tertiary.default);
    display_color_entry("On Tertiary", &palette.on_tertiary.default);
    display_color_entry("Tertiary Container", &palette.tertiary_container.default);
    display_color_entry("On Tertiary Container", &palette.on_tertiary_container.default);
    display_color_entry("Error", &palette.error.default);
    display_color_entry("On Error", &palette.on_error.default);
    display_color_entry("Error Container", &palette.error_container.default);
    display_color_entry("On Error Container", &palette.on_error_container.default);
    display_color_entry("Background", &palette.background.default);
    display_color_entry("On Background", &palette.on_background.default);
    display_color_entry("Surface", &palette.surface.default);
    display_color_entry("On Surface", &palette.on_surface.default);
    display_color_entry("Surface Variant", &palette.surface_variant.default);
    display_color_entry("On Surface Variant", &palette.on_surface_variant.default);
    display_color_entry("Surface Container Lowest", &palette.surface_container_lowest.default);
    display_color_entry("Surface Container Low", &palette.surface_container_low.default);
    display_color_entry("Surface Container", &palette.surface_container.default);
    display_color_entry("Surface Container High", &palette.surface_container_high.default);
    display_color_entry("Surface Container Highest", &palette.surface_container_highest.default);
    display_color_entry("Outline", &palette.outline.default);
    display_color_entry("Outline Variant", &palette.outline_variant.default);
    display_color_entry("Shadow", &palette.shadow.default);

    Ok(())
}

/// Display a single color entry with its various representations
fn display_color_entry(name: &str, color: &crate::theme::ColorFormat) {
    // Print color name
    println!("{}", name.bold());

    // Print color sample using terminal background color
    // Creating a color block to visualize the color
    let color_sample = format!(
        "{:<15}",
        format!("RGB({},{},{})", color.red, color.green, color.blue)
            .truecolor(color.red, color.green, color.blue)
    );
    println!("  Color Sample: {}", color_sample);

    // Print various color formats
    println!("  HEX:   {}", color.hex.bold());
    println!("  RGB:   {}", color.rgb);
    println!("  RGBA:  {}", color.rgba);
    println!("  HSL:   {}", color.hsl);
    println!("  HSLA:  {}", color.hsla);
    println!("  Hex (stripped): #{}", color.hex_stripped);
    println!("  Red:   {}", color.red);
    println!("  Green: {}", color.green);
    println!("  Blue:  {}", color.blue);
    println!("  Alpha: {}", color.alpha);
    println!("  Hue:   {:.1}", color.hue);
    println!("  Saturation: {:.1}%", color.saturation);
    println!("  Lightness: {:.1}%", color.lightness);
    println!();
}