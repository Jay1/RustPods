//! Battery indicator component for displaying AirPods battery status

use crate::ui::{Message, theme::{self, Theme}};
use iced::{
    widget::{Column, Row, Text, Svg},
    Element, Alignment, Length,
};

/// Creates a battery indicator view with icon, percentage, and label
pub fn view(label: &str, level: Option<u8>, is_charging: bool) -> Element<'static, Message, iced::Renderer<Theme>> {
    // Root Column with center alignment and spacing
    Column::new()
        .align_items(Alignment::Center)
        .spacing(5)
        .push({
            // First Element: Icon & Charging Indicator Row
            let mut icon_row = Row::new()
                .align_items(Alignment::Center)
                .push(
                    // Battery SVG icon with dynamic color
                    Svg::new(battery_svg(level))
                        .width(Length::Fixed(24.0))
                        .height(Length::Fixed(24.0))
                );
            
            // Add lightning bolt if charging
            if is_charging {
                icon_row = icon_row.push(
                    Svg::new(iced::widget::svg::Handle::from_memory(crate::assets::ui::CHARGING_ICON))
                        .width(Length::Fixed(12.0))
                        .height(Length::Fixed(12.0))
                );
            }
            
            icon_row
        })
        .push(
            // Second Element: Percentage Text
            Text::new(
                match level {
                    Some(value) => format!("{}%", value),
                    None => "--".to_string(),
                }
            )
            .style(theme::TEXT)
            .size(16.0)
        )
        .push(
            // Third Element: Label Text
            Text::new(label.to_string())
                .style(theme::SUBTEXT1)
                .size(14.0)
        )
        .into()
}

/// Helper function to generate battery SVG handle based on battery level
fn battery_svg(level: Option<u8>) -> iced::widget::svg::Handle {
    // Convert battery level to percentage (0.0 to 1.0)
    let percentage = match level {
        Some(level) => (level as f32) / 100.0,
        None => 0.0, // Show empty battery for unknown level
    };
    
    // Determine color based on battery level
    let color = get_battery_color(level);
    
    // Generate the SVG string with the battery icon and color
    let svg_string = battery_icon_svg_string(percentage, false, &color);
    
    // Create SVG handle from the generated string
    iced::widget::svg::Handle::from_memory(svg_string.into_bytes())
}

/// Generates an SVG string for a battery icon
/// 
/// - `percentage`: The battery fill level, from 0.0 (empty) to 1.0 (full)
/// - `charging`: Whether to show the charging indicator
/// - `color`: The color to use for the battery icon
fn battery_icon_svg_string(percentage: f32, charging: bool, color: &str) -> String {
    use std::fmt::Write;
    
    // Clamp percentage between 0.0 and 1.0
    let p = percentage.clamp(0.0, 1.0);

    // Define the inner dimensions for the fill level
    let fill_area_x = 4.0;
    let fill_area_y_start = 6.0; // Top of the fillable area
    let fill_area_width = 8.0;
    let fill_area_max_height = 14.0; // Max height of the fillable area

    let actual_fill_height = fill_area_max_height * p;
    // Calculate Y position for the fill rectangle (grows from the bottom)
    let fill_y_position = fill_area_y_start + (fill_area_max_height - actual_fill_height);

    let mut svg_string = String::new();
    write!(
        &mut svg_string,
        r#"<svg width="16" height="24" viewBox="0 0 16 24" xmlns="http://www.w3.org/2000/svg">"#,
    ).unwrap();
    
    // Main battery body outline
    write!(
        &mut svg_string,
        r#"<path d="M13 5H3C2.44772 5 2 5.44772 2 6V20C2 20.5523 2.44772 21 3 21H13C13.5523 21 14 20.5523 14 20V6C14 5.44772 13.5523 5 13 5Z" stroke="{}" stroke-width="2" fill="none"/>"#,
        color
    ).unwrap();
    
    // Battery terminal
    write!(
        &mut svg_string,
        r#"<path d="M6 3C6 2.44772 6.44772 2 7 2H9C9.55228 2 10 2.44772 10 3V5H6V3Z" fill="{}"/>"#,
        color
    ).unwrap();

    // Fill level rectangle (only draw if there's a visible fill)
    if p > 0.01 {
        write!(
            &mut svg_string,
            r#"<rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" fill="{}" rx="1"/>"#,
            fill_area_x,
            fill_y_position,
            fill_area_width,
            actual_fill_height,
            color
        ).unwrap();
    }

    // Add charging bolt if charging
    if charging {
        write!(
            &mut svg_string,
            r#"<path d="M9 10L7 14H9L7 18L11 13H8.5L10 10H9Z" fill="{}"/>"#,
            color
        ).unwrap();
    }

    write!(&mut svg_string, r#"</svg>"#).unwrap();
    svg_string
}

/// Get the appropriate color hex string for the battery based on its level
fn get_battery_color(level: Option<u8>) -> String {
    match level {
        Some(level) if level > 20 => "#a6e3a1".to_string(), // Green for good battery (>20%)
        Some(level) if level > 10 => "#f9e2af".to_string(), // Yellow for medium battery (>10%)
        Some(_) => "#f38ba8".to_string(),                   // Red for low battery (<=10%)
        None => "#7f849c".to_string(),                      // Gray for unknown battery
    }
} 