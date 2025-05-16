# RustPods UI Theming Guide

## Overview

RustPods uses a custom theming system based on the Catppuccin Mocha color palette, implemented in `src/ui/theme.rs`. This guide documents the theme structure, color choices, and best practices for extending or fixing theming in the project.

## Color Palette

The Catppuccin Mocha palette provides a warm, pastel look with good contrast and readability. Colors are defined as `pub static` values for use throughout the UI. Example colors:

- `ROSEWATER`, `FLAMINGO`, `PINK`, `MAUVE`, `RED`, `MAROON`, `PEACH`, `YELLOW`, `GREEN`, `TEAL`, `SKY`, `SAPPHIRE`, `BLUE`, `LAVENDER`
- Base colors: `TEXT`, `SUBTEXT1`, `SUBTEXT0`, `OVERLAY2`, `OVERLAY1`, `OVERLAY0`, `SURFACE2`, `SURFACE1`, `SURFACE0`, `BASE`, `MANTLE`, `CRUST`
- Light theme variants: `LIGHT_BG`, `LIGHT_SURFACE`, `LIGHT_TEXT`, `LIGHT_ACCENT`

## Theme Enum

The `Theme` enum defines the available UI themes:

```rust
pub enum Theme {
    Light,
    Dark,
    System,
    CatppuccinMocha,
}
```

## Implementing StyleSheet Traits

The theme module implements Iced's `StyleSheet` traits for all major widgets:
- `application::StyleSheet`
- `button::StyleSheet`
- `container::StyleSheet`
- `text_input::StyleSheet`
- `text::StyleSheet`
- `rule::StyleSheet`
- `scrollable::StyleSheet`
- `progress_bar::StyleSheet`
- `checkbox::StyleSheet`
- `slider::StyleSheet`
- `pick_list::StyleSheet`
- `menu::StyleSheet`

Each trait implementation provides custom colors, border radii, and other style properties for each theme variant.

## Widget Theming Conventions

- Use the color constants from the palette for all widget backgrounds, borders, and text.
- For `Option<Color>` fields (e.g., `background` in `checkbox::Appearance`), use `Some(color)` or `None` as appropriate.
- For `Color` fields (e.g., `icon_color`, `text_color`), use the color directly.
- When darkening or lightening a color for hover/active states, adjust the `.a` (alpha) or RGB values as needed.
- Always match the expected types for each field in the widget's `Appearance` struct.

## Common Linter Errors and Fixes

- **mismatched types: expected `Option<Color>`, found `Color`**
  - Use `Some(color)` for fields expecting `Option<Color>`.
- **mismatched types: expected `Color`, found `Option<Color>`**
  - Use the color directly, not wrapped in `Some()`.
- **no field `a` on type `iced::Background`**
  - Only adjust `.a` on `Color`, not on `Background`.
- **expected `Color`, found `Option<Color>`**
  - Unwrap the option or provide a default color.

## Recent Fixes

- Fixed the `checkbox::StyleSheet` implementation:
  - `background` is now set as `Some(bg)`.
  - `icon_color` and `text_color` are set as `Color` (not `Option<Color>`).
  - The hover state darkens the background using `.map(|mut c| { c.a = 0.95; c })`.
- Updated all widget theming to use the correct types for each field.

## Extending the Theme

- Add new color constants to the palette as needed.
- Implement additional `StyleSheet` traits for new widgets following the same conventions.
- Reference this guide and `src/ui/theme.rs` for examples.

---

_Last updated: [DATE]_ 