//! Test helpers and mocks for UI components
//! This module contains simplified versions and test doubles for UI components
//! to make testing easier without requiring a full GUI environment

use std::sync::Arc;
use std::collections::HashMap;
use chrono::Utc;

use crate::ui::Message;
use crate::ui::state_manager::StateManager;
use crate::config::AppConfig;
use crate::bluetooth::AirPodsBatteryStatus;
use crate::airpods::{AirPodsBattery, ChargingStatus as AirPodsCharging};

/// A simplified test version of UI components that would normally require
/// the full Iced framework
pub struct TestDouble;

/// A test version of the window visibility manager that avoids complex Iced dependencies
#[derive(Debug)]
pub struct MockWindowVisibilityManager {
    /// Whether the window is visible
    pub visible: bool,
    /// Last position
    pub position: Option<(f32, f32, f32, f32)>
}

impl Default for MockWindowVisibilityManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MockWindowVisibilityManager {
    /// Create a new mock window visibility manager
    pub fn new() -> Self {
        Self {
            visible: true,
            position: None
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
    
    /// Toggle visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }
}

/// A test version of the system tray
#[derive(Debug, Clone)]
pub struct MockSystemTray {
    /// Whether the system tray is connected
    pub is_connected: bool,
    /// Last battery status
    pub battery_status: Option<AirPodsBatteryStatus>,
    /// The menu items
    pub menu_items: Vec<String>,
    /// Called actions
    pub actions: Vec<String>
}

impl Default for MockSystemTray {
    fn default() -> Self {
        Self::new()
    }
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
            actions: Vec::new()
        }
    }
    
    /// Update the battery status
    pub fn update_battery(&mut self, battery: AirPodsBatteryStatus) {
        self.battery_status = Some(battery);
        self.actions.push("update_battery".to_string());
    }
    
    /// Update the connection status
    pub fn update_connection(&mut self, connected: bool) {
        self.is_connected = connected;
        self.actions.push(format!("update_connection: {}", connected));
    }
    
    /// Process a click on a menu item
    pub fn process_click(&mut self, menu_item: &str) -> Option<Message> {
        self.actions.push(format!("click: {}", menu_item));
        
        match menu_item {
            "Show" => Some(Message::ShowWindow),
            "Settings" => Some(Message::OpenSettings),
            "Exit" => Some(Message::Exit),
            _ => None
        }
    }
}

/// Create a test battery status
pub fn create_test_battery() -> AirPodsBatteryStatus {
    AirPodsBatteryStatus {
        battery: AirPodsBattery {
            left: Some(75),
            right: Some(80),
            case: Some(90),
            charging: AirPodsCharging {
                left: true,
                right: true,
                case: true,
            },
        },
        last_updated: Utc::now(),
    }
}

/// Create a test state manager
pub fn create_test_state_manager() -> Arc<StateManager> {
    let (tx, _) = tokio::sync::mpsc::unbounded_channel();
    Arc::new(StateManager::new(tx))
}

/// Create a test configuration
pub fn create_test_config() -> AppConfig {
    AppConfig::default()
}

/// A validator function for testing
pub fn test_validator(input: &str) -> Result<(), String> {
    if input.is_empty() {
        Err("Field cannot be empty".to_string())
    } else {
        Ok(())
    }
}

/// A test form data structure
pub struct TestForm {
    /// Field values
    pub values: HashMap<String, String>,
    /// Validation errors
    pub errors: HashMap<String, String>,
}

impl Default for TestForm {
    fn default() -> Self {
        Self::new()
    }
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
            if value.parse::<i32>().is_err() {
                self.errors.insert(name.to_string(), "Must be a number".to_string());
            } else if let Ok(num) = value.parse::<i32>() {
                if !(0..=100).contains(&num) {
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