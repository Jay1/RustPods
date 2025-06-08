//! Integration tests for UI components and state (post-refactor)
//! Updated for native C++ AirPods battery helper and new state/message model

use btleplug::api::BDAddr;
use std::collections::HashMap;
use std::time::Instant;

use iced::Application;
use rustpods::bluetooth::DiscoveredDevice;
use rustpods::config::AppConfig;
use rustpods::ui::state::AppState;

/// Test the full state update flow with simulated device events (paired devices)
#[test]
fn test_state_device_flow() {
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut state = AppState::new(tx);
    // Note: devices may not be empty due to CLI scanner integration
    // Create a paired device
    let paired_addr = BDAddr::from([1, 2, 3, 4, 5, 6]);
    let paired_addr_str = paired_addr.to_string();
    let paired_device = DiscoveredDevice {
        address: paired_addr,
        name: Some("Paired Device".to_string()),
        rssi: Some(-60),
        manufacturer_data: HashMap::new(),
        is_potential_airpods: false,
        last_seen: Instant::now(),
        is_connected: true,
        service_data: HashMap::new(),
        services: Vec::new(),
        tx_power_level: None,
    };
    // Add device to state
    let initial_device_count = state.devices.len();
    state.update_device(paired_device.clone());
    // Should have at least the test device (may have more from CLI scanner)
    assert!(state.devices.len() > initial_device_count);
    assert!(state.devices.contains_key(&paired_addr_str));
    // Select the paired device
    state.select_device(paired_addr_str.clone());
    assert_eq!(state.selected_device, Some(paired_addr_str.clone()));
    let selected = state.get_selected_device().unwrap();
    assert_eq!(selected.address, paired_addr);
    // Update RSSI for the paired device
    let updated_device = DiscoveredDevice {
        address: paired_addr,
        name: Some("Paired Device".to_string()),
        rssi: Some(-55),
        manufacturer_data: HashMap::new(),
        is_potential_airpods: false,
        last_seen: Instant::now(),
        is_connected: true,
        service_data: HashMap::new(),
        services: Vec::new(),
        tx_power_level: None,
    };
    state.update_device(updated_device);
    // Device count should remain the same (just updated existing device)
    assert!(state.devices.len() > initial_device_count);
    let updated = state.devices.get(&paired_addr_str).unwrap();
    assert_eq!(updated.rssi, Some(-55));
    let selected = state.get_selected_device().unwrap();
    assert_eq!(selected.rssi, Some(-55));
}

/// Test visibility toggling affects the state correctly
#[test]
fn test_visibility_toggle() {
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut state = AppState::new(tx);
    assert!(state.visible); // Default is visible (true)
    state.toggle_visibility();
    assert!(!state.visible);
    state.toggle_visibility();
    assert!(state.visible);
}

/// Test that default config is used with default state
#[test]
fn test_default_config() {
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let state = AppState::new(tx);
    let default_config = AppConfig::default();
    assert_eq!(
        state.config.bluetooth.scan_duration,
        default_config.bluetooth.scan_duration
    );
    assert_eq!(
        state.config.bluetooth.scan_interval,
        default_config.bluetooth.scan_interval
    );
    assert_eq!(
        state.config.bluetooth.auto_scan_on_startup,
        default_config.bluetooth.auto_scan_on_startup
    );
}

/// Create a mock paired device for testing
fn create_test_device(address: &str) -> DiscoveredDevice {
    let addr = if address.contains(':') {
        let bytes: Vec<&str> = address.split(':').collect();
        let mut addr_bytes = [0u8; 6];
        for (i, byte) in bytes.iter().enumerate() {
            addr_bytes[i] = u8::from_str_radix(byte, 16).unwrap_or(0);
        }
        BDAddr::from(addr_bytes)
    } else {
        BDAddr::from([1, 2, 3, 4, 5, 6])
    };
    DiscoveredDevice {
        address: addr,
        name: Some(format!("Test Device {}", address)),
        rssi: Some(-70),
        manufacturer_data: HashMap::new(),
        is_potential_airpods: false,
        last_seen: std::time::Instant::now(),
        is_connected: true,
        service_data: HashMap::new(),
        services: Vec::new(),
        tx_power_level: None,
    }
}

#[test]
fn test_app_state_defaults() {
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let state = AppState::new(tx);
    assert!(state.visible); // Default is visible (true)
                            // Note: devices may not be empty due to CLI scanner integration
    let default_config = AppConfig::default();
    assert_eq!(
        state.config.bluetooth.scan_duration,
        default_config.bluetooth.scan_duration
    );
    assert_eq!(
        state.config.bluetooth.scan_interval,
        default_config.bluetooth.scan_interval
    );
    assert_eq!(
        state.config.bluetooth.auto_scan_on_startup,
        default_config.bluetooth.auto_scan_on_startup
    );
}

#[test]
fn test_device_update_and_selection() {
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut state = AppState::new(tx);
    let device1 = create_test_device("11:22:33:44:55:66");
    let device2 = create_test_device("AA:BB:CC:DD:EE:FF");
    state.update_device(device1.clone());
    state.update_device(device2.clone());
    state.select_device(device1.address.to_string());
    assert_eq!(state.selected_device, Some(device1.address.to_string()));
    state.select_device(device2.address.to_string());
    assert_eq!(state.selected_device, Some(device2.address.to_string()));
    // Try to select non-existent device
    state.select_device("00:00:00:00:00:00".to_string());
    // Selection should not change
    assert_eq!(state.selected_device, Some(device2.address.to_string()));
}

/// Test overlays: status and toast
#[test]
fn test_app_state_status_and_toast() {
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut state = AppState::new(tx);
    state.status_message = Some("Status!".to_string());
    state.toast_message = Some("Toast!".to_string());

    // Test that view works with messages
    {
        let _element = state.view();
    } // Drop the element before mutating state

    state.clear_status_message();
    assert!(state.status_message.is_none());
    state.clear_toast_message();
    assert!(state.toast_message.is_none());
}
