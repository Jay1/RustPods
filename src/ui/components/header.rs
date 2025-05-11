use iced::widget::{button, row, text, container};
use iced::{Length};

use crate::ui::{Message, UiComponent};

/// Component for the application header
#[derive(Debug, Clone)]
pub struct Header {
    /// Whether scanning is active
    is_scanning: bool,
    /// Whether automatic scanning is enabled
    auto_scan: bool,
}

impl Header {
    /// Create a new header
    pub fn new(is_scanning: bool, auto_scan: bool) -> Self {
        Self {
            is_scanning,
            auto_scan,
        }
    }
}

impl UiComponent for Header {
    fn view(&self) -> iced::Element<'static, Message, iced::Renderer<crate::ui::theme::Theme>> {
        let title = text("RustPods - AirPods Battery Monitor")
            .size(28)
            .width(Length::Shrink);

        let scan_button = if self.is_scanning {
            button("Stop Scan")
                .on_press(Message::StopScan)
        } else {
            button("Start Scan")
                .on_press(Message::StartScan)
        };

        // Wrap the button in a container for styling
        let styled_scan_button = scan_button;

        // Iced 0.13 checkbox only takes 2 arguments - create a toggleable button instead
        let toggle_text = if self.auto_scan { "Auto-scan: On" } else { "Auto-scan: Off" };
        let toggle_button = button(text(toggle_text))
            .on_press(Message::ToggleAutoScan(!self.auto_scan))
            .width(Length::Shrink);
        
        let auto_scan_toggle = toggle_button;

        let header_row = row![
            title,
            styled_scan_button,
            auto_scan_toggle,
        ]
        .spacing(20)
        .padding(20)
        .width(Length::Fill);

        container(header_row)
            .width(Length::Fill)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_creation() {
        // Create with scanning active
        let header = Header::new(true, true);
        assert!(header.is_scanning);
        assert!(header.auto_scan);

        // Create with scanning inactive
        let header = Header::new(false, false);
        assert!(!header.is_scanning);
        assert!(!header.auto_scan);
    }
} 