use crate::config::{AppConfig, LogLevel};
use crate::ui::theme as ui_theme;
use crate::ui::Message;
use iced::Length;
use iced::Renderer;
use iced::{
    widget::{Checkbox, Column, Container, PickList, Row, Slider, Text},
    Element,
};
use std::convert::TryInto;
use std::time::Duration;

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
        println!("[DEBUG] SettingsView::update_config called");
        self.config = config;
        println!("[DEBUG] SettingsView::config updated");
    }

    /// Update bluetooth settings
    pub fn update_bluetooth_setting(&mut self, setting: BluetoothSetting) {
        println!(
            "[DEBUG] SettingsView::update_bluetooth_setting called: {:?}",
            setting
        );
        match setting {
            BluetoothSetting::AutoScanOnStartup(value) => {
                self.config.bluetooth.auto_scan_on_startup = value;
            }
            BluetoothSetting::ScanDuration(value) => {
                self.config.bluetooth.scan_duration = std::time::Duration::from_secs(value as u64);
            }
            BluetoothSetting::ScanInterval(value) => {
                self.config.bluetooth.scan_interval = std::time::Duration::from_secs(value as u64);
            }
            BluetoothSetting::BatteryRefreshInterval(value) => {
                self.config.bluetooth.battery_refresh_interval = Duration::from_secs(value as u64);
            }
            BluetoothSetting::MinRssi(value) => {
                self.config.bluetooth.min_rssi = Some(value.try_into().unwrap_or(-70));
            }
            BluetoothSetting::AutoReconnect(value) => {
                self.config.bluetooth.auto_reconnect = value;
            }
            BluetoothSetting::ReconnectAttempts(value) => {
                self.config.bluetooth.reconnect_attempts = value.try_into().unwrap_or(3);
            }
        }
    }

    /// Bluetooth settings section
    pub fn bluetooth_settings(&self) -> Element<'_, Message, Renderer<ui_theme::Theme>> {
        let title = Text::new("Bluetooth").size(20).style(ui_theme::TEXT);

        // Device pairing section
        let pairing_section = if let Some(paired_device) = &self.config.bluetooth.paired_device_id {
            Column::new()
                .spacing(10)
                .push(Text::new("Paired Device").style(ui_theme::TEXT).size(16))
                .push(Text::new(format!("Device: {}", paired_device)).style(ui_theme::TEXT))
                .push(
                    iced::widget::button("Unpair Device")
                        .on_press(Message::UnpairDevice)
                        .style(iced::theme::Button::Destructive),
                )
        } else {
            Column::new()
                .spacing(10)
                .push(Text::new("No Device Paired").style(ui_theme::TEXT).size(16))
                .push(
                    Text::new("Use the main interface to pair with your AirPods")
                        .style(ui_theme::SUBTEXT1),
                )
        };

        let scan_duration_seconds = self.config.bluetooth.scan_duration.as_secs() as i32;
        let scan_duration = Column::new()
            .spacing(5)
            .push(Text::new("Scan duration (seconds)").style(ui_theme::TEXT))
            .push(
                Row::new()
                    .spacing(10)
                    .push(
                        Slider::new(1..=60, scan_duration_seconds, move |value| {
                            Message::UpdateBluetoothSetting(BluetoothSetting::ScanDuration(value))
                        })
                        .width(Length::Fill),
                    )
                    .push(Text::new(scan_duration_seconds.to_string()).style(ui_theme::TEXT)),
            );

        let scan_interval_seconds = self.config.bluetooth.scan_interval.as_secs() as i32;
        let scan_interval = Column::new()
            .spacing(5)
            .push(Text::new("Scan interval (seconds)").style(ui_theme::TEXT))
            .push(
                Row::new()
                    .spacing(10)
                    .push(
                        Slider::new(5..=600, scan_interval_seconds, move |value| {
                            Message::UpdateBluetoothSetting(BluetoothSetting::ScanInterval(value))
                        })
                        .width(Length::Fill),
                    )
                    .push(Text::new(scan_interval_seconds.to_string()).style(ui_theme::TEXT)),
            );

        let battery_refresh_seconds =
            self.config.bluetooth.battery_refresh_interval.as_secs() as i32;
        let battery_refresh = Column::new()
            .spacing(5)
            .push(Text::new("Battery refresh interval (seconds)").style(ui_theme::TEXT))
            .push(
                Row::new()
                    .spacing(10)
                    .push(
                        Slider::new(3..=120, battery_refresh_seconds, move |value| {
                            Message::UpdateBluetoothSetting(
                                BluetoothSetting::BatteryRefreshInterval(value),
                            )
                        })
                        .width(Length::Fill),
                    )
                    .push(Text::new(battery_refresh_seconds.to_string()).style(ui_theme::TEXT)),
            );

        Column::new()
            .spacing(20)
            .push(title)
            .push(pairing_section)
            .push(scan_duration)
            .push(scan_interval)
            .push(battery_refresh)
            .into()
    }

    /// UI settings section  
    pub fn ui_settings(&self) -> Element<'_, Message, Renderer<ui_theme::Theme>> {
        let title = Text::new("Settings").size(20).style(ui_theme::TEXT);

        let minimize_to_tray = Checkbox::new(
            "Minimize to tray when X is pressed",
            self.config.ui.minimize_to_tray_on_close,
            |value| Message::UpdateUiSetting(UiSetting::MinimizeToTrayOnClose(value)),
        );

        let notice = Text::new("Note: System tray has been improved for better reliability")
            .size(12)
            .style(crate::ui::theme::TEXT);

        Container::new(
            Column::new()
                .spacing(15)
                .push(title)
                .push(minimize_to_tray)
                .push(notice)
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

        let minimize_option =
            Checkbox::new("Start minimized", self.config.ui.start_minimized, |value| {
                Message::UpdateSystemSetting(SystemSetting::StartMinimized(value))
            });

        // Clone to avoid reference issues
        let log_error = LogLevel::Error;
        let log_warn = LogLevel::Warn;
        let log_info = LogLevel::Info;
        let log_debug = LogLevel::Debug;
        let log_trace = LogLevel::Trace;

        let log_options = vec![log_error, log_warn, log_info, log_debug, log_trace];

        let log_level_picker = Row::new()
            .spacing(10)
            .push(
                Text::new("Log Level:")
                    .width(Length::Fill)
                    .style(ui_theme::TEXT),
            )
            .push(
                PickList::new(
                    log_options,
                    Some(self.config.system.log_level.clone()),
                    |level| Message::UpdateSystemSetting(SystemSetting::LogLevel(level)),
                )
                .width(Length::FillPortion(2)),
            );

        let telemetry_option = Checkbox::new(
            "Enable anonymous usage telemetry",
            self.config.system.enable_telemetry,
            |value| Message::UpdateSystemSetting(SystemSetting::EnableTelemetry(value)),
        );

        // Information text about telemetry
        let telemetry_info = Text::new(
            "Anonymous usage data helps improve the application.\nNo personal information is collected."
        ).size(12).style(ui_theme::TEXT);

        Container::new(
            Column::new()
                .spacing(15)
                .push(title)
                .push(startup_option)
                .push(minimize_option)
                .push(log_level_picker)
                .push(telemetry_option)
                .push(telemetry_info)
                .width(Length::Fill),
        )
        .width(Length::Fill)
        .into()
    }
}

/// Bluetooth settings enum
#[derive(Debug, Clone, Copy, PartialEq)]
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
    /// Start minimized
    StartMinimized(bool),
    /// Log level
    LogLevel(LogLevel),
    /// Enable telemetry
    EnableTelemetry(bool),
}
