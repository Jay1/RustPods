//! Integration tests for UI state management

use std::collections::HashMap;
use std::time::Instant;
use std::sync::Arc;

use btleplug::api::BDAddr;

use iced::Application;

use rustpods::ui::{state::AppState, Message};
use rustpods::bluetooth::DiscoveredDevice;
use rustpods::airpods::{
    DetectedAirPods, AirPodsType, AirPodsBattery, ChargingStatus
};
use rustpods::ui::state_manager::StateManager;
use tokio::sync::mpsc::unbounded_channel;

/// Helper to create a test AirPods device
fn create_test_airpods(device_type: AirPodsType, left: Option<u8>, right: Option<u8>, case: Option<u8>) -> DetectedAirPods {
    DetectedAirPods {
        address: BDAddr::from([1, 2, 3, 4, 5, 6]),
        name: Some(format!("Test {:?}", device_type)),
        device_type,
        battery: Some(AirPodsBattery {
            left,
            right,
            case,
            charging: None,
        }),
        rssi: Some(-60),
        last_seen: Instant::now(),
        is_connected: false,
    }
}

/// Helper to create a test device
fn create_test_device(address: [u8; 6], name: &str, rssi: i16) -> DiscoveredDevice {
    DiscoveredDevice {
        address: BDAddr::from(address),
        name: Some(name.to_string()),
        rssi: Some(rssi),
        manufacturer_data: HashMap::new(),
        is_potential_airpods: false,
        last_seen: Instant::now(),
        is_connected: false,
        service_data: HashMap::new(),
        services: Vec::new(),
        tx_power_level: None,
    }
}

#[test]
fn test_app_state_initialization() {
    let app_state = AppState::default();
    
    // Visibility should be the opposite of start_minimized setting
    assert_eq!(app_state.visible, !app_state.config.ui.start_minimized);
    
    // Default state should not be scanning
    assert!(!app_state.is_scanning);
    
    // Default state should have auto scan enabled
    assert!(app_state.auto_scan);
    
    // Should have no devices initially
    assert!(app_state.devices.is_empty());
    
    // Should have no device selected
    assert!(app_state.selected_device.is_none());
}

#[test]
fn test_app_state_visibility() {
    let mut app_state = AppState::default();
    
    // Initial visibility should be opposite of start_minimized
    let initial_visibility = !app_state.config.ui.start_minimized;
    assert_eq!(app_state.visible, initial_visibility);
    
    // Toggle to opposite state
    app_state.toggle_visibility();
    assert_eq!(app_state.visible, !initial_visibility);
    
    // Toggle back to initial state
    app_state.toggle_visibility();
    assert_eq!(app_state.visible, initial_visibility);
}

#[test]
fn test_app_state_update_devices() {
    // Create a new AppState
    let (tx, _rx) = unbounded_channel();
    let (mut app_state, _) = AppState::new(Arc::new(StateManager::new(tx)));
    
    // Create test devices
    let device1 = create_test_device([1, 2, 3, 4, 5, 6], "Device 1", -60);
    let device2 = create_test_device([6, 5, 4, 3, 2, 1], "Device 2", -70);
    
    // Update devices
    app_state.update_device(device1.clone());
    assert_eq!(app_state.devices.len(), 1);
    assert!(app_state.devices.contains_key(&device1.address.to_string()));
    
    app_state.update_device(device2.clone());
    assert_eq!(app_state.devices.len(), 2);
    assert!(app_state.devices.contains_key(&device2.address.to_string()));
    
    // Update existing device
    let mut updated_device1 = device1.clone();
    updated_device1.rssi = Some(-55); // Better signal
    app_state.update_device(updated_device1.clone());
    
    // Should still have 2 devices
    assert_eq!(app_state.devices.len(), 2);
    
    // Check that the device was updated
    let stored_device = app_state.devices.get(&device1.address.to_string()).unwrap();
    assert_eq!(stored_device.rssi, Some(-55));
}

#[test]
fn test_app_state_select_device() {
    // Create a new AppState
    let (tx, _rx) = unbounded_channel();
    let (mut app_state, _) = AppState::new(Arc::new(StateManager::new(tx)));
    
    // Create test device
    let device = create_test_device([1, 2, 3, 4, 5, 6], "Device 1", -60);
    let addr_str = device.address.to_string();
    
    // Update device
    app_state.update_device(device.clone());
    assert_eq!(app_state.devices.len(), 1);
    
    // Select device
    app_state.select_device(addr_str.clone());
    assert_eq!(app_state.selected_device, Some(addr_str));
    
    // Test getting selected device
    let selected_device = app_state.get_selected_device();
    assert!(selected_device.is_some());
    assert_eq!(selected_device.unwrap().address, device.address);
}

#[test]
fn test_app_state_message_handling() {
    // Create a new AppState
    let (tx, _rx) = unbounded_channel();
    let (mut app_state, _) = AppState::new(Arc::new(StateManager::new(tx)));
    
    // Test starting scan message
    let _ = app_state.update(Message::StartScan);
    assert!(app_state.is_scanning);
    
    // Test stop scan message
    let _ = app_state.update(Message::StopScan);
    assert!(!app_state.is_scanning);
    
    // Test device update message
    let device = create_test_device([1, 2, 3, 4, 5, 6], "Test Device", -60);
    let _ = app_state.update(Message::DeviceDiscovered(device.clone()));
    assert_eq!(app_state.devices.len(), 1);
    
    // Test device selection message
    let addr_str = device.address.to_string();
    let _ = app_state.update(Message::SelectDevice(addr_str.clone()));
    assert_eq!(app_state.selected_device, Some(addr_str));
    
    // Test auto scan toggle message
    assert!(app_state.auto_scan); // Default is true
    let _ = app_state.update(Message::ToggleAutoScan(false));
    assert!(!app_state.auto_scan);
} 