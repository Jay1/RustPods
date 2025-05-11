use crate::config::{AppConfig, Theme, LogLevel};
use crate::ui::Message;
use iced::{widget::{Column, Row, Text, Container, Checkbox, Slider, PickList}, Element};
use iced::{Length};
use std::convert::TryInto;

/// Settings view component
#[derive(Debug, Clone)]
pub struct SettingsView {
    config: AppConfig,
}

impl SettingsView {
    /// Create a new settings view
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }
    
    /// Get a copy of the config
    pub fn config(&self) -> AppConfig {
        self.config.clone()
    }
    
    /// Update the config
    pub fn update_config(&mut self, config: AppConfig) {
        self.config = config;
    }
    
    /// Update bluetooth settings
    pub fn update_bluetooth_setting(&mut self, setting: BluetoothSetting) {
        match setting {
            BluetoothSetting::AutoScanOnStartup(value) => {
                self.config.bluetooth.auto_scan_on_startup = value;
            },
            BluetoothSetting::ScanDuration(value) => {
                self.config.bluetooth.scan_duration = std::time::Duration::from_secs(value as u64);
            },
            BluetoothSetting::ScanInterval(value) => {
                self.config.bluetooth.scan_interval = std::time::Duration::from_secs(value as u64);
            },
            BluetoothSetting::BatteryRefreshInterval(value) => {
                self.config.bluetooth.battery_refresh_interval = value as u64;
            },
            BluetoothSetting::MinRssi(value) => {
                self.config.bluetooth.min_rssi = Some(value.try_into().unwrap_or(-70));
            },
            BluetoothSetting::AutoReconnect(value) => {
                self.config.bluetooth.auto_reconnect = value;
            },
            BluetoothSetting::ReconnectAttempts(value) => {
                self.config.bluetooth.reconnect_attempts = value.try_into().unwrap_or(3);
            },
        }
    }

    /// Bluetooth settings section
    pub fn bluetooth_settings(&self) -> Element<Message> {
        let title = Text::new("Bluetooth")
            .size(20)
            .style(iced::Color::from_rgb(0.2, 0.2, 0.8));
            
        let auto_scan = Checkbox::new(
            "Auto scan on startup",
            self.config.bluetooth.auto_scan_on_startup,
            move |value| {
                Message::UpdateBluetoothSetting(BluetoothSetting::AutoScanOnStartup(value))
            }
        );
        
        let scan_duration_seconds = self.config.bluetooth.scan_duration.as_secs() as i32;
        let scan_duration = Column::new()
            .spacing(5)
            .push(Text::new("Scan duration (seconds)"))
            .push(
                Row::new()
                    .spacing(10)
                    .push(
                        Slider::new(
                            1..=60,
                            scan_duration_seconds,
                            move |value| {
                                Message::UpdateBluetoothSetting(BluetoothSetting::ScanDuration(value))
                            }
                        )
                        .width(Length::Fill)
                    )
                    .push(Text::new(scan_duration_seconds.to_string()))
            );
            
        let scan_interval_seconds = self.config.bluetooth.scan_interval.as_secs() as i32;
        let scan_interval = Column::new()
            .spacing(5)
            .push(Text::new("Scan interval (seconds)"))
            .push(
                Row::new()
                    .spacing(10)
                    .push(
                        Slider::new(
                            5..=600,
                            scan_interval_seconds,
                            move |value| {
                                Message::UpdateBluetoothSetting(BluetoothSetting::ScanInterval(value))
                            }
                        )
                        .width(Length::Fill)
                    )
                    .push(Text::new(scan_interval_seconds.to_string()))
            );
            
        let battery_refresh_seconds = self.config.bluetooth.battery_refresh_interval as i32;
        let battery_refresh = Column::new()
            .spacing(5)
            .push(Text::new("Battery refresh interval (seconds)"))
            .push(
                Row::new()
                    .spacing(10)
                    .push(
                        Slider::new(
                            3..=120,
                            battery_refresh_seconds,
                            move |value| {
                                Message::UpdateBluetoothSetting(BluetoothSetting::BatteryRefreshInterval(value))
                            }
                        )
                        .width(Length::Fill)
                    )
                    .push(Text::new(battery_refresh_seconds.to_string()))
            );
            
        let min_rssi_value = self.config.bluetooth.min_rssi.unwrap_or(-70) as i32;
        let min_rssi = Column::new()
            .spacing(5)
            .push(Text::new("Minimum RSSI (signal strength)"))
            .push(
                Row::new()
                    .spacing(10)
                    .push(
                        Slider::new(
                            -100..=-40,
                            min_rssi_value,
                            move |value| {
                                Message::UpdateBluetoothSetting(BluetoothSetting::MinRssi(value))
                            }
                        )
                        .width(Length::Fill)
                    )
                    .push(Text::new(format!("{} dBm", min_rssi_value)))
            );
            
        let auto_reconnect = Checkbox::new(
            "Auto reconnect to devices",
            self.config.bluetooth.auto_reconnect,
            move |value| {
                Message::UpdateBluetoothSetting(BluetoothSetting::AutoReconnect(value))
            }
        );
        
        let reconnect_attempts = Column::new()
            .spacing(5)
            .push(Text::new("Reconnect attempts"))
            .push(
                Row::new()
                    .spacing(10)
                    .push(
                        Slider::new(
                            1..=10,
                            self.config.bluetooth.reconnect_attempts as i32,
                            move |value| {
                                Message::UpdateBluetoothSetting(BluetoothSetting::ReconnectAttempts(value))
                            }
                        )
                        .width(Length::Fill)
                    )
                    .push(Text::new(self.config.bluetooth.reconnect_attempts.to_string()))
            );
            
        Container::new(
            Column::new()
                .spacing(15)
                .push(title)
                .push(auto_scan)
                .push(scan_duration)
                .push(scan_interval)
                .push(battery_refresh)
                .push(min_rssi)
                .push(auto_reconnect)
                .push(reconnect_attempts)
                .width(Length::Fill)
        )
        .width(Length::Fill)
        .into()
    }
    
    /// UI settings section
    pub fn ui_settings(&self) -> Element<Message> {
        let title = Text::new("User Interface")
            .size(20)
            .style(iced::Color::from_rgb(0.2, 0.8, 0.2));
            
        // Clone the themes to avoid reference issues
        let theme_light = Theme::Light;
        let theme_dark = Theme::Dark;
        let theme_system = Theme::System;
        let theme_options = vec![theme_light, theme_dark, theme_system];
            
        let theme_picker = Row::new()
            .spacing(10)
            .push(Text::new("Theme:").width(Length::Fill))
            .push(
                PickList::new(
                    theme_options,
                    Some(self.config.ui.theme.clone()),
                    |theme| Message::UpdateUiSetting(UiSetting::Theme(theme))
                )
                .width(Length::FillPortion(2))
            );
            
        let show_notifications = Checkbox::new(
            "Show battery notifications",
            self.config.ui.show_notifications,
            |value| Message::UpdateUiSetting(UiSetting::ShowNotifications(value))
        );
        
        let start_minimized = Checkbox::new(
            "Start minimized to system tray",
            self.config.ui.start_minimized,
            |value| Message::UpdateUiSetting(UiSetting::StartMinimized(value))
        );
        
        let show_percentage = Checkbox::new(
            "Show battery percentage in system tray icon",
            self.config.ui.show_percentage_in_tray,
            |value| Message::UpdateUiSetting(UiSetting::ShowPercentageInTray(value))
        );
        
        let show_low_battery = Checkbox::new(
            "Show low battery warnings",
            self.config.ui.show_low_battery_warning,
            |value| Message::UpdateUiSetting(UiSetting::ShowLowBatteryWarning(value))
        );
        
        let battery_threshold = Column::new()
            .spacing(5)
            .push(Text::new("Low battery threshold (%)"))
            .push(
                Row::new()
                    .spacing(10)
                    .push(
                        Slider::new(
                            5..=50,
                            self.config.ui.low_battery_threshold as i32,
                            move |value| {
                                Message::UpdateUiSetting(UiSetting::LowBatteryThreshold(value as u8))
                            }
                        )
                        .width(Length::Fill)
                    )
                    .push(Text::new(format!("{}%", self.config.ui.low_battery_threshold)))
            );
        
        Container::new(
            Column::new()
                .spacing(15)
                .push(title)
                .push(theme_picker)
                .push(show_notifications)
                .push(start_minimized)
                .push(show_percentage)
                .push(show_low_battery)
                .push(battery_threshold)
                .width(Length::Fill)
        )
        .width(Length::Fill)
        .into()
    }
    
    /// System settings section
    pub fn system_settings(&self) -> Element<Message> {
        let title = Text::new("System")
            .size(20)
            .style(iced::Color::from_rgb(0.8, 0.2, 0.2));
            
        let startup_option = Checkbox::new(
            "Start on system startup",
            self.config.system.launch_at_startup,
            |value| Message::UpdateSystemSetting(SystemSetting::StartOnBoot(value))
        );
        
        let minimize_option = Checkbox::new(
            "Start minimized",
            self.config.ui.start_minimized,
            |value| Message::UpdateSystemSetting(SystemSetting::StartMinimized(value))
        );
        
        // Clone to avoid reference issues
        let log_error = LogLevel::Error;
        let log_warn = LogLevel::Warn;
        let log_info = LogLevel::Info;
        let log_debug = LogLevel::Debug;
        let log_trace = LogLevel::Trace;
        
        let log_options = vec![log_error, log_warn, log_info, log_debug, log_trace];
        
        let log_level_picker = Row::new()
            .spacing(10)
            .push(Text::new("Log Level:").width(Length::Fill))
            .push(
                PickList::new(
                    log_options,
                    Some(self.config.system.log_level.clone()),
                    |level| Message::UpdateSystemSetting(SystemSetting::LogLevel(level))
                )
                .width(Length::FillPortion(2))
            );
        
        let telemetry_option = Checkbox::new(
            "Enable anonymous usage telemetry",
            self.config.system.enable_telemetry,
            |value| Message::UpdateSystemSetting(SystemSetting::EnableTelemetry(value))
        );
        
        // Information text about telemetry
        let telemetry_info = Text::new(
            "Anonymous usage data helps improve the application.\nNo personal information is collected."
        ).size(12);
        
        Container::new(
            Column::new()
                .spacing(15)
                .push(title)
                .push(startup_option)
                .push(minimize_option)
                .push(log_level_picker)
                .push(telemetry_option)
                .push(telemetry_info)
                .width(Length::Fill)
        )
        .width(Length::Fill)
        .into()
    }
}

/// Bluetooth settings enum
#[derive(Debug, Clone, Copy)]
pub enum BluetoothSetting {
    /// Auto scan on startup
    AutoScanOnStartup(bool),
    /// Scan duration in seconds
    ScanDuration(i32),
    /// Scan interval in seconds
    ScanInterval(i32),
    /// Battery refresh interval in seconds
    BatteryRefreshInterval(i32),
    /// Minimum RSSI value
    MinRssi(i32),
    /// Auto reconnect
    AutoReconnect(bool),
    /// Number of reconnect attempts
    ReconnectAttempts(i32),
}

/// UI settings enum
#[derive(Debug, Clone)]
pub enum UiSetting {
    /// Theme
    Theme(Theme),
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
}

/// System settings enum
#[derive(Debug, Clone)]
pub enum SystemSetting {
    /// Start on boot
    StartOnBoot(bool),
    /// Start minimized
    StartMinimized(bool),
    /// Log level
    LogLevel(LogLevel),
    /// Enable telemetry
    EnableTelemetry(bool),
} 