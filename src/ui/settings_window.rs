//! Settings window implementation for RustPods

use crate::config::AppConfig;
use crate::ui::Message;
use crate::ui::theme::{self, Theme};
use crate::ui::UiComponent;
use crate::ui::components::settings_view::SettingsView;

use iced::{
    widget::{button, column, container, row, scrollable, text, Column, Container, Rule, rule},
    Element, Length, Padding, Alignment, Background, Border, Color
};

/// Represents the settings window of the application
#[derive(Debug)]
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
    /// Bluetooth settings
    Bluetooth,
    /// UI settings
    Interface,
    /// System settings
    System,
}

impl SettingsWindow {
    /// Create a new settings window
    pub fn new(config: AppConfig) -> Self {
        let settings_view = SettingsView::new(config.clone());
        
        Self {
            config,
            settings_view,
            has_changes: false,
            selected_tab: SettingsTab::Bluetooth,
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
            .style(CloseButtonStyle)
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
                .padding(Padding::new(10, 10, 10, 10));
                
            if selected {
                btn.style(SelectedTabStyle)
            } else {
                btn.style(UnselectedTabStyle)
                    .on_press(Message::SelectSettingsTab(tab))
            }
        };
        
        // Get tab accent colors based on tab category
        let bluetooth_color = theme::BLUE;
        let interface_color = theme::GREEN;
        let system_color = theme::MAUVE;
        
        // Create a highlight indicator for the active tab
        let highlight_indicator = |active: bool, color: Color| {
            let height = if active { 3 } else { 0 };
            container(
                iced::widget::Space::with_height(Length::Units(height))
            )
            .width(Length::Fill)
            .style(move |_: &iced::Theme| container::Appearance {
                background: Some(color.into()),
                ..Default::default()
            })
        };
        
        column![
            row![
                tab_button("Bluetooth", SettingsTab::Bluetooth, self.selected_tab == SettingsTab::Bluetooth),
                tab_button("Interface", SettingsTab::Interface, self.selected_tab == SettingsTab::Interface),
                tab_button("System", SettingsTab::System, self.selected_tab == SettingsTab::System)
            ]
            .spacing(2)
            .padding(Padding::new(15, 15, 0, 0)),
            row![
                highlight_indicator(self.selected_tab == SettingsTab::Bluetooth, bluetooth_color),
                highlight_indicator(self.selected_tab == SettingsTab::Interface, interface_color),
                highlight_indicator(self.selected_tab == SettingsTab::System, system_color)
            ]
        ]
        .spacing(0)
        .into()
    }
    
    /// Get tab-specific accent color
    fn tab_accent_color(&self) -> Color {
        match self.selected_tab {
            SettingsTab::Bluetooth => theme::BLUE,
            SettingsTab::Interface => theme::GREEN,
            SettingsTab::System => theme::MAUVE,
        }
    }
    
    /// Create the content view based on selected tab
    fn content_view(&self) -> Element<Message> {
        let content = match self.selected_tab {
            SettingsTab::Bluetooth => self.settings_view.bluetooth_settings(),
            SettingsTab::Interface => self.settings_view.ui_settings(),
            SettingsTab::System => self.settings_view.system_settings(),
        };
        
        // Use the accent color for the tab
        let accent_color = self.tab_accent_color();
        
        container(
            scrollable(
                container(content)
                    .padding(20)
                    .width(Length::Fill)
            )
            .height(Length::Fill)
            .scroller_width(4)
            // Apply the scrollbar style
            .style(move |_: &iced::Theme| {
                scrollable::Appearance {
                    scrollbar: scrollable::Scrollbar {
                        background: Some(theme::SURFACE0.into()),
                        border_radius: 2.0.into(),
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                        scroller: scrollable::Scroller {
                            color: accent_color,
                            border_radius: 2.0.into(),
                            border_width: 0.0,
                            border_color: Color::TRANSPARENT,
                        }
                    },
                    ..Default::default()
                }
            })
        )
        .height(Length::Fill)
        .width(Length::Fill)
        .style(ContentContainerStyle)
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
            .style(ErrorContainerStyle)
            .width(Length::Fill)
            .into()
        })
    }
    
    /// Create the action buttons (Save, Cancel, Reset)
    fn action_buttons(&self) -> Element<Message> {
        let save_button = button(text("Save").size(16))
            .padding(10)
            .style(if self.has_changes {
                SaveButtonStyle
            } else {
                DisabledButtonStyle
            });
            
        let save_button = if self.has_changes {
            save_button.on_press(Message::SaveSettings)
        } else {
            save_button
        };
            
        let cancel_button = button(text("Cancel").size(16))
            .padding(10)
            .style(CancelButtonStyle)
            .on_press(Message::CloseSettings);
            
        let reset_button = button(text("Reset to Defaults").size(16))
            .padding(10)
            .style(ResetButtonStyle)
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
    fn view(&self) -> Element<'static, Message, iced::Renderer<theme::Theme>> {
        let mut content = column![
            self.header_view(),
            Rule::horizontal(1).style(|_: &iced::Theme| rule::Appearance {
                color: theme::OVERLAY0,
                width: 1,
                radius: 0.0.into(),
                fill_mode: rule::FillMode::Full,
            }),
            self.tab_navigation(),
            self.content_view(),
        ]
        .spacing(0)
        .height(Length::Fill)
        .width(Length::Fill);
        
        // Add error message if present
        if let Some(error_element) = self.error_message() {
            content = content.push(error_element);
        }
        
        // Add horizontal rule and action buttons
        content = content
            .push(Rule::horizontal(1).style(|_: &iced::Theme| rule::Appearance {
                color: theme::OVERLAY0,
                width: 1,
                radius: 0.0.into(),
                fill_mode: rule::FillMode::Full,
            }))
            .push(self.action_buttons());
        
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(WindowContainerStyle)
            .into()
    }
}

// Custom styles for the settings window components
struct WindowContainerStyle;
struct ContentContainerStyle;
struct SelectedTabStyle;
struct UnselectedTabStyle;
struct SaveButtonStyle;
struct DisabledButtonStyle;
struct ErrorContainerStyle;
struct CloseButtonStyle;
struct CancelButtonStyle;
struct ResetButtonStyle;

impl container::StyleSheet for WindowContainerStyle {
    type Style = iced::Theme;
    
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(theme::BASE.into()),
            text_color: Some(theme::TEXT),
            border_radius: 8.0.into(),
            border_width: 1.0,
            border_color: theme::SURFACE1,
        }
    }
}

impl container::StyleSheet for ContentContainerStyle {
    type Style = iced::Theme;
    
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(theme::SURFACE0.into()),
            text_color: Some(theme::TEXT),
            border_radius: 0.0.into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }
}

impl container::StyleSheet for ErrorContainerStyle {
    type Style = iced::Theme;
    
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(theme::RED.with_alpha(0.1).into()),
            text_color: Some(theme::RED),
            border_radius: 4.0.into(),
            border_width: 1.0,
            border_color: theme::RED,
        }
    }
}

impl button::StyleSheet for SelectedTabStyle {
    type Style = iced::Theme;
    
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(theme::SURFACE0.into()),
            text_color: theme::LAVENDER,
            border_radius: 4.0.into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }
    
    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);
        button::Appearance { ..active }
    }
    
    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);
        button::Appearance { ..active }
    }
    
    fn disabled(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);
        button::Appearance {
            text_color: theme::OVERLAY1,
            ..active
        }
    }
}

impl button::StyleSheet for UnselectedTabStyle {
    type Style = iced::Theme;
    
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(theme::MANTLE.into()),
            text_color: theme::TEXT,
            border_radius: 4.0.into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }
    
    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);
        
        button::Appearance {
            background: Some(theme::SURFACE0.into()),
            text_color: theme::SUBTEXT1,
            ..active
        }
    }
    
    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        let hovered = self.hovered(style);
        button::Appearance {
            background: Some(theme::SURFACE1.into()),
            ..hovered
        }
    }
    
    fn disabled(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);
        button::Appearance {
            text_color: theme::OVERLAY1,
            ..active
        }
    }
}

impl button::StyleSheet for SaveButtonStyle {
    type Style = iced::Theme;
    
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(theme::BLUE.into()),
            text_color: theme::CRUST,
            border_radius: 4.0.into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }
    
    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);
        
        button::Appearance {
            background: Some(theme::SAPPHIRE.into()),
            ..active
        }
    }
    
    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);
        
        button::Appearance {
            background: Some(theme::LAVENDER.into()),
            ..active
        }
    }
    
    fn disabled(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);
        button::Appearance {
            background: Some(theme::OVERLAY0.into()),
            text_color: theme::OVERLAY1,
            ..active
        }
    }
}

impl button::StyleSheet for DisabledButtonStyle {
    type Style = iced::Theme;
    
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(theme::SURFACE1.into()),
            text_color: theme::SUBTEXT0,
            border_radius: 4.0.into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }
    
    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        self.active(style)
    }
    
    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        self.active(style)
    }
    
    fn disabled(&self, style: &Self::Style) -> button::Appearance {
        self.active(style)
    }
}

impl button::StyleSheet for CloseButtonStyle {
    type Style = iced::Theme;
    
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(theme::OVERLAY0.into()),
            text_color: theme::TEXT,
            border_radius: 4.0.into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }
    
    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);
        
        button::Appearance {
            background: Some(theme::RED.into()),
            text_color: theme::CRUST,
            ..active
        }
    }
    
    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        let hovered = self.hovered(style);
        
        button::Appearance {
            background: Some(theme::MAROON.into()),
            ..hovered
        }
    }
}

impl button::StyleSheet for CancelButtonStyle {
    type Style = iced::Theme;
    
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(theme::SURFACE1.into()),
            text_color: theme::TEXT,
            border_radius: 4.0.into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }
    
    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);
        
        button::Appearance {
            background: Some(theme::SURFACE2.into()),
            ..active
        }
    }
    
    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        let hovered = self.hovered(style);
        
        button::Appearance {
            background: Some(theme::OVERLAY0.into()),
            ..hovered
        }
    }
}

impl button::StyleSheet for ResetButtonStyle {
    type Style = iced::Theme;
    
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(theme::SURFACE0.into()),
            text_color: theme::SUBTEXT0,
            border_radius: 4.0.into(),
            border_width: 1.0,
            border_color: theme::OVERLAY0,
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }
    
    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);
        
        button::Appearance {
            background: Some(theme::PEACH.with_alpha(0.1).into()),
            text_color: theme::PEACH,
            border_color: theme::PEACH,
            ..active
        }
    }
    
    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        let hovered = self.hovered(style);
        
        button::Appearance {
            background: Some(theme::PEACH.with_alpha(0.2).into()),
            ..hovered
        }
    }
} 