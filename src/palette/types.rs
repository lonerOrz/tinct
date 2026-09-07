//! Core color types for palette generation

use crate::color::Color;
use std::collections::HashMap;
use std::str::FromStr;

/// Exhaustive Material Design 3 color roles.
/// String representation is snake_case to match template keys (e.g., "primary", "on_secondary_container").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumIter)]
pub enum ColorRole {
    Primary,
    OnPrimary,
    PrimaryContainer,
    OnPrimaryContainer,
    PrimaryFixed,
    PrimaryFixedDim,
    OnPrimaryFixed,
    OnPrimaryFixedVariant,
    Secondary,
    OnSecondary,
    SecondaryContainer,
    OnSecondaryContainer,
    SecondaryFixed,
    SecondaryFixedDim,
    OnSecondaryFixed,
    OnSecondaryFixedVariant,
    Tertiary,
    OnTertiary,
    TertiaryContainer,
    OnTertiaryContainer,
    TertiaryFixed,
    TertiaryFixedDim,
    OnTertiaryFixed,
    OnTertiaryFixedVariant,
    Error,
    OnError,
    ErrorContainer,
    OnErrorContainer,
    Background,
    OnBackground,
    Surface,
    OnSurface,
    SurfaceVariant,
    OnSurfaceVariant,
    SurfaceContainerLowest,
    SurfaceContainerLow,
    SurfaceContainer,
    SurfaceContainerHigh,
    SurfaceContainerHighest,
    InverseSurface,
    InverseOnSurface,
    InversePrimary,
    SurfaceDim,
    SurfaceBright,
    SurfaceTint,
    Outline,
    OutlineVariant,
    Shadow,
    Scrim,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl std::fmt::Display for ColorRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorRole::Primary => write!(f, "primary"),
            ColorRole::OnPrimary => write!(f, "on_primary"),
            ColorRole::PrimaryContainer => write!(f, "primary_container"),
            ColorRole::OnPrimaryContainer => write!(f, "on_primary_container"),
            ColorRole::PrimaryFixed => write!(f, "primary_fixed"),
            ColorRole::PrimaryFixedDim => write!(f, "primary_fixed_dim"),
            ColorRole::OnPrimaryFixed => write!(f, "on_primary_fixed"),
            ColorRole::OnPrimaryFixedVariant => write!(f, "on_primary_fixed_variant"),
            ColorRole::Secondary => write!(f, "secondary"),
            ColorRole::OnSecondary => write!(f, "on_secondary"),
            ColorRole::SecondaryContainer => write!(f, "secondary_container"),
            ColorRole::OnSecondaryContainer => write!(f, "on_secondary_container"),
            ColorRole::SecondaryFixed => write!(f, "secondary_fixed"),
            ColorRole::SecondaryFixedDim => write!(f, "secondary_fixed_dim"),
            ColorRole::OnSecondaryFixed => write!(f, "on_secondary_fixed"),
            ColorRole::OnSecondaryFixedVariant => write!(f, "on_secondary_fixed_variant"),
            ColorRole::Tertiary => write!(f, "tertiary"),
            ColorRole::OnTertiary => write!(f, "on_tertiary"),
            ColorRole::TertiaryContainer => write!(f, "tertiary_container"),
            ColorRole::OnTertiaryContainer => write!(f, "on_tertiary_container"),
            ColorRole::TertiaryFixed => write!(f, "tertiary_fixed"),
            ColorRole::TertiaryFixedDim => write!(f, "tertiary_fixed_dim"),
            ColorRole::OnTertiaryFixed => write!(f, "on_tertiary_fixed"),
            ColorRole::OnTertiaryFixedVariant => write!(f, "on_tertiary_fixed_variant"),
            ColorRole::Error => write!(f, "error"),
            ColorRole::OnError => write!(f, "on_error"),
            ColorRole::ErrorContainer => write!(f, "error_container"),
            ColorRole::OnErrorContainer => write!(f, "on_error_container"),
            ColorRole::Background => write!(f, "background"),
            ColorRole::OnBackground => write!(f, "on_background"),
            ColorRole::Surface => write!(f, "surface"),
            ColorRole::OnSurface => write!(f, "on_surface"),
            ColorRole::SurfaceVariant => write!(f, "surface_variant"),
            ColorRole::OnSurfaceVariant => write!(f, "on_surface_variant"),
            ColorRole::SurfaceContainerLowest => write!(f, "surface_container_lowest"),
            ColorRole::SurfaceContainerLow => write!(f, "surface_container_low"),
            ColorRole::SurfaceContainer => write!(f, "surface_container"),
            ColorRole::SurfaceContainerHigh => write!(f, "surface_container_high"),
            ColorRole::SurfaceContainerHighest => write!(f, "surface_container_highest"),
            ColorRole::InverseSurface => write!(f, "inverse_surface"),
            ColorRole::InverseOnSurface => write!(f, "inverse_on_surface"),
            ColorRole::InversePrimary => write!(f, "inverse_primary"),
            ColorRole::SurfaceDim => write!(f, "surface_dim"),
            ColorRole::SurfaceBright => write!(f, "surface_bright"),
            ColorRole::SurfaceTint => write!(f, "surface_tint"),
            ColorRole::Outline => write!(f, "outline"),
            ColorRole::OutlineVariant => write!(f, "outline_variant"),
            ColorRole::Shadow => write!(f, "shadow"),
            ColorRole::Scrim => write!(f, "scrim"),
            ColorRole::Black => write!(f, "black"),
            ColorRole::Red => write!(f, "red"),
            ColorRole::Green => write!(f, "green"),
            ColorRole::Yellow => write!(f, "yellow"),
            ColorRole::Blue => write!(f, "blue"),
            ColorRole::Magenta => write!(f, "magenta"),
            ColorRole::Cyan => write!(f, "cyan"),
            ColorRole::White => write!(f, "white"),
            ColorRole::BrightBlack => write!(f, "bright_black"),
            ColorRole::BrightRed => write!(f, "bright_red"),
            ColorRole::BrightGreen => write!(f, "bright_green"),
            ColorRole::BrightYellow => write!(f, "bright_yellow"),
            ColorRole::BrightBlue => write!(f, "bright_blue"),
            ColorRole::BrightMagenta => write!(f, "bright_magenta"),
            ColorRole::BrightCyan => write!(f, "bright_cyan"),
            ColorRole::BrightWhite => write!(f, "bright_white"),
        }
    }
}

/// Complete MD3 color palette stored as role→color map.
#[derive(Debug, Clone, Default)]
pub struct Palette {
    colors: HashMap<ColorRole, Color>,
}

impl Palette {
    /// Construct a new Palette from role→color pairs.
    pub fn new(colors: HashMap<ColorRole, Color>) -> Self {
        Self { colors }
    }

    /// Insert a single role.
    pub fn insert(&mut self, role: ColorRole, color: Color) {
        self.colors.insert(role, color);
    }

    /// Look up a role by name (snake_case string key).
    pub fn get(&self, role_name: &str) -> Option<&Color> {
        role_name
            .parse::<ColorRole>()
            .ok()
            .and_then(|r| self.colors.get(&r))
    }

    /// Get all roles as a string-keyed map for template rendering and preview.
    pub fn to_map(&self) -> HashMap<String, Color> {
        self.colors
            .iter()
            .map(|(role, color)| (role.to_string(), *color))
            .collect()
    }

    /// Iterate over all roles.
    pub fn iter(&self) -> impl Iterator<Item = (&ColorRole, &Color)> {
        self.colors.iter()
    }

    /// Create an empty palette.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Check if a role is present.
    pub fn contains(&self, role: &ColorRole) -> bool {
        self.colors.contains_key(role)
    }
}

impl FromStr for ColorRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "primary" => Ok(ColorRole::Primary),
            "on_primary" => Ok(ColorRole::OnPrimary),
            "primary_container" => Ok(ColorRole::PrimaryContainer),
            "on_primary_container" => Ok(ColorRole::OnPrimaryContainer),
            "primary_fixed" => Ok(ColorRole::PrimaryFixed),
            "primary_fixed_dim" => Ok(ColorRole::PrimaryFixedDim),
            "on_primary_fixed" => Ok(ColorRole::OnPrimaryFixed),
            "on_primary_fixed_variant" => Ok(ColorRole::OnPrimaryFixedVariant),
            "secondary" => Ok(ColorRole::Secondary),
            "on_secondary" => Ok(ColorRole::OnSecondary),
            "secondary_container" => Ok(ColorRole::SecondaryContainer),
            "on_secondary_container" => Ok(ColorRole::OnSecondaryContainer),
            "secondary_fixed" => Ok(ColorRole::SecondaryFixed),
            "secondary_fixed_dim" => Ok(ColorRole::SecondaryFixedDim),
            "on_secondary_fixed" => Ok(ColorRole::OnSecondaryFixed),
            "on_secondary_fixed_variant" => Ok(ColorRole::OnSecondaryFixedVariant),
            "tertiary" => Ok(ColorRole::Tertiary),
            "on_tertiary" => Ok(ColorRole::OnTertiary),
            "tertiary_container" => Ok(ColorRole::TertiaryContainer),
            "on_tertiary_container" => Ok(ColorRole::OnTertiaryContainer),
            "tertiary_fixed" => Ok(ColorRole::TertiaryFixed),
            "tertiary_fixed_dim" => Ok(ColorRole::TertiaryFixedDim),
            "on_tertiary_fixed" => Ok(ColorRole::OnTertiaryFixed),
            "on_tertiary_fixed_variant" => Ok(ColorRole::OnTertiaryFixedVariant),
            "error" => Ok(ColorRole::Error),
            "on_error" => Ok(ColorRole::OnError),
            "error_container" => Ok(ColorRole::ErrorContainer),
            "on_error_container" => Ok(ColorRole::OnErrorContainer),
            "background" => Ok(ColorRole::Background),
            "on_background" => Ok(ColorRole::OnBackground),
            "surface" => Ok(ColorRole::Surface),
            "on_surface" => Ok(ColorRole::OnSurface),
            "surface_variant" => Ok(ColorRole::SurfaceVariant),
            "on_surface_variant" => Ok(ColorRole::OnSurfaceVariant),
            "surface_container_lowest" => Ok(ColorRole::SurfaceContainerLowest),
            "surface_container_low" => Ok(ColorRole::SurfaceContainerLow),
            "surface_container" => Ok(ColorRole::SurfaceContainer),
            "surface_container_high" => Ok(ColorRole::SurfaceContainerHigh),
            "surface_container_highest" => Ok(ColorRole::SurfaceContainerHighest),
            "inverse_surface" => Ok(ColorRole::InverseSurface),
            "inverse_on_surface" => Ok(ColorRole::InverseOnSurface),
            "inverse_primary" => Ok(ColorRole::InversePrimary),
            "surface_dim" => Ok(ColorRole::SurfaceDim),
            "surface_bright" => Ok(ColorRole::SurfaceBright),
            "surface_tint" => Ok(ColorRole::SurfaceTint),
            "outline" => Ok(ColorRole::Outline),
            "outline_variant" => Ok(ColorRole::OutlineVariant),
            "shadow" => Ok(ColorRole::Shadow),
            "scrim" => Ok(ColorRole::Scrim),
            "black" => Ok(ColorRole::Black),
            "red" => Ok(ColorRole::Red),
            "green" => Ok(ColorRole::Green),
            "yellow" => Ok(ColorRole::Yellow),
            "blue" => Ok(ColorRole::Blue),
            "magenta" => Ok(ColorRole::Magenta),
            "cyan" => Ok(ColorRole::Cyan),
            "white" => Ok(ColorRole::White),
            "bright_black" => Ok(ColorRole::BrightBlack),
            "bright_red" => Ok(ColorRole::BrightRed),
            "bright_green" => Ok(ColorRole::BrightGreen),
            "bright_yellow" => Ok(ColorRole::BrightYellow),
            "bright_blue" => Ok(ColorRole::BrightBlue),
            "bright_magenta" => Ok(ColorRole::BrightMagenta),
            "bright_cyan" => Ok(ColorRole::BrightCyan),
            "bright_white" => Ok(ColorRole::BrightWhite),
            _ => Err(format!("Unknown color role: {}", s)),
        }
    }
}
