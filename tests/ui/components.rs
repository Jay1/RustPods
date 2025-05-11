//! Integration tests for UI components

use std::collections::HashMap;
use std::time::Instant;

use btleplug::api::BDAddr;
use iced::Sandbox;

use rustpods::ui::AppState;
use rustpods::ui::Message;
use rustpods::bluetooth::DiscoveredDevice;
use rustpods::airpods::{
    DetectedAirPods, AirPodsType, AirPodsBattery, ChargingStatus
};

/// Test that the AppState default implementation works correctly
#[test]
fn test_app_state_default() {
    let state = AppState::default();
    
    // Verify initial state
    assert!(state.devices.is_empty());
    assert_eq!(state.selected_device, None);
    assert!(!state.is_scanning);
    assert!(state.auto_scan);
}

/// Test that we can add and retrieve devices
#[test]
fn test_device_management() {
    let mut state = AppState::default();
    
    // Create a test device
    let device = DiscoveredDevice {
        address: BDAddr::from([1, 2, 3, 4, 5, 6]),
        name: Some("Test Device".to_string()),
        rssi: Some(-50),
        manufacturer_data: HashMap::new(),
        is_potential_airpods: false,
        last_seen: Instant::now(),
    };
    
    // Add the device
    state.update_device(device);
    
    // Check device was added
    assert_eq!(state.devices.len(), 1);
    
    // Select the device
    let addr_str = BDAddr::from([1, 2, 3, 4, 5, 6]).to_string();
    state.select_device(addr_str.clone());
    
    // Check device was selected
    assert_eq!(state.selected_device, Some(addr_str));
}

/// Test message handling
#[test]
fn test_message_handling() {
    let mut state = AppState::default();
    
    // Test toggle scanning message
    state.is_scanning = false;
    state.update(Message::StartScan);
    assert!(state.is_scanning);
    
    state.update(Message::StopScan);
    assert!(!state.is_scanning);
    
    // Test toggle auto-scan
    state.auto_scan = false;
    state.update(Message::ToggleAutoScan(true));
    assert!(state.auto_scan);
    
    state.update(Message::ToggleAutoScan(false));
    assert!(!state.auto_scan);
    
    // Test device selection
    let device = DiscoveredDevice {
        address: BDAddr::from([1, 2, 3, 4, 5, 6]),
        name: Some("Test Device".to_string()),
        rssi: Some(-50),
        manufacturer_data: HashMap::new(),
        is_potential_airpods: false,
        last_seen: Instant::now(),
    };
    
    state.update_device(device);
    
    let addr_str = BDAddr::from([1, 2, 3, 4, 5, 6]).to_string();
    state.update(Message::SelectDevice(addr_str.clone()));
    
    assert_eq!(state.selected_device, Some(addr_str));
}

/// Helper to create a test device
fn create_test_device(
    address: [u8; 6],
    name: Option<&str>,
    rssi: Option<i16>,
    is_airpods: bool
) -> DiscoveredDevice {
    DiscoveredDevice {
        address: BDAddr::from(address),
        name: name.map(|s| s.to_string()),
        rssi,
        manufacturer_data: HashMap::new(),
        is_potential_airpods: is_airpods,
        last_seen: Instant::now(),
    }
}

/// Helper to create test AirPods
fn create_test_airpods(
    address: [u8; 6],
    name: Option<&str>,
    airpods_type: AirPodsType,
    battery_left: Option<u8>,
    battery_right: Option<u8>,
    battery_case: Option<u8>
) -> DetectedAirPods {
    DetectedAirPods {
        address: BDAddr::from(address),
        name: name.map(|s| s.to_string()),
        device_type: airpods_type,
        battery: AirPodsBattery {
            left: battery_left,
            right: battery_right,
            case: battery_case,
            charging: ChargingStatus {
                left: false,
                right: false,
                case: true,
            },
        },
        rssi: Some(-60),
        raw_data: vec![1, 2, 3, 4, 5],
    }
}

#[test]
fn test_app_state_toggle_visibility() {
    let mut state = AppState::default();
    assert!(!state.visible); // Default is false
    
    state.toggle_visibility();
    assert!(state.visible); // Now true after toggle
    
    state.toggle_visibility();
    assert!(!state.visible); // Now false again
}

#[test]
fn test_app_state_update_device() {
    let mut state = AppState::default();
    let device = create_test_device([1, 2, 3, 4, 5, 6], Some("Test Device"), Some(-60), false);
    
    // Store the address string for comparisons
    let addr_str = device.address.to_string();
    
    // Add the device
    state.update_device(device.clone());
    
    // Check if the device was added
    assert_eq!(state.devices.len(), 1);
    assert!(state.devices.contains_key(&addr_str));
    
    // Update the device with different properties
    let updated_device = create_test_device([1, 2, 3, 4, 5, 6], Some("Updated Name"), Some(-50), true);
    state.update_device(updated_device);
    
    // Check if the device was updated
    assert_eq!(state.devices.len(), 1);
    assert_eq!(state.devices.get(&addr_str).unwrap().name, Some("Updated Name".to_string()));
    assert_eq!(state.devices.get(&addr_str).unwrap().rssi, Some(-50));
    assert!(state.devices.get(&addr_str).unwrap().is_potential_airpods);
}

// Additional UI tests would require more complex setup with mock rendering
// Since Iced's Element is hard to test directly, we focus on state updates
// For full UI testing, consider using a headless browser or similar approach 