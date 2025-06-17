//! Minimal, error-free UI entrypoint for RustPods using Iced.
// Only use fields in iced::Settings that are supported by all common Iced versions.
// This file should be rust-analyzer error free.

use crate::ui::state::AppState;
use crate::ui::utils::load_window_icon;
use crate::ui::window_management::{DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH};
use iced::Application;

/// Runs the UI application with system tray support
pub fn run_ui() -> iced::Result {
    run_ui_with_options(false)
}

/// Runs the UI application with optional test mode
pub fn run_ui_with_options(_test_battery: bool) -> iced::Result {
    // Create a channel for communication between UI and controller
    let (controller_sender, controller_receiver) = tokio::sync::mpsc::unbounded_channel();

    // Load the application icon with error handling
    let icon = load_window_icon();

    // Run the Iced application using AppState with fixed window properties
    AppState::run(iced::Settings {
        window: iced::window::Settings {
            size: (DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT),
            min_size: Some((DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)),
            max_size: Some((DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)),
            resizable: false,
            decorations: false, // Custom title bar
            transparent: false,
            icon,
            ..Default::default()
        },
        flags: (controller_sender, controller_receiver),
        id: None,
        default_font: iced::Font::with_name("SpaceMono Nerd Font"),
        default_text_size: 16.0,
        antialiasing: false,
        exit_on_close_request: false, // Allow custom handling of close requests for system tray
    })
}
