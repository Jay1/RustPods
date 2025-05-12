//! UI module for the application

// Module exports
mod app;
mod state_app;
pub mod components;
pub mod state;
pub mod state_manager;
mod message;
mod system_tray;
mod system_tray_controller;
mod main_window;
mod settings_window;
pub mod theme;
pub mod keyboard_shortcuts;
pub mod window_management;
pub mod window_visibility;
pub mod form_validation;
pub mod test_helpers;

// Re-exports for easier access
pub use app::{run_ui, view, subscription};
pub use state_app::run_state_ui;
pub use state::AppState;
pub use message::Message;
pub use system_tray::SystemTray;
pub use system_tray_controller::SystemTrayController;
pub use main_window::MainWindow;
pub use settings_window::{SettingsWindow, SettingsTab};
pub use state_manager::StateManager;
pub use keyboard_shortcuts::{KeyboardShortcut, KeyboardShortcutManager, handle_events};
pub use window_management::{WindowInteraction, DragRegion};
pub use window_visibility::{WindowVisibilityManager, WindowPosition};
pub use form_validation::{ValidationRule, FormValidator};

/// Manager for UI operations
pub struct UiManager;

/// Trait for UI components that can be rendered
pub trait UiComponent {
    /// Convert the component to an element
    fn view(&self) -> iced::Element<'_, Message, iced::Renderer<theme::Theme>>;
} 