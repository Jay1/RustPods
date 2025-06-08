//! Connection status wrapper component
//!
//! A wrapper around ConnectionStatus that owns it and can render it without borrowing issues.

use iced::Element;

use crate::ui::theme::Theme;
use crate::ui::{Message, UiComponent};

/// A wrapper for ConnectionStatus that owns it and can render it
#[derive(Debug, Clone)]
pub struct ConnectionStatusWrapper {
    /// Whether the device is connected
    pub is_connected: bool,
    /// Whether scanning is in progress
    pub is_scanning: bool,
    /// Animation progress (0.0-1.0)
    pub animation_progress: f32,
}

impl ConnectionStatusWrapper {
    /// Create a new connection status wrapper
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

    /// Render the connection status directly
    pub fn render(&self) -> Element<'static, Message, iced::Renderer<Theme>> {
        // Create a representation of the connection status that doesn't borrow 'self'
        match self.is_scanning {
            true => self.render_scanning_status(),
            false => self.render_connection_status(),
        }
    }

    // Helper method to render the scanning status
    fn render_scanning_status(&self) -> Element<'static, Message, iced::Renderer<Theme>> {
        let text = "Scanning for devices...";
        let color = crate::ui::theme::BLUE;
        self.render_status(text, color, true)
    }

    // Helper method to render the connection status
    fn render_connection_status(&self) -> Element<'static, Message, iced::Renderer<Theme>> {
        let (text, color) = if self.is_connected {
            ("Connected", crate::ui::theme::GREEN)
        } else {
            ("No device connected", crate::ui::theme::RED)
        };
        self.render_status(text, color, false)
    }

    // Helper method to render the status with given text and color
    fn render_status(
        &self,
        status_text: &'static str,
        color: iced::Color,
        is_scanning: bool,
    ) -> Element<'static, Message, iced::Renderer<Theme>> {
        use iced::alignment;
        use iced::widget::{container, row, text};

        // Clone progress for use in the rendering
        let progress = self.animation_progress;

        // Status indicator dot
        let dot_size = if is_scanning {
            // Pulsing effect for scanning
            let pulse = (1.0 + (progress * 2.0 * std::f32::consts::PI).sin()) * 0.5;
            8.0 + (4.0 * pulse) // Size between 8 and 12px
        } else {
            10.0 // Fixed size
        };

        // Create status dot
        let status_dot = container(iced::widget::Space::new(
            iced::Length::Fixed(dot_size),
            iced::Length::Fixed(dot_size),
        ))
        .style(iced::theme::Container::Custom(Box::new(
            move |_: &iced::Theme| iced::widget::container::Appearance {
                background: Some(color.into()),
                border_radius: dot_size.into(),
                border_width: 0.0,
                border_color: iced::Color::TRANSPARENT,
                text_color: None
            },
        )))
        .width(iced::Length::Fixed(dot_size))
        .height(iced::Length::Fixed(dot_size))
        .center_y();

        // Create text with appropriate color
        let status_label = text(status_text).style(color).size(16);

        // Create additional scanning animation if scanning
        let scanning_animation = if is_scanning {
            // Add a loading animation
            let dots = ".".repeat(((progress * 3.0) as usize % 4) + 1);
            text(dots)
                .style(crate::ui::theme::BLUE)
                .size(16)
                .width(iced::Length::Fixed(30.0))
                .horizontal_alignment(alignment::Horizontal::Left)
        } else {
            // Empty space for consistency
            text("").size(16).width(iced::Length::Fixed(30.0))
        };

        // Store colors in owned variables that can be moved into the closure
        let bg_color = crate::ui::theme::SURFACE0;
        let border_color = crate::ui::theme::OVERLAY0;
        let text_color = crate::ui::theme::SUBTEXT0;

        // Combine elements
        container(
            row![
                status_dot,
                iced::widget::Space::new(iced::Length::Fixed(10.0), iced::Length::Fixed(1.0)),
                status_label,
                scanning_animation,
            ]
            .spacing(5)
            .align_items(alignment::Alignment::Center),
        )
        .style(iced::theme::Container::Custom(Box::new(
            move |_: &iced::Theme| iced::widget::container::Appearance {
                background: Some(bg_color.into()),
                border_radius: 4.0.into(),
                border_width: 1.0,
                border_color,
                text_color: Some(text_color)
            },
        )))
        .padding(8)
        .into()
    }
}

impl UiComponent for ConnectionStatusWrapper {
    fn view(&self) -> Element<'_, Message, iced::Renderer<Theme>> {
        self.render()
    }
}

// Implement From<ConnectionStatusWrapper> for Element to allow direct use in column! macro
impl From<ConnectionStatusWrapper> for Element<'_, Message, iced::Renderer<Theme>> {
    fn from(wrapper: ConnectionStatusWrapper) -> Self {
        wrapper.render()
    }
}
