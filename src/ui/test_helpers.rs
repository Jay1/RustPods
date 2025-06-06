//! Test helpers and mocks for UI components
//! This module contains simplified versions and test doubles for UI components
//! to make testing easier without requiring a full GUI environment

use std::sync::Arc;
use std::collections::HashMap;

use crate::ui::message::Message;
use crate::ui::state_manager::StateManager;
use crate::config::AppConfig;
use crate::bluetooth::AirPodsBatteryStatus;
use crate::airpods::{AirPodsBattery, AirPodsChargingState};

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
    pub actions: Vec<String>,
    /// Tooltip text
    pub tooltip: String,
    /// Icon type
    pub icon_type: TrayIconType,
    /// Theme mode
    pub theme_mode: ThemeMode,
    /// Notifications sent
    pub notifications: Vec<String>,
    /// Low battery threshold
    pub low_battery_threshold: u8,
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
            actions: Vec::new(),
            tooltip: "RustPods".to_string(),
            icon_type: TrayIconType::Disconnected,
            theme_mode: ThemeMode::Dark,
            notifications: Vec::new(),
            low_battery_threshold: 20,
        }
    }
    
    /// Update the battery status
    pub fn update_battery(&mut self, battery: AirPodsBatteryStatus) {
        self.battery_status = Some(battery.clone());
        self.actions.push("update_battery".to_string());
        
        // Update tooltip
        self.update_tooltip(&battery);
        
        // Update icon based on battery
        self.update_icon_for_battery(&battery);
        
        // Check for low battery notifications
        self.check_low_battery(&battery);
    }
    
    /// Update the tooltip based on battery status
    fn update_tooltip(&mut self, battery: &AirPodsBatteryStatus) {
        let mut tooltip = "RustPods".to_string();
        
        // Add overall battery level if available
        if let Some(left) = battery.battery.left {
            tooltip.push_str(&format!(" - {}%", left));
        }
        
        tooltip.push('\n');
        
        // Add detailed battery levels
        tooltip.push_str(&format!(
            "Left: {}\nRight: {}\nCase: {}", 
            battery.battery.left.map_or("N/A".to_string(), |v| format!("{}%", v)),
            battery.battery.right.map_or("N/A".to_string(), |v| format!("{}%", v)),
            battery.battery.case.map_or("N/A".to_string(), |v| format!("{}%", v)),
        ));
        
        self.tooltip = tooltip;
        self.actions.push("update_tooltip".to_string());
    }
    
    /// Update icon based on battery status
    fn update_icon_for_battery(&mut self, battery: &AirPodsBatteryStatus) {
        // Default to disconnected
        let mut icon = TrayIconType::Disconnected;
        
        if !self.is_connected {
            // Keep disconnected if not connected
            icon = TrayIconType::Disconnected;
        } else if let Some(charging) = &battery.battery.charging {
            // Check for charging status
            if *charging != crate::airpods::AirPodsChargingState::NotCharging {
                icon = TrayIconType::Charging;
            } else {
                // Check for low battery
                let min_battery = Self::get_min_battery_level(&battery.battery);
                if let Some(level) = min_battery {
                    if level <= self.low_battery_threshold {
                        icon = TrayIconType::LowBattery;
                    } else {
                        icon = TrayIconType::BatteryLevel(level);
                    }
                }
            }
        } else if let Some(left) = battery.battery.left {
            // Use left earbud battery level for icon
            if left <= self.low_battery_threshold {
                icon = TrayIconType::LowBattery;
            } else {
                icon = TrayIconType::BatteryLevel(left);
            }
        }
        
        self.icon_type = icon.clone();
        self.actions.push(format!("update_icon: {:?}", icon));
    }
    
    /// Check for low battery and send notifications
    fn check_low_battery(&mut self, battery: &AirPodsBatteryStatus) {
        // Check left earbud
        if let Some(left) = battery.battery.left {
            if left <= self.low_battery_threshold {
                self.notifications.push(format!("Low Battery Warning: Left AirPod at {}%", left));
            }
        }
        
        // Check right earbud
        if let Some(right) = battery.battery.right {
            if right <= self.low_battery_threshold {
                self.notifications.push(format!("Low Battery Warning: Right AirPod at {}%", right));
            }
        }
        
        // Check case
        if let Some(case) = battery.battery.case {
            if case <= self.low_battery_threshold {
                self.notifications.push(format!("Low Battery Warning: Case at {}%", case));
            }
        }
    }
    
    /// Get minimum battery level across all components
    fn get_min_battery_level(battery: &crate::airpods::AirPodsBattery) -> Option<u8> {
        let mut levels = Vec::new();
        
        if let Some(left) = battery.left {
            levels.push(left);
        }
        
        if let Some(right) = battery.right {
            levels.push(right);
        }
        
        if let Some(case) = battery.case {
            levels.push(case);
        }
        
        if levels.is_empty() {
            None
        } else {
            levels.iter().min().copied()
        }
    }
    
    /// Handle battery update (alias for update_battery)
    pub fn handle_battery_update(&mut self, battery: AirPodsBatteryStatus) -> Result<(), String> {
        self.update_battery(battery);
        Ok(())
    }
    
    /// Connect to state manager
    pub fn connect_state_manager(&mut self, _state_manager: Arc<StateManager>) -> Result<(), String> {
        self.actions.push("connect_state_manager".to_string());
        Ok(())
    }
    
    /// Update the connection status
    pub fn update_connection(&mut self, connected: bool) {
        self.is_connected = connected;
        self.actions.push(format!("update_connection: {}", connected));
    }
    
    /// Update the theme mode
    pub fn update_theme(&mut self, mode: ThemeMode) {
        self.theme_mode = mode;
        self.actions.push(format!("update_theme: {:?}", mode));
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: &crate::config::AppConfig) {
        // Update theme based on config
        let theme_mode = match config.ui.theme {
            crate::config::Theme::Light => ThemeMode::Light,
            crate::config::Theme::Dark => ThemeMode::Dark,
            crate::config::Theme::System => ThemeMode::System,
        };
        
        self.update_theme(theme_mode);
        self.actions.push("update_config".to_string());
    }
    
    /// Process a click on a menu item
    pub fn process_click(&mut self, menu_item: &str) -> Option<Message> {
        self.actions.push(format!("click: {}", menu_item));
        match menu_item {
            "Settings" => Some(Message::SaveSettings),
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
            charging: Some(AirPodsChargingState::BothBudsCharging),
        },
        last_updated: std::time::Instant::now(),
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