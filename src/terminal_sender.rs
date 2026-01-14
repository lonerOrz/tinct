use crate::palette_generator::Palette;
use std::fs::File;
use std::io::Write;

/// Trait for platform-specific terminal sequence sending
pub trait TerminalSender {
    fn send_sequences(&self, palette: &Palette) -> Result<(), String>;
}

/// ANSI escape sequence generator
pub struct AnsiSequenceGenerator;

impl AnsiSequenceGenerator {
    /// Generate ANSI escape sequence for setting a color
    pub fn set_color(hex_color: &str, index: u32) -> String {
        format!("\x1B]4;{index};{hex_color}\x1B\\")
    }

    /// Generate ANSI escape sequence for special colors
    pub fn set_special(hex_color: &str, index: u32) -> String {
        format!("\x1B]{index};{hex_color}\x1B\\")
    }

    /// Generate all terminal sequences from a palette
    pub fn generate_sequences(palette: &Palette) -> String {
        // Generate sequences for standard colors (0-15)
        let standard_colors = [
            Self::set_color(&palette.black.default.hex, 0), // Black
            Self::set_color(&palette.red.default.hex, 1),   // Red
            Self::set_color(&palette.green.default.hex, 2), // Green
            Self::set_color(&palette.yellow.default.hex, 3), // Yellow
            Self::set_color(&palette.blue.default.hex, 4),  // Blue
            Self::set_color(&palette.magenta.default.hex, 5), // Magenta
            Self::set_color(&palette.cyan.default.hex, 6),  // Cyan
            Self::set_color(&palette.white.default.hex, 7), // White
            Self::set_color(&palette.bright_black.default.hex, 8), // Bright Black
            Self::set_color(&palette.bright_red.default.hex, 9), // Bright Red
            Self::set_color(&palette.bright_green.default.hex, 10), // Bright Green
            Self::set_color(&palette.bright_yellow.default.hex, 11), // Bright Yellow
            Self::set_color(&palette.bright_blue.default.hex, 12), // Bright Blue
            Self::set_color(&palette.bright_magenta.default.hex, 13), // Bright Magenta
            Self::set_color(&palette.bright_cyan.default.hex, 14), // Bright Cyan
            Self::set_color(&palette.bright_white.default.hex, 15), // Bright White
        ]
        .join("");

        // Generate sequences for special colors
        let bg = [
            Self::set_special(&palette.background.default.hex, 11), // Background
            Self::set_special(&palette.background.default.hex, 19), // Another background variant
            Self::set_color(&palette.background.default.hex, 232),  // Background color as color 232
            Self::set_color(&palette.background.default.hex, 257),  // Background color as color 257
            Self::set_special(&palette.background.default.hex, 708), // Border color
        ]
        .join("");

        let fg = [
            Self::set_special(&palette.on_background.default.hex, 10), // Foreground
            Self::set_special(&palette.on_background.default.hex, 17), // Another foreground variant
            Self::set_color(&palette.on_background.default.hex, 256), // Foreground color as color 256
        ]
        .join("");

        let cursor = [
            Self::set_special(&palette.on_background.default.hex, 12), // Cursor foreground
            Self::set_special(&palette.on_background.default.hex, 13), // Mouse foreground
        ]
        .join("");

        format!("{}{}{}{}", standard_colors, bg, fg, cursor)
    }
}

#[cfg(target_family = "unix")]
pub struct UnixTerminalSender;

#[cfg(target_family = "unix")]
impl TerminalSender for UnixTerminalSender {
    fn send_sequences(&self, palette: &Palette) -> Result<(), String> {
        use glob::glob;
        use std::path::PathBuf;

        let sequences = AnsiSequenceGenerator::generate_sequences(palette);

        // Define cache path and create sequence file
        let cache_dir = std::env::var("HOME")
            .map(|home| PathBuf::from(home).join(".cache"))
            .unwrap_or_else(|_| PathBuf::from("/tmp"));

        let seq_file = cache_dir.join("tinct/sequences");

        // Create cache directory if it doesn't exist
        if let Some(parent) = seq_file.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Could not create cache directory: {}", e))?;
        }

        // Write sequences to cache file
        if let Err(e) = File::create(&seq_file).and_then(|mut o| o.write_all(sequences.as_bytes()))
        {
            eprintln!("Warning: Couldn't create sequence file: {e}");
        }

        // Define TTY pattern based on OS
        #[cfg(target_os = "macos")]
        let tty_pattern = "/dev/ttys00[0-9]*";

        #[cfg(not(target_os = "macos"))]
        let tty_pattern = "/dev/pts/[0-9]*";

        // Find all active TTY devices
        let devices = glob(tty_pattern).map_err(|e| format!("Error finding TTY devices: {}", e))?;

        // Send sequences to each device
        for entry in devices {
            match entry {
                Ok(path) => {
                    if let Err(e) = File::create(&path)
                        .and_then(|mut file| file.write_all(sequences.as_bytes()))
                    {
                        eprintln!("Warning: Could not write to {}: {}", path.display(), e);
                        continue;
                    }
                }
                Err(e) => return Err(format!("Error while sending sequences to terminals: {}", e)),
            }
        }

        Ok(())
    }
}

#[cfg(target_family = "windows")]
pub struct WindowsTerminalSender;

#[cfg(target_family = "windows")]
impl TerminalSender for WindowsTerminalSender {
    fn send_sequences(&self, palette: &Palette) -> Result<(), String> {
        // Windows implementation would use Windows Console API
        // For now, just log that this is not implemented
        eprintln!("Terminal sequence sending is not yet implemented for Windows");
        Ok(())
    }
}

/// Platform-agnostic function to send terminal sequences
pub fn send_terminal_sequences(palette: &Palette) -> Result<(), String> {
    #[cfg(target_family = "unix")]
    {
        let sender = UnixTerminalSender;
        sender.send_sequences(palette)
    }

    #[cfg(target_family = "windows")]
    {
        let sender = WindowsTerminalSender;
        sender.send_sequences(palette)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette_generator::{ColorFormat, ColorEntry};

    #[test]
    fn test_set_color_sequence() {
        let sequence = AnsiSequenceGenerator::set_color("#FF5722", 0);
        assert_eq!(sequence, "\x1B]4;0;#FF5722\x1B\\");
    }

    #[test]
    fn test_set_special_sequence() {
        let sequence = AnsiSequenceGenerator::set_special("#FF5722", 10);
        assert_eq!(sequence, "\x1B]10;#FF5722\x1B\\");
    }

    #[test]
    fn test_generate_sequences() {
        // Create a minimal palette for testing
        let color_format = ColorFormat {
            hex: "#FF5722".to_string(),
            hex_stripped: "FF5722".to_string(),
            hex8: "#FF5722FF".to_string(),
            hex8_stripped: "FF5722FF".to_string(),
            rgb: "rgb(255, 87, 34)".to_string(),
            rgba: "rgba(255, 87, 34, 1.0)".to_string(),
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

        let palette = Palette {
            primary: ColorEntry { default: color_format.clone() },
            on_primary: ColorEntry { default: color_format.clone() },
            primary_container: ColorEntry { default: color_format.clone() },
            on_primary_container: ColorEntry { default: color_format.clone() },
            primary_fixed: ColorEntry { default: color_format.clone() },
            primary_fixed_dim: ColorEntry { default: color_format.clone() },
            on_primary_fixed: ColorEntry { default: color_format.clone() },
            on_primary_fixed_variant: ColorEntry { default: color_format.clone() },
            secondary: ColorEntry { default: color_format.clone() },
            on_secondary: ColorEntry { default: color_format.clone() },
            secondary_container: ColorEntry { default: color_format.clone() },
            on_secondary_container: ColorEntry { default: color_format.clone() },
            secondary_fixed: ColorEntry { default: color_format.clone() },
            secondary_fixed_dim: ColorEntry { default: color_format.clone() },
            on_secondary_fixed: ColorEntry { default: color_format.clone() },
            on_secondary_fixed_variant: ColorEntry { default: color_format.clone() },
            tertiary: ColorEntry { default: color_format.clone() },
            on_tertiary: ColorEntry { default: color_format.clone() },
            tertiary_container: ColorEntry { default: color_format.clone() },
            on_tertiary_container: ColorEntry { default: color_format.clone() },
            tertiary_fixed: ColorEntry { default: color_format.clone() },
            tertiary_fixed_dim: ColorEntry { default: color_format.clone() },
            on_tertiary_fixed: ColorEntry { default: color_format.clone() },
            on_tertiary_fixed_variant: ColorEntry { default: color_format.clone() },
            error: ColorEntry { default: color_format.clone() },
            on_error: ColorEntry { default: color_format.clone() },
            error_container: ColorEntry { default: color_format.clone() },
            on_error_container: ColorEntry { default: color_format.clone() },
            background: ColorEntry { default: color_format.clone() },
            on_background: ColorEntry { default: color_format.clone() },
            surface: ColorEntry { default: color_format.clone() },
            on_surface: ColorEntry { default: color_format.clone() },
            surface_variant: ColorEntry { default: color_format.clone() },
            on_surface_variant: ColorEntry { default: color_format.clone() },
            surface_container_lowest: ColorEntry { default: color_format.clone() },
            surface_container_low: ColorEntry { default: color_format.clone() },
            surface_container: ColorEntry { default: color_format.clone() },
            surface_container_high: ColorEntry { default: color_format.clone() },
            surface_container_highest: ColorEntry { default: color_format.clone() },
            inverse_surface: ColorEntry { default: color_format.clone() },
            inverse_on_surface: ColorEntry { default: color_format.clone() },
            inverse_primary: ColorEntry { default: color_format.clone() },
            surface_dim: ColorEntry { default: color_format.clone() },
            surface_bright: ColorEntry { default: color_format.clone() },
            outline: ColorEntry { default: color_format.clone() },
            outline_variant: ColorEntry { default: color_format.clone() },
            shadow: ColorEntry { default: color_format.clone() },
            scrim: ColorEntry { default: color_format.clone() },
            // Terminal colors
            black: ColorEntry { default: color_format.clone() },
            red: ColorEntry { default: color_format.clone() },
            green: ColorEntry { default: color_format.clone() },
            yellow: ColorEntry { default: color_format.clone() },
            blue: ColorEntry { default: color_format.clone() },
            magenta: ColorEntry { default: color_format.clone() },
            cyan: ColorEntry { default: color_format.clone() },
            white: ColorEntry { default: color_format.clone() },
            bright_black: ColorEntry { default: color_format.clone() },
            bright_red: ColorEntry { default: color_format.clone() },
            bright_green: ColorEntry { default: color_format.clone() },
            bright_yellow: ColorEntry { default: color_format.clone() },
            bright_blue: ColorEntry { default: color_format.clone() },
            bright_magenta: ColorEntry { default: color_format.clone() },
            bright_cyan: ColorEntry { default: color_format.clone() },
            bright_white: ColorEntry { default: color_format },
        };

        let sequences = AnsiSequenceGenerator::generate_sequences(&palette);

        // Check that the sequences contain expected color values
        assert!(sequences.contains("#FF5722"));
        assert!(sequences.contains("\x1B]4;0;#FF5722\x1B\\"));
        assert!(sequences.contains("\x1B]11;#FF5722\x1B\\"));
    }
}
