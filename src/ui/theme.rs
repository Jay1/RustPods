//! Theme module implementing the Catppuccin Mocha color scheme for the RustPods UI
//! 
//! This module provides color constants and theme implementations for the Iced UI framework
//! using the Catppuccin Mocha color palette. Catppuccin is a soothing pastel theme designed
//! to be warm and soft, while maintaining good contrast and readability.
//!
//! The module implements StyleSheet traits for various Iced widgets to ensure consistent
//! theming across the application.

use iced::{Color, application, widget::{button, container, text_input, text, rule, scrollable, progress_bar}};

// Catppuccin Mocha Palette - using static instead of const due to Color::from_rgb8 not being const
pub static ROSEWATER: Color = Color::from_rgb(0xf5 as f32 / 255.0, 0xe0 as f32 / 255.0, 0xdc as f32 / 255.0);
pub static FLAMINGO: Color = Color::from_rgb(0xf2 as f32 / 255.0, 0xcd as f32 / 255.0, 0xcd as f32 / 255.0);
pub static PINK: Color = Color::from_rgb(0xf5 as f32 / 255.0, 0xc2 as f32 / 255.0, 0xe7 as f32 / 255.0);
pub static MAUVE: Color = Color::from_rgb(0xcb as f32 / 255.0, 0xa6 as f32 / 255.0, 0xf7 as f32 / 255.0);
pub static RED: Color = Color::from_rgb(0xf3 as f32 / 255.0, 0x8b as f32 / 255.0, 0xa8 as f32 / 255.0);
pub static MAROON: Color = Color::from_rgb(0xeb as f32 / 255.0, 0xa0 as f32 / 255.0, 0xac as f32 / 255.0);
pub static PEACH: Color = Color::from_rgb(0xfa as f32 / 255.0, 0xb3 as f32 / 255.0, 0x87 as f32 / 255.0);
pub static YELLOW: Color = Color::from_rgb(0xf9 as f32 / 255.0, 0xe2 as f32 / 255.0, 0xaf as f32 / 255.0);
pub static GREEN: Color = Color::from_rgb(0xa6 as f32 / 255.0, 0xe3 as f32 / 255.0, 0xa1 as f32 / 255.0);
pub static TEAL: Color = Color::from_rgb(0x94 as f32 / 255.0, 0xe2 as f32 / 255.0, 0xd5 as f32 / 255.0);
pub static SKY: Color = Color::from_rgb(0x89 as f32 / 255.0, 0xdc as f32 / 255.0, 0xeb as f32 / 255.0);
pub static SAPPHIRE: Color = Color::from_rgb(0x74 as f32 / 255.0, 0xc7 as f32 / 255.0, 0xec as f32 / 255.0);
pub static BLUE: Color = Color::from_rgb(0x89 as f32 / 255.0, 0xb4 as f32 / 255.0, 0xfa as f32 / 255.0);
pub static LAVENDER: Color = Color::from_rgb(0xb4 as f32 / 255.0, 0xbe as f32 / 255.0, 0xfe as f32 / 255.0);

// Base colors
pub static TEXT: Color = Color::from_rgb(0xcd as f32 / 255.0, 0xd6 as f32 / 255.0, 0xf4 as f32 / 255.0);
pub static SUBTEXT1: Color = Color::from_rgb(0xba as f32 / 255.0, 0xc2 as f32 / 255.0, 0xde as f32 / 255.0);
pub static SUBTEXT0: Color = Color::from_rgb(0xa6 as f32 / 255.0, 0xad as f32 / 255.0, 0xc8 as f32 / 255.0);
pub static OVERLAY2: Color = Color::from_rgb(0x9a as f32 / 255.0, 0xa1 as f32 / 255.0, 0xb9 as f32 / 255.0);
pub static OVERLAY1: Color = Color::from_rgb(0x7f as f32 / 255.0, 0x84 as f32 / 255.0, 0x9c as f32 / 255.0);
pub static OVERLAY0: Color = Color::from_rgb(0x6c as f32 / 255.0, 0x70 as f32 / 255.0, 0x86 as f32 / 255.0);
pub static SURFACE2: Color = Color::from_rgb(0x58 as f32 / 255.0, 0x5b as f32 / 255.0, 0x70 as f32 / 255.0);
pub static SURFACE1: Color = Color::from_rgb(0x45 as f32 / 255.0, 0x47 as f32 / 255.0, 0x59 as f32 / 255.0);
pub static SURFACE0: Color = Color::from_rgb(0x31 as f32 / 255.0, 0x32 as f32 / 255.0, 0x44 as f32 / 255.0);
pub static BASE: Color = Color::from_rgb(0x1e as f32 / 255.0, 0x1e as f32 / 255.0, 0x2e as f32 / 255.0);
pub static MANTLE: Color = Color::from_rgb(0x18 as f32 / 255.0, 0x18 as f32 / 255.0, 0x1b as f32 / 255.0);
pub static CRUST: Color = Color::from_rgb(0x11 as f32 / 255.0, 0x11 as f32 / 255.0, 0x19 as f32 / 255.0);

/// Theme variants for the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    /// Catppuccin Mocha theme
    #[default]
    CatppuccinMocha,
}

impl application::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> application::Appearance {
        match self {
            Theme::CatppuccinMocha => application::Appearance {
                background_color: BASE,
                text_color: TEXT,
            },
        }
    }
}

impl button::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        match self {
            Theme::CatppuccinMocha => button::Appearance {
                background: Some(SURFACE0.into()),
                border_radius: 2.0.into(),
                border_width: 1.0,
                border_color: OVERLAY0,
                text_color: TEXT,
                ..Default::default()
            },
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        match self {
            Theme::CatppuccinMocha => button::Appearance {
                background: Some(SURFACE1.into()),
                border_color: OVERLAY1,
                text_color: SUBTEXT1,
                ..self.active(style)
            },
        }
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        match self {
            Theme::CatppuccinMocha => button::Appearance {
                background: Some(SURFACE2.into()),
                ..self.active(style)
            },
        }
    }

    fn disabled(&self, _style: &Self::Style) -> button::Appearance {
        match self {
            Theme::CatppuccinMocha => button::Appearance {
                background: Some(MANTLE.into()),
                text_color: OVERLAY0,
                ..Default::default()
            },
        }
    }
}

impl container::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        match self {
            Theme::CatppuccinMocha => container::Appearance {
                text_color: Some(TEXT),
                background: None,
                border_radius: 0.0.into(),
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
        }
    }
}

impl text_input::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _style: &Self::Style) -> text_input::Appearance {
        match self {
            Theme::CatppuccinMocha => text_input::Appearance {
                background: SURFACE0.into(),
                border_radius: 2.0.into(),
                border_width: 1.0,
                border_color: OVERLAY0,
                icon_color: TEXT,
            },
        }
    }

    fn focused(&self, style: &Self::Style) -> text_input::Appearance {
        match self {
            Theme::CatppuccinMocha => text_input::Appearance {
                border_color: BLUE,
                ..self.active(style)
            },
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        match self {
            Theme::CatppuccinMocha => OVERLAY1,
        }
    }

    fn value_color(&self, _style: &Self::Style) -> Color {
        match self {
            Theme::CatppuccinMocha => TEXT,
        }
    }

    fn selection_color(&self, _style: &Self::Style) -> Color {
        match self {
            Theme::CatppuccinMocha => SURFACE2,
        }
    }

    fn disabled(&self, style: &Self::Style) -> text_input::Appearance {
        match self {
            Theme::CatppuccinMocha => text_input::Appearance {
                background: MANTLE.into(),
                border_color: OVERLAY0,
                ..self.active(style)
            },
        }
    }

    fn disabled_color(&self, _style: &Self::Style) -> Color {
        match self {
            Theme::CatppuccinMocha => OVERLAY0,
        }
    }
}

impl text::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: Self::Style) -> text::Appearance {
        match self {
            Theme::CatppuccinMocha => text::Appearance {
                color: Some(TEXT),
            },
        }
    }
}

impl rule::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> rule::Appearance {
        match self {
            Theme::CatppuccinMocha => rule::Appearance {
                color: OVERLAY0,
                width: 1,
                radius: 0.0.into(),
                fill_mode: rule::FillMode::Full,
            },
        }
    }
}

impl scrollable::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _style: &Self::Style) -> scrollable::Scrollbar {
        match self {
            Theme::CatppuccinMocha => scrollable::Scrollbar {
                background: Some(SURFACE0.into()),
                border_radius: 2.0.into(),
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
                scroller: scrollable::Scroller {
                    color: OVERLAY1,
                    border_radius: 2.0.into(),
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                },
            },
        }
    }

    fn hovered(&self, style: &Self::Style, is_mouse_over_scrollbar: bool) -> scrollable::Scrollbar {
        match self {
            Theme::CatppuccinMocha => {
                let mut scrollbar = self.active(style);
                
                if is_mouse_over_scrollbar {
                    scrollbar.scroller.color = OVERLAY2;
                }
                
                scrollbar
            },
        }
    }

    fn dragging(&self, style: &Self::Style) -> scrollable::Scrollbar {
        match self {
            Theme::CatppuccinMocha => {
                let mut scrollbar = self.active(style);
                scrollbar.scroller.color = OVERLAY0;
                scrollbar
            },
        }
    }
}

impl progress_bar::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> progress_bar::Appearance {
        match self {
            Theme::CatppuccinMocha => progress_bar::Appearance {
                background: SURFACE0.into(),
                bar: BLUE.into(),
                border_radius: 2.0.into(),
            },
        }
    }
} 