//! Material Design 3 color constants
//!
//! This module contains constants used in the MD3 color generation algorithm.
//! These values are based on the Material Design 3 specification.

// ============================================================================
// Container Tone Constants
// ============================================================================

/// Primary container tone in dark mode
pub const PRIMARY_CONTAINER_TONE_DARK: f64 = 30.0;
/// Primary container tone in light mode
pub const PRIMARY_CONTAINER_TONE_LIGHT: f64 = 90.0;

/// Secondary container tone in dark mode
pub const SECONDARY_CONTAINER_TONE_DARK: f64 = 20.0;
/// Secondary container tone in light mode
pub const SECONDARY_CONTAINER_TONE_LIGHT: f64 = 95.0;

/// Tertiary container tone in dark mode
pub const TERTIARY_CONTAINER_TONE_DARK: f64 = 25.0;
/// Tertiary container tone in light mode
pub const TERTIARY_CONTAINER_TONE_LIGHT: f64 = 95.0;

/// Container chroma reduction factor (multiply base chroma by this value)
pub const CONTAINER_CHROMA_FACTOR: f64 = 0.4;

// ============================================================================
// Surface Tone Constants
// ============================================================================

/// Surface tone in dark mode
pub const SURFACE_TONE_DARK: f64 = 6.0;
/// Surface tone in light mode
pub const SURFACE_TONE_LIGHT: f64 = 98.0;

/// Surface variant hue shift from base surface
pub const SURFACE_VARIANT_HUE_SHIFT: f64 = 15.0;
/// Surface variant chroma
pub const SURFACE_VARIANT_CHROMA: f64 = 5.0;
/// Surface variant tone in dark mode
pub const SURFACE_VARIANT_TONE_DARK: f64 = 10.0;
/// Surface variant tone in light mode
pub const SURFACE_VARIANT_TONE_LIGHT: f64 = 94.0;

/// Surface container lowest tone offset (dark mode step)
pub const SURFACE_CONTAINER_STEP_DARK: f64 = 2.0;
/// Surface container lowest tone offset (light mode step)
pub const SURFACE_CONTAINER_STEP_LIGHT: f64 = 4.0;

/// Surface dim tone in dark mode
pub const SURFACE_DIM_TONE_DARK: f64 = 6.0;
/// Surface dim tone in light mode
pub const SURFACE_DIM_TONE_LIGHT: f64 = 87.0;

/// Surface bright tone in dark mode
pub const SURFACE_BRIGHT_TONE_DARK: f64 = 24.0;
/// Surface bright tone in light mode
pub const SURFACE_BRIGHT_TONE_LIGHT: f64 = 100.0;

// ============================================================================
// Fixed Accent Color Constants
// ============================================================================

/// Minimum chroma for fixed accent colors
pub const FIXED_MIN_CHROMA: f64 = 12.0;

/// Fixed color chroma reduction factor
pub const FIXED_CHROMA_FACTOR: f64 = 0.9;

/// Fixed color dim chroma reduction factor
pub const FIXED_DIM_CHROMA_FACTOR: f64 = 0.7;

/// Fixed color tone base multiplier
pub const FIXED_TONE_BASE_MULTIPLIER: f64 = 0.8;

/// Fixed color tone offset
pub const FIXED_TONE_OFFSET: f64 = 18.0;

/// Fixed color dim tone multiplier
pub const FIXED_DIM_TONE_MULTIPLIER: f64 = 0.7;

/// Fixed color dim tone offset
pub const FIXED_DIM_TONE_OFFSET: f64 = 25.0;

/// Fixed color tone clamp minimum
pub const FIXED_TONE_CLAMP_MIN: f64 = 20.0;

/// Fixed color tone clamp maximum
pub const FIXED_TONE_CLAMP_MAX: f64 = 90.0;

/// Fixed color variant hue shift
pub const FIXED_VARIANT_HUE_SHIFT: f64 = 20.0;

/// Fixed color variant tone threshold (for determining variant tone)
pub const FIXED_VARIANT_TONE_THRESHOLD: f64 = 60.0;

/// Fixed color variant tone when source is bright
pub const FIXED_VARIANT_TONE_BRIGHT: f64 = 45.0;

/// Fixed color variant tone when source is dark
pub const FIXED_VARIANT_TONE_DARK: f64 = 65.0;

/// Fixed color variant chroma factor
pub const FIXED_VARIANT_CHROMA_FACTOR: f64 = 0.6;

/// Minimum chroma for fixed color variant
pub const FIXED_VARIANT_MIN_CHROMA: f64 = 8.0;

// ============================================================================
// Inverse Color Constants
// ============================================================================

/// Inverse surface tone in dark mode
pub const INVERSE_SURFACE_TONE_DARK: f64 = 90.0;

/// Inverse surface tone in light mode
pub const INVERSE_SURFACE_TONE_LIGHT: f64 = 20.0;

/// Inverse primary tone in dark mode
pub const INVERSE_PRIMARY_TONE_DARK: f64 = 40.0;

/// Inverse primary tone in light mode
pub const INVERSE_PRIMARY_TONE_LIGHT: f64 = 80.0;

// ============================================================================
// Error Color Constants
// ============================================================================

/// Error container chroma
pub const ERROR_CONTAINER_CHROMA: f64 = 30.0;

/// Error container tone in dark mode
pub const ERROR_CONTAINER_TONE_DARK: f64 = 30.0;

/// Error container tone in light mode
pub const ERROR_CONTAINER_TONE_LIGHT: f64 = 95.0;

// ============================================================================
// Outline Color Constants
// ============================================================================

/// Outline chroma
pub const OUTLINE_CHROMA: f64 = 10.0;

/// Outline tone in dark mode
pub const OUTLINE_TONE_DARK: f64 = 60.0;

/// Outline tone in light mode
pub const OUTLINE_TONE_LIGHT: f64 = 50.0;

/// Outline variant chroma
pub const OUTLINE_VARIANT_CHROMA: f64 = 5.0;

/// Outline variant tone in dark mode
pub const OUTLINE_VARIANT_TONE_DARK: f64 = 30.0;

/// Outline variant tone in light mode
pub const OUTLINE_VARIANT_TONE_LIGHT: f64 = 80.0;

// ============================================================================
// Algorithm Constants
// ============================================================================

/// Hue wrap-around value (degrees in a circle)
pub const HUE_WRAP_AROUND: f64 = 360.0;

/// Chroma clamp minimum
pub const CHROMA_CLAMP_MIN: f64 = 0.0;

/// Chroma clamp maximum
pub const CHROMA_CLAMP_MAX: f64 = 200.0;

/// Tone clamp minimum
pub const TONE_CLAMP_MIN: f64 = 0.0;

/// Tone clamp maximum
pub const TONE_CLAMP_MAX: f64 = 100.0;

/// Container tone clamp minimum
pub const CONTAINER_TONE_CLAMP_MIN: f64 = 4.0;

/// Container tone clamp maximum
pub const CONTAINER_TONE_CLAMP_MAX: f64 = 100.0;

// ============================================================================
// Alpha/Hex Conversion Constants
// ============================================================================

/// Alpha scale factor (for converting 0-255 to 0.0-1.0)
pub const ALPHA_SCALE: f64 = 255.0;

// ============================================================================
// Default Fallback Colors
// ============================================================================

/// Default error color hex
pub const DEFAULT_ERROR_COLOR: &str = "#f44336";

/// Default scrim color in dark mode (50% opacity black)
pub const SCRIM_COLOR_DARK: &str = "#00000080";

/// Default scrim color in light mode (30% opacity dark gray)
pub const SCRIM_COLOR_LIGHT: &str = "#1111114D";
