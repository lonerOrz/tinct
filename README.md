# tinct - Theme Injector

A theme injector tool that applies Material Design 3 color palettes to various configuration files.

![Preview](Assets/preview.png)

## Description

tinct is a command-line utility that generates themed configuration files based on Material Design 3 color specifications. It reads color themes from JSON files and injects the appropriate color values into template files, producing themed output files for various applications.

## Features

- Material Design 3 compliant color generation using official algorithms
- Support for light and dark themes
- Template-based theme injection
- Color preview functionality
- Configurable via TOML files with algorithm parameters
- Support for post-processing hooks
- Modular architecture for easy extensibility
- **Backward compatible** with legacy theme formats
- **Smart color generation** from single seed color
- Consistent alpha values (0.0-1.0 range)
- Format-preserving color filters
- HSL-based color adjustments

## Installation

### 1. Build from source

```bash
git clone https://github.com/lonerOrz/tinct.git
cd tinct
cargo build --release
```

### 2. Install via Nix (for Nix or NixOS users)

```bash
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    tinct.url = "github:lonerOrz/tinct";
  };

  outputs =
    inputs@{
      self,
      flake-utils,
      nixpkgs,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [ inputs.tinct.packages.${system}.tinct ];
        };
      }
    );
}
```

## Usage

Basic usage:

```bash
tinct --theme <theme-name>
```

With custom options:

```bash
tinct -t MyTheme -c config.toml -m light -p
```

Options:

- `-c, --config`: Path to the TOML config file (defaults to `~/.config/tinct/config.toml`)
- `-t, --theme`: Path to theme.json file or theme name in themes/ folder
- `-m, --mode`: Theme mode override (dark/light, defaults to dark)
- `-p, --preview`: Show color preview instead of processing templates
- `--log-level`: Logging level (quiet/normal/verbose, defaults to normal)

## Theme Format

tinct supports both new and legacy theme formats for backward compatibility.

### New Format (Recommended)

The new format uses a single seed color to generate the complete Material Design 3 palette using official algorithms.

**Minimal example:**
```json
{
  "seed": "#b8bb26"
}
```

**With color overrides:**
```json
{
  "seed": "#b8bb26",
  "error": "#fb4934",
  "surface": "#282828",
  "background": "#282828",
  "outline": "#928374"
}
```

**Available override options:**

| Field | Description | Example |
|-------|-------------|---------|
| `seed` | **Required.** Seed color for palette generation | `"#b8bb26"` |
| `primary` | Override primary color | `"#b8bb26"` |
| `secondary` | Override secondary color | `"#fabd2f"` |
| `tertiary` | Override tertiary color | `"#83a598"` |
| `error` | Override error color | `"#fb4934"` |
| `surface` | Override surface color | `"#282828"` |
| `background` | Override background color | `"#282828"` |
| `surface_variant` | Override surface variant | `"#3c3836"` |
| `outline` | Override outline color | `"#928374"` |
| `outline_variant` | Override outline variant | `"#bdae93"` |
| `shadow` | Override shadow color | `"#000000"` |
| `scrim` | Override scrim color | `"#00000080"` |
| `inverse_surface` | Override inverse surface | `"#ebdbb2"` |
| `inverse_on_surface` | Override inverse on surface | `"#3c3836"` |
| `inverse_primary` | Override inverse primary | `"#b8bb26"` |

### Legacy Format (Still Supported)

The legacy format specifies all colors explicitly for both dark and light modes.

```json
{
  "dark": {
    "mPrimary": "#b8bb26",
    "mOnPrimary": "#282828",
    "mSecondary": "#fabd2f",
    "mOnSecondary": "#282828",
    "mTertiary": "#83a598",
    "mOnTertiary": "#282828",
    "mError": "#fb4934",
    "mOnError": "#282828",
    "mSurface": "#282828",
    "mOnSurface": "#fbf1c7",
    "mSurfaceVariant": "#3c3836",
    "mOnSurfaceVariant": "#ebdbb2",
    "mOutline": "#928374",
    "mShadow": "#000000"
  },
  "light": {
    "mPrimary": "#98971a",
    "mOnPrimary": "#fbf1c7",
    "mSecondary": "#d79921",
    "mOnSecondary": "#fbf1c7",
    "mTertiary": "#458588",
    "mOnTertiary": "#fbf1c7",
    "mError": "#cc241d",
    "mOnError": "#fbf1c7",
    "mSurface": "#fbf1c7",
    "mOnSurface": "#3c3836"
  }
}
```

**Key differences:**

| Aspect | New Format | Legacy Format |
|--------|-----------|---------------|
| File size | ~5 lines | ~50 lines |
| Maintenance | Automatic generation | Manual specification |
| Color consistency | MD3 algorithm guaranteed | Depends on manual tuning |
| Flexibility | Seed + optional overrides | Full manual control |
| Compatibility | ✅ Recommended | ✅ Still supported |

### Algorithm Configuration

You can adjust the color generation algorithm in your `config.toml`:

```toml
[algorithm]
hue_shift = 0               # Rotate hue by degrees (-180 to 180)
saturation_adjustment = 0   # Adjust saturation percentage (-100 to 100)
```

**Algorithm parameters:**

| Parameter | Range | Default | Effect |
|-----------|-------|---------|--------|
| `hue_shift` | -180 ~ 180 | `0` | Rotates all colors' hue |
| `saturation_adjustment` | -100 ~ 100 | `0` | Adjusts color saturation (chroma) |

**Examples:**

```toml
# Warm Gruvbox theme
[algorithm]
hue_shift = 15
saturation_adjustment = 10

# Cool Nord theme
[algorithm]
hue_shift = -10
saturation_adjustment = -20

# High saturation theme
[algorithm]
saturation_adjustment = 50
```

**Notes:**
- `hue_shift = 30` rotates colors 30° toward orange
- `saturation_adjustment = 50` increases saturation by 50%
- `saturation_adjustment = -50` decreases saturation by 50% (more muted)
- `lightness_adjustment` is not supported (would break MD3 contrast ratios)

### Example Themes

**Gruvbox Dark:**
```json
{
  "seed": "#b8bb26",
  "error": "#fb4934",
  "surface": "#282828",
  "background": "#282828"
}
```

**Nord:**
```json
{
  "seed": "#88c0d0",
  "error": "#bf616a",
  "surface": "#2e3440",
  "background": "#2e3440"
}
```

**Dracula:**
```json
{
  "seed": "#bd93f9",
  "error": "#ff5555",
  "surface": "#282a36",
  "background": "#282a36"
}
```


## Template Color Format

In tinct's template files, you can use the following color formats to reference colors from your theme.

### Color Roles

Available color roles include:

- `primary` - Primary brand color
- `on_primary` - Text/icon color that appears on top of primary
- `primary_container` - Container color matching the primary
- `on_primary_container` - Text/icon color that appears on top of primary container
- `secondary` - Secondary brand color
- `on_secondary` - Text/icon color that appears on top of secondary
- `secondary_container` - Container color matching the secondary
- `on_secondary_container` - Text/icon color that appears on top of secondary container
- `tertiary` - Tertiary brand color
- `on_tertiary` - Text/icon color that appears on top of tertiary
- `tertiary_container` - Container color matching the tertiary
- `on_tertiary_container` - Text/icon color that appears on top of tertiary container
- `error` - Error state color
- `on_error` - Text/icon color that appears on top of error
- `error_container` - Container color matching the error
- `on_error_container` - Text/icon color that appears on top of error container
- `background` - Background color
- `on_background` - Text/icon color that appears on top of background
- `surface` - Surface color
- `on_surface` - Text/icon color that appears on top of surface
- `surface_variant` - Variant surface color
- `on_surface_variant` - Text/icon color that appears on top of surface variant
- `surface_container_lowest` - Lowest level surface container
- `surface_container_low` - Low level surface container
- `surface_container` - Standard surface container
- `surface_container_high` - High level surface container
- `surface_container_highest` - Highest level surface container
- `inverse_surface` - Inverse surface color
- `inverse_on_surface` - Text/icon color for inverse surface
- `inverse_primary` - Inverse primary color
- `surface_dim` - Dimmed surface color
- `surface_bright` - Bright surface color
- `outline` - Outline/border color
- `outline_variant` - Variant outline color
- `shadow` - Shadow color
- `scrim` - Scrim overlay color

### Color Format Attributes

For each color role, you can use the following format attributes:

| Attribute    | Example Placeholder                           | Output Example             |
| ------------ | --------------------------------------------- | -------------------------- |
| Hex complete | `{{colors.primary.default.hex}}`              | `#ff5722`                  |
| Hex stripped | `{{colors.primary.default.hex_stripped}}`     | `ff5722`                   |
| Hex8 complete | `{{colors.primary.default.hex8}}`             | `#ff5722ff`                |
| Hex8 stripped | `{{colors.primary.default.hex8_stripped}}`    | `ff5722ff`                 |
| RGB          | `{{colors.primary.default.rgb}}`              | `rgb(255, 87, 34)`         |
| RGBA         | `{{colors.primary.default.rgba}}`             | `rgba(255, 87, 34, 1.0)`   |
| Red          | `{{colors.primary.default.red}}`              | `255`                      |
| Green        | `{{colors.primary.default.green}}`            | `87`                       |
| Blue         | `{{colors.primary.default.blue}}`             | `34`                       |
| Alpha        | `{{colors.primary.default.alpha}}`            | `1.0`                      |
| HSL          | `{{colors.primary.default.hsl}}`              | `hsl(14, 100%, 57%)`       |
| HSLA         | `{{colors.primary.default.hsla}}`             | `hsla(14, 100%, 57%, 1.0)` |
| Hue          | `{{colors.primary.default.hue}}`              | `14`                       |
| Saturation   | `{{colors.primary.default.saturation}}`       | `100`                      |
| Lightness    | `{{colors.primary.default.lightness}}`        | `57`                       |

### Template Filters

tinct supports a modular filter system to transform color values:

| Filter     | Example Placeholder                                 | Output Example           |
| ---------- | --------------------------------------------------- | ------------------------ |
| Set Alpha  | `{{colors.primary.default.rgba \| set_alpha: 0.5}}` | `rgba(255, 87, 34, 0.5)` |
| Lighten    | `{{colors.primary.default.rgb \| lighten: 10}}`     | Lightened RGB color      |
| Darken     | `{{colors.primary.default.rgb \| darken: 10}}`      | Darkened RGB color       |
| Saturate   | `{{colors.primary.default.rgb \| saturate: 10}}`    | More saturated RGB color |
| Desaturate | `{{colors.primary.default.rgb \| desaturate: 10}}`  | Less saturated RGB color |

**Note:** If you want to use transparency in `rgba()`, you need to reference the `.red`, `.green`, `.blue` components separately, otherwise it will generate invalid CSS.

### Mode-related Placeholders

- `{{mode}}` → `"dark"` or `"light"`
- `{{is_dark}}` → `"true"` or `"false"`
- `{{is_light}}` → `"true"` or `"false"`

### Usage Examples

- **Hex Colors**

```css
.primary-button {
    background-color: {{colors.primary.default.hex}};
    color: {{colors.on_primary.default.hex}};
}
```

- **RGB Colors**

```css
.surface-background {
    background-color: {{colors.surface.default.rgb}};
    border: 1px solid {{colors.outline.default.hex}};
}
```

- **RGBA Colors (with components)**

```css
.semi-transparent-overlay {
  background-color: rgba(
    {{colors.surface.default.red}},
    {{colors.surface.default.green}},
    {{colors.surface.default.blue}},
    0.8
  );
}
```

- **HSL Colors**

```css
.accent-element {
    background-color: {{colors.tertiary.default.hsl}};
}
```

- **Stripped Hex**

```css
.styled-border {
    border-color: #{{colors.outline.default.hex_stripped}};
}
```

- **Conditional Styling**

```css
@media (prefers-color-scheme: {{mode}}) {
    body {
        background-color: {{colors.background.default.hex}};
    }
}
```

These formats allow you to flexibly use various color representations in your templates to accommodate the requirements of different application configuration files.

## License

BSD 3-Clause License

---

> If you find `tinct` useful, please give it a ⭐ and share! 🎉
