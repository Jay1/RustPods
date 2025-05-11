//! Integration tests for UI components and state

use btleplug::api::BDAddr;
use std::collections::HashMap;
use std::time::Instant;

use iced::Sandbox;

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
        address: airpods_addr,
        name: Some("AirPods Pro".to_string()),
        rssi: Some(-55),
        manufacturer_data: mfg_data.clone(),
        is_potential_airpods: true,
        last_seen: Instant::now(),
    };
    
    // Create a regular Bluetooth device
    let bt_addr = BDAddr::from([6, 5, 4, 3, 2, 1]);
    let bt_addr_str = bt_addr.to_string();
    let bt_device = DiscoveredDevice {
        address: bt_addr,
        name: Some("Regular BT Device".to_string()),
        rssi: Some(-70),
        manufacturer_data: HashMap::new(),
        is_potential_airpods: false,
        last_seen: Instant::now(),
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
        address: airpods_addr,
        name: Some("AirPods Pro".to_string()),
        rssi: Some(-40), // Better signal now
        manufacturer_data: mfg_data,
        is_potential_airpods: true,
        last_seen: Instant::now(),
    };
    
    state.update_device(updated_airpods);
    assert_eq!(state.devices.len(), 2); // Still just 2 devices
    
    // Verify the update worked
    let updated_device = state.devices.get(&airpods_addr_str).unwrap();
    assert_eq!(updated_device.rssi, Some(-40));
    
    // Verify the selected device also reflects the update
    let selected = state.get_selected_device().unwrap();
    assert_eq!(selected.rssi, Some(-40));
}

/// Test visibility toggling affects the state correctly
#[test]
fn test_visibility_toggle() {
    let mut state = AppState::default();
    assert!(state.visible); // Default is visible for integration testing
    
    // Toggle visibility
    state.toggle_visibility();
    assert!(!state.visible);
    
    // Toggle back
    state.toggle_visibility();
    assert!(state.visible);
}

/// Test that state handles messages correctly
#[test]
fn test_message_handling() {
    // Create a state instance
    let mut state = AppState::default();
    
    // Test StartScan message
    state.update(Message::StartScan);
    assert!(state.is_scanning);
    
    // Test StopScan message
    state.update(Message::StopScan);
    assert!(!state.is_scanning);
    
    // Test ToggleAutoScan message
    state.update(Message::ToggleAutoScan(false));
    assert!(!state.auto_scan);
    
    state.update(Message::ToggleAutoScan(true));
    assert!(state.auto_scan);
    
    // Test ToggleVisibility message
    let initial_visibility = state.visible;
    state.update(Message::ToggleVisibility);
    assert_ne!(state.visible, initial_visibility);
}

/// Test that default config is used with default state
#[test]
fn test_default_config() {
    let state = AppState::default();
    
    // Verify default config values
    let default_config = AppConfig::default();
    assert_eq!(state.config.scan_duration, default_config.scan_duration);
    assert_eq!(state.config.scan_interval, default_config.scan_interval);
    assert_eq!(state.config.auto_scan_on_startup, default_config.auto_scan_on_startup);
} 