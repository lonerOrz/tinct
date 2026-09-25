use clap::{ArgGroup, Parser};
use std::path::PathBuf;

use tinct::SchemeType;

#[derive(Parser, Debug)]
#[command(version, about = "Material Design 3 theme injector", long_about = None)]
#[command(group(
    ArgGroup::new("source")
        .required(true)
        .args(["theme", "seed", "image"])
))]
pub struct CliArgs {
    /// Path to the TOML config file [default: $XDG_CONFIG_HOME/tinct/config.toml]
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Path to theme.json file or theme name in themes/ folder
    #[arg(short, long)]
    pub theme: Option<String>,

    /// Seed color for generating palette (e.g., "#7aa2f7")
    #[arg(short, long)]
    pub seed: Option<String>,

    /// Path to wallpaper image for color extraction (PNG/JPG/WebP)
    #[arg(short = 'i', long, value_name = "IMAGE")]
    pub image: Option<PathBuf>,

    /// MD3 scheme variant used to build the palette (tonal-spot, vibrant, content, ...).
    /// For image sources it also selects the extraction pipeline. If not provided, the
    /// config file value (`[image] scheme_type`) is used, defaulting to tonal-spot.
    #[arg(long, value_name = "SCHEME")]
    pub scheme_type: Option<SchemeType>,

    /// Theme mode override
    #[arg(short, long, value_enum, default_value = "dark")]
    pub mode: tinct::Mode,

    /// Show color preview instead of processing templates
    #[arg(short, long)]
    pub preview: bool,

    /// Logging level: quiet, normal, verbose
    #[arg(long, value_enum, default_value = "normal")]
    pub log_level: tinct::LogLevel,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arg_group_requires_exactly_one_source() {
        // No source at all → error.
        assert!(CliArgs::try_parse_from(["tinct"]).is_err());

        // Two sources → error (mutually exclusive).
        assert!(CliArgs::try_parse_from(["tinct", "--seed", "#fff", "--theme", "x"]).is_err());

        // Exactly one → ok.
        assert!(CliArgs::try_parse_from(["tinct", "--seed", "#fff"]).is_ok());
        assert!(CliArgs::try_parse_from(["tinct", "-t", "mytheme"]).is_ok());
        assert!(CliArgs::try_parse_from(["tinct", "--image", "wall.png"]).is_ok());
    }
}
