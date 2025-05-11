//! UI module for the application

// Module exports
mod app;
pub mod components;
pub mod state;
mod message;
mod system_tray;
mod main_window;
mod settings_window;
pub mod theme;

// Re-exports for easier access
pub use app::{run_ui, view, subscription};
pub use state::AppState;
pub use message::Message;
pub use system_tray::SystemTray;
pub use main_window::MainWindow;
pub use settings_window::{SettingsWindow, SettingsTab};

/// Manager for UI operations
pub struct UiManager;

/// Trait for UI components that can be rendered
pub trait UiComponent {
    /// Convert the component to an element
    fn view(&self) -> iced::Element<'static, Message, iced::Renderer<theme::Theme>>;
} 