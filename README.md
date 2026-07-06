# tinct - Theme Injector

A theme injector tool that applies Material Design 3 color palettes to various configuration files.

![Preview](Assets/preview.png)

## Description

tinct is a command-line utility that generates themed configuration files based on Material Design 3 color specifications. It reads color themes from JSON files and injects the appropriate color values into template files, producing themed output files for various applications.

## Features

- Material Design 3 compliant color generation using official algorithms
- **Wallpaper-based color extraction** — extract source colors from images with multiple scheme types
- Support for light and dark themes
- Template-based theme injection with **parallel processing** (10x faster)
- Color preview functionality
- Configurable via TOML files with algorithm parameters
  - `contrast_level` for accessibility
  - `color_harmony` modes (md3, analogous, complementary, triadic, split-complementary)
- Support for post-processing hooks
- Modular architecture for easy extensibility
- **Smart color generation** from single seed color
- Consistent alpha values (0.0-1.0 range)
- Format-preserving color filters
- HSL-based color adjustments
- **Simplified theme format** - no more dark/light nesting

## Installation

### 1. Build from source

```bash
git clone https://github.com/lonerOrz/tinct.git
cd tinct
cargo build --release
```

### 2. Install via Nix (for Nix or NixOS users)

```nix
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
- `-s, --seed`: Seed color for generating palette (e.g., `"#7aa2f7"`)
- `-i, --image`: Path to wallpaper image for color extraction (PNG/JPG/WebP)
- `-m, --mode`: Theme mode override (dark/light, defaults to dark)
- `-p, --preview`: Show color preview instead of processing templates
- `--scheme-type`: Color scheme for image extraction (tonal-spot, vibrant, faithful, muted, dysfunctional, content, fruit-salad, rainbow, monochrome)
- `--skip-sequences`: Skip sending ANSI escape sequences to update terminal colors
- `--log-level`: Logging level (quiet/normal/verbose, defaults to normal)

## Theme Format

tinct supports simplified theme formats using Material Design 3 color generation.

### Format 1: Seed Only (Simplest)

The simplest format uses only a seed color to generate the complete Material Design 3 palette.

```json
{
  "seed": "#7aa2f7"
}
```

### Format 2: Overrides Only

You can also specify override colors directly. The `Primary` color will be used as the seed.

```json
{
  "Primary": "#7aa2f7",
  "Secondary": "#bb9af7",
  "Tertiary": "#9ece6a"
}
```

### Format 3: Seed + Overrides (Recommended)

Combine seed with color overrides for precise control.

```json
{
  "seed": "#7aa2f7",
  "Primary": "#7aa2f7",
  "Secondary": "#bb9af7",
  "Tertiary": "#9ece6a",
  "Error": "#f7768e",
  "Surface": "#1a1b26",
  "Background": "#1a1b26"
}
```

**Available override options:**

| Field              | Description                       | Example       |
| ------------------ | --------------------------------- | ------------- |
| `seed`             | Seed color for palette generation | `"#7aa2f7"`   |
| `Primary`          | Override primary color            | `"#7aa2f7"`   |
| `Secondary`        | Override secondary color          | `"#bb9af7"`   |
| `Tertiary`         | Override tertiary color           | `"#9ece6a"`   |
| `Error`            | Override error color              | `"#f7768e"`   |
| `Surface`          | Override surface color            | `"#1a1b26"`   |
| `Background`       | Override background color         | `"#1a1b26"`   |
| `SurfaceVariant`   | Override surface variant          | `"#24283b"`   |
| `Outline`          | Override outline color            | `"#565f89"`   |
| `OutlineVariant`   | Override outline variant          | `"#b4b5b9"`   |
| `Shadow`           | Override shadow color             | `"#000000"`   |
| `Scrim`            | Override scrim color              | `"#00000080"` |
| `InverseSurface`   | Override inverse surface          | `"#ebdbb2"`   |
| `InverseOnSurface` | Override inverse on surface       | `"#3c3836"`   |
| `InversePrimary`   | Override inverse primary          | `"#7aa2f7"`   |

**Note:** Color names support both lowercase (`"primary"`) and PascalCase (`"Primary"`).

## Configuration File

The configuration file is written in TOML format and is located at `~/.config/tinct/config.toml` by default. It contains template injection definitions, color generation algorithm tuning, and image extraction preferences.

### 1. Template File Configurations

Configure the template paths, destination output paths, and execution hooks for each application.

```toml
[templates.alacritty]
input_path = "~/.config/tinct/templates/alacritty.toml"
output_path = "~/.config/alacritty/alacritty.toml"
post_hook = "echo 'Alacritty theme injected successfully!'"

[templates.waybar]
input_path = "~/.config/tinct/templates/waybar.css"
output_path = "~/.config/waybar/style.css"
post_hook = "killall -SIGUSR2 waybar"
```

### 2. Algorithm Configuration

You can adjust the color generation algorithm behavior:

```toml
[algorithm]
hue_shift = 0               # Rotate hue by degrees (-180 to 180)
saturation_adjustment = 0   # Adjust saturation percentage (-100 to 100)
contrast_level = 0.0        # MD3 contrast level (-1.0 to 1.0)
color_harmony = "md3"       # Harmony mode (md3, analogous, complementary, triadic, split-complementary)
```

**Algorithm parameters:**

| Parameter               | Range      | Default | Effect                               |
| ----------------------- | ---------- | ------- | ------------------------------------ |
| `hue_shift`             | -180 ~ 180 | `0`     | Rotates all colors' hue              |
| `saturation_adjustment` | -100 ~ 100 | `0`     | Adjusts color saturation (chroma)    |
| `contrast_level`        | -1.0 ~ 1.0 | `0.0`   | MD3 contrast level for accessibility |
| `color_harmony`         | see below  | `md3`   | Secondary/tertiary hue relationships |

**Color Harmony modes:**

| Mode                  | Description                | Secondary Hue         | Tertiary Hue          |
| --------------------- | -------------------------- | --------------------- | --------------------- |
| `md3`                 | Material Design 3 standard | MD3 hue table (2-20°) | MD3 hue table (5-40°) |
| `analogous`           | Close, harmonious colors   | +15°                  | +30°                  |
| `complementary`       | Opposite colors            | +180°                 | +180°                 |
| `triadic`             | Evenly spaced              | +120°                 | +240°                 |
| `split-complementary` | Split opposite             | +150°                 | +210°                 |

### 3. Image Configuration

Extract colors from wallpaper images using the `[image]` section:

```toml
[image]
scheme_type = "vibrant"    # Color extraction scheme
```

**Scheme types:**

| Scheme          | Pipeline                | Description              |
| --------------- | ----------------------- | ------------------------ |
| `tonal-spot`    | Wu + WSMeans + Score    | MD3 standard, balanced   |
| `vibrant`       | K-means + Chroma        | High saturation colors   |
| `faithful`      | K-means + Count         | Area-dominant colors     |
| `muted`         | K-means + Muted         | Low saturation, subtle   |
| `dysfunctional` | K-means + Dysfunctional | 2nd most dominant family |
| `content`       | Wu + WSMeans + Score    | MD3 Content variant      |
| `fruit-salad`   | Wu + WSMeans + Score    | MD3 Fruit Salad variant  |
| `rainbow`       | Wu + WSMeans + Score    | MD3 Rainbow variant      |
| `monochrome`    | Wu + WSMeans + Score    | MD3 Monochrome variant   |

**Priority chain:** CLI `--scheme-type` > config `[image].scheme_type` > default (`tonal-spot`)

**Notes:**

- `hue_shift = 30` rotates colors 30° toward orange
- `saturation_adjustment = 50` increases saturation by 50%
- `saturation_adjustment = -50` decreases saturation by 50% (more muted)
- `lightness_adjustment` is not supported (would break MD3 contrast ratios)

## Template Color Format

In tinct's template files, you can use the following color formats to reference colors from your theme.

### Color Roles

#### Material Design 3 Color Roles

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

#### Terminal ANSI Color Roles

You can also use these standard ANSI terminal color values, mapped intelligently from the generated palette:

- `black` / `bright_black`
- `red` / `bright_red`
- `green` / `bright_green`
- `yellow` / `bright_yellow`
- `blue` / `bright_blue`
- `magenta` / `bright_magenta`
- `cyan` / `bright_cyan`
- `white` / `bright_white`

### Color Format Attributes

For each color role, you can use the following format attributes:

| Attribute     | Example Placeholder                        | Output Example             |
| ------------- | ------------------------------------------ | -------------------------- |
| Hex complete  | `{{colors.primary.default.hex}}`           | `#ff5722`                  |
| Hex stripped  | `{{colors.primary.default.hex_stripped}}`  | `ff5722`                   |
| Hex8 complete | `{{colors.primary.default.hex8}}`          | `#ff5722ff`                |
| Hex8 stripped | `{{colors.primary.default.hex8_stripped}}` | `ff5722ff`                 |
| RGB           | `{{colors.primary.default.rgb}}`           | `rgb(255, 87, 34)`         |
| RGBA          | `{{colors.primary.default.rgba}}`          | `rgba(255, 87, 34, 1.0)`   |
| Red           | `{{colors.primary.default.red}}`           | `255`                      |
| Green         | `{{colors.primary.default.green}}`         | `87`                       |
| Blue          | `{{colors.primary.default.blue}}`          | `34`                       |
| Alpha         | `{{colors.primary.default.alpha}}`         | `1.0`                      |
| HSL           | `{{colors.primary.default.hsl}}`           | `hsl(14, 100%, 57%)`       |
| HSLA          | `{{colors.primary.default.hsla}}`          | `hsla(14, 100%, 57%, 1.0)` |
| Hue           | `{{colors.primary.default.hue}}`           | `14`                       |
| Saturation    | `{{colors.primary.default.saturation}}`    | `100`                      |
| Lightness     | `{{colors.primary.default.lightness}}`     | `57`                       |

### Template Filters

tinct supports a modular filter system to transform color values.

> **Note on Syntax**: To ensure the parser parses filters correctly, do not put whitespace directly after the pipe `|` character (e.g. use `{{colors.primary.default.rgba|set_alpha:0.5}}` instead of `| set_alpha`).

| Filter     | Example Placeholder                              | Output Example           |
| ---------- | ------------------------------------------------ | ------------------------ |
| Set Alpha  | `{{colors.primary.default.rgba\|set_alpha:0.5}}` | `rgba(255, 87, 34, 0.5)` |
| Lighten    | `{{colors.primary.default.rgb\|lighten:10}}`     | Lightened RGB color      |
| Darken     | `{{colors.primary.default.rgb\|darken:10}}`      | Darkened RGB color       |
| Saturate   | `{{colors.primary.default.rgb\|saturate:10}}`    | More saturated RGB color |
| Desaturate | `{{colors.primary.default.rgb\|desaturate:10}}`  | Less saturated RGB color |

**Note on Transparency:** The most convenient way to add transparency to a color is by using the `|set_alpha` filter directly on the role placeholder. However, you can also manually format them in CSS by accessing individual RGB components separately:

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

```

```
