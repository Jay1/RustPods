//! Common test helpers for all UI component tests
//! Contains mock implementations and utility functions

use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Instant, Duration};
use std::path::PathBuf;
use iced::{Point, Size, Rectangle};
use btleplug::api::BDAddr;

use rustpods::ui::Message;
use rustpods::bluetooth::AirPodsBatteryStatus;
use rustpods::airpods::{AirPodsBattery, AirPodsChargingState};
use rustpods::config::{AppConfig, Theme, Theme as ConfigTheme};
use rustpods::config::{BluetoothConfig, UiConfig, SystemConfig, LogLevel};
use rustpods::config::app_config::BatteryConfig;
use rustpods::ui::window_visibility::WindowPosition;
use rustpods::ui::state_manager::{StateManager, Action};

/// Mock system tray error type
#[derive(Debug, thiserror::Error)]
pub enum MockSystemTrayError {
    #[error("Generic error: {0}")]
    Generic(String),
    
    #[error("Failed to create tray item: {0}")]
    Creation(String),
    
    #[error("Failed to add menu item: {0}")]
    MenuItem(String),
    
    #[error("Failed to set icon: {0}")]
    SetIcon(String),
    
    #[error("Failed to set tooltip: {0}")]
    SetTooltip(String),
    
    #[error("Failed to handle tray event: {0}")]
    EventHandling(String),
    
    #[error("Registry error: {0}")]
    Registry(String),
    
    #[error("Failed to connect to state manager: {0}")]
    StateManagerError(String),
    
    #[error("Failed to update battery status: {0}")]
    BatteryUpdateError(String),
    
    #[error("Failed to clean up resources: {0}")]
    CleanupError(String),
    
    #[error("Notification error: {0}")]
    NotificationError(String),
    
    #[error("Icon resource error: {0}")]
    ResourceError(String),
}

/// Alias to match actual SystemTrayError
pub type SystemTrayError = MockSystemTrayError;

/// Create a test AppConfig for testing
pub fn create_test_config() -> AppConfig {
    AppConfig {
        bluetooth: BluetoothConfig {
            auto_scan_on_startup: true,
            scan_duration: Duration::from_secs(5),
            scan_interval: Duration::from_secs(30),
            battery_refresh_interval: 60,
            min_rssi: Some(-80),
            auto_reconnect: true,
            reconnect_attempts: 3,
            adaptive_polling: false,
        },
        ui: UiConfig {
            show_notifications: true,
            start_minimized: false,
            theme: Theme::Dark,
            show_percentage_in_tray: true,
            show_low_battery_warning: true,
            low_battery_threshold: 20,
            remember_window_position: true,
            last_window_position: None,
            minimize_to_tray_on_close: true,
            minimize_on_blur: false,
            auto_hide_timeout: None,
        },
        system: SystemConfig {
            launch_at_startup: true,
            log_level: LogLevel::Info,
            enable_telemetry: false,
            auto_save_interval: Some(300),
            enable_crash_recovery: true,
        },
        battery: BatteryConfig {
            low_threshold: 20,
            smoothing_enabled: true,
            change_threshold: 5,
            notify_low: true,
            notify_charged: true,
        },
        settings_path: PathBuf::from("settings.json"),
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
            charging: Some(AirPodsChargingState::CaseCharging),
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
    
    /// Convert Rectangle to WindowPosition
    pub fn rect_to_position(rect: Rectangle) -> WindowPosition {
        WindowPosition {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }
    
    /// Convert WindowPosition to Rectangle
    pub fn position_to_rect(pos: WindowPosition) -> Rectangle {
        Rectangle::new(
            Point::new(pos.x, pos.y),
            Size::new(pos.width, pos.height),
        )
    }
    
    /// Update method to trigger auto-hide
    pub fn update(&mut self, rect: Rectangle) -> Option<Message> {
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

/// Mock tray icon types to match the real ones
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

/// Mock theme modes to match the real ones
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    /// Light theme
    Light,
    /// Dark theme
    Dark,
    /// System theme
    System,
}

impl From<ThemeMode> for ConfigTheme {
    fn from(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Light => ConfigTheme::Light,
            ThemeMode::Dark => ConfigTheme::Dark,
            ThemeMode::System => ConfigTheme::System,
        }
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
    /// Notifications that have been sent
    pub notifications: Vec<String>,
    /// Associated state manager
    pub state_manager: Option<Arc<StateManager>>,
    /// Whether the tray is registered for startup
    pub startup_registered: bool,
    /// Application configuration
    pub config: AppConfig,
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
            tooltip: "RustPods".to_string(),
            icon_type: TrayIconType::Disconnected,
            theme_mode: ThemeMode::Dark,
            show_percentage: true,
            low_battery_threshold: 20,
            notifications: Vec::new(),
            state_manager: None,
            startup_registered: false,
            config: create_test_config(),
        }
    }
    
    /// Set state manager
    pub fn set_state_manager(&mut self, state_manager: Arc<StateManager>) -> Result<(), MockSystemTrayError> {
        self.state_manager = Some(state_manager);
        self.actions.push("set_state_manager".to_string());
        Ok(())
    }
    
    /// Process click
    pub fn process_click(&mut self, item: &str) -> Option<Message> {
        self.actions.push(format!("click: {}", item));
        
        match item {
            "Show" => Some(Message::ShowWindow),
            "Hide" => Some(Message::HideWindow),
            "Start Scan" => Some(Message::StartScan),
            "Stop Scan" => Some(Message::StopScan),
            "Settings" => Some(Message::SaveSettings),
            "Exit" => Some(Message::Exit),
            _ => None,
        }
    }
    
    /// Update connection status
    pub fn update_connection(&mut self, connected: bool) -> Result<(), MockSystemTrayError> {
        self.is_connected = connected;
        self.actions.push(format!("update_connection: {}", connected));
        
        // Update icon based on connection status
        if !connected {
            self.icon_type = TrayIconType::Disconnected;
            self.actions.push("update_icon: Disconnected".to_string());
        }
        
        Ok(())
    }
    
    /// Update battery status
    pub fn update_battery(&mut self, status: AirPodsBatteryStatus) -> Result<(), MockSystemTrayError> {
        // Save battery status
        self.battery_status = Some(status.clone());
        self.actions.push("update_battery".to_string());
        
        // Update icon if connected
        if self.is_connected {
            self.update_icon_for_battery(&status);
        }
        
        // Update tooltip
        self.update_tooltip_with_battery(&status);
        
        // Check for low battery levels and send notifications
        self.check_battery_levels(&status);
        
        Ok(())
    }
    
    /// Update tooltip with battery status
    fn update_tooltip_with_battery(&mut self, battery: &AirPodsBatteryStatus) {
        let mut tooltip = String::from("RustPods");
        
        // Add battery percentage to title if enabled
        if self.show_percentage {
            if let Some(min_level) = Self::get_min_battery_level(&battery.battery) {
                tooltip.push_str(&format!(" - {}%", min_level));
            }
        }
        
        // Add individual battery levels
        tooltip.push_str("\nLeft: ");
        if let Some(left) = &battery.battery.left {
            tooltip.push_str(&format!("{}%", left));
        } else {
            tooltip.push_str("N/A");
        }
        
        tooltip.push_str("\nRight: ");
        if let Some(right) = &battery.battery.right {
            tooltip.push_str(&format!("{}%", right));
        } else {
            tooltip.push_str("N/A");
        }
        
        tooltip.push_str("\nCase: ");
        if let Some(case) = &battery.battery.case {
            tooltip.push_str(&format!("{}%", case));
        } else {
            tooltip.push_str("N/A");
        }
        
        self.tooltip = tooltip;
        self.actions.push("update_tooltip".to_string());
    }
    
    /// Update icon for battery
    pub fn update_icon_for_battery(&mut self, battery: &AirPodsBatteryStatus) {
        // Default to disconnected
        let mut icon = TrayIconType::Disconnected;
        
        if !self.is_connected {
            // Keep disconnected if not connected
            icon = TrayIconType::Disconnected;
        } else if let Some(charging) = &battery.battery.charging {
            // Check for charging status
            if *charging != AirPodsChargingState::NotCharging {
                icon = TrayIconType::Charging;
            } else {
                // Check for low battery
                let min_level = Self::get_min_battery_level(&battery.battery).unwrap_or(100);
                
                if min_level <= self.low_battery_threshold {
                    icon = TrayIconType::LowBattery;
                } else {
                    icon = TrayIconType::BatteryLevel(min_level);
                }
            }
        }
        
        // Update the icon type
        self.icon_type = icon.clone();
        self.actions.push(format!("update_icon: {:?}", icon));
    }
    
    /// Update tray icon
    pub fn update_icon(&mut self, connected: bool) -> Result<(), MockSystemTrayError> {
        if connected {
            // Use battery status if available
            if let Some(battery) = &self.battery_status.clone() {
                self.update_icon_for_battery(battery);
            } else {
                self.icon_type = TrayIconType::BatteryLevel(100);
            }
        } else {
            self.icon_type = TrayIconType::Disconnected;
        }
        
        self.actions.push(format!("update_icon: {:?}", self.icon_type));
        Ok(())
    }
    
    /// Check battery levels for low battery notifications
    fn check_battery_levels(&mut self, battery: &AirPodsBatteryStatus) {
        // Check left AirPod
        if let Some(left) = battery.battery.left {
            if left <= self.low_battery_threshold {
                let notification = format!("Low Battery: Left AirPod at {}%", left);
                self.notifications.push(notification);
            }
        }
        
        // Check right AirPod
        if let Some(right) = battery.battery.right {
            if right <= self.low_battery_threshold {
                let notification = format!("Low Battery: Right AirPod at {}%", right);
                self.notifications.push(notification);
            }
        }
        
        // Check case
        if let Some(case) = battery.battery.case {
            if case <= self.low_battery_threshold {
                let notification = format!("Low Battery: Case at {}%", case);
                self.notifications.push(notification);
            }
        }
    }
    
    /// Update theme mode
    pub fn update_theme(&mut self, theme_mode: ThemeMode) -> Result<(), MockSystemTrayError> {
        self.theme_mode = theme_mode;
        self.actions.push(format!("update_theme: {:?}", theme_mode));
        Ok(())
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: &AppConfig) -> Result<(), MockSystemTrayError> {
        // Update show percentage setting
        self.show_percentage = config.ui.show_percentage_in_tray;
        
        // Update theme mode based on config
        let theme_mode = match config.ui.theme {
            ConfigTheme::Light => ThemeMode::Light,
            ConfigTheme::Dark => ThemeMode::Dark,
            ConfigTheme::System => ThemeMode::System,
        };
        
        // Update theme
        self.update_theme(theme_mode)?;
        
        // Update low battery threshold
        self.low_battery_threshold = config.battery.low_threshold;
        
        self.actions.push("update_config".to_string());
        Ok(())
    }
    
    /// Process state update
    pub fn process_state_update(&mut self) -> Result<(), MockSystemTrayError> {
        if let Some(state_manager) = &self.state_manager {
            // Get device state
            let device_state = state_manager.get_device_state();
            
            // Update connected status
            let was_connected = self.is_connected;
            self.is_connected = device_state.selected_device.is_some();
            
            // If connection status changed, update icon
            if was_connected != self.is_connected {
                self.update_icon(self.is_connected)?;
            }
            
            self.actions.push("process_state_update".to_string());
        }
        
        Ok(())
    }
    
    /// Helper to get minimum battery level
    fn get_min_battery_level(battery: &AirPodsBattery) -> Option<u8> {
        let levels = vec![battery.left, battery.right];
        levels.iter().filter_map(|&x| x).min()
    }
    
    /// Set startup enabled
    pub fn set_startup_enabled(&mut self, enabled: bool) -> Result<(), MockSystemTrayError> {
        self.startup_registered = enabled;
        self.actions.push(format!("set_startup_enabled: {}", enabled));
        Ok(())
    }
    
    /// Check if startup is registered
    pub fn is_startup_registered(&self) -> bool {
        self.startup_registered
    }
    
    /// Cleanup resources
    pub fn cleanup(&mut self) -> Result<(), MockSystemTrayError> {
        self.actions.push("cleanup".to_string());
        Ok(())
    }

    /// Connect to state manager
    pub fn connect_state_manager(&mut self, state_manager: Arc<StateManager>) -> Result<(), MockSystemTrayError> {
        self.state_manager = Some(state_manager);
        self.actions.push("connect_state_manager".to_string());
        Ok(())
    }

    /// Handle battery update 
    pub fn handle_battery_update(&mut self, battery: AirPodsBatteryStatus) -> Result<(), MockSystemTrayError> {
        // Call update_battery which handles all the business logic
        self.update_battery(battery)
    }

    /// Send notification
    pub fn send_notification(&mut self, title: &str, message: &str) -> Result<(), MockSystemTrayError> {
        let notification = format!("{}: {}", title, message);
        self.notifications.push(notification);
        self.actions.push("send_notification".to_string());
        Ok(())
    }

    /// Get current connection state
    pub fn is_connected(&self) -> bool {
        self.is_connected
    }

    /// Get current tooltip
    pub fn get_tooltip(&self) -> &str {
        &self.tooltip
    }

    /// Set tooltip
    pub fn set_tooltip(&mut self, tooltip: &str) -> Result<(), MockSystemTrayError> {
        self.tooltip = tooltip.to_string();
        self.actions.push("set_tooltip".to_string());
        Ok(())
    }

    /// Add menu item
    pub fn add_menu_item(&mut self, item: &str) -> Result<(), MockSystemTrayError> {
        self.menu_items.push(item.to_string());
        self.actions.push(format!("add_menu_item: {}", item));
        Ok(())
    }

    /// Remove menu item
    pub fn remove_menu_item(&mut self, item: &str) -> Result<(), MockSystemTrayError> {
        if let Some(index) = self.menu_items.iter().position(|x| x == item) {
            self.menu_items.remove(index);
            self.actions.push(format!("remove_menu_item: {}", item));
        }
        Ok(())
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