//! Core color types for palette generation

use crate::color::Color;
use std::collections::HashMap;

/// Exhaustive Material Design 3 color roles.
/// String representation is snake_case to match template keys (e.g., "primary", "on_secondary_container").
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumIter, strum::Display, strum::EnumString,
)]
#[strum(serialize_all = "snake_case")]
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
