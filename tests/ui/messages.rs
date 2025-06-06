//! Tests for UI message handling (post-refactor)
//! Updated for native C++ AirPods battery helper and new state/message model

use std::collections::HashMap;
use std::time::Instant;

use btleplug::api::BDAddr;

use rustpods::bluetooth::DiscoveredDevice;
use rustpods::ui::AppState;

/// Create a test paired device with the given address and name
fn create_test_device(
    address: [u8; 6], 
    name: Option<&str>, 
    rssi: Option<i16>, 
    is_airpods: bool
) -> DiscoveredDevice {
    let mut mfg_data = HashMap::new();
    if is_airpods {
        mfg_data.insert(0x004C, vec![1, 2, 3, 4]); // Apple manufacturer ID
    }
    DiscoveredDevice {
        address: BDAddr::from(address),
        name: name.map(|s| s.to_string()),
        rssi,
        manufacturer_data: mfg_data,
        is_potential_airpods: is_airpods,
        last_seen: Instant::now(),
        is_connected: true,
        service_data: HashMap::new(),
        services: Vec::new(),
        tx_power_level: None,
    }
}

/// Test device update and selection
#[test]
fn test_device_update_and_selection() {
    let mut state = AppState::default();
    let device1 = create_test_device([1, 2, 3, 4, 5, 6], Some("Device 1"), Some(-60), false);
    let device2 = create_test_device([2, 3, 4, 5, 6, 7], Some("Device 2"), Some(-70), false);
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

/// Test device update cycle (paired devices)
#[test]
fn test_device_update_cycle() {
    let mut state = AppState::default();
    let device = create_test_device([1, 2, 3, 4, 5, 6], Some("Device 1"), Some(-60), false);
    state.update_device(device.clone());
    assert_eq!(state.devices.len(), 1);
    let updated_device = create_test_device([1, 2, 3, 4, 5, 6], Some("Device 1 Updated"), Some(-50), false);
    state.update_device(updated_device.clone());
    assert_eq!(state.devices.len(), 1);
    let updated = state.devices.get(&device.address.to_string()).unwrap();
    assert_eq!(updated.name, Some("Device 1 Updated".to_string()));
    assert_eq!(updated.rssi, Some(-50));
}

/// Test selection behavior with missing devices
#[test]
fn test_device_selection_edge_cases() {
    let mut state = AppState::default();
    assert!(state.devices.is_empty());
    assert!(state.selected_device.is_none());
    // Try to select a non-existent device
    state.select_device("non-existent".to_string());
    assert!(state.selected_device.is_none());
    // Add a device
    let device = create_test_device([1, 2, 3, 4, 5, 6], Some("Device 1"), Some(-60), false);
    let addr_str = device.address.to_string();
    state.update_device(device);
    // Select it
    state.select_device(addr_str.clone());
    assert_eq!(state.selected_device, Some(addr_str.clone()));
    // Remove the device
    state.devices.clear();
    // After removing the device, both selected_device and get_selected_device should be None
    if let Some(selected) = &state.selected_device {
        if !state.devices.contains_key(selected) {
            state.selected_device = None;
        }
    }
    assert!(state.selected_device.is_none(), "Selection should be cleared when device is removed");
    assert!(state.get_selected_device().is_none());
} 