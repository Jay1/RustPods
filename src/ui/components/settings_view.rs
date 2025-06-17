use crate::config::AppConfig;
use crate::ui::theme as ui_theme;
use crate::ui::Message;
use iced::Length;
use iced::Renderer;
use iced::{
    widget::{Checkbox, Column, Container, Row, Text},
    Element,
};

/// Settings view component
#[derive(Debug, Clone)]
pub struct SettingsView {
    config: AppConfig,
    /// Current connected devices for display
    connected_devices: Vec<String>,
}

impl SettingsView {
    /// Create a new settings view
    pub fn new(config: AppConfig) -> Self {
        Self { 
            config,
            connected_devices: Vec::new(),
        }
    }

    /// Get a copy of the config
    pub fn config(&self) -> AppConfig {
        self.config.clone()
    }

    /// Update the config
    pub fn update_config(&mut self, config: AppConfig) {
        crate::debug_log!("ui", "SettingsView::update_config called");
        self.config = config;
        crate::debug_log!("ui", "SettingsView::config updated");
    }

    /// Update connected devices list
    pub fn update_connected_devices(&mut self, devices: Vec<String>) {
        self.connected_devices = devices;
    }

    /// Update bluetooth settings
    pub fn update_bluetooth_setting(&mut self, setting: BluetoothSetting) {
        crate::debug_log!(
            "ui",
            "SettingsView::update_bluetooth_setting called: {:?}",
            setting
        );
        match setting {
            BluetoothSetting::DeviceName(value) => {
                self.config.bluetooth.paired_device_name = if value.trim().is_empty() {
                    None
                } else {
                    Some(value.trim().to_string())
                };
            }
        }
    }

    /// Bluetooth settings section
    pub fn bluetooth_settings(&self) -> Element<'_, Message, Renderer<ui_theme::Theme>> {
        let title = Text::new("Device Settings").size(20).style(ui_theme::TEXT);

        // Device naming section - show if we have connected devices
        let device_section = if !self.connected_devices.is_empty() {
            let current_device_name = self.connected_devices.first().unwrap();
            let display_name = self.config.bluetooth.paired_device_name
                .as_ref()
                .unwrap_or(current_device_name);
            
            let device_name_input = iced::widget::text_input(
                "Enter custom device name...",
                self.config.bluetooth.paired_device_name.as_deref().unwrap_or(""),
            )
            .on_input(Message::SetDeviceName)
            .width(Length::Fill);

            Column::new()
                .spacing(15)
                .push(Text::new("Connected Device").style(ui_theme::TEXT).size(16))
                .push(Text::new(format!("Device: {}", display_name)).style(ui_theme::TEXT))
                .push(
                    Row::new()
                        .spacing(10)
                        .push(Text::new("Custom Name:").style(ui_theme::TEXT).width(Length::Fixed(120.0)))
                        .push(device_name_input)
                )
        } else {
            Column::new()
                .spacing(10)
                .push(Text::new("No Device Connected").style(ui_theme::TEXT).size(16))
                .push(
                    Text::new("Connect your AirPods to customize device settings")
                        .style(ui_theme::SUBTEXT1),
                )
        };

        // Battery Intelligence section
        let intelligence_section = Column::new()
            .spacing(15)
            .push(Text::new("Battery Intelligence").style(ui_theme::TEXT).size(16))
            .push(
                Text::new("Manage battery learning profiles and data")
                    .style(ui_theme::SUBTEXT1)
            )
            .push(
                Row::new()
                    .spacing(10)
                    .push(
                        iced::widget::button("Open Profile Folder")
                            .on_press(Message::OpenProfileFolder)
                            .style(iced::theme::Button::Secondary),
                    )
                    .push(
                        iced::widget::button("Purge All Profiles")
                            .on_press(Message::PurgeProfiles)
                            .style(iced::theme::Button::Destructive),
                    )
            );

        Column::new()
            .spacing(25)
            .push(title)
            .push(device_section)
            .push(intelligence_section)
            .into()
    }

    /// UI settings section  
    pub fn ui_settings(&self) -> Element<'_, Message, Renderer<ui_theme::Theme>> {
        let title = Text::new("Interface").size(20).style(ui_theme::TEXT);

        let minimize_to_tray = Checkbox::new(
            "Minimize to tray on close",
            self.config.ui.minimize_to_tray_on_close,
            |value| Message::UpdateUiSetting(UiSetting::MinimizeToTrayOnClose(value)),
        );

        Container::new(
            Column::new()
                .spacing(15)
                .push(title)
                .push(minimize_to_tray)
                .width(Length::Fill),
        )
        .width(Length::Fill)
        .into()
    }

    /// System settings section
    pub fn system_settings(&self) -> Element<'_, Message, Renderer<ui_theme::Theme>> {
        let title = Text::new("System").size(20).style(ui_theme::TEXT);

        let startup_option = Checkbox::new(
            "Start on system startup",
            self.config.system.launch_at_startup,
            |value| Message::UpdateSystemSetting(SystemSetting::StartOnBoot(value)),
        );

        Container::new(
            Column::new()
                .spacing(15)
                .push(title)
                .push(startup_option)
                .width(Length::Fill),
        )
        .width(Length::Fill)
        .into()
    }
}

/// Bluetooth settings enum
#[derive(Debug, Clone, PartialEq)]
pub enum BluetoothSetting {
    /// Custom device name
    DeviceName(String),
}

/// UI settings enum
#[derive(Debug, Clone, PartialEq)]
pub enum UiSetting {
    /// Theme
    Theme(ui_theme::Theme),
    /// Show notifications
    ShowNotifications(bool),
    /// Start minimized
    StartMinimized(bool),
    /// Show percentage in tray
    ShowPercentageInTray(bool),
    /// Show low battery warning
    ShowLowBatteryWarning(bool),
    /// Low battery threshold
    LowBatteryThreshold(u8),
    /// Minimize to tray when close button is pressed
    MinimizeToTrayOnClose(bool),
}

/// System settings enum
#[derive(Debug, Clone, PartialEq)]
pub enum SystemSetting {
    /// Start on boot
    StartOnBoot(bool),
}
