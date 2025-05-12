//! Connection status component with animated transitions
//!
//! Displays connection status (connected, disconnected, scanning) with animations.

use iced::widget::{container, row, text};
use iced::{alignment, Color, Element, Length};

use crate::ui::{Message, UiComponent};
use crate::ui::theme::Theme;
use crate::ui::theme::{GREEN, RED, BLUE, OVERLAY0, SUBTEXT0, SURFACE0};

/// A UI component that displays the current connection status
#[derive(Clone, Debug)]
pub struct ConnectionStatus {
    /// Whether the device is connected
    pub is_connected: bool,
    /// Whether scanning is in progress
    pub is_scanning: bool,
    /// Animation progress (0.0-1.0)
    pub animation_progress: f32,
}

impl ConnectionStatus {
    /// Create a new connection status component
    pub fn new(connected: bool, scanning: bool) -> Self {
        Self {
            is_connected: connected,
            is_scanning: scanning,
            animation_progress: 0.0,
        }
    }
    
    /// Set the animation progress
    pub fn with_animation_progress(mut self, progress: f32) -> Self {
        self.animation_progress = progress;
        self
    }
    
    /// Get the connection status text
    fn status_text(&self) -> &'static str {
        if self.is_scanning {
            "Scanning for devices..."
        } else if self.is_connected {
            "Connected"
        } else {
            "No device connected"
        }
    }
    
    /// Get the connection status color
    fn status_color(&self) -> Color {
        if self.is_scanning {
            // Pulse between two blues during scanning
            let pulse = (1.0 + (self.animation_progress * 3.0 * std::f32::consts::PI).sin()) * 0.5;
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
        } else if self.is_connected {
            GREEN
        } else {
            RED
        }
    }
    
    /// Create the status indicator dot
    fn create_status_indicator(&self) -> Element<'_, Message, iced::Renderer<Theme>> {
        // Status indicator dot
        let status_color = self.status_color();
        
        let dot_size = if self.is_scanning {
            // Pulsing effect for scanning
            let pulse = (1.0 + (self.animation_progress * 2.0 * std::f32::consts::PI).sin()) * 0.5;
            8.0 + (4.0 * pulse) // Size between 8 and 12px
        } else {
            10.0 // Fixed size
        };
        
        // Clone the color and size to own them before moving into the closure
        let status_color = status_color;
        let dot_size_value = dot_size;
        
        container(iced::widget::Space::new(
            Length::Fixed(dot_size),
            Length::Fixed(dot_size)
        ))
        .style(iced::theme::Container::Custom(Box::new(move |_: &iced::Theme| {
            iced::widget::container::Appearance {
                background: Some(status_color.into()),
                border_radius: dot_size_value.into(),
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
                ..Default::default()
            }
        })))
        .width(Length::Fixed(dot_size))
        .height(Length::Fixed(dot_size))
        .center_y()
        .into()
    }
}

impl UiComponent for ConnectionStatus {
    fn view(&self) -> Element<'_, Message, iced::Renderer<Theme>> {
        let status_text = self.status_text();
        let status_color = self.status_color();
        
        // Create status dot
        let status_dot = self.create_status_indicator();
        
        // Create text with appropriate color
        let status_label = text(status_text)
            .style(status_color)
            .size(16);
        
        // Create additional scanning animation if scanning
        let scanning_animation = if self.is_scanning {
            // Add a loading animation
            let dots = ".".repeat(((self.animation_progress * 3.0) as usize % 4) + 1);
            text(dots)
                .style(BLUE)
                .size(16)
                .width(Length::Fixed(30.0))
                .horizontal_alignment(alignment::Horizontal::Left)
        } else {
            // Empty space for consistency
            text("")
                .size(16)
                .width(Length::Fixed(30.0))
        };
        
        // Store colors in owned variables that can be moved into the closure
        let bg_color = SURFACE0;
        let border_color = OVERLAY0;
        let text_color = SUBTEXT0;
        
        // Combine elements
        container(
            row![
                status_dot,
                iced::widget::Space::new(Length::Fixed(10.0), Length::Fixed(1.0)),
                status_label,
                scanning_animation,
            ]
            .spacing(5)
            .align_items(alignment::Alignment::Center)
        )
        .style(iced::theme::Container::Custom(Box::new(move |_: &iced::Theme| {
            iced::widget::container::Appearance {
                background: Some(bg_color.into()),
                border_radius: 4.0.into(),
                border_width: 1.0,
                border_color,
                text_color: Some(text_color),
                ..Default::default()
            }
        })))
        .padding(8)
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_connection_status_text() {
        // Connected state
        let connected = ConnectionStatus::new(true, false);
        assert_eq!(connected.status_text(), "Connected");
        
        // Disconnected state
        let disconnected = ConnectionStatus::new(false, false);
        assert_eq!(disconnected.status_text(), "No device connected");
        
        // Scanning state (takes precedence over connected state)
        let scanning = ConnectionStatus::new(false, true);
        assert_eq!(scanning.status_text(), "Scanning for devices...");
        
        // Scanning while connected (scanning takes precedence)
        let scanning_connected = ConnectionStatus::new(true, true);
        assert_eq!(scanning_connected.status_text(), "Scanning for devices...");
    }
    
    #[test]
    fn test_animation_progress() {
        let status = ConnectionStatus::new(true, false)
            .with_animation_progress(0.5);
        
        assert_eq!(status.animation_progress, 0.5);
    }
} 