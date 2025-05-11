//! UI module for the application

// Module exports
mod app;
pub mod components;
pub mod state;
mod message;
mod system_tray;

// Re-exports for easier access
pub use app::run_ui;
pub use state::AppState;
pub use message::Message;
pub use system_tray::SystemTray;

/// Manager for UI operations
pub struct UiManager;

/// Trait for UI components that can be rendered
pub trait UiComponent {
    /// Render the component
    fn view(&self) -> iced::Element<'_, Message>;
} 