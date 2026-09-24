//! Configuration loading, parsing, and path canonicalization.
//!
//! [`Config::load`] is the single entry point: it locates the config file
//! (honouring `$XDG_CONFIG_HOME`), reads it, parses it and rewrites every
//! path relative to the config file's own directory so behaviour is identical
//! no matter which directory `tinct` is run from.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::core::{Error, Result};
use crate::image::{ImageFilter, Quantizer, SchemeType};

/// Image extraction configuration.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ImageConfig {
    /// Scheme type for color extraction/derivation (tonal-spot, vibrant, ...).
    #[serde(default)]
    pub scheme_type: Option<SchemeType>,

    /// Maximum number of representative clusters returned for terminal hue
    /// snapping. `None` uses the algorithm default.
    #[serde(default)]
    pub max_colors: Option<usize>,

    /// Clusters whose share of the image is below this fraction (0.0..=1.0)
    /// are discarded. `0.0` keeps every cluster.
    #[serde(default)]
    pub min_population: f64,

    /// Optional pre-quantization pixel filter.
    #[serde(default)]
    pub filter: ImageFilter,

    /// Quantizer/refinement used by the M3 extraction pipeline.
    #[serde(default)]
    pub quantizer: Quantizer,
}

/// Color-generation tuning knobs.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct AlgorithmConfig {
    /// Saturation adjustment (-100 to 100)
    #[serde(default)]
    pub saturation_adjustment: i8,

    /// Hue shift (-180 to 180)
    #[serde(default)]
    pub hue_shift: i16,

    /// MD3 contrast level (-1.0 to 1.0)
    #[serde(default)]
    pub contrast_level: f64,

    /// Force the seed to this HCT tone (0..=100) before generation. `None`
    /// keeps the seed's own tone (the official behaviour).
    #[serde(default)]
    pub seed_tone: Option<f64>,

    /// Minimum seed chroma (0..=120). Greyscale seeds are lifted to it so a
    /// near-neutral input still yields a chromatic palette. `0.0` disables it.
    #[serde(default)]
    pub chroma_floor: f64,

    /// MD3 variant override for dark mode. Takes precedence over the base
    /// `--scheme-type`/`[image] scheme_type` for this mode only.
    #[serde(default)]
    pub variant_dark: Option<SchemeType>,

    /// MD3 variant override for light mode (see [`Self::variant_dark`]).
    #[serde(default)]
    pub variant_light: Option<SchemeType>,
}

/// Which side of the terminal the ANSI palette is tuned for.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum AnsiPalette {
    /// Bright variants are lighter than their normal counterparts.
    #[default]
    Dark,
    /// Bright variants are darker, for light-background terminals.
    Light,
}

/// Overrides for the six chromatic ANSI anchors.
///
/// Each value is a hex color; its CIE LCh hue/lightness/chroma replace the
/// built-in wallust anchor, so a slot with no matching source color renders
/// from this color instead. Invalid hex values are ignored with a warning.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AnsiAnchors {
    #[serde(default)]
    pub red: Option<String>,
    #[serde(default)]
    pub green: Option<String>,
    #[serde(default)]
    pub yellow: Option<String>,
    #[serde(default)]
    pub blue: Option<String>,
    #[serde(default)]
    pub magenta: Option<String>,
    #[serde(default)]
    pub cyan: Option<String>,
}

/// Terminal (ANSI 16-color) tuning knobs.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnsiConfig {
    /// Whether bright variants are lighter (`dark`) or darker (`light`).
    #[serde(default)]
    pub palette: AnsiPalette,

    /// Blend weight of the source color against the slot anchor in `0.0..=1.0`.
    /// `0.0` = anchor only, `1.0` = source only. Defaults to wallust's `2/3`.
    #[serde(default = "default_source_weight")]
    pub source_weight: f32,

    /// Candidates at or below this CIE LCh chroma are ignored for hue snapping.
    #[serde(default = "default_chroma_threshold")]
    pub chroma_threshold: f32,

    /// Lightness delta applied to `bright_*` variants.
    #[serde(default = "default_brightness_delta")]
    pub brightness_delta: f32,

    /// Chroma multiplier applied to `bright_*` variants.
    #[serde(default = "default_bright_chroma_multiplier")]
    pub bright_chroma_multiplier: f32,

    /// Minimum WCAG contrast ratio between a chromatic slot and the background.
    #[serde(default = "default_contrast_target")]
    pub contrast_target: f32,

    /// Pin the terminal background (ANSI 0). Invalid hex is ignored.
    #[serde(default)]
    pub background: Option<String>,

    /// Pin the terminal foreground (ANSI 7). Invalid hex is ignored.
    #[serde(default)]
    pub foreground: Option<String>,

    /// Per-slot anchor color overrides.
    #[serde(default)]
    pub anchors: AnsiAnchors,
}

impl Default for AnsiConfig {
    fn default() -> Self {
        Self {
            palette: AnsiPalette::default(),
            source_weight: default_source_weight(),
            chroma_threshold: default_chroma_threshold(),
            brightness_delta: default_brightness_delta(),
            bright_chroma_multiplier: default_bright_chroma_multiplier(),
            contrast_target: default_contrast_target(),
            background: None,
            foreground: None,
            anchors: AnsiAnchors::default(),
        }
    }
}

fn default_source_weight() -> f32 {
    2.0 / 3.0
}

fn default_chroma_threshold() -> f32 {
    12.0
}

fn default_brightness_delta() -> f32 {
    8.0
}

fn default_bright_chroma_multiplier() -> f32 {
    1.2
}

fn default_contrast_target() -> f32 {
    3.0
}

/// A single template-injection target.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigSection {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    #[serde(rename = "post_hook", default)]
    pub post_hook: Option<String>,
}

/// Fully resolved configuration.
///
/// Constructed by [`Config::load`]; every path is already absolute (tilde
/// expanded, relative paths bound to [`Config::config_dir`]). Template groups
/// are flattened into dotted keys such as `"templates.alacritty"`.
#[derive(Debug, Clone)]
pub struct Config {
    /// Absolute path of the config file that was loaded.
    pub config_path: PathBuf,
    /// Directory containing the config file — the base for relative paths.
    pub config_dir: PathBuf,
    pub algorithm: AlgorithmConfig,
    pub image: ImageConfig,
    pub ansi: AnsiConfig,
    pub sections: HashMap<String, ConfigSection>,
}

/// Result of [`Config::load_or_init`].
#[derive(Debug)]
pub enum ConfigLoad {
    /// An existing config file was found and parsed.
    Loaded(Box<Config>),
    /// No config existed at the default location; a commented default was
    /// written here. The caller should tell the user and stop.
    CreatedDefault(PathBuf),
}

impl Config {
    /// Commented default configuration written on first run.
    ///
    /// It parses to exactly the built-in defaults, so a freshly scaffolded
    /// file can be edited incrementally without breaking anything.
    pub const DEFAULT_TOML: &'static str = r##"# tinct configuration
#
# Generated on first run. Every option is shown with its default value;
# uncomment or override what you need, then run tinct again.
#
# Paths in [templates.*] are resolved relative to this file's directory,
# so tinct behaves the same no matter where it is invoked from.

# ---------------------------------------------------------------------------
# [algorithm] - color generation tuning (feeds both the UI and terminal palettes)
# ---------------------------------------------------------------------------
[algorithm]
# Saturation adjustment applied to the seed, in percent (-100..=100).
saturation_adjustment = 0
# Hue shift applied to the seed, in degrees (-180..=180).
hue_shift = 0
# Material Design 3 contrast level (-1.0..=1.0).
contrast_level = 0.0
# Minimum seed chroma (0..=120); lifts near-greyscale seeds to stay chromatic.
chroma_floor = 0.0
# Force the seed to this HCT tone (0..=100). Unset keeps the seed's own tone.
# seed_tone = 50.0
# Per-mode MD3 variant overrides (tonal-spot, vibrant, content, faithful, ...).
# variant_dark = "vibrant"
# variant_light = "tonal-spot"

# ---------------------------------------------------------------------------
# [image] - wallpaper extraction (used with --image)
# ---------------------------------------------------------------------------
[image]
# MD3 scheme used for both extraction and palette derivation.
scheme_type = "tonal-spot"
# Maximum number of representative clusters. Unset uses the algorithm default.
# max_colors = 32
# Discard clusters covering less than this fraction of the image (0.0..=1.0).
min_population = 0.0
# Optional pre-quantization pixel filter: "none", "saturation", "brightness".
filter = "none"
# Quantizer/refinement: "wsmeans" (accurate) or "wu" (fast).
quantizer = "wsmeans"

# ---------------------------------------------------------------------------
# [ansi] - terminal 16-color palette tuning
# ---------------------------------------------------------------------------
[ansi]
# Bright variants are lighter ("dark") or darker ("light").
palette = "dark"
# Blend weight of the source color against the slot anchor (0.0..=1.0).
source_weight = 0.6667
# Candidates at or below this CIE LCh chroma are ignored for hue snapping.
chroma_threshold = 12.0
# Lightness delta applied to bright_* variants.
brightness_delta = 8.0
# Chroma multiplier applied to bright_* variants.
bright_chroma_multiplier = 1.2
# Minimum WCAG contrast ratio between a chromatic slot and the background.
contrast_target = 3.0
# Pin the terminal background (ANSI 0) and foreground (ANSI 7).
# background = "#101010"
# foreground = "#F0F0F0"

# Override the six chromatic anchors. Their CIE LCh hue/lightness/chroma are
# taken from these colors; unset anchors use the built-in wallust values.
[ansi.anchors]
# red = "#E06C75"
# green = "#98C379"
# yellow = "#E5C07B"
# blue = "#61AFEF"
# magenta = "#C678DD"
# cyan = "#56B6C2"

# ---------------------------------------------------------------------------
# Template injection targets
# ---------------------------------------------------------------------------
# Each section needs input_path (template) and output_path (rendered file);
# post_hook is optional and runs after the file is written.
#
# [templates.alacritty]
# input_path = "./templates/alacritty.toml"
# output_path = "~/.config/alacritty/colors.toml"
# post_hook = "pkill -USR1 alacritty"
"##;

    /// Unified entry point: locate, read, parse and canonicalize a config.
    ///
    /// When `custom_path` is `None`, the default location is used:
    /// `$XDG_CONFIG_HOME/tinct/config.toml`, falling back to
    /// `$HOME/.config/tinct/config.toml`.
    pub fn load(custom_path: Option<&Path>) -> Result<Self> {
        let config_path = match custom_path {
            Some(p) => p.to_path_buf(),
            None => Self::default_config_path()?,
        };

        let content = fs::read_to_string(&config_path).map_err(|e| {
            Error::Config(format!(
                "Could not read config file '{}': {}",
                config_path.display(),
                e
            ))
        })?;

        // Canonicalize so `config_dir` is absolute even for relative `--config`.
        let config_path = fs::canonicalize(&config_path).unwrap_or(config_path);

        Self::parse(&content, &config_path)
    }

    /// Load the config, scaffolding a commented default when it is missing.
    ///
    /// The default is only ever created at the standard location (no
    /// `--config`). An explicitly supplied path that does not exist is an
    /// error: tinct never guesses where to write.
    pub fn load_or_init(custom_path: Option<&Path>) -> Result<ConfigLoad> {
        match custom_path {
            Some(path) => Self::load_or_init_at(path.to_path_buf(), true),
            None => Self::load_or_init_at(Self::default_config_path()?, false),
        }
    }

    fn load_or_init_at(path: PathBuf, explicit: bool) -> Result<ConfigLoad> {
        if path.exists() {
            return Self::load(Some(&path)).map(|c| ConfigLoad::Loaded(Box::new(c)));
        }

        if explicit {
            return Err(Error::Config(format!(
                "Config file not found: {}",
                path.display()
            )));
        }

        Self::write_default(&path)?;
        Ok(ConfigLoad::CreatedDefault(path))
    }

    /// Write [`Self::DEFAULT_TOML`] to `path`, creating parent directories.
    pub fn write_default(path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                Error::Config(format!(
                    "Could not create config directory '{}': {}",
                    parent.display(),
                    e
                ))
            })?;
        }

        fs::write(path, Self::DEFAULT_TOML).map_err(|e| {
            Error::Config(format!(
                "Could not write default config '{}': {}",
                path.display(),
                e
            ))
        })
    }

    /// Default config file location, following the XDG Base Directory spec.
    pub fn default_config_path() -> Result<PathBuf> {
        let base = config_home().ok_or_else(|| {
            Error::Config("Neither $XDG_CONFIG_HOME nor $HOME is set".to_string())
        })?;
        Ok(base.join("tinct").join("config.toml"))
    }

    /// Parse TOML content and bind every relative path to `config_file`'s parent.
    pub fn parse(content: &str, config_file: &Path) -> Result<Self> {
        let raw: toml::Table = toml::from_str(content)
            .map_err(|e| Error::Config(format!("Invalid TOML format: {}", e)))?;

        let config_dir = config_file
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();

        let algorithm = match raw.get("algorithm") {
            Some(value) => value
                .clone()
                .try_into::<AlgorithmConfig>()
                .map_err(|e| Error::Config(format!("Invalid [algorithm] section: {}", e)))?,
            None => AlgorithmConfig::default(),
        };

        let image = match raw.get("image") {
            Some(value) => value
                .clone()
                .try_into::<ImageConfig>()
                .map_err(|e| Error::Config(format!("Invalid [image] section: {}", e)))?,
            None => ImageConfig::default(),
        };

        let ansi = match raw.get("ansi") {
            Some(value) => value
                .clone()
                .try_into::<AnsiConfig>()
                .map_err(|e| Error::Config(format!("Invalid [ansi] section: {}", e)))?,
            None => AnsiConfig::default(),
        };

        let mut sections = HashMap::new();
        for (group_key, group_val) in &raw {
            // Reserved configuration tables, not template groups.
            if group_key == "algorithm" || group_key == "image" || group_key == "ansi" {
                continue;
            }

            let toml::Value::Table(group) = group_val else {
                continue;
            };

            for (section_name, section_val) in group {
                // Deserialize directly — a section missing `input_path` or
                // `output_path` simply fails and is skipped.
                let Ok(mut section) = section_val.clone().try_into::<ConfigSection>() else {
                    continue;
                };
                canonicalize_section_paths(&mut section, &config_dir);
                sections.insert(format!("{}.{}", group_key, section_name), section);
            }
        }

        Ok(Config {
            config_path: config_file.to_path_buf(),
            config_dir,
            algorithm,
            image,
            ansi,
            sections,
        })
    }
}

/// Expand `~` and bind a relative path to `base_dir`.
fn resolve_path(path: &Path, base_dir: &Path) -> PathBuf {
    let expanded = shellexpand::tilde(&path.to_string_lossy()).into_owned();
    let expanded = Path::new(&expanded);
    if expanded.is_absolute() {
        expanded.to_path_buf()
    } else {
        base_dir.join(expanded)
    }
}

/// Rewrite a section's paths so they are absolute and config-relative.
fn canonicalize_section_paths(section: &mut ConfigSection, base_dir: &Path) {
    section.input_path = resolve_path(&section.input_path, base_dir);
    section.output_path = resolve_path(&section.output_path, base_dir);

    // A relative hook script (`./reload.sh`) is bound to the config directory.
    if let Some(hook) = section.post_hook.as_mut()
        && let Some(relative) = hook.strip_prefix("./")
    {
        let joined = base_dir.join(relative);
        *hook = if joined.exists() {
            fs::canonicalize(&joined).unwrap_or(joined)
        } else {
            joined
        }
        .to_string_lossy()
        .into_owned();
    }
}

/// The user's config base directory: `$XDG_CONFIG_HOME`, else `$HOME/.config`.
fn config_home() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME")
        && !xdg.is_empty()
    {
        return Some(PathBuf::from(xdg));
    }
    std::env::var("HOME")
        .ok()
        .filter(|h| !h.is_empty())
        .map(|home| PathBuf::from(home).join(".config"))
}

/// Resolve a theme name or path to an existing file.
///
/// Search order:
/// 1. Absolute path (as given)
/// 2. Relative path from the current directory
/// 3. User themes directory (`$XDG_CONFIG_HOME/tinct/themes/<name>.json`,
///    falling back to `~/.config/...`)
pub fn resolve_theme_path(theme_name: &str) -> Result<PathBuf> {
    let candidate = Path::new(theme_name);
    if candidate.is_absolute() && candidate.exists() {
        return Ok(candidate.to_path_buf());
    }

    // Relative path (must be a file, not a directory)
    if candidate.exists() && candidate.is_file() {
        return Ok(fs::canonicalize(candidate).unwrap_or_else(|_| candidate.to_path_buf()));
    }

    // User themes directory
    if let Some(base) = config_home() {
        let user_theme = base
            .join("tinct")
            .join("themes")
            .join(format!("{}.json", theme_name));
        if user_theme.exists() {
            return Ok(user_theme);
        }
    }

    Err(Error::Config(format!(
        "Theme '{}' not found in any of these locations:\n  - Current directory\n  - Project themes/ directory\n  - $XDG_CONFIG_HOME/tinct/themes/ (or ~/.config/tinct/themes/)",
        theme_name
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(content: &str) -> Config {
        Config::parse(content, Path::new("/etc/tinct/config.toml")).unwrap()
    }

    #[test]
    fn test_config_parse_minimal() {
        let config = parse(
            r#"
[templates.test]
input_path = "input.css"
output_path = "output.css"
"#,
        );
        assert!(config.sections.contains_key("templates.test"));
    }

    #[test]
    fn test_config_parse_with_algorithm() {
        let config = parse(
            r#"
[algorithm]
saturation_adjustment = 10
hue_shift = 15

[templates.test]
input_path = "input.css"
output_path = "output.css"
"#,
        );
        assert_eq!(config.algorithm.saturation_adjustment, 10);
        assert_eq!(config.algorithm.hue_shift, 15);
    }

    #[test]
    fn test_config_parse_multiple_sections() {
        let config = parse(
            r#"
[section1.test]
input_path = "input1.css"
output_path = "output1.css"

[section2.production]
input_path = "input2.css"
output_path = "output2.css"
post_hook = "./script.sh"
"#,
        );
        assert!(config.sections.contains_key("section1.test"));
        assert!(config.sections.contains_key("section2.production"));
        assert_eq!(
            config.sections["section2.production"].post_hook.as_deref(),
            Some("/etc/tinct/script.sh")
        );
    }

    #[test]
    fn test_algorithm_config_defaults() {
        let config = parse(
            r#"
[algorithm]
"#,
        );
        assert_eq!(config.algorithm.saturation_adjustment, 0);
        assert_eq!(config.algorithm.hue_shift, 0);
        assert_eq!(config.algorithm.contrast_level, 0.0);
    }

    #[test]
    fn test_invalid_algorithm_value_is_an_error() {
        let result = Config::parse(
            r#"
[algorithm]
saturation_adjustment = 999
"#,
            Path::new("/etc/tinct/config.toml"),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("[algorithm]"));
    }

    #[test]
    fn test_config_parse_missing_required_fields() {
        let config = parse(
            r#"
[incomplete.test]
some_field = "value"
"#,
        );
        assert!(config.sections.is_empty());
    }

    #[test]
    fn test_relative_paths_bind_to_config_dir() {
        let config = parse(
            r#"
[templates.test]
input_path = "input.css"
output_path = "dist/output.css"
"#,
        );
        let section = &config.sections["templates.test"];
        assert_eq!(section.input_path, PathBuf::from("/etc/tinct/input.css"));
        assert_eq!(
            section.output_path,
            PathBuf::from("/etc/tinct/dist/output.css")
        );
    }

    #[test]
    fn test_absolute_paths_left_untouched() {
        let config = parse(
            r#"
[templates.test]
input_path = "/opt/tpl/input.css"
output_path = "/opt/out/output.css"
"#,
        );
        let section = &config.sections["templates.test"];
        assert_eq!(section.input_path, PathBuf::from("/opt/tpl/input.css"));
        assert_eq!(section.output_path, PathBuf::from("/opt/out/output.css"));
    }

    #[test]
    fn test_config_parse_with_nested_groups() {
        let config = parse(
            r#"
[frontend.templates]
input_path = "src/templates/*.hbs"
output_path = "dist/"

[backend.templates]
input_path = "server/**/*.tmpl"
output_path = "generated/"
"#,
        );
        assert!(config.sections.contains_key("frontend.templates"));
        assert!(config.sections.contains_key("backend.templates"));
    }

    #[test]
    fn test_config_parses_image_scheme_type() {
        let config = parse(
            r#"
[image]
scheme_type = "vibrant"
"#,
        );
        assert_eq!(config.image.scheme_type, Some(SchemeType::Vibrant));
    }

    #[test]
    fn test_config_parses_ansi_section() {
        let config = parse(
            r##"
[ansi]
palette = "light"
source_weight = 0.5
chroma_threshold = 20.0
brightness_delta = 10.0
bright_chroma_multiplier = 1.5
contrast_target = 4.5
background = "#101010"
foreground = "#F0F0F0"

[ansi.anchors]
red = "#E06C75"
"##,
        );
        assert_eq!(config.ansi.palette, AnsiPalette::Light);
        assert!((config.ansi.source_weight - 0.5).abs() < f32::EPSILON);
        assert!((config.ansi.chroma_threshold - 20.0).abs() < f32::EPSILON);
        assert!((config.ansi.brightness_delta - 10.0).abs() < f32::EPSILON);
        assert!((config.ansi.bright_chroma_multiplier - 1.5).abs() < f32::EPSILON);
        assert!((config.ansi.contrast_target - 4.5).abs() < f32::EPSILON);
        assert_eq!(config.ansi.background.as_deref(), Some("#101010"));
        assert_eq!(config.ansi.foreground.as_deref(), Some("#F0F0F0"));
        assert_eq!(config.ansi.anchors.red.as_deref(), Some("#E06C75"));
        assert_eq!(config.ansi.anchors.green, None);
    }

    #[test]
    fn test_ansi_defaults_and_reserved_table() {
        let config = parse(
            r#"
[templates.test]
input_path = "in.css"
output_path = "out.css"
"#,
        );
        assert_eq!(config.ansi.palette, AnsiPalette::Dark);
        assert!((config.ansi.source_weight - 2.0 / 3.0).abs() < 1e-6);
        assert!((config.ansi.chroma_threshold - 12.0).abs() < f32::EPSILON);
        assert!((config.ansi.brightness_delta - 8.0).abs() < f32::EPSILON);
        assert!((config.ansi.bright_chroma_multiplier - 1.2).abs() < f32::EPSILON);
        assert!((config.ansi.contrast_target - 3.0).abs() < f32::EPSILON);
        // [ansi] must not be mistaken for a template group.
        assert!(!config.sections.contains_key("ansi.palette"));
        assert_eq!(config.sections.len(), 1);
    }

    #[test]
    fn test_config_parses_algorithm_extras() {
        let config = parse(
            r#"
[algorithm]
seed_tone = 50.0
chroma_floor = 12.0
variant_dark = "vibrant"
variant_light = "tonal-spot"
"#,
        );
        assert_eq!(config.algorithm.seed_tone, Some(50.0));
        assert!((config.algorithm.chroma_floor - 12.0).abs() < f64::EPSILON);
        assert_eq!(config.algorithm.variant_dark, Some(SchemeType::Vibrant));
        assert_eq!(config.algorithm.variant_light, Some(SchemeType::TonalSpot));
    }

    #[test]
    fn test_config_parses_image_extras() {
        let config = parse(
            r#"
[image]
max_colors = 16
min_population = 0.02
filter = "saturation"
quantizer = "wu"
"#,
        );
        assert_eq!(config.image.max_colors, Some(16));
        assert!((config.image.min_population - 0.02).abs() < f64::EPSILON);
        assert_eq!(config.image.filter, ImageFilter::Saturation);
        assert_eq!(config.image.quantizer, Quantizer::Wu);
    }

    #[test]
    fn test_resolve_theme_path_missing() {
        let result = resolve_theme_path("nonexistent_theme_xyz");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_default_toml_parses_with_defaults() {
        let config = Config::parse(Config::DEFAULT_TOML, Path::new("/etc/tinct/config.toml"))
            .expect("default template must be valid TOML");
        assert_eq!(config.algorithm.saturation_adjustment, 0);
        assert_eq!(config.algorithm.hue_shift, 0);
        assert_eq!(config.algorithm.chroma_floor, 0.0);
        assert_eq!(config.image.scheme_type, Some(SchemeType::TonalSpot));
        assert_eq!(config.image.filter, ImageFilter::None);
        assert_eq!(config.image.quantizer, Quantizer::Wsmeans);
        assert_eq!(config.ansi.palette, AnsiPalette::Dark);
        assert!((config.ansi.source_weight - 2.0 / 3.0).abs() < 1e-3);
        // Every template section is commented out in the scaffold.
        assert!(config.sections.is_empty());
    }

    #[test]
    fn test_write_default_creates_parents() {
        let dir = std::env::temp_dir().join(format!("tinct-cfg-write-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let path = dir.join("nested").join("config.toml");

        Config::write_default(&path).unwrap();
        assert!(path.exists());

        let content = fs::read_to_string(&path).unwrap();
        assert!(Config::parse(&content, &path).is_ok());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_or_init_creates_default_at_missing_path() {
        let dir = std::env::temp_dir().join(format!("tinct-cfg-init-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let path = dir.join("config.toml");

        match Config::load_or_init_at(path.clone(), false).unwrap() {
            ConfigLoad::CreatedDefault(created) => assert_eq!(created, path),
            ConfigLoad::Loaded(_) => panic!("expected a freshly created config"),
        }
        assert!(path.exists());

        // A second call now finds and loads the file.
        assert!(matches!(
            Config::load_or_init_at(path, false).unwrap(),
            ConfigLoad::Loaded(_)
        ));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_or_init_errors_for_explicit_missing_path() {
        let path = std::env::temp_dir().join("tinct-cfg-definitely-missing.toml");
        let _ = fs::remove_file(&path);

        let result = Config::load_or_init_at(path, true);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
