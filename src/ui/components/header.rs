use iced::widget::{row, text, container};
use iced::{Length};

use crate::ui::{UiComponent};

/// Component for the application header
#[derive(Debug, Clone)]
pub struct Header;

impl Header {
    /// Create a new header
    pub fn new() -> Self {
        Self
    }
}

impl UiComponent for Header {
    fn view(&self) -> iced::Element<'static, crate::ui::Message, iced::Renderer<crate::ui::theme::Theme>> {
        let title = text("RustPods - AirPods Battery Monitor")
            .size(28)
            .width(Length::Shrink);

        let header_row = row![
            title,
        ]
        .spacing(20)
        .padding(20)
        .width(Length::Fill);

        container(header_row)
            .width(Length::Fill)
            .into()
    }
} 