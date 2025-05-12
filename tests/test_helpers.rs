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
use rustpods::bluetooth::{AirPodsBatteryStatus, AirPodsBattery, AirPodsCharging};

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
        },
        charging: AirPodsCharging {
            left: false,
            right: false,
            case: true,
        },
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
}

impl MockSystemTray {
    /// Create a new mock system tray
    pub fn new() -> Self {
        Self {
            is_connected: false,
            battery_status: None,
            menu_items: vec![
                "Show".to_string(),
                "Settings".to_string(),
                "Exit".to_string(),
            ],
            actions: Vec::new(),
        }
    }
    
    /// Update the battery status
    pub fn handle_battery_update(&mut self, battery: AirPodsBatteryStatus) -> Result<(), String> {
        self.battery_status = Some(battery);
        self.actions.push("update_battery".to_string());
        Ok(())
    }
    
    /// Connect to state manager
    pub fn connect_state_manager(&mut self, _state_manager: Arc<StateManager>) -> Result<(), String> {
        self.is_connected = true;
        self.actions.push("connect_state_manager".to_string());
        Ok(())
    }
    
    /// Process a click on a menu item
    pub fn process_click(&mut self, menu_item: &str) -> Option<Message> {
        self.actions.push(format!("click: {}", menu_item));
        
        match menu_item {
            "Show" => Some(Message::ToggleVisibility),
            "Settings" => Some(Message::OpenSettings),
            "Exit" => Some(Message::Exit),
            _ => None,
        }
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