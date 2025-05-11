use std::collections::HashMap;
use std::time::Instant;
use iced::{Sandbox, Command};

use crate::bluetooth::DiscoveredDevice;
use crate::config::AppConfig;
use crate::ui::Message;

/// Main application state
#[derive(Debug, Clone)]
pub struct AppState {
    /// Whether the application window is visible
    pub visible: bool,
    
    /// Whether we're currently scanning for devices
    pub is_scanning: bool,
    
    /// Whether automatic scanning is enabled
    pub auto_scan: bool,
    
    /// Discovered Bluetooth devices
    pub devices: HashMap<String, DiscoveredDevice>,
    
    /// Currently selected device
    pub selected_device: Option<String>,
    
    /// Application configuration
    pub config: AppConfig,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            visible: true,
            is_scanning: false,
            auto_scan: true,
            devices: HashMap::new(),
            selected_device: None,
            config: AppConfig::default(),
        }
    }
}

impl Sandbox for AppState {
    type Message = Message;
    
    fn new() -> Self {
        Self::default()
    }
    
    fn title(&self) -> String {
        String::from("RustPods")
    }
    
    fn update(&mut self, message: Message) {
        match message {
            Message::ToggleVisibility => {
                self.toggle_visibility();
            }
            Message::Exit => {
                // Exit the application - handled by runner
            }
            Message::DeviceDiscovered(device) => {
                self.update_device(device);
            }
            Message::DeviceUpdated(device) => {
                self.update_device(device);
            }
            Message::SelectDevice(address) => {
                self.select_device(address);
            }
            Message::StartScan => {
                self.is_scanning = true;
            }
            Message::StopScan => {
                self.is_scanning = false;
            }
            Message::ToggleAutoScan(enabled) => {
                self.auto_scan = enabled;
            }
            Message::Tick => {
                // Periodic update
            }
        }
    }
    
    fn view(&self) -> iced::Element<Message> {
        crate::ui::app::view(self)
    }
}

impl AppState {
    /// Toggle the visibility of the application
    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }
    
    /// Update a device in the devices list
    pub fn update_device(&mut self, device: DiscoveredDevice) {
        let address = device.address.to_string();
        self.devices.insert(address, device);
    }
    
    /// Select a device by address
    pub fn select_device(&mut self, address: String) {
        self.selected_device = Some(address);
    }
    
    /// Get the currently selected device
    pub fn get_selected_device(&self) -> Option<&DiscoveredDevice> {
        self.selected_device.as_ref().and_then(|addr| self.devices.get(addr))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use btleplug::api::BDAddr;
    use std::time::Instant;
    
    #[test]
    fn test_default_state() {
        let state = AppState::default();
        assert!(state.visible);
        assert!(!state.is_scanning);
        assert!(state.auto_scan);
        assert!(state.devices.is_empty());
        assert_eq!(state.selected_device, None);
    }
    
    #[test]
    fn test_toggle_visibility() {
        let mut state = AppState::default();
        assert!(state.visible);
        
        state.toggle_visibility();
        assert!(!state.visible);
        
        state.toggle_visibility();
        assert!(state.visible);
    }
    
    #[test]
    fn test_update_device() {
        let mut state = AppState::default();
        assert!(state.devices.is_empty());
        
        let addr = BDAddr::from([1, 2, 3, 4, 5, 6]);
        let addr_str = addr.to_string();
        
        let device = DiscoveredDevice {
            address: addr,
            name: Some("Test Device".to_string()),
            rssi: Some(-60),
            manufacturer_data: HashMap::new(),
            is_potential_airpods: false,
            last_seen: Instant::now(),
        };
        
        state.update_device(device.clone());
        assert_eq!(state.devices.len(), 1);
        assert!(state.devices.contains_key(&addr_str));
        
        // Update existing device
        let updated_device = DiscoveredDevice {
            rssi: Some(-50),
            ..device
        };
        
        state.update_device(updated_device);
        assert_eq!(state.devices.len(), 1);
        assert_eq!(state.devices.get(&addr_str).unwrap().rssi, Some(-50));
    }
    
    #[test]
    fn test_select_device() {
        let mut state = AppState::default();
        let addr = BDAddr::from([1, 2, 3, 4, 5, 6]);
        let addr_str = addr.to_string();
        
        // Device not in the list yet
        state.select_device(addr_str.clone());
        assert_eq!(state.selected_device, Some(addr_str.clone()));
        
        // Add the device
        let device = DiscoveredDevice {
            address: addr,
            name: Some("Test Device".to_string()),
            rssi: Some(-60),
            manufacturer_data: HashMap::new(),
            is_potential_airpods: false,
            last_seen: Instant::now(),
        };
        
        state.update_device(device);
        
        // Now select it again
        let addr_str2 = addr.to_string(); // Create a new string to avoid using the moved value
        state.select_device(addr_str2.clone());
        assert_eq!(state.selected_device, Some(addr_str2));
    }
    
    #[test]
    fn test_get_selected_device() {
        let mut state = AppState::default();
        let addr = BDAddr::from([1, 2, 3, 4, 5, 6]);
        let addr_str = addr.to_string();
        
        // No selected device
        assert!(state.get_selected_device().is_none());
        
        // Add a device
        let device = DiscoveredDevice {
            address: addr,
            name: Some("Test Device".to_string()),
            rssi: Some(-60),
            manufacturer_data: HashMap::new(),
            is_potential_airpods: false,
            last_seen: Instant::now(),
        };
        
        state.update_device(device.clone());
        state.select_device(addr_str);
        
        // Get the selected device
        let selected = state.get_selected_device();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, Some("Test Device".to_string()));
    }
} 