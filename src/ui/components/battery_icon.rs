//! Battery icon and visualization components
//!
//! These components provide various ways to display battery status using
//! the Catppuccin Mocha theme colors.

use iced::{
    alignment,
    widget::{column, container, progress_bar, row, text, Svg},
    Alignment, Color, Element, Length,
};

use crate::ui::theme;
use crate::ui::theme::Theme;
use crate::ui::Message;

/// Create a battery display row with label, level and charging indicator
pub fn battery_display_row<'a>(
    label: &str,
    level: Option<u8>,
    is_charging: bool,
    animation_progress: f32,
) -> Element<'a, Message, iced::Renderer<Theme>> {
    // Create the label
    let label_element = text(label).size(16).width(Length::Fixed(50.0));

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
        text("").size(16).width(Length::Fixed(20.0))
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
    _animation_progress: f32,
) -> Element<'a, Message, iced::Renderer<Theme>> {
    // Get battery level as percentage (0.0 to 1.0)
    let battery_level = level.unwrap_or(0);
    let percentage = battery_level as f32 / 100.0;

    // Determine color based on level and charging
    let color = if is_charging {
        theme::BLUE
    } else if battery_level <= 20 {
        theme::RED
    } else if battery_level <= 50 {
        theme::YELLOW
    } else {
        theme::GREEN
    };

    // Convert color to hex string for SVG
    let hex_color = format!(
        "#{:02X}{:02X}{:02X}",
        (color.r * 255.0) as u8,
        (color.g * 255.0) as u8,
        (color.b * 255.0) as u8
    );

    // Generate custom colored SVG string for the battery icon
    let svg_string = create_colored_battery_svg(percentage, is_charging, &hex_color);

    // Convert string to bytes for Handle::from_memory
    let svg_bytes = svg_string.into_bytes();

    // Create SVG element using the actual size parameter (bug fix!)
    let svg_element = Svg::new(iced::widget::svg::Handle::from_memory(svg_bytes))
        .width(Length::Fixed(size)) // Use the actual size parameter
        .height(Length::Fixed(size * 0.6)); // Maintain aspect ratio for horizontal battery

    // Return the SVG directly without container wrapper since colors are baked into SVG
    svg_element.into()
}

/// Create a colored SVG battery icon with specific hex color
fn create_colored_battery_svg(percentage: f32, charging: bool, hex_color: &str) -> String {
    // Clamp percentage between 0.0 and 1.0
    let p = percentage.clamp(0.0, 1.0);

    // Define horizontal battery dimensions - better proportions for visibility
    let battery_width = 48.0; // Reasonable width
    let battery_height = 24.0; // Reasonable height
    let battery_x = 4.0;
    let battery_y = 6.0;

    // Terminal dimensions (small nub on the right)
    let terminal_width = 4.0;
    let terminal_height = 10.0;
    let terminal_x = battery_x + battery_width;
    let terminal_y = battery_y + 7.0; // Centered vertically

    // Fill area (inside the battery)
    let fill_padding = 2.0;
    let fill_x = battery_x + fill_padding;
    let fill_y = battery_y + fill_padding;
    let fill_max_width = battery_width - (fill_padding * 2.0);
    let fill_height = battery_height - (fill_padding * 2.0);
    let fill_width = fill_max_width * p;

    let mut svg_string = String::new();
    use std::fmt::Write;

    // Define neutral color for battery outline - much lighter/whiter for visibility
    let gray_color = "#CDD6F4"; // Much lighter - almost white for excellent visibility

    write!(
        &mut svg_string,
        r#"<svg width="60" height="36" viewBox="0 0 60 36" xmlns="http://www.w3.org/2000/svg">"#,
    )
    .unwrap();

    // Main battery body outline (horizontal rectangle) - MUCH thicker stroke for visibility
    write!(
        &mut svg_string,
        r#"<rect x="{}" y="{}" width="{}" height="{}" stroke="{}" stroke-width="4.0" fill="none" rx="2"/>"#,
        battery_x, battery_y, battery_width, battery_height, gray_color
    ).unwrap();

    // Battery terminal (small nub on the right) - thicker for visibility
    write!(
        &mut svg_string,
        r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" stroke="{}" stroke-width="2.0" rx="1"/>"#,
        terminal_x, terminal_y, terminal_width, terminal_height, gray_color, gray_color
    ).unwrap();

    // Fill level rectangle with the actual battery level color (grows from left to right)
    if p > 0.01 {
        write!(
            &mut svg_string,
            r#"<rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" fill="{}" rx="1"/>"#,
            fill_x, fill_y, fill_width, fill_height, hex_color
        )
        .unwrap();
    }

    // Add charging bolt if charging (use same color as fill) - better sized and positioned
    if charging {
        // Lightning bolt positioned in center, properly sized
        write!(
            &mut svg_string,
            r#"<path d="M26 14L22 20H26L22 26L32 20H28L32 14Z" fill="{}" stroke="{}" stroke-width="1"/>"#,
            hex_color, gray_color
        ).unwrap();
    }

    write!(&mut svg_string, r#"</svg>"#).unwrap();
    svg_string
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
            .align_items(alignment::Alignment::Center),
    )
    .width(Length::Fixed(size * 1.2))
    .height(Length::Fixed(size * 0.8))
    .center_x()
    .center_y()
    .into()
}

/// Get a color for the battery level
#[allow(dead_code)]
fn battery_color(level: Option<u8>, is_charging: bool, animation_progress: f32) -> Color {
    if is_charging {
        // Pulse between two blues
        let pulse = (1.0 + (animation_progress * 2.0 * std::f32::consts::PI).sin()) * 0.5;
        let base_color = theme::BLUE;
        let highlight_color =
            Color::from_rgb(base_color.r * 1.2, base_color.g * 1.2, base_color.b * 1.2);

        Color {
            r: base_color.r + (highlight_color.r - base_color.r) * pulse,
            g: base_color.g + (highlight_color.g - base_color.g) * pulse,
            b: base_color.b + (highlight_color.b - base_color.b) * pulse,
            a: 1.0,
        }
    } else if let Some(level) = level {
        if level <= 20 {
            theme::RED
        } else if level <= 50 {
            theme::PEACH
        } else {
            theme::GREEN
        }
    } else {
        theme::OVERLAY1
    }
}

/// Get a style for the battery level progress bar
fn battery_level_style(level: Option<u8>, is_charging: bool) -> iced::theme::ProgressBar {
    // Determine the color now, outside the closure
    let color = if is_charging {
        theme::BLUE
    } else if let Some(level) = level {
        if level <= 20 {
            theme::RED
        } else if level <= 50 {
            theme::PEACH
        } else {
            theme::GREEN
        }
    } else {
        theme::OVERLAY1
    };

    // Create a new color that is owned and can be moved into the closure
    /* color already defined */
    let bg_color = theme::SURFACE1;

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
        theme::BLUE // Blue for charging
    } else if let Some(level) = level {
        if level <= 20 {
            theme::RED // Red for low battery
        } else if level <= 50 {
            theme::YELLOW // Yellow for medium battery
        } else {
            theme::GREEN // Green for good battery
        }
    } else {
        theme::TEXT // White-ish text for unknown battery
    }
}

/// Get a pulsing color for charging animation
fn pulse_color(pulse: f32) -> iced::Color {
    let base_color = theme::BLUE;
    let factor = (pulse * std::f32::consts::PI).sin() * 0.4 + 0.6; // Range: 0.2 - 1.0

    iced::Color {
        r: base_color.r,
        g: base_color.g,
        b: base_color.b,
        a: factor,
    }
}

/// Create a circular progress SVG for battery display
fn create_circular_battery_svg(level: u8, is_charging: bool) -> String {
    // Clamp level between 0 and 100
    let level = level.min(100);
    
    // SVG circle parameters - increased by 50%
    let radius = 48.0;  // Was 32.0
    let stroke_width = 12.0;  // Was 8.0, increased proportionally
    let center = 60.0; // Was 40.0, SVG center point
    let circumference = 2.0 * std::f32::consts::PI * radius;
    
    // Calculate progress arc length (starting from top, clockwise)
    let progress = (level as f32 / 100.0) * circumference;
    let dash_offset = circumference - progress;
    
    // Catppuccin Mocha theme colors
    let bg_color = "#45475a"; // SURFACE1 - dark, subtle color
    let progress_color = "#cdd6f4"; // TEXT - bright, contrasting color  
    let charging_color = "#f9e2af"; // YELLOW - bright color for lightning bolt
    
    let mut svg = String::new();
    use std::fmt::Write;
    
    // Start SVG - increased size from 80x80 to 120x120
    write!(&mut svg, r#"<svg width="120" height="120" viewBox="0 0 120 120" xmlns="http://www.w3.org/2000/svg">"#).unwrap();
    
    // Background circle
    write!(&mut svg, 
        r#"<circle cx="{}" cy="{}" r="{}" fill="none" stroke="{}" stroke-width="{}"/>"#,
        center, center, radius, bg_color, stroke_width
    ).unwrap();
    
    // Progress arc (only if level > 0)
    if level > 0 {
        write!(&mut svg,
            r#"<circle cx="{}" cy="{}" r="{}" fill="none" stroke="{}" stroke-width="{}" stroke-dasharray="{}" stroke-dashoffset="{}" stroke-linecap="round" transform="rotate(-90 {} {})"/>"#,
            center, center, radius, progress_color, stroke_width, circumference, dash_offset, center, center
        ).unwrap();
    }
    
    // Charging lightning bolt icon - scaled and repositioned for larger circle
    if is_charging {
        write!(&mut svg,
            r#"<path d="M67.5 45L52.5 63H63L56.25 75L75 57H64.5L70.5 45Z" fill="{}" stroke="none"/>"#,
            charging_color
        ).unwrap();
    }
    
    write!(&mut svg, r#"</svg>"#).unwrap();
    svg
}

/// Create a minimalist circular battery widget inspired by modern UI design
pub fn view_circular_battery_widget<'a>(
    level: u8,
    is_charging: bool,
) -> Element<'a, Message, iced::Renderer<Theme>> {
    // Store Catppuccin Mocha theme colors in owned variables that can be moved into the closure
    let bg_color = theme::BASE; // Dark background
    let border_color = theme::SURFACE0; // Subtle border
    let text_color = theme::TEXT; // Light text

    // Create circular progress SVG
    let svg_string = create_circular_battery_svg(level, is_charging);
    let svg_bytes = svg_string.into_bytes();
    let svg_element = Svg::new(iced::widget::svg::Handle::from_memory(svg_bytes))
        .width(Length::Fixed(120.0))  // Increased from 80.0 to 120.0
        .height(Length::Fixed(120.0)); // Increased from 80.0 to 120.0

    // Create the main container with fixed dimensions
    let main_container = container(
        column![
            // Circular battery progress indicator
            svg_element,
            // Battery percentage text (keeping same size as requested)
            text(format!("{}%", level))
                .size(24)
                .style(text_color)
        ]
        .spacing(10)
        .align_items(Alignment::Center)
    )
    .width(Length::Fixed(200.0))     // Increased from 160.0 to accommodate larger ring
    .height(Length::Fixed(200.0))    // Increased from 160.0 to accommodate larger ring
    .style(iced::theme::Container::Custom(Box::new(
        move |_: &iced::Theme| container::Appearance {
            background: Some(bg_color.into()),
            border_radius: 24.0.into(),
            border_width: 1.0,
            border_color,
            text_color: None,
        },
    )))
    .center_x()
    .center_y();

    main_container.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_circular_battery_svg() {
        // Test SVG generation with different battery levels
        let svg_25 = create_circular_battery_svg(25, false);
        assert!(svg_25.contains("svg"));
        assert!(svg_25.contains("circle"));
        
        let svg_75_charging = create_circular_battery_svg(75, true);
        assert!(svg_75_charging.contains("svg"));
        assert!(svg_75_charging.contains("circle"));
        assert!(svg_75_charging.contains("path")); // Lightning bolt
        
        // Test edge cases
        let svg_0 = create_circular_battery_svg(0, false);
        assert!(svg_0.contains("svg"));
        
        let svg_100 = create_circular_battery_svg(100, false);
        assert!(svg_100.contains("svg"));
        
        // Test clamping
        let svg_over_100 = create_circular_battery_svg(150, true);
        assert!(svg_over_100.contains("svg"));
    }

    #[test]
    fn test_view_circular_battery_widget() {
        // Test widget creation with various parameters
        let widget_25 = view_circular_battery_widget(25, false);
        let widget_75_charging = view_circular_battery_widget(75, true);
        let widget_0 = view_circular_battery_widget(0, false);
        let widget_100 = view_circular_battery_widget(100, false);
        
        // Widgets should be created without panicking
        let _ = widget_25;
        let _ = widget_75_charging;
        let _ = widget_0;
        let _ = widget_100;
    }
}
