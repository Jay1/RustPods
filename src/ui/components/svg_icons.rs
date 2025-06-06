use std::fmt::Write;
use iced::Color;

/// Generates an SVG string for a refresh/scan icon with an optional rotation animation
///
/// - `animated`: Whether to include a rotation animation
/// - `progress`: If animated, the animation progress from 0.0 to 1.0
///
/// The icon uses `currentColor` for its stroke, allowing it to be
/// styled by Iced's color properties.
pub fn refresh_icon_svg_string(animated: bool, progress: f32) -> String {
    // Calculate rotation based on progress if animated
    let rotation = if animated {
        progress * 360.0
    } else {
        0.0
    };

    let mut svg_string = String::new();
    
    // SVG header with viewBox
    write!(
        &mut svg_string,
        r#"<svg width="24" height="24" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">"#,
    ).unwrap();
    
    // Add a rotate transform if animated
    if animated {
        write!(
            &mut svg_string,
            r#"<g transform="rotate({:.1} 12 12)">"#,
            rotation
        ).unwrap();
    }
    
    // Refresh icon path (circular arrows)
    write!(
        &mut svg_string,
        r#"<path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z" 
        fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>"#,
    ).unwrap();
    
    // Close the transform group if animated
    if animated {
        write!(&mut svg_string, r#"</g>"#).unwrap();
    }
    
    // Add animation if needed
    if animated {
        write!(
            &mut svg_string,
            r#"<animateTransform attributeName="transform" attributeType="XML" type="rotate" from="0 12 12" to="360 12 12" dur="1s" repeatCount="indefinite"/>"#,
        ).unwrap();
    }
    
    // Close the SVG tag
    write!(&mut svg_string, r#"</svg>"#).unwrap();
    svg_string
}

/// Generates an SVG string for a battery icon using the approach from the example
///
/// - `percentage`: The battery fill level, from 0.0 (empty) to 1.0 (full)
/// - `charging`: Whether to show the charging indicator
///
/// The icon uses `currentColor` for its stroke and fill, allowing it to be
/// styled by Iced's color properties.
pub fn battery_icon_svg_string(percentage: f32, charging: bool) -> String {
    // Clamp percentage between 0.0 and 1.0
    let p = percentage.clamp(0.0, 1.0);

    // Define the inner dimensions for the fill level
    let fill_area_x = 4.0;
    let fill_area_y_start = 6.0; // Top of the fillable area
    let fill_area_width = 8.0;
    let fill_area_max_height = 14.0; // Max height of the fillable area

    let actual_fill_height = fill_area_max_height * p;
    // Calculate Y position for the fill rectangle (grows from the bottom)
    let fill_y_position =
        fill_area_y_start + (fill_area_max_height - actual_fill_height);

    let mut svg_string = String::new();
    write!(
        &mut svg_string,
        r#"<svg width="16" height="24" viewBox="0 0 16 24" xmlns="http://www.w3.org/2000/svg">"#,
    ).unwrap();
    
    // Main battery body outline
    write!(
        &mut svg_string,
        r#"<path d="M13 5H3C2.44772 5 2 5.44772 2 6V20C2 20.5523 2.44772 21 3 21H13C13.5523 21 14 20.5523 14 20V6C14 5.44772 13.5523 5 13 5Z" stroke="currentColor" stroke-width="2" fill="none"/>"#,
    ).unwrap();
    
    // Battery terminal
    write!(
        &mut svg_string,
        r#"<path d="M6 3C6 2.44772 6.44772 2 7 2H9C9.55228 2 10 2.44772 10 3V5H6V3Z" fill="currentColor"/>"#,
    ).unwrap();

    // Fill level rectangle (only draw if there's a visible fill)
    if p > 0.01 {
        write!(
            &mut svg_string,
            r#"<rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" fill="currentColor" rx="1"/>"#,
            fill_area_x,
            fill_y_position,
            fill_area_width,
            actual_fill_height
        ).unwrap();
    }

    // Add charging bolt if charging
    if charging {
        write!(
            &mut svg_string,
            r#"<path d="M9 10L7 14H9L7 18L11 13H8.5L10 10H9Z" fill="currentColor"/>"#,
        ).unwrap();
    }

    write!(&mut svg_string, r#"</svg>"#).unwrap();
    svg_string
}

/// Generates an SVG string for a gear/settings icon (Catppuccin Mocha themed)
pub fn settings_icon_svg_string(color: Color) -> String {
    let mut svg_string = String::new();
    let hex = format!("#{:02X}{:02X}{:02X}",
        (color.r * 255.0) as u8,
        (color.g * 255.0) as u8,
        (color.b * 255.0) as u8
    );
    write!(&mut svg_string, r#"<svg width='24' height='24' viewBox='0 0 24 24' fill='none' xmlns='http://www.w3.org/2000/svg'>"#).unwrap();
    write!(&mut svg_string, r#"<g stroke='{}' stroke-width='1.7' stroke-linecap='round' stroke-linejoin='round' fill='{}'>"#, hex, hex).unwrap();
    write!(&mut svg_string, r#"<circle cx='12' cy='12' r='3.5' stroke='{}' stroke-width='2' fill='{}'/>"#, hex, hex).unwrap();
    write!(&mut svg_string, r#"<path d='M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33h.09a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51h.09a1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82v.09a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z' stroke='{}' fill='{}'/>"#, hex, hex).unwrap();
    write!(&mut svg_string, r#"</g></svg>"#).unwrap();
    svg_string
}

/// Generates an SVG string for a generic headset icon (for unknown devices)
pub fn headset_icon_svg_string() -> String {
    let mut svg_string = String::new();
    use std::fmt::Write;
    write!(&mut svg_string, r#"<svg width='32' height='32' viewBox='0 0 32 32' fill='none' xmlns='http://www.w3.org/2000/svg'>"#).unwrap();
    // Headband
    write!(&mut svg_string, r#"<path d='M8 20V16C8 10.4772 12.4772 6 18 6C23.5228 6 28 10.4772 28 16V20' stroke='currentColor' stroke-width='2' fill='none'/>"#).unwrap();
    // Left earcup
    write!(&mut svg_string, r#"<rect x='4' y='20' width='6' height='8' rx='2' fill='currentColor'/>"#).unwrap();
    // Right earcup
    write!(&mut svg_string, r#"<rect x='22' y='20' width='6' height='8' rx='2' fill='currentColor'/>"#).unwrap();
    // Mic (optional, subtle)
    write!(&mut svg_string, r#"<path d='M6 28C6 29.1046 6.89543 30 8 30H24C25.1046 30 26 29.1046 26 28' stroke='currentColor' stroke-width='1.5' fill='none'/>"#).unwrap();
    write!(&mut svg_string, "</svg>").unwrap();
    svg_string
} 