use iced::widget::{container, row, column, text, Space, progress_bar};
use iced::{Element, Length, Color};

use crate::ui::Message;

/// Create a battery display row with color-coded indicators
pub fn battery_display_row(
    label: &str, 
    level: Option<u8>, 
    is_charging: bool
) -> Element<'static, Message> {
    match level {
        Some(level) => {
            // Determine color based on battery level and charging status
            let color = if is_charging {
                Color::from_rgb(0.2, 0.6, 0.8) // Blue for charging
            } else if level <= 20 {
                Color::from_rgb(0.8, 0.2, 0.2) // Red for low battery
            } else if level <= 50 {
                Color::from_rgb(0.9, 0.6, 0.1) // Yellow/orange for medium battery
            } else {
                Color::from_rgb(0.2, 0.7, 0.2) // Green for good battery
            };
            
            let level_f32 = level as f32 / 100.0;
            
            row![
                // Label
                text(label).width(Length::Shrink),
                
                Space::with_width(Length::Fill),
                
                // Battery visual representation
                progress_bar(0.0..=1.0, level_f32)
                    .width(Length::Fixed(100.0))
                    .height(Length::Fixed(16.0))
                    .style(progress_style(color)),
                
                Space::with_width(Length::Fixed(10.0)),
                
                // Percentage text and charging indicator
                text(format!("{}%{}", level, if is_charging { " ⚡" } else { "" }))
                    .style(iced::theme::Text::Color(color))
                    .width(Length::Shrink),
            ]
            .spacing(10)
            .align_items(iced::Alignment::Center)
            .width(Length::Fill)
            .into()
        }
        None => {
            row![
                text(label)
                    .width(Length::Shrink),
                Space::with_width(Length::Fill),
                text("Not connected")
                    .style(iced::theme::Text::Color(Color::from_rgb(0.7, 0.7, 0.7)))
                    .width(Length::Shrink)
            ]
            .spacing(10)
            .padding(5)
            .align_items(iced::Alignment::Center)
            .width(Length::Fill)
            .into()
        }
    }
}

/// Create a battery icon display in a row with a label
pub fn battery_with_label(
    label: &str,
    level: Option<u8>,
    is_charging: bool
) -> Element<'static, Message> {
    let battery = battery_icon_display(level.map(|l| (l, is_charging)));
    
    column![
        text(label)
            .size(14)
            .horizontal_alignment(iced::alignment::Horizontal::Center),
        battery,
    ]
    .spacing(5)
    .align_items(iced::Alignment::Center)
    .into()
}

/// Create a compact battery icon display
pub fn battery_icon_display(battery: Option<(u8, bool)>) -> Element<'static, Message> {
    match battery {
        Some((level, is_charging)) => {
            // Determine color based on battery level and charging status
            let color = if is_charging {
                Color::from_rgb(0.2, 0.6, 0.8) // Blue for charging
            } else if level <= 20 {
                Color::from_rgb(0.8, 0.2, 0.2) // Red for low battery
            } else if level <= 50 {
                Color::from_rgb(0.9, 0.6, 0.1) // Yellow/orange for medium battery
            } else {
                Color::from_rgb(0.2, 0.7, 0.2) // Green for good battery
            };
            
            // Create a simple representation using a progress bar
            container(
                row![
                    progress_bar(0.0..=1.0, level as f32 / 100.0)
                        .style(progress_style(color))
                        .width(Length::Fill),
                    
                    // Add charging indicator if needed
                    if is_charging {
                        text("⚡").size(12)
                    } else {
                        text("").size(0)  // Empty text instead of Space
                    }
                ]
                .spacing(2)
                .width(Length::Fill)
                .height(Length::Fill)
            )
            .width(Length::Fixed(30.0))
            .height(Length::Fixed(15.0))
            .padding(2)
            .into()
        }
        None => {
            container(
                progress_bar(0.0..=1.0, 0.0)
                    .style(progress_style(Color::from_rgb(0.3, 0.3, 0.3)))
                    .width(Length::Fill)
            )
            .width(Length::Fixed(30.0))
            .height(Length::Fixed(15.0))
            .padding(2)
            .into()
        }
    }
}

/// Helper function to create a progress bar style with a given color
fn progress_style(color: Color) -> impl Fn(&iced::Theme) -> iced::widget::progress_bar::Appearance + 'static {
    move |_theme: &iced::Theme| {
        iced::widget::progress_bar::Appearance {
            background: iced::Background::Color(Color::from_rgb(0.2, 0.2, 0.2)),
            bar: iced::Background::Color(color),
            border_radius: 4.0.into(),
        }
    }
} 