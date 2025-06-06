//! Minimal, error-free UI entrypoint for RustPods using Iced.
// Only use fields in iced::Settings that are supported by all common Iced versions.
// This file should be rust-analyzer error free.

use crate::ui::state::AppState;
use crate::ui::window_management::{DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH};
use iced::Application;

/// Runs the UI application
pub fn run_ui() -> iced::Result {
    // Create a channel for communication between UI and controller
    let (controller_sender, controller_receiver) = tokio::sync::mpsc::unbounded_channel();

    // Run the Iced application using AppState
    AppState::run(iced::Settings {
        window: iced::window::Settings {
            size: (DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT),
            ..Default::default()
        },
        flags: (controller_sender, controller_receiver),
        id: None,
        default_font: iced::Font::default(),
        default_text_size: 16.0,
        antialiasing: false,
        exit_on_close_request: true,
    })
}