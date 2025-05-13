//! Battery icon and visualization components
//! 
//! These components provide various ways to display battery status using
//! the Catppuccin Mocha theme colors.

use iced::widget::{container, progress_bar, row, text};
use iced::{alignment, Color, Element, Length};

use crate::ui::Message;
use crate::ui::theme::Theme;
use crate::ui::theme::{RED, PEACH, GREEN, BLUE, SURFACE1, OVERLAY1};

// Add missing color constants at the top level
const YELLOW: Color = Color {
    r: 0.98,
    g: 0.84,
    b: 0.24,
    a: 1.0,
};

const WHITE: Color = Color {
    r: 0.98,
    g: 0.98,
    b: 0.98,
    a: 1.0,
};

/// Create a battery display row with label, level and charging indicator
pub fn battery_display_row<'a>(
    label: &str,
    level: Option<u8>,
    is_charging: bool,
    animation_progress: f32,
) -> Element<'a, Message, iced::Renderer<Theme>> {
    // Create the label
    let label_element = text(label)
        .size(16)
        .width(Length::Fixed(50.0));
    
    // Create the level text
    let level_text = match level {
        Some(level) => format!("{}%", level),
        None => "N/A".to_string(),
    };
    
    let level_element = text(level_text)
        .size(16)
        .width(Length::Fixed(50.0))
        .horizontal_alignment(alignment::Horizontal::Right);
    
    // Create the charging indicator
    let charging_element = if is_charging {
        // Pulse animation for charging icon
        let pulse = (1.0 + (animation_progress * 3.0 * std::f32::consts::PI).sin()) * 0.5;
        text("⚡")
            .size(16)
            .style(pulse_color(pulse))
            .width(Length::Fixed(20.0))
    } else {
        text("")
            .size(16)
            .width(Length::Fixed(20.0))
    };
    
    // Create the progress bar
    let level_f32 = level.unwrap_or(0) as f32 / 100.0;
    let progress = progress_bar(0.0..=1.0, level_f32)
        .style(battery_level_style(level, is_charging))
        .height(18.0);
    
    // Combine everything into a row
    row![
        label_element,
        progress.width(Length::Fill),
        level_element,
        charging_element,
    ]
    .spacing(10)
    .align_items(alignment::Alignment::Center)
    .width(Length::Fill)
    .into()
}

/// Create a battery icon with the given level
pub fn battery_icon_display<'a>(
    level: Option<u8>,
    is_charging: bool,
    size: f32,
    animation_progress: f32,
) -> Element<'a, Message, iced::Renderer<Theme>> {
    // Create a simplified battery icon using a container
    let battery_level = level.unwrap_or(0) as f32 / 100.0;
    
    // Determine color based on level and charging
    let color = battery_color(level, is_charging, animation_progress);
    
    // Create owned copies of colors and values for closures
    let fill_color = color;
    let border_color = OVERLAY1;
    let bg_color = SURFACE1;
    let cap_color = OVERLAY1;
    let width = size * 0.8 * battery_level;
    
    // Create the battery body
    let battery_body = container(
        // Inner fill representing the charge level
        container(iced::widget::Space::new(
            Length::Fixed(width), 
            Length::Fixed(size * 0.4)
        ))
        .style(iced::theme::Container::Custom(Box::new(move |_: &iced::Theme| {
            iced::widget::container::Appearance {
                background: Some(fill_color.into()),
                ..Default::default()
            }
        })))
        .width(Length::Fixed(width))
        .height(Length::Fixed(size * 0.4))
        .align_x(alignment::Horizontal::Left)
    )
    .style(iced::theme::Container::Custom(Box::new(move |_: &iced::Theme| {
        iced::widget::container::Appearance {
            background: Some(bg_color.into()),
            border_radius: 2.0.into(),
            border_width: 1.0,
            border_color,
            ..Default::default()
        }
    })))
    .width(Length::Fixed(size * 0.8))
    .height(Length::Fixed(size * 0.4))
    .padding(1);
    
    // Create the battery cap
    let battery_cap = container(iced::widget::Space::new(
        Length::Fixed(size * 0.1), 
        Length::Fixed(size * 0.2)
    ))
    .style(iced::theme::Container::Custom(Box::new(move |_: &iced::Theme| {
        iced::widget::container::Appearance {
            background: Some(cap_color.into()),
            border_radius: 2.0.into(),
            ..Default::default()
        }
    })))
    .width(Length::Fixed(size * 0.1))
    .height(Length::Fixed(size * 0.2));
    
    // Create charging indicator
    let charging_indicator = if is_charging {
        // Pulse animation for charging icon
        let pulse = (1.0 + (animation_progress * 3.0 * std::f32::consts::PI).sin()) * 0.5;
        text("⚡")
            .size((size * 0.3) as u16)
            .style(pulse_color(pulse))
    } else {
        text("")
            .size((size * 0.3) as u16)
    };
    
    // Combine battery body and cap
    row![
        battery_body,
        battery_cap,
        charging_indicator,
    ]
    .spacing(2)
    .align_items(alignment::Alignment::Center)
    .into()
}

/// Create a battery icon with a label
pub fn battery_with_label<'a>(
    label: &str,
    level: Option<u8>,
    is_charging: bool,
    size: f32,
    animation_progress: f32,
) -> Element<'a, Message, iced::Renderer<Theme>> {
    let icon = battery_icon_display(level, is_charging, size, animation_progress);
    
    // Create level text
    let level_text = match level {
        Some(level) => format!("{}%", level),
        None => "N/A".to_string(),
    };
    
    let text_element = text(format!("{}: {}", label, level_text))
        .size((size * 0.25) as u16)
        .style(battery_text_style(level, is_charging));
    
    // Combine icon and text
    container(
        iced::widget::Column::new()
            .push(icon)
            .push(text_element)
            .spacing(5)
            .align_items(alignment::Alignment::Center)
    )
    .width(Length::Fixed(size * 1.2))
    .height(Length::Fixed(size * 0.8))
    .center_x()
    .center_y()
    .into()
}

/// Get a color for the battery level
fn battery_color(level: Option<u8>, is_charging: bool, animation_progress: f32) -> Color {
    if is_charging {
        // Pulse between two blues
        let pulse = (1.0 + (animation_progress * 2.0 * std::f32::consts::PI).sin()) * 0.5;
        let base_color = BLUE;
        let highlight_color = Color::from_rgb(
            base_color.r * 1.2,
            base_color.g * 1.2,
            base_color.b * 1.2
        );
        
        Color {
            r: base_color.r + (highlight_color.r - base_color.r) * pulse,
            g: base_color.g + (highlight_color.g - base_color.g) * pulse,
            b: base_color.b + (highlight_color.b - base_color.b) * pulse,
            a: 1.0,
        }
    } else if let Some(level) = level {
        if level <= 20 {
            RED
        } else if level <= 50 {
            PEACH
        } else {
            GREEN
        }
    } else {
        OVERLAY1
    }
}

/// Get a style for the battery level progress bar
fn battery_level_style(level: Option<u8>, is_charging: bool) -> iced::theme::ProgressBar {
    // Determine the color now, outside the closure
    let color = if is_charging {
        BLUE
    } else if let Some(level) = level {
        if level <= 20 {
            RED
        } else if level <= 50 {
            PEACH
        } else {
            GREEN
        }
    } else {
        OVERLAY1
    };
    
    // Create a new color that is owned and can be moved into the closure
    let color = color;
    let bg_color = SURFACE1;
    
    iced::theme::ProgressBar::Custom(Box::new(move |_: &iced::Theme| {
        iced::widget::progress_bar::Appearance {
            background: bg_color.into(),
            bar: color.into(),
            border_radius: 2.0.into(),
        }
    }))
}

/// Get a text style for the battery level
fn battery_text_style(level: Option<u8>, is_charging: bool) -> iced::Color {
    if is_charging {
        BLUE // Blue for charging
    } else if let Some(level) = level {
        if level <= 20 {
            RED // Red for low battery
        } else if level <= 50 {
            YELLOW // Yellow for medium battery
        } else {
            GREEN // Green for good battery
        }
    } else {
        WHITE // White for unknown battery
    }
}

/// Get a pulsing color for charging animation
fn pulse_color(pulse: f32) -> iced::Color {
    let base_color = BLUE;
    let factor = (pulse * std::f32::consts::PI).sin() * 0.4 + 0.6; // Range: 0.2 - 1.0
    
    iced::Color {
        r: base_color.r,
        g: base_color.g,
        b: base_color.b,
        a: factor,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_battery_color() {
        // Test charging color - use the exact same logic as the function to verify
        let animation_progress = 0.0;
        let pulse = (1.0 + (animation_progress * 2.0 * std::f32::consts::PI).sin()) * 0.5;
        let base_color = BLUE;
        let highlight_color = Color::from_rgb(
            base_color.r * 1.2,
            base_color.g * 1.2,
            base_color.b * 1.2
        );
        
        let expected_charging_color = Color {
            r: base_color.r + (highlight_color.r - base_color.r) * pulse,
            g: base_color.g + (highlight_color.g - base_color.g) * pulse,
            b: base_color.b + (highlight_color.b - base_color.b) * pulse,
            a: 1.0,
        };
        
        let actual_charging_color = battery_color(Some(50), true, 0.0);
        assert_eq!(actual_charging_color, expected_charging_color);
        
        // Test low battery color
        let low_color = battery_color(Some(10), false, 0.0);
        assert_eq!(low_color, RED);
        
        // Test medium battery color
        let medium_color = battery_color(Some(40), false, 0.0);
        assert_eq!(medium_color, PEACH);
        
        // Test high battery color
        let high_color = battery_color(Some(80), false, 0.0);
        assert_eq!(high_color, GREEN);
        
        // Test unknown battery color
        let unknown_color = battery_color(None, false, 0.0);
        assert_eq!(unknown_color, OVERLAY1);
    }
} 