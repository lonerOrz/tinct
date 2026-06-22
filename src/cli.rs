use clap::Parser;

use tinct::SchemeTypeCli;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    /// Path to the TOML config file
    #[arg(short, long)]
    pub config: Option<String>,

    /// Path to theme.json file or theme name in themes/ folder
    #[arg(short, long)]
    pub theme: Option<String>,

    /// Seed color for generating palette (e.g., "#7aa2f7")
    #[arg(short, long)]
    pub seed: Option<String>,

    /// Path to wallpaper image for color extraction (PNG/JPG/WebP)
    #[arg(short = 'i', long)]
    pub image: Option<String>,

    /// Color scheme type for image extraction (tonal-spot, vibrant, faithful, etc.)
    /// If not provided, uses config file or defaults to tonal-spot
    #[arg(long, value_name = "SCHEME")]
    pub scheme_type: Option<SchemeTypeCli>,

    /// Theme mode override
    #[arg(short, long, value_enum, default_value = "dark")]
    pub mode: tinct::Mode,

    /// Show color preview instead of processing templates
    #[arg(short, long)]
    pub preview: bool,

    /// Skip sending ANSI escape sequences to update terminal colors
    #[arg(long)]
    pub skip_sequences: bool,

    /// Logging level: quiet, normal, verbose
    #[arg(long, value_enum, default_value = "normal")]
    pub log_level: tinct::LogLevel,
}

impl CliArgs {
    /// Validate that either --theme, --seed, or --image is provided
    pub fn validate(&self) -> Result<(), String> {
        let has_theme = self.theme.is_some();
        let has_seed = self.seed.is_some();
        let has_image = self.image.is_some();

        let count = [has_theme, has_seed, has_image]
            .iter()
            .filter(|&&x| x)
            .count();

        if count == 0 {
            return Err("Either --theme, --seed, or --image must be provided".to_string());
        }
        if count > 1 {
            return Err(
                "--theme, --seed, and --image are mutually exclusive, use only one".to_string(),
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tinct::image::SchemeType;

    #[test]
    fn test_cli_args_derive() {
        let args = CliArgs {
            config: Some("custom.toml".to_string()),
            theme: Some("mytheme".to_string()),
            seed: None,
            image: None,
            scheme_type: Some(SchemeTypeCli(SchemeType::TonalSpot)),
            mode: tinct::Mode::Light,
            preview: true,
            skip_sequences: false,
            log_level: tinct::LogLevel::Verbose,
        };
        assert_eq!(args.theme, Some("mytheme".to_string()));
        assert_eq!(args.mode, tinct::Mode::Light);
        assert!(args.preview);
    }

    #[test]
    fn test_log_level_variants() {
        let _ = tinct::LogLevel::Quiet;
        let _ = tinct::LogLevel::Normal;
        let _ = tinct::LogLevel::Verbose;
    }
}
