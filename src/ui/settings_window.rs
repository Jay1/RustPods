//! Settings window implementation for RustPods

use crate::config::AppConfig;
use crate::ui::Message;
use crate::ui::theme::{self, Theme};
use crate::ui::UiComponent;
use crate::ui::components::settings_view::SettingsView;

use iced::{
    widget::{button, column, container, row, scrollable, text},
    Element, Length, Alignment, Color
};

/// Represents the settings window of the application
#[derive(Debug, Clone)]
pub struct SettingsWindow {
    /// Application configuration
    config: AppConfig,
    /// Current settings view
    settings_view: SettingsView,
    /// Whether changes have been made
    has_changes: bool,
    /// Currently selected tab
    selected_tab: SettingsTab,
    /// Current validation error
    validation_error: Option<String>,
}

/// Settings tab categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    /// General settings
    General,
    /// Bluetooth settings
    Bluetooth,
    /// Advanced settings
    Advanced,
    /// About
    About,
}

impl SettingsWindow {
    /// Create a new settings window
    pub fn new(config: AppConfig) -> Self {
        let settings_view = SettingsView::new(config.clone());
        
        Self {
            config,
            settings_view,
            has_changes: false,
            selected_tab: SettingsTab::General,
            validation_error: None,
        }
    }
    
    /// Get the current configuration
    pub fn config(&self) -> AppConfig {
        self.settings_view.config()
    }
    
    /// Update the configuration
    pub fn update_config(&mut self, config: AppConfig) {
        self.config = config.clone();
        self.settings_view.update_config(config);
        self.has_changes = false;
        self.validation_error = None;
    }
    
    /// Mark that changes have been made
    pub fn mark_changed(&mut self) {
        self.has_changes = true;
    }
    
    /// Check if there are unsaved changes
    pub fn has_changes(&self) -> bool {
        self.has_changes
    }
    
    /// Set a validation error
    pub fn set_validation_error(&mut self, error: Option<String>) {
        self.validation_error = error;
    }
    
    /// Select a tab
    pub fn select_tab(&mut self, tab: SettingsTab) {
        self.selected_tab = tab;
    }
    
    /// Create the header view
    fn header_view(&self) -> Element<Message> {
        let title = text("Settings")
            .size(24)
            .style(theme::TEXT);
            
        let close_button = button(text("✕").size(16))
            .padding(8)
            .style(iced::theme::Button::Secondary)
            .on_press(Message::CloseSettings);
            
        row![
            title,
            iced::widget::Space::with_width(Length::Fill),
            close_button
        ]
        .spacing(10)
        .align_items(Alignment::Center)
        .padding(15)
        .into()
    }
    
    /// Create the tab navigation view
    fn tab_navigation(&self) -> Element<Message> {
        // Helper function to create a tab button
        let tab_button = |label: &str, tab: SettingsTab, selected: bool| {
            let label_text = text(label).size(16);
            
            let btn = button(label_text)
                .width(Length::Fill)
                .padding(10);
                
            if selected {
                btn.style(iced::theme::Button::Primary)
            } else {
                btn.style(iced::theme::Button::Secondary)
                    .on_press(Message::SelectSettingsTab(tab))
            }
        };
        
        // Get tab accent colors based on tab category
        let general_color = theme::BLUE;
        let bluetooth_color = theme::GREEN;
        let advanced_color = theme::MAUVE;
        
        // Create a highlight indicator for the active tab
        let highlight_indicator = |active: bool, color: Color| {
            let height = if active { 3 } else { 0 };
            container(
                iced::widget::Space::with_height(Length::Fixed(height as f32))
            )
            .width(Length::Fill)
            .style(move |_: &iced::Theme| container::Appearance {
                background: Some(color.into()),
                ..Default::default()
            })
        };
        
        column![
            row![
                tab_button("General", SettingsTab::General, self.selected_tab == SettingsTab::General),
                tab_button("Bluetooth", SettingsTab::Bluetooth, self.selected_tab == SettingsTab::Bluetooth),
                tab_button("Advanced", SettingsTab::Advanced, self.selected_tab == SettingsTab::Advanced),
                tab_button("About", SettingsTab::About, self.selected_tab == SettingsTab::About)
            ]
            .spacing(2)
            .padding(15),
            row![
                highlight_indicator(self.selected_tab == SettingsTab::General, general_color),
                highlight_indicator(self.selected_tab == SettingsTab::Bluetooth, bluetooth_color),
                highlight_indicator(self.selected_tab == SettingsTab::Advanced, advanced_color)
            ]
        ]
        .spacing(0)
        .into()
    }
    
    /// Get tab-specific accent color
    fn tab_accent_color(&self) -> Color {
        match self.selected_tab {
            SettingsTab::General => theme::BLUE,
            SettingsTab::Bluetooth => theme::GREEN,
            SettingsTab::Advanced => theme::MAUVE,
            SettingsTab::About => theme::MAUVE,
        }
    }
    
    /// Create the content view based on selected tab
    fn content_view(&self) -> Element<Message> {
        let content = match self.selected_tab {
            SettingsTab::General => self.settings_view.ui_settings(),
            SettingsTab::Bluetooth => self.settings_view.bluetooth_settings(),
            SettingsTab::Advanced => self.settings_view.system_settings(),
            SettingsTab::About => text("About RustPods: Battery monitoring application").into(),
        };
        
        // Use the accent color for the tab
        let _accent_color = self.tab_accent_color();
        
        container(
            scrollable(
                container(content)
                    .padding(20)
                    .width(Length::Fill)
            )
            .height(Length::Fill)
            .style(iced::theme::Scrollable::Default)
        )
        .height(Length::Fill)
        .width(Length::Fill)
        .style(iced::theme::Container::Box)
        .into()
    }
    
    /// Create an error message element
    fn error_message(&self) -> Option<Element<Message>> {
        self.validation_error.as_ref().map(|error| {
            container(
                text(error)
                    .size(14)
                    .style(theme::RED)
            )
            .padding(10)
            .style(iced::theme::Container::Box)
            .width(Length::Fill)
            .into()
        })
    }
    
    /// Create the action buttons (Save, Cancel, Reset)
    fn action_buttons(&self) -> Element<Message> {
        let save_button = button(text("Save").size(16))
            .padding(10)
            .style(if self.has_changes {
                iced::theme::Button::Primary
            } else {
                iced::theme::Button::Secondary
            });
            
        let save_button = if self.has_changes {
            save_button.on_press(Message::SaveSettings)
        } else {
            save_button
        };
            
        let cancel_button = button(text("Cancel").size(16))
            .padding(10)
            .style(iced::theme::Button::Secondary)
            .on_press(Message::CloseSettings);
            
        let reset_button = button(text("Reset to Defaults").size(16))
            .padding(10)
            .style(iced::theme::Button::Destructive)
            .on_press(Message::ResetSettings);
            
        row![
            reset_button,
            iced::widget::Space::with_width(Length::Fill),
            cancel_button,
            save_button
        ]
        .spacing(10)
        .padding(15)
        .into()
    }
}

impl UiComponent for SettingsWindow {
    fn view(&self) -> Element<'_, Message, iced::Renderer<Theme>> {
        // Temporary workaround to get past the compilation error
        // Just return a simple text element
        // TODO: Implement full settings UI once the type issues are resolved
        text("Settings Window - Under Construction")
            .size(24)
            .into()
    }
} 