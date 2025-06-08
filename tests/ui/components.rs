//! Integration tests for UI components (post-refactor)
//! Updated for native C++ AirPods battery helper and new state/message model

use std::collections::HashMap;
use std::time::Instant;

use btleplug::api::BDAddr;

use rustpods::bluetooth::DiscoveredDevice;
use rustpods::ui::AppState;

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
fn test_app_state_default() {
    let state = AppState::default();
    // Verify initial state
    assert!(state.devices.is_empty());
    assert_eq!(state.selected_device, None);
}

#[test]
fn test_device_management() {
    let mut state = AppState::default();
    let device = create_test_device([1, 2, 3, 4, 5, 6], "Test Device", -50);
    state.update_device(device);
    assert_eq!(state.devices.len(), 1);
    let addr_str = BDAddr::from([1, 2, 3, 4, 5, 6]).to_string();
    state.select_device(addr_str.clone());
    assert_eq!(state.selected_device, Some(addr_str));
}

#[test]
fn test_app_state_toggle_visibility() {
    let mut state = AppState::default();
    // AppState always starts with visible = true regardless of start_minimized config
    // The start_minimized config is handled at the application startup level
    assert!(state.visible);
    state.toggle_visibility();
    assert!(!state.visible);
    state.toggle_visibility();
    assert!(state.visible);
}

#[test]
fn test_app_state_update_device() {
    let mut state = AppState::default();
    let device = create_test_device([1, 2, 3, 4, 5, 6], "Test Device", -60);
    let addr_str = device.address.to_string();
    state.update_device(device.clone());
    assert_eq!(state.devices.len(), 1);
    assert!(state.devices.contains_key(&addr_str));
    let mut updated_device = device.clone();
    updated_device.name = Some("Updated Name".to_string());
    updated_device.rssi = Some(-50);
    state.update_device(updated_device);
    assert_eq!(state.devices.len(), 1);
    assert_eq!(
        state.devices.get(&addr_str).unwrap().name,
        Some("Updated Name".to_string())
    );
    assert_eq!(state.devices.get(&addr_str).unwrap().rssi, Some(-50));
}

#[test]
fn test_toast_notification_system() {
    // TODO: Implement a test for the toast/notification system
    // Should check creation, display, and close/timeout behavior
    assert!(true); // Stub
}

// Additional UI tests would require more complex setup with mock rendering
// Since Iced's Element is hard to test directly, we focus on state updates
// For full UI testing, consider using a headless browser or similar approach
