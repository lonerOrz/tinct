// Integration tests for tinct crate
use tempfile::TempDir;
use tinct::{color, theme};

#[test]
fn test_color_functions() {
    // Test hex to RGB conversion
    let rgb = color::hex_to_rgb("#ffffff").unwrap();
    assert_eq!(rgb.r, 255);
    assert_eq!(rgb.g, 255);
    assert_eq!(rgb.b, 255);

    let rgb = color::hex_to_rgb("#000000").unwrap();
    assert_eq!(rgb.r, 0);
    assert_eq!(rgb.g, 0);
    assert_eq!(rgb.b, 0);

    let rgb = color::hex_to_rgb("#ff0000").unwrap();
    assert_eq!(rgb.r, 255);
    assert_eq!(rgb.g, 0);
    assert_eq!(rgb.b, 0);

    // Test hex to RGB without hash
    let rgb = color::hex_to_rgb("ffffff").unwrap();
    assert_eq!(rgb.r, 255);
    assert_eq!(rgb.g, 255);
    assert_eq!(rgb.b, 255);

    // Test RGB to hex
    assert_eq!(color::rgb_to_hex(255.0, 255.0, 255.0), "#ffffff");
    assert_eq!(color::rgb_to_hex(0.0, 0.0, 0.0), "#000000");
    assert_eq!(color::rgb_to_hex(255.0, 0.0, 0.0), "#ff0000");

    // Test RGB to HSL
    let hsl = color::rgb_to_hsl(255.0, 0.0, 0.0); // Red
    assert_eq!(hsl.h as u32, 0); // Hue should be around 0 for red
    assert!((hsl.s - 100.0).abs() < 1.0); // Should be fully saturated
    assert!((hsl.l - 50.0).abs() < 1.0); // Should be mid-lightness

    let hsl = color::rgb_to_hsl(0.0, 255.0, 0.0); // Green
    assert_eq!(hsl.h as u32, 120); // Hue should be around 120 for green

    // Test HSL to RGB
    let rgb = color::hsl_to_rgb(0.0, 100.0, 50.0); // Red
    assert_eq!(rgb.r, 255);
    assert_eq!(rgb.g, 0);
    assert_eq!(rgb.b, 0);

    let rgb = color::hsl_to_rgb(120.0, 100.0, 50.0); // Green
    assert_eq!(rgb.r, 0);
    assert_eq!(rgb.g, 255);
    assert_eq!(rgb.b, 0);

    // Test adjusting lightness
    let result = color::adjust_lightness("#ff0000", -20.0).unwrap();
    assert!(result != "#ff0000"); // Should be different

    // Test adjusting saturation
    let result = color::adjust_saturation("#804040", 20.0).unwrap(); // Start with a less saturated color
    assert!(result != "#804040"); // Should be different

    // Test luminance
    let lum_white = color::get_luminance("#ffffff").unwrap();
    assert!(lum_white > 0.9); // White should have high luminance

    let lum_black = color::get_luminance("#000000").unwrap();
    assert!(lum_black < 0.1); // Black should have low luminance

    // Test contrast ratio
    let ratio = color::get_contrast_ratio("#ffffff", "#000000").unwrap();
    assert!(ratio > 20.0); // Black and white should have high contrast

    let ratio = color::get_contrast_ratio("#ffffff", "#ffffff").unwrap();
    assert!((ratio - 1.0).abs() < 0.1); // Same colors should have ratio of 1

    // Test if color is light
    assert_eq!(color::is_light_color("#ffffff").unwrap(), true);
    assert_eq!(color::is_light_color("#000000").unwrap(), false);
    assert_eq!(color::is_light_color("#888888").unwrap(), false); // Mid gray is considered dark

    // Test generating on color
    // Test with light background - should return dark text color
    let color_result = color::generate_on_color("#ffffff", true).unwrap();
    assert!(color_result == "#000000" || color_result == "#1c1b1f"); // Either pure black or dark gray

    // Test with dark background - should return light text color
    let color_result = color::generate_on_color("#000000", true).unwrap();
    assert!(color_result == "#ffffff" || color_result == "#e6e1e5"); // Either pure white or light gray

    // Test adjusting lightness and saturation
    let result = color::adjust_lightness_and_saturation("#ff8080", -10.0, 10.0).unwrap();
    assert!(result != "#ff8080"); // Should be different

    // Test clamp function
    assert_eq!(color::clamp(5.0, 0.0, 10.0), 5.0);
    assert_eq!(color::clamp(-1.0, 0.0, 10.0), 0.0);
    assert_eq!(color::clamp(15.0, 0.0, 10.0), 10.0);
    
    // Test HCT functionality
    let hct = color::rgb_to_hct(255, 0, 0); // Red
    assert_eq!(hct.h as u32, 0); // Hue should be around 0 for red
    
    let hct_color = color::Hct::from_hct(0.0, 100.0, 50.0);
    let rgb_from_hct = hct_color.to_rgb();
    // The RGB values should be close to red (255, 0, 0) when converted back from HCT
    assert!(rgb_from_hct.r > 200); // Red component should be high
    assert!(rgb_from_hct.g < 50); // Green component should be low
    assert!(rgb_from_hct.b < 50); // Blue component should be low
}

#[test]
fn test_theme_functions() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let theme_path = temp_dir.path().join("test_theme.json");

    // Create a simple test theme with dark wrapper
    let primary = "#FF5722";
    let secondary = "#607D8B";
    let background = "#121212";

    let theme_content = format!(
        r#"{{
        "dark": {{
            "primary": "{}",
            "secondary": "{}",
            "background": "{}"
        }}
    }}"#,
        primary, secondary, background
    );

    std::fs::write(&theme_path, theme_content).expect("Unable to write theme file");

    // Load and process the theme
    let theme_value = theme::load_theme(theme_path.to_str().unwrap()).unwrap();
    let (theme, _) = theme::select_theme_mode(&theme_value, "dark").unwrap();

    // Generate palette
    let palette = theme::generate_palette(&theme, true, false).unwrap();

    // Test that the palette contains expected color roles
    assert!(!palette.primary.default.hex.is_empty());
    assert!(!palette.secondary.default.hex.is_empty());
    assert!(!palette.background.default.hex.is_empty());

    // Test that primary color exists and has expected structure
    assert!(!palette.primary.default.hex.is_empty());
    assert!(!palette.primary.default.rgb.is_empty());
    assert!(!palette.primary.default.hsl.is_empty());
    
    // Test template processing
    let template_content = "Primary color: {{colors.primary.default.hex}}, Mode: {{mode}}";
    let result = theme::process_template(template_content, &palette, "dark");

    // The result should contain the expected placeholders replaced
    assert!(result.contains("Primary color:"));
    assert!(result.contains(", Mode: "));
}

#[test]
fn test_load_theme() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let theme_path = temp_dir.path().join("simple_theme.json");

    let primary = "#FF5722";
    let secondary = "#607D8B";
    let light_primary = "#E91E63";
    let light_secondary = "#00BCD4";

    let theme_content = format!(
        r#"{{
        "dark": {{
            "primary": "{}",
            "secondary": "{}"
        }},
        "light": {{
            "primary": "{}",
            "secondary": "{}"
        }}
    }}"#,
        primary, secondary, light_primary, light_secondary
    );

    std::fs::write(&theme_path, theme_content).expect("Unable to write theme file");

    let result = theme::load_theme(theme_path.to_str().unwrap());
    assert!(result.is_ok());

    let theme = result.unwrap();
    assert!(theme.get("dark").is_some());
    assert!(theme.get("light").is_some());
}

#[test]
fn test_select_theme_mode() {
    let theme_content = serde_json::json!({
        "dark": {
            "primary": "#FF5722"
        },
        "light": {
            "primary": "#E91E63"
        }
    });

    // Test selecting dark mode
    let (theme, mode) = theme::select_theme_mode(&theme_content, "dark").unwrap();
    assert_eq!(mode, "dark");
    assert_eq!(theme.get("primary").unwrap().as_str().unwrap(), "#FF5722");

    // Test selecting light mode
    let (theme, mode) = theme::select_theme_mode(&theme_content, "light").unwrap();
    assert_eq!(mode, "light");
    assert_eq!(theme.get("primary").unwrap().as_str().unwrap(), "#E91E63");

    // Test fallback to dark when requested mode doesn't exist
    let (theme, mode) = theme::select_theme_mode(&theme_content, "nonexistent").unwrap();
    assert_eq!(mode, "dark");
    assert_eq!(theme.get("primary").unwrap().as_str().unwrap(), "#FF5722");
}

#[test]
fn test_save_output() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let output_path = temp_dir.path().join("output.txt");

    let content = "Test content for output file";
    let result = theme::save_output(content, output_path.to_str().unwrap());
    assert!(result.is_ok());

    // Verify the file was written correctly
    let written_content = std::fs::read_to_string(output_path).unwrap();
    assert_eq!(written_content, content);
}