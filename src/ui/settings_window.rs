//! Settings window implementation for RustPods

use crate::config::AppConfig;
use crate::ui::Message;
use crate::ui::theme::{self, Theme};
use crate::ui::UiComponent;
use crate::ui::components::UiSetting;
use iced::{
    widget::{
        button, column, container, row, text, Space, Checkbox
    },
    Element, Length, Alignment
};

/// Represents the settings window of the application
#[derive(Debug, Clone)]
pub struct SettingsWindow {
    /// Application configuration
    config: AppConfig,
    /// Whether changes have been made
    has_changes: bool,
}

impl SettingsWindow {
    /// Create a new settings window
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            has_changes: false,
        }
    }
    
    /// Get the current configuration
    pub fn config(&self) -> &AppConfig {
        &self.config
    }
    
    /// Update the configuration
    pub fn update_config(&mut self, config: AppConfig) {
        self.config = config;
        self.has_changes = false;
    }
    
    /// Mark that changes have been made
    pub fn mark_changed(&mut self) {
        self.has_changes = true;
    }
    
    /// Check if there are unsaved changes
    pub fn has_changes(&self) -> bool {
        self.has_changes
    }
    
    /// Set a validation error (for compatibility)
    pub fn set_validation_error(&mut self, _error: Option<String>) {
        // No-op for simplified settings window
    }
}

impl UiComponent for SettingsWindow {
    fn view(&self) -> Element<'_, Message, iced::Renderer<Theme>> {
        // Header with back button and title
        let header = row![
            button(text("← Back").size(14))
                .on_press(Message::CloseSettings)
                .style(iced::theme::Button::Secondary)
                .padding([5, 10]),
            Space::with_width(Length::Fixed(10.0)),
            text("Settings")
                .size(24)
                .style(theme::TEXT),
            Space::with_width(Length::Fill),
        ]
        .align_items(Alignment::Center);
        
        // Minimize to tray checkbox
        let minimize_checkbox = Checkbox::new(
            "Minimize to tray on close",
            self.config.ui.minimize_to_tray_on_close,
            |value| Message::UpdateUiSetting(UiSetting::MinimizeToTrayOnClose(value))
        );
        
        // Settings info text
        let info_text = text("Settings are saved automatically when changed")
            .size(12)
            .style(theme::SUBTEXT1);
        
        // Action buttons - Save applies changes and closes, Cancel discards changes
        let save_button = button(
            text("Save & Close")
                .style(theme::TEXT)
                .size(14)
        )
            .on_press(Message::SaveSettings)
            .style(iced::theme::Button::Primary)
            .padding(10);
            
        let cancel_button = button(
            text("Cancel")
                .style(theme::TEXT) 
                .size(14)
        )
            .on_press(Message::CloseSettings)
            .style(iced::theme::Button::Secondary)
            .padding(10);
        
        let actions = row![
            Space::with_width(Length::Fill),
            cancel_button,
            save_button
        ]
        .spacing(10)
        .align_items(Alignment::Center);
        
        let content = column![
            header,
            Space::with_height(Length::Fixed(20.0)),
            minimize_checkbox,
            Space::with_height(Length::Fixed(15.0)),
            info_text,
            Space::with_height(Length::Fixed(30.0)),
            actions
        ]
        .spacing(15)
        .padding(25)
        .align_items(Alignment::Start);
        
        // Use the same fixed dimensions as the main popup (420×320)
        container(content)
            .width(Length::Fixed(420.0))
            .height(Length::Fixed(320.0))
            .style(iced::theme::Container::Box)
            .into()
    }
} 