use std::time::Duration;
use iced::widget::{column, container, text};
use iced::{Element, Settings, Sandbox};
use crate::ui::{AppState, Message};

/// Runs the UI application
pub fn run_ui() -> iced::Result {
    // Import the Sandbox trait to have run() available
    use iced::Sandbox;
    AppState::run(Settings::default())
}

/// Creates a subscription for updating the UI
fn subscription(_state: &AppState) -> iced::Subscription<Message> {
    // Create a timer subscription that ticks every second
    iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick)
}

/// Creates the user interface
pub fn view(_state: &AppState) -> Element<Message> {
    let content = column![
        text("RustPods").size(30),
        text("AirPods Battery Monitor").size(20),
        text("MVP Version").size(16),
    ].spacing(10);
    
    container(content)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .padding(20)
        .center_x()
        .center_y()
        .into()
} 