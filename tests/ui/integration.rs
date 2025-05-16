//! Integration tests for UI components and state

use btleplug::api::BDAddr;
use std::collections::HashMap;
use std::time::Instant;


use iced::Application;

use rustpods::bluetooth::DiscoveredDevice;
use rustpods::config::AppConfig;
use rustpods::ui::state::AppState;
use rustpods::ui::Message;

/// Test the full state update flow with simulated device events
#[test]
fn test_state_device_flow() {
    // Create initial state
    let mut state = AppState::default();
    assert!(state.devices.is_empty());
    
    // Create a test device that might be AirPods
    let airpods_addr = BDAddr::from([1, 2, 3, 4, 5, 6]);
    let airpods_addr_str = airpods_addr.to_string();
    let mut mfg_data = HashMap::new();
    mfg_data.insert(0x004C, vec![1, 2, 3, 4]); // Apple manufacturer ID
    
    let airpods_device = DiscoveredDevice {
        address: BDAddr::from([0x01, 0x02, 0x03, 0x04, 0x05, 0x06]),
        name: Some("AirPods Pro".to_string()),
        rssi: Some(-60),
        manufacturer_data: HashMap::new(),
        is_potential_airpods: true,
        last_seen: Instant::now(),
        is_connected: false,
        service_data: HashMap::new(),
        services: Vec::new(),
        tx_power_level: None,
    };
    
    // Create a regular Bluetooth device
    let bt_addr = BDAddr::from([6, 5, 4, 3, 2, 1]);
    let bt_addr_str = bt_addr.to_string();
    let bt_device = DiscoveredDevice {
        address: BDAddr::from([0x11, 0x22, 0x33, 0x44, 0x55, 0x66]),
        name: Some("Bluetooth Speaker".to_string()),
        rssi: Some(-70),
        manufacturer_data: HashMap::new(),
        is_potential_airpods: false,
        last_seen: Instant::now(),
        is_connected: false,
        service_data: HashMap::new(),
        services: Vec::new(),
        tx_power_level: None,
    };
    
    // Add devices to state
    state.update_device(airpods_device);
    assert_eq!(state.devices.len(), 1);
    assert!(state.devices.contains_key(&airpods_addr_str));
    
    state.update_device(bt_device);
    assert_eq!(state.devices.len(), 2);
    assert!(state.devices.contains_key(&bt_addr_str));
    
    // Select the AirPods device
    state.select_device(airpods_addr_str.clone());
    assert_eq!(state.selected_device, Some(airpods_addr_str.clone()));
    
    let selected = state.get_selected_device().unwrap();
    assert_eq!(selected.address, airpods_addr);
    assert!(selected.is_potential_airpods);
    
    // Update RSSI for the AirPods device
    let updated_airpods = DiscoveredDevice {
        address: BDAddr::from([0x01, 0x02, 0x03, 0x04, 0x05, 0x06]),
        name: Some("AirPods Pro".to_string()),
        rssi: Some(-55),
        manufacturer_data: HashMap::new(),
        is_potential_airpods: true,
        last_seen: Instant::now(),
        is_connected: false,
        service_data: HashMap::new(),
        services: Vec::new(),
        tx_power_level: None,
    };
    
    state.update_device(updated_airpods);
    assert_eq!(state.devices.len(), 2); // Still just 2 devices
    
    // Verify the update worked
    let updated_device = state.devices.get(&airpods_addr_str).unwrap();
    assert_eq!(updated_device.rssi, Some(-55));
    
    // Verify the selected device also reflects the update
    let selected = state.get_selected_device().unwrap();
    assert_eq!(selected.rssi, Some(-55));
}

/// Test visibility toggling affects the state correctly
#[test]
fn test_visibility_toggle() {
    let mut state = AppState::default();
    assert!(!state.visible); // Default is NOT visible
    
    // Toggle visibility
    state.toggle_visibility();
    assert!(state.visible);
    
    // Toggle back
    state.toggle_visibility();
    assert!(!state.visible);
}

/// Test that state handles messages correctly
#[test]
fn test_message_handling() {
    // Create a state instance
    let mut state = AppState::default();
    
    // Test StartScan message
    let _ = state.update(Message::StartScan);
    assert!(state.is_scanning);
    
    // Test StopScan message
    let _ = state.update(Message::StopScan);
    assert!(!state.is_scanning);
    
    // Test ToggleAutoScan message
    let _ = state.update(Message::ToggleAutoScan(false));
    assert!(!state.auto_scan);
    
    let _ = state.update(Message::ToggleAutoScan(true));
    assert!(state.auto_scan);
    
    // Test ToggleVisibility message
    let initial_visibility = state.visible;
    let _ = state.update(Message::ToggleVisibility);
    assert_ne!(state.visible, initial_visibility);
}

/// Test that default config is used with default state
#[test]
fn test_default_config() {
    let state = AppState::default();
    
    // Verify default config values
    let default_config = AppConfig::default();
    assert_eq!(state.config.bluetooth.scan_duration, default_config.bluetooth.scan_duration);
    assert_eq!(state.config.bluetooth.scan_interval, default_config.bluetooth.scan_interval);
    assert_eq!(state.config.bluetooth.auto_scan_on_startup, default_config.bluetooth.auto_scan_on_startup);
}

// Create a mock device for testing
fn create_test_device(address: &str) -> DiscoveredDevice {
    // Parse the address using BDAddr::from or directly create from bytes
    let addr = if address.contains(':') {
        // Parse from string like "11:22:33:44:55:66"
        let bytes: Vec<&str> = address.split(':').collect();
        let mut addr_bytes = [0u8; 6];
        for (i, byte) in bytes.iter().enumerate() {
            addr_bytes[i] = u8::from_str_radix(byte, 16).unwrap_or(0);
        }
        BDAddr::from(addr_bytes)
    } else {
        // Just for tests, create a simple address if not in correct format
        BDAddr::from([1, 2, 3, 4, 5, 6])
    };
    
    DiscoveredDevice {
        address: addr,
        name: Some(format!("Test Device {}", address)),
        rssi: Some(-70),
        manufacturer_data: HashMap::new(),
        is_potential_airpods: false,
        last_seen: std::time::Instant::now(),
        is_connected: false,
        service_data: HashMap::new(),
        services: Vec::new(),
        tx_power_level: None,
    }
}

#[test]
fn test_app_state_defaults() {
    let state = AppState::default();
    
    // Check initial state
    assert!(!state.visible);
    assert!(!state.is_scanning);
    assert!(state.auto_scan);
    assert!(state.devices.is_empty());
    assert_eq!(state.selected_device, None);
    
    // Verify config matches default
    let default_config = AppConfig::default();
    assert_eq!(state.config.bluetooth.scan_duration, default_config.bluetooth.scan_duration);
    assert_eq!(state.config.bluetooth.scan_interval, default_config.bluetooth.scan_interval);
    assert_eq!(state.config.bluetooth.auto_scan_on_startup, default_config.bluetooth.auto_scan_on_startup);
}

#[test]
fn test_toggle_visibility() {
    let mut state = AppState::default();
    assert!(!state.visible);
    
    // Toggle visibility
    let _ = state.update(Message::ToggleVisibility);
    assert!(state.visible);
    
    // Toggle again
    let _ = state.update(Message::ToggleVisibility);
    assert!(!state.visible);
}

#[test]
fn test_device_discovery() {
    let mut state = AppState::default();
    let device = create_test_device("11:22:33:44:55:66");
    
    // Add device
    let _ = state.update(Message::DeviceDiscovered(device.clone()));
    
    // Verify device was added
    assert_eq!(state.devices.len(), 1);
    assert!(state.devices.contains_key(&device.address.to_string()));
    
    // Compare device data
    let stored_device = state.devices.get(&device.address.to_string()).unwrap();
    assert_eq!(stored_device.name, device.name);
    assert_eq!(stored_device.rssi, device.rssi);
}

#[test]
fn test_device_selection() {
    let mut state = AppState::default();
    let device1 = create_test_device("11:22:33:44:55:66");
    let device2 = create_test_device("AA:BB:CC:DD:EE:FF");
    
    // Add devices
    let _ = state.update(Message::DeviceDiscovered(device1.clone()));
    let _ = state.update(Message::DeviceDiscovered(device2.clone()));
    
    // Select first device
    let _ = state.update(Message::SelectDevice(device1.address.to_string()));
    assert_eq!(state.selected_device, Some(device1.address.to_string()));
    
    // Select second device
    let _ = state.update(Message::SelectDevice(device2.address.to_string()));
    assert_eq!(state.selected_device, Some(device2.address.to_string()));
    
    // Try to select non-existent device
    let _ = state.update(Message::SelectDevice("00:00:00:00:00:00".to_string()));
    // Selection should not change
    assert_eq!(state.selected_device, Some(device2.address.to_string()));
}

#[test]
fn test_scanning_state() {
    let mut state = AppState::default();
    assert!(!state.is_scanning);
    
    // Start scanning
    let _ = state.update(Message::StartScan);
    assert!(state.is_scanning);
    
    // Stop scanning
    let _ = state.update(Message::StopScan);
    assert!(!state.is_scanning);
    
    // ScanStarted message
    let _ = state.update(Message::ScanStarted);
    assert!(state.is_scanning);
    
    // ScanCompleted message
    let _ = state.update(Message::ScanCompleted);
    assert!(!state.is_scanning);
}

#[test]
fn test_start_stop_scan() {
    let mut state = AppState::default();
    
    // Check that we start scanning correctly
    assert!(!state.is_scanning);
    let _ = state.update(Message::StartScan);
    assert!(state.is_scanning);
    
    // Check that we stop scanning correctly
    let _ = state.update(Message::StopScan);
    assert!(!state.is_scanning);
    
    // Check auto scan toggling
    let _ = state.update(Message::ToggleAutoScan(false));
    assert!(!state.auto_scan);
    
    let _ = state.update(Message::ToggleAutoScan(true));
    assert!(state.auto_scan);
    
    // First make sure we're visible
    let _ = state.update(Message::ToggleVisibility);
    assert!(state.visible);
    
    // Now check visibility toggling
    let _ = state.update(Message::ToggleVisibility);
    assert!(!state.visible);
} 