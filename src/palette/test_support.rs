//! Shared fixtures for palette-layer tests.

/// The 16 ANSI terminal colour roles, in canonical terminal order.
///
/// Kept in one place so the palette tests that assert "every ANSI role exists"
/// cannot drift apart from each other.
pub const ANSI_ROLE_NAMES: [&str; 16] = [
    "black",
    "red",
    "green",
    "yellow",
    "blue",
    "magenta",
    "cyan",
    "white",
    "bright_black",
    "bright_red",
    "bright_green",
    "bright_yellow",
    "bright_blue",
    "bright_magenta",
    "bright_cyan",
    "bright_white",
];
