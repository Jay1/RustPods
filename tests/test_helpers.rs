//! Common test helpers for all UI component tests
//! Contains mock implementations and utility functions

use std::sync::Arc;
use std::collections::HashMap;
use std::time::Duration;
use chrono::{DateTime, Utc};
use iced::{Point, Size, Rectangle};
use btleplug::api::BDAddr;

use rustpods::ui::Message;
use rustpods::ui::state_manager::{StateManager, Action};
use rustpods::config::{AppConfig, BluetoothConfig, UiConfig, SystemConfig};
use rustpods::ui::theme::Theme;
use rustpods::bluetooth::AirPodsBatteryStatus;
use rustpods::airpods::AirPodsBattery;
use rustpods::bluetooth::AirPodsCharging;

/// Create a test AppConfig for testing
pub fn create_test_config() -> AppConfig {
    AppConfig {
        bluetooth: BluetoothConfig {
            auto_scan_on_startup: true,
            scan_duration: Duration::from_secs(5),
            scan_interval: Duration::from_secs(30),
            battery_refresh_interval: Duration::from_secs(60),
            min_rssi: -80,
            auto_reconnect: true,
            reconnect_attempts: 3,
        },
        ui: UiConfig {
            show_notifications: true,
            start_minimized: false,
            theme: Theme::CatppuccinMocha,
            show_percentage_in_tray: true,
            show_low_battery_warning: true,
            low_battery_threshold: 20,
        },
        system: SystemConfig {
            launch_at_startup: true,
            log_level: "info".to_string(),
            enable_telemetry: false,
        },
    }
}

/// Create a test state manager for testing
pub fn create_test_state_manager() -> Arc<StateManager> {
    let (tx, _) = tokio::sync::mpsc::unbounded_channel();
    Arc::new(StateManager::new(tx))
}

/// Create a test battery status for testing
pub fn create_test_battery() -> AirPodsBatteryStatus {
    AirPodsBatteryStatus {
        battery: AirPodsBattery {
            left: Some(75),
            right: Some(80),
            case: Some(90),
            charging: AirPodsCharging {
                left: false,
                right: false,
                case: true,
            },
        },
        last_updated: std::time::Instant::now(),
    }
}

/// A mock implementation of the window visibility manager that doesn't depend on Iced
#[derive(Debug, Clone)]
pub struct MockWindowVisibilityManager {
    /// Whether the window is visible
    pub visible: bool,
    /// Last window position
    pub position: Option<WindowPosition>,
    /// Auto hide timeout
    pub auto_hide_timeout: Option<Duration>,
}

/// Window position storage
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowPosition {
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
    /// Width
    pub width: f32,
    /// Height
    pub height: f32,
}

impl From<Rectangle> for WindowPosition {
    fn from(rect: Rectangle) -> Self {
        Self {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }
}

impl From<WindowPosition> for Rectangle {
    fn from(pos: WindowPosition) -> Self {
        Rectangle::new(
            Point::new(pos.x, pos.y),
            Size::new(pos.width, pos.height),
        )
    }
}

impl MockWindowVisibilityManager {
    /// Create a new mock window visibility manager
    pub fn new() -> Self {
        Self {
            visible: true,
            position: None,
            auto_hide_timeout: None,
        }
    }
    
    /// Show the window
    pub fn show(&mut self) {
        self.visible = true;
    }
    
    /// Hide the window
    pub fn hide(&mut self) {
        self.visible = false;
    }
    
    /// Toggle window visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }
    
    /// Set the window position
    pub fn set_position(&mut self, position: WindowPosition) {
        self.position = Some(position);
    }
    
    /// Get the window position
    pub fn get_position(&self) -> Option<WindowPosition> {
        self.position
    }
    
    /// Set auto hide timeout
    pub fn with_auto_hide_timeout(mut self, timeout: Duration) -> Self {
        self.auto_hide_timeout = Some(timeout);
        self
    }
    
    /// Handle focus event
    pub fn handle_focus(&mut self) {
        // In a mock, we just record the event
    }
    
    /// Handle blur event
    pub fn handle_blur(&mut self) {
        // In a mock, we just record the event
        if self.auto_hide_timeout.is_some() {
            // Auto-hide would be triggered after timeout
        }
    }
    
    /// Update method to trigger auto-hide
    pub fn update(&mut self, _rect: Rectangle) -> Option<Message> {
        if self.auto_hide_timeout.is_some() && self.visible {
            self.visible = false;
            Some(Message::HideWindow)
        } else {
            None
        }
    }
    
    /// Check if the window is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }
    
    /// Get the last position
    pub fn last_position(&self) -> Option<WindowPosition> {
        self.position
    }
}

/// A mock implementation of the system tray
#[derive(Debug, Clone)]
pub struct MockSystemTray {
    /// Whether the system tray is connected
    pub is_connected: bool,
    /// Last battery status
    pub battery_status: Option<AirPodsBatteryStatus>,
    /// The menu items
    pub menu_items: Vec<String>,
    /// Called actions
    pub actions: Vec<String>,
    /// Current tooltip text
    pub tooltip: String,
    /// Current icon type
    pub icon_type: TrayIconType,
    /// Current theme mode
    pub theme_mode: ThemeMode,
    /// Show percentage in tray
    pub show_percentage: bool,
    /// Low battery threshold
    pub low_battery_threshold: u8,
    /// Notifications shown
    pub notifications: Vec<String>,
}

/// Mock tray icon types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrayIconType {
    /// Disconnected icon
    Disconnected,
    /// Connected with battery level
    BatteryLevel(u8),
    /// Charging icon
    Charging,
    /// Low battery icon
    LowBattery,
}

/// Mock theme modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    /// Light theme
    Light,
    /// Dark theme
    Dark,
    /// System theme
    System,
}

impl MockSystemTray {
    /// Create a new mock system tray
    pub fn new() -> Self {
        Self {
            is_connected: false,
            battery_status: None,
            menu_items: vec![
                "Show".to_string(),
                "Hide".to_string(),
                "Start Scan".to_string(),
                "Stop Scan".to_string(),
                "Settings".to_string(),
                "Exit".to_string(),
            ],
            actions: Vec::new(),
            tooltip: "RustPods - Disconnected".to_string(),
            icon_type: TrayIconType::Disconnected,
            theme_mode: ThemeMode::Dark,
            show_percentage: true,
            low_battery_threshold: 20,
            notifications: Vec::new(),
        }
    }
    
    /// Update with battery status
    pub fn update_battery(&mut self, battery: AirPodsBatteryStatus) {
        self.battery_status = Some(battery.clone());
        self.actions.push("update_battery".to_string());
        
        // Update tooltip
        self.update_tooltip_with_battery(
            battery.battery.left,
            battery.battery.right,
            battery.battery.case
        );
        
        // Update icon
        self.update_icon_with_battery(&battery);
        
        // Check for low battery
        self.check_low_battery(&battery);
    }
    
    /// Update connection status
    pub fn update_connection(&mut self, connected: bool) {
        self.is_connected = connected;
        self.actions.push(format!("update_connection: {}", connected));
        
        if !connected {
            self.icon_type = TrayIconType::Disconnected;
            self.tooltip = "RustPods - Disconnected".to_string();
        }
    }
    
    /// Handle battery update
    pub fn handle_battery_update(&mut self, battery: AirPodsBatteryStatus) -> Result<(), String> {
        self.update_battery(battery);
        Ok(())
    }
    
    /// Connect to state manager
    pub fn connect_state_manager(&mut self, _state_manager: Arc<StateManager>) -> Result<(), String> {
        self.actions.push("connect_state_manager".to_string());
        Ok(())
    }
    
    /// Process menu item click
    pub fn process_click(&mut self, menu_item: &str) -> Option<Message> {
        self.actions.push(format!("click: {}", menu_item));
        
        match menu_item {
            "Show" => Some(Message::ShowWindow),
            "Hide" => Some(Message::HideWindow),
            "Start Scan" => Some(Message::StartScan),
            "Stop Scan" => Some(Message::StopScan),
            "Settings" => Some(Message::ShowSettings),
            "Exit" => Some(Message::Exit),
            _ => None,
        }
    }
    
    /// Process system tray actions and convert to state manager actions
    pub fn process_system_tray_action(&mut self, action: &str, state_manager: &Arc<StateManager>) {
        self.actions.push(format!("process_action: {}", action));
        
        match action {
            "toggle_window" => {
                state_manager.dispatch(Action::ToggleVisibility);
            },
            "show_window" => {
                state_manager.dispatch(Action::ShowWindow);
            },
            "hide_window" => {
                state_manager.dispatch(Action::HideWindow);
            },
            "start_scan" => {
                state_manager.dispatch(Action::StartScanning);
            },
            "stop_scan" => {
                state_manager.dispatch(Action::StopScanning);
            },
            "exit" => {
                state_manager.dispatch(Action::Exit);
            },
            _ => {}
        }
    }
    
    /// Handle window state change triggered by another component
    pub fn handle_window_state_change(&mut self, visible: bool) -> Option<Message> {
        self.actions.push(format!("window_state_change: {}", visible));
        
        // Update menu items if needed
        let show_hide_index = if visible { 1 } else { 0 };
        if self.menu_items.len() > show_hide_index {
            // Swap Show/Hide menu items based on visibility
            if visible && self.menu_items[0] == "Show" {
                self.menu_items[0] = "Hide".to_string();
            } else if !visible && self.menu_items[0] == "Hide" {
                self.menu_items[0] = "Show".to_string();
            }
        }
        
        None
    }
    
    /// Update tooltip with battery info
    pub fn update_tooltip_with_battery(&mut self, left: Option<u8>, right: Option<u8>, case: Option<u8>) {
        let left_str = left.map_or("N/A".to_string(), |v| format!("{}%", v));
        let right_str = right.map_or("N/A".to_string(), |v| format!("{}%", v));
        let case_str = case.map_or("N/A".to_string(), |v| format!("{}%", v));
        
        self.tooltip = format!("RustPods - Connected\nLeft: {}\nRight: {}\nCase: {}", 
                              left_str, right_str, case_str);
        
        self.actions.push("update_tooltip".to_string());
    }
    
    /// Update icon based on battery status
    pub fn update_icon_with_battery(&mut self, battery: &AirPodsBatteryStatus) {
        // Determine whether any component is charging
        let is_charging = battery.battery.charging.left || battery.battery.charging.right || battery.battery.charging.case;
        
        // Get minimum battery level for icon
        let min_level = [battery.battery.left, battery.battery.right]
            .iter()
            .filter_map(|&level| level)
            .min()
            .unwrap_or(100);
        
        // Determine icon type
        if is_charging {
            self.icon_type = TrayIconType::Charging;
        } else if min_level <= self.low_battery_threshold {
            self.icon_type = TrayIconType::LowBattery;
        } else {
            self.icon_type = TrayIconType::BatteryLevel(min_level);
        }
        
        // If showing percentage, update the tooltip prefix
        if self.show_percentage {
            let prefix = if is_charging {
                format!("RustPods - {}% ⚡", min_level)
            } else if min_level <= self.low_battery_threshold {
                format!("RustPods - {}% ⚠️", min_level)
            } else {
                format!("RustPods - {}%", min_level)
            };
            
            if let Some(first_line_end) = self.tooltip.find('\n') {
                let rest = &self.tooltip[first_line_end..];
                self.tooltip = format!("{}{}", prefix, rest);
            }
        }
        
        self.actions.push(format!("update_icon: {:?}", self.icon_type));
    }
    
    /// Check for low battery and show notification if needed
    pub fn check_low_battery(&mut self, battery: &AirPodsBatteryStatus) {
        // Get the components with their battery levels
        let components = [
            ("Left AirPod", battery.battery.left),
            ("Right AirPod", battery.battery.right),
            ("Case", battery.battery.case),
        ];
        
        // Check each component for low battery
        for (name, level) in components.iter() {
            if let Some(battery_level) = level {
                if *battery_level <= self.low_battery_threshold {
                    let notification = format!("Low Battery Warning: {} at {}%", name, battery_level);
                    self.show_notification(&notification);
                }
            }
        }
    }
    
    /// Show a notification
    pub fn show_notification(&mut self, message: &str) {
        self.notifications.push(message.to_string());
        self.actions.push(format!("notification: {}", message));
    }
    
    /// Update theme mode
    pub fn update_theme(&mut self, theme: ThemeMode) {
        self.theme_mode = theme;
        self.actions.push(format!("update_theme: {:?}", theme));
    }
    
    /// Update config settings
    pub fn update_config(&mut self, config: &AppConfig) {
        // Update show percentage setting
        self.show_percentage = config.ui.show_percentage_in_tray;
        
        // Update theme mode
        match config.ui.theme {
            rustpods::config::Theme::Light => self.theme_mode = ThemeMode::Light,
            rustpods::config::Theme::Dark => self.theme_mode = ThemeMode::Dark,
            rustpods::config::Theme::System => self.theme_mode = ThemeMode::System,
            rustpods::config::Theme::CatppuccinMocha => self.theme_mode = ThemeMode::Dark,
        }
        
        // Update low battery threshold
        self.low_battery_threshold = config.ui.low_battery_threshold;
        
        self.actions.push("update_config".to_string());
    }
}

/// A test form for validation testing
pub struct TestForm {
    /// Field values
    pub values: HashMap<String, String>,
    /// Validation errors
    pub errors: HashMap<String, String>,
}

impl TestForm {
    /// Create a new test form
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            errors: HashMap::new(),
        }
    }
    
    /// Set a field value
    pub fn set_field(&mut self, name: &str, value: &str) {
        self.values.insert(name.to_string(), value.to_string());
        self.validate_field(name);
    }
    
    /// Validate a field
    pub fn validate_field(&mut self, name: &str) {
        let value = self.values.get(name).cloned().unwrap_or_default();
        
        // Simple validation logic for testing
        if name == "required_field" && value.is_empty() {
            self.errors.insert(name.to_string(), "Field is required".to_string());
        } else if name == "number_field" {
            if let Err(_) = value.parse::<i32>() {
                self.errors.insert(name.to_string(), "Must be a number".to_string());
            } else if let Ok(num) = value.parse::<i32>() {
                if num < 0 || num > 100 {
                    self.errors.insert(name.to_string(), "Number must be between 0 and 100".to_string());
                } else {
                    self.errors.remove(name);
                }
            }
        } else {
            self.errors.remove(name);
        }
    }
    
    /// Check if the form is valid
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

/// A validator function for testing
pub fn test_validator(input: &str) -> Result<(), String> {
    if input.is_empty() {
        Err("Field cannot be empty".to_string())
    } else {
        Ok(())
    }
} 