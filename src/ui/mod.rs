//! UI module for the application

// Module exports
mod app;
pub mod components;
mod message;
pub mod state;
pub mod state_manager;
mod system_tray;
// mod system_tray_controller; // Keep controller disabled for now
pub mod form_validation;
pub mod keyboard_shortcuts;
mod main_window;
mod settings_window;
pub mod test_helpers;
pub mod theme;
pub mod utils;
pub mod window_management;
pub mod window_visibility;

// Re-exports for easier access
pub use app::{run_ui, run_ui_with_options};
pub use message::Message;
pub use state::AppState;
pub use system_tray::SystemTray;
// pub use system_tray_controller::SystemTrayController; // Keep controller disabled
pub use form_validation::{FormValidator, ValidationRule};
pub use keyboard_shortcuts::{handle_events, KeyboardShortcut, KeyboardShortcutManager};
pub use main_window::MainWindow;
pub use settings_window::SettingsWindow;
pub use state_manager::StateManager;
pub use window_management::{DragRegion, WindowInteraction};
pub use window_visibility::{WindowPosition, WindowVisibilityManager};

/// Manager for UI operations
pub struct UiManager;

/// Trait for UI components that can be rendered
pub trait UiComponent {
    /// Convert the component to an element
    fn view(&self) -> iced::Element<'_, Message, iced::Renderer<theme::Theme>>;
}
