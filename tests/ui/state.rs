//! Integration tests for UI state management (post-refactor)
//! Updated for native C++ AirPods battery helper and new state/message model

use std::collections::HashMap;
use std::time::Instant;

use btleplug::api::BDAddr;

use rustpods::ui::state::AppState;
use rustpods::bluetooth::DiscoveredDevice;

/// Helper to create a test device (paired)
fn create_test_device(address: [u8; 6], name: &str, rssi: i16) -> DiscoveredDevice {
    DiscoveredDevice {
        address: BDAddr::from(address),
        name: Some(name.to_string()),
        rssi: Some(rssi),
        manufacturer_data: HashMap::new(),
        is_potential_airpods: false,
        last_seen: Instant::now(),
        is_connected: true,
        service_data: HashMap::new(),
        services: Vec::new(),
        tx_power_level: None,
    }
}

#[test]
fn test_app_state_initialization() {
    let app_state = AppState::default();
    // Application starts visible by default (system tray handles minimized behavior)
    assert_eq!(app_state.visible, true);
    // Should have no devices initially
    assert!(app_state.devices.is_empty());
    // Should have no device selected
    assert!(app_state.selected_device.is_none());
}

#[test]
fn test_app_state_visibility() {
    let mut app_state = AppState::default();
    let initial_visibility = true; // Always starts visible
    assert_eq!(app_state.visible, initial_visibility);
    app_state.toggle_visibility();
    assert_eq!(app_state.visible, !initial_visibility);
    app_state.toggle_visibility();
    assert_eq!(app_state.visible, initial_visibility);
}

#[test]
fn test_app_state_update_devices() {
    let mut app_state = AppState::default();
    let device1 = create_test_device([1, 2, 3, 4, 5, 6], "Device 1", -60);
    let device2 = create_test_device([6, 5, 4, 3, 2, 1], "Device 2", -70);
    app_state.update_device(device1.clone());
    assert_eq!(app_state.devices.len(), 1);
    assert!(app_state.devices.contains_key(&device1.address.to_string()));
    app_state.update_device(device2.clone());
    assert_eq!(app_state.devices.len(), 2);
    assert!(app_state.devices.contains_key(&device2.address.to_string()));
    // Update existing device
    let mut updated_device1 = device1.clone();
    updated_device1.rssi = Some(-55);
    app_state.update_device(updated_device1.clone());
    assert_eq!(app_state.devices.len(), 2);
    let stored_device = app_state.devices.get(&device1.address.to_string()).unwrap();
    assert_eq!(stored_device.rssi, Some(-55));
}

#[test]
fn test_app_state_select_device() {
    let mut app_state = AppState::default();
    let device = create_test_device([1, 2, 3, 4, 5, 6], "Device 1", -60);
    let addr_str = device.address.to_string();
    app_state.update_device(device.clone());
    assert_eq!(app_state.devices.len(), 1);
    app_state.select_device(addr_str.clone());
    assert_eq!(app_state.selected_device, Some(addr_str));
    let selected_device = app_state.get_selected_device();
    assert!(selected_device.is_some());
    assert_eq!(selected_device.unwrap().address, device.address);
} 