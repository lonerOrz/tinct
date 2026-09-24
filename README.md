# tinct - Theme Injector

A theme injector tool that applies Material Design 3 color palettes to various configuration files.

![Preview](.github/assets/preview.png)

## Description

tinct is a command-line utility that generates themed configuration files based on Material Design 3 color specifications. It reads color themes from JSON files and injects the appropriate color values into template files, producing themed output files for various applications.

## Features

- Material Design 3 compliant color generation using the official Material You algorithms
- **Wallpaper-based color extraction** — extract source colors from images with multiple scheme types
- **Scheme selection for every source** — `--scheme-type` chooses the MD3 scheme variant (tonal-spot, vibrant, content, …) used to build the palette
- **Input-aware terminal palette** — the 16 ANSI colors snap real colors from your wallpaper, theme file, or seed onto the CIE LCh hue wheel (wallust-inspired), with vivid hue anchors and a readable bright/dark ladder
- Support for light and dark themes
- Template-based theme injection with **parallel processing** (10x faster)
- Color preview functionality
- Configurable via TOML files
  - `[algorithm]` — seed nudging (`hue_shift`, `saturation_adjustment`, `contrast_level`, `seed_tone`, `chroma_floor`) plus per-mode `variant_dark` / `variant_light`
  - `[image]` — extraction tuning (`max_colors`, `min_population`, `filter`, `quantizer`)
  - `[ansi]` — terminal palette tuning (`palette`, `source_weight`, `chroma_threshold`, `contrast_target`, colour pins and anchors)
- Support for post-processing hooks
- Layered architecture (domain / infrastructure / presentation) with high cohesion and low coupling
- **Smart color generation** from single seed color
- Consistent alpha values (0.0-1.0 range)
- Perceptually uniform HCT-based color filters
- **Simplified theme format** - no more dark/light nesting

## Project layout

The crate is organised in layers so every concern has exactly one home:

```text
src/
├── main.rs, cli.rs     # entry point + argument parsing
├── pipeline.rs         # application orchestration
├── core/               # domain: Color/HCT, filters, Error, Mode, Theme
├── palette/            # domain: MD3 generation (dynamic.rs), ANSI, params
├── image/              # domain: quantization + source-color extraction
├── template/           # domain: placeholder rendering + color filters
├── config/             # infrastructure: TOML loading + path canonicalization
├── output/             # infrastructure: file writing
└── ui/                 # presentation: logging + color preview
```

`lib.rs` re-exports the public API, including backward-compatible aliases
(`tinct::color`, `tinct::log`, `tinct::preview`) for modules that moved during
the reorganisation.

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

- `-c, --config`: Path to the TOML config file (defaults to `$XDG_CONFIG_HOME/tinct/config.toml`, falling back to `~/.config/tinct/config.toml`)
- `-t, --theme`: Path to theme.json file or theme name in themes/ folder
- `-s, --seed`: Seed color for generating palette (e.g., `"#7aa2f7"`)
- `-i, --image`: Path to wallpaper image for color extraction (PNG/JPG/WebP)
- `-m, --mode`: Theme mode override (dark/light, defaults to dark)
- `-p, --preview`: Show color preview instead of processing templates
- `--scheme-type`: MD3 scheme variant used to generate the palette (also the extraction algorithm for images): tonal-spot, vibrant, faithful, muted, dysfunctional, content, fruit-salad, rainbow, monochrome
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

The configuration file is written in TOML format and is located at `$XDG_CONFIG_HOME/tinct/config.toml` (falling back to `~/.config/tinct/config.toml`) by default. It contains template injection definitions, color generation algorithm tuning, and image extraction preferences.

On first run, if the default config file does not exist, `tinct` writes a fully commented default there, prints its path, and exits — edit it and run again. An explicit `--config <path>` that does not exist is an error: `tinct` never writes to a path you did not ask for. Every option has a built-in default, so a partially filled config (or none of the optional sections at all) is valid.

Relative `input_path` / `output_path` values are resolved against the directory containing the config file, so `tinct` behaves identically no matter which directory you run it from. `~` is expanded.

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
hue_shift = 0               # Rotate the seed hue by degrees (-180 to 180)
saturation_adjustment = 0   # Scale the seed chroma by percentage (-100 to 100)
contrast_level = 0.0        # MD3 contrast level (-1.0 to 1.0)
# seed_tone = 50            # Force the seed's HCT tone (0-100); unset keeps it
# chroma_floor = 20.0       # Minimum seed chroma (0-120); rescues grey seeds
# variant_dark = "vibrant"  # MD3 variant used only in dark mode
# variant_light = "tonal-spot"  # MD3 variant used only in light mode
# color_harmony = "md3"     # DEPRECATED and ignored; use --scheme-type
```

**Algorithm parameters:**

| Parameter               | Range      | Default | Effect                                      |
| ----------------------- | ---------- | ------- | ------------------------------------------- |
| `hue_shift`             | -180 ~ 180 | `0`     | Rotates the seed hue before generation      |
| `saturation_adjustment` | -100 ~ 100 | `0`     | Scales seed chroma (`-100` → grey, `+100` → double) |
| `contrast_level`        | -1.0 ~ 1.0 | `0.0`   | MD3 contrast level for accessibility        |
| `seed_tone`             | 0 ~ 100    | unset   | Force the seed's HCT tone before generation |
| `chroma_floor`          | 0 ~ 120    | `0`     | Minimum seed chroma; grey seeds still yield a lively palette |
| `variant_dark`          | scheme name | unset  | MD3 variant used in dark mode only          |
| `variant_light`         | scheme name | unset  | MD3 variant used in light mode only         |

These parameters only **nudge the seed color**. All secondary/tertiary/neutral/error relationships are defined by the selected MD3 scheme and are no longer hand-tuned — that is what keeps the output faithful to Material You.

**Scheme precedence:** the base scheme is `--scheme-type` (CLI) > `[image].scheme_type` (config) > `tonal-spot`. `variant_dark` / `variant_light` then override that base for the respective mode; unset modes keep the base.

**Deprecated:** `color_harmony` (analogous, complementary, triadic, split-complementary) is no longer part of the algorithm — if present in an old config file it is simply ignored. The old hue-table relationships were not part of MD3. Use `--scheme-type` / `[image].scheme_type` to pick the scheme instead.

### 3. Image Configuration

Extract colors from wallpaper images using the `[image]` section. The selected scheme is also used to generate the palette:

```toml
[image]
scheme_type = "vibrant"    # Extraction algorithm and MD3 scheme variant
# max_colors = 64          # Cap on returned colour clusters (default 32)
# min_population = 0.01    # Drop clusters below this share of total pixels (0-1)
# filter = "saturation"    # Pixel pre-filter: "none" | "saturation" | "brightness"
# quantizer = "wsmeans"    # M3 pipeline quantizer: "wsmeans" | "wu"
```

**Scheme types:**

| Scheme          | Extraction pipeline     | MD3 variant   | Description              |
| --------------- | ----------------------- | ------------- | ------------------------ |
| `tonal-spot`    | Wu + WSMeans + Score    | Tonal Spot    | MD3 standard, balanced   |
| `vibrant`       | K-means + Chroma        | Vibrant       | High saturation colors   |
| `faithful`      | K-means + Count         | Fidelity      | Area-dominant colors     |
| `muted`         | K-means + Muted         | Neutral       | Low saturation, subtle   |
| `dysfunctional` | K-means + Dysfunctional | Expressive    | 2nd most dominant family |
| `content`       | Wu + WSMeans + Score    | Content       | MD3 Content variant      |
| `fruit-salad`   | Wu + WSMeans + Score    | Fruit Salad   | MD3 Fruit Salad variant  |
| `rainbow`       | Wu + WSMeans + Score    | Rainbow       | MD3 Rainbow variant      |
| `monochrome`    | Wu + WSMeans + Score    | Monochrome    | MD3 Monochrome variant   |

**Extraction options:**

| Option           | Range                    | Default   | Effect                                              |
| ---------------- | ------------------------ | --------- | --------------------------------------------------- |
| `max_colors`     | 1+                       | `32`      | Cap on the number of returned colour clusters       |
| `min_population` | 0.0 ~ 1.0                | `0.0`     | Drop clusters below this fraction of the total pixels |
| `filter`         | `none`/`saturation`/`brightness` | `none` | Pre-filter pixels before clustering          |
| `quantizer`      | `wsmeans` / `wu`         | `wsmeans` | `wu` skips WSMeans refinement (faster, coarser)     |

**Priority chain:** CLI `--scheme-type` > config `[image].scheme_type` > default (`tonal-spot`)

The resolved `--scheme-type` applies to **every** theme source — seeds and theme files, not just images (where it additionally picks the extraction pipeline). The scheme variant is what produces the MD3-correct secondary (desaturated, ~16 chroma), tertiary (seed hue + 60°), and neutrals (4–8 chroma).

For the non-M3 names (`vibrant`, `faithful`, `muted`, `dysfunctional`) the name does double duty. For example:

- `--image wallpaper.png --scheme-type faithful` — extraction runs the area-dominant (K-means + Count) pipeline, then the palette is built with the `Fidelity` variant.
- `--seed "#6750A4" --scheme-type faithful` — extraction is skipped and the palette is built directly with `Fidelity`.

Same scheme, same output variant; only the seed source differs. The mapping is: `faithful → Fidelity`, `muted → Neutral`, `dysfunctional → Expressive`, `vibrant → Vibrant`.

**Notes:**

- `hue_shift = 30` rotates the seed 30° toward orange
- `saturation_adjustment = 50` scales seed chroma up by 50%
- `saturation_adjustment = -50` reduces seed chroma by 50% (more muted)
- `lightness_adjustment` is not supported (would break MD3 contrast ratios)

### 4. Terminal (ANSI) Configuration

The UI/MD3 roles are always pure Material You. The **16-color terminal palette** is generated separately (wallust-inspired): each chromatic slot snaps to a hue actually present in the source, so a wallpaper or theme yields a matching terminal. Tune it via `[ansi]`:

```toml
[ansi]
palette = "dark"              # "dark" | "light" — bright-colour orientation
source_weight = 0.6667        # 0.0 = anchor hue, 1.0 = source hue (matched slots)
chroma_threshold = 12.0       # Candidates below this chroma are not trusted
brightness_delta = 8.0        # Lightness delta applied to bright variants
bright_chroma_multiplier = 1.2  # Chroma scale for bright variants
contrast_target = 3.0         # Minimum WCAG contrast ratio against the background
# background = "#101010"      # Pin ANSI black (colour 0)
# foreground = "#F0F0F0"      # Pin ANSI white (colour 7)

[ansi.anchors]                # Power-user: override the six hue anchors
# red = "#E06C75"
# green = "#98C379"
# yellow = "#E5C07B"
# blue = "#61AFEF"
# magenta = "#C678DD"
# cyan = "#56B6C2"
```

| Option                   | Range          | Default  | Effect                                          |
| ------------------------ | -------------- | -------- | ----------------------------------------------- |
| `palette`                | `dark`/`light` | `dark`   | Bright variants get lighter (`dark`) or darker (`light`) |
| `source_weight`          | 0.0 ~ 1.0      | `0.6667` | Blend between the fixed anchor and the source hue |
| `chroma_threshold`       | 0+             | `12.0`   | Ignore source candidates below this chroma      |
| `brightness_delta`       | 0+             | `8.0`    | Lightness shift for bright variants             |
| `bright_chroma_multiplier` | 0+           | `1.2`    | Chroma multiplier for bright variants           |
| `contrast_target`        | 0.0 ~ 21.0     | `3.0`    | Minimum contrast ratio against the background   |
| `background` / `foreground` | hex colour  | unset    | Pin ANSI black (0) / white (7)                  |
| `anchors.<hue>`          | hex colour     | unset    | Override an individual hue anchor               |

Setting `contrast_target = 0.0` disables the contrast nudge; `source_weight = 0.0` reproduces fixed, wallust-like anchors. All values are clamped to their valid range.

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

The desktop/UI roles above are pure Material You. A terminal, however, needs
sixteen *recognizable* colors, so the ANSI palette is generated separately using
a CIE LCh pipeline in the spirit of [wallust](https://codeberg.org/explosion-mental/wallust)
(MIT © explosion-mental).

The chromatic slots are **derived from your input, not from a fixed palette**.
The set of candidate colors depends on the source:

| Source                    | Candidate colors                                                    |
| ------------------------- | ------------------------------------------------------------------- |
| `--image <path>`          | the representative clusters extracted from the wallpaper            |
| `-t <theme.json>`         | **every hex value in the theme file**, including custom/extra keys  |
| `--seed <hex>`            | the seed color only                                                 |

Each candidate color is matched to the ANSI slot whose hue range contains it, and
that slot inherits the source's **hue**. The source's chroma and lightness are
then blended with the slot's vivid anchor using wallust's
`(anchor + 2·average) / 3` weighting — your input tints the terminal, but every
slot keeps the vividness and legibility the anchor encodes. When a slot has no
matching candidate (a monochrome wallpaper, or `--seed`), it falls back to the
**full anchor**, so the palette never collapses into a washed-out approximation.

- `black` / `bright_black` / `white` / `bright_white` — a fixed greyscale ladder
  taken from the scheme's neutral palette, so backgrounds and text always match
  the mode.
- `red` / `green` / `yellow` / `blue` / `magenta` / `cyan` — the six chromatic
  slots. Each owns a hue range on the CIE LCh color wheel (red 0–60°, yellow
  61–120°, green 121–180°, cyan 181–210°, blue 211–300°, magenta 301–360°). A
  matching input color contributes its hue; near-neutral colors (low-chroma
  surfaces, greys) are ignored so they don't contaminate the accents, and each
  candidate is consumed by at most one slot.
- `bright_*` variants — the same hue as their normal counterpart, but with a
  **lighter** CIE L\* (in dark *and* light modes) and **higher** chroma, so they
  read as vivid emphasised colors rather than darker, muddier ones.

The anchors are the recognizable colors wallust converges on (red ≈ `#F90000`,
green ≈ `#009850`, blue ≈ `#0076FD`, …). Every chromatic slot is gamut-mapped back
into sRGB and then nudged in lightness until it clears a WCAG 3:1 contrast ratio
against the terminal background, so colors stay legible without washing out.

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
