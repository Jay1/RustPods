//! Integration tests for UI state management

use rustpods::ui::state::AppState;
use rustpods::ui::Message;
use rustpods::ui::theme::Theme;
use rustpods::bluetooth::{BleEvent, DiscoveredDevice, AirPodsBatteryStatus};
use rustpods::airpods::{DetectedAirPods, AirPodsType, AirPodsBattery, AirPodsChargingState};
use rustpods::ui::state_manager::{ConnectionState, StateManager};
use btleplug::api::BDAddr;
use iced::{Command, Application};
use std::sync::Arc;
use std::time::Instant;
use std::collections::HashMap;

use crate::test_helpers;

/// Test default AppState initialization
#[test]
fn test_app_state_default() {
    // Create a test state manager
    let state_manager = test_helpers::create_test_state_manager();
    
    // Initialize AppState
    let (state, _) = AppState::new(state_manager);
    
    // Verify default values
    assert!(!state.is_scanning, "Default state should not be scanning");
    assert!(state.auto_scan, "Default state should have auto_scan enabled");
    assert!(state.devices.is_empty(), "Default state should have no devices");
    assert_eq!(state.selected_device, None, "Default state should have no selected device");
    assert!(!state.show_settings, "Default state should not be showing settings");
    // Theme is set in the theme() method, not in a field
}

/// Test state visibility toggle
#[test]
fn test_app_state_visibility_toggle() {
    // Create a test state manager
    let state_manager = test_helpers::create_test_state_manager();
    
    // Initialize AppState
    let (mut state, _) = AppState::new(state_manager);
    
    // AppState defaults to visible true, so first toggle it to false
    state.update(Message::ToggleVisibility);
    assert!(!state.visible, "Visibility should be toggled to false");
    
    // Toggle back to true
    state.update(Message::ToggleVisibility);
    assert!(state.visible, "Visibility should be toggled back to true");
}

/// Test scanning state management
#[test]
fn test_scanning_state() {
    // Create a test state manager
    let state_manager = test_helpers::create_test_state_manager();
    
    // Initialize AppState
    let (mut state, _) = AppState::new(state_manager);
    
    // Start scanning
    state.update(Message::StartScan);
    assert!(state.is_scanning, "State should reflect scanning in progress");
    
    // Process scan started event
    state.update(Message::ScanStarted);
    assert!(state.is_scanning, "State should still be scanning after ScanStarted event");
    
    // Stop scanning
    state.update(Message::StopScan);
    
    // Process scan stopped event
    state.update(Message::ScanStopped);
    assert!(!state.is_scanning, "State should not be scanning after ScanStopped event");
}

/// Test device discovery and selection
#[test]
fn test_device_discovery_and_selection() {
    // Create a test state manager
    let state_manager = test_helpers::create_test_state_manager();
    
    // Initialize AppState
    let (mut state, _) = AppState::new(state_manager);
    
    // Create a test device
    let device = create_test_airpods();
    // Convert to string for use in app state
    let device_address = device.address.to_string();
    
    // Create a DiscoveredDevice for the test
    let discovered_device = DiscoveredDevice {
        address: device.address,
        name: device.name.clone(),
        rssi: device.rssi,
        manufacturer_data: HashMap::new(),
        is_potential_airpods: true,
        last_seen: std::time::Instant::now(),
        is_connected: false,
        service_data: HashMap::new(),
        services: vec![],
        tx_power_level: None,
    };
    
    // Process device discovered event
    state.update(Message::DeviceDiscovered(discovered_device.clone()));
    
    // Verify device was added
    assert_eq!(state.devices.len(), 1, "State should contain one device");
    assert!(state.devices.contains_key(&device_address), "Device should be in the state's devices map");
    
    // Select device
    state.update(Message::SelectDevice(device_address.clone()));
    
    // Verify selection
    assert_eq!(state.selected_device, Some(device_address.clone()), "Device should be selected");
    
    // Deselect by passing empty string
    state.update(Message::SelectDevice(String::new()));
    
    // Verify deselection
    assert_eq!(state.selected_device, None, "No device should be selected");
}

/// Test battery update handling
#[test]
fn test_battery_update_handling() {
    // Create a test state manager
    let state_manager = test_helpers::create_test_state_manager();
    
    // Initialize AppState
    let (mut state, _) = AppState::new(state_manager);
    
    // Create a test device with battery
    let mut device = create_test_airpods();
    let device_address = device.address.to_string();
    
    // Create a DiscoveredDevice for the test
    let discovered_device = DiscoveredDevice {
        address: device.address,
        name: device.name.clone(),
        rssi: device.rssi,
        manufacturer_data: HashMap::new(),
        is_potential_airpods: true,
        last_seen: std::time::Instant::now(),
        is_connected: false,
        service_data: HashMap::new(),
        services: vec![],
        tx_power_level: None,
    };
    
    // Add device to state
    state.update(Message::DeviceDiscovered(discovered_device.clone()));
    
    // Select the device first so that MainWindow will be updated
    state.update(Message::SelectDevice(device_address.clone()));
    
    // Create updated device with new battery info
    if let Some(battery) = &mut device.battery {
        battery.left = Some(80);
        battery.right = Some(85);
        battery.case = Some(95);
    }
    
    // Create battery status object
    let battery_status = AirPodsBatteryStatus::new(device.battery.clone().unwrap_or_default());
    
    // Update battery status directly
    state.update(Message::BatteryStatusUpdated(battery_status.clone()));
    
    // Verify that the battery status was set
    assert!(state.battery_status.is_some(), "Battery status should be set");
    if let Some(status) = &state.battery_status {
        // Compare values in the status
        if let Some(left) = status.battery.left {
            assert_eq!(left, 80, "Left earbud battery level should be 80%");
        }
        if let Some(right) = status.battery.right {
            assert_eq!(right, 85, "Right earbud battery level should be 85%");
        }
        if let Some(case) = status.battery.case {
            assert_eq!(case, 95, "Case battery level should be 95%");
        }
    }
}

/// Test settings visibility toggle
#[test]
fn test_settings_visibility() {
    // Create a test state manager
    let state_manager = test_helpers::create_test_state_manager();
    
    // Initialize AppState
    let (mut state, _) = AppState::new(state_manager);
    
    // Initially settings should be hidden
    assert!(!state.show_settings, "Settings should be hidden by default");
    
    // Show settings
    state.update(Message::OpenSettings);
    assert!(state.show_settings, "Settings should be visible after opening");
    
    // Hide settings
    state.update(Message::CloseSettings);
    assert!(!state.show_settings, "Settings should be hidden after closing");
}

/// Test theme-related functionality
#[test]
fn test_theme_handling() {
    // Create a test state manager
    let state_manager = test_helpers::create_test_state_manager();
    
    // Initialize AppState
    let (state, _) = AppState::new(state_manager);
    
    // Test theme() method returns the expected default theme
    assert_eq!(state.theme(), Theme::CatppuccinMocha);
}

/// Test device connection state
#[test]
fn test_device_connection() {
    // Create a test state manager
    let state_manager = test_helpers::create_test_state_manager();
    
    // Initialize AppState
    let (mut state, _) = AppState::new(state_manager);
    
    // Create a test device
    let device = create_test_airpods();
    let device_address = device.address.to_string();
    
    // Create a DiscoveredDevice for the test
    let discovered_device = DiscoveredDevice {
        address: device.address,
        name: device.name.clone(),
        rssi: device.rssi,
        manufacturer_data: HashMap::new(),
        is_potential_airpods: true,
        last_seen: std::time::Instant::now(),
        is_connected: false,
        service_data: HashMap::new(),
        services: vec![],
        tx_power_level: None,
    };
    
    // Add device to state
    state.update(Message::DeviceDiscovered(discovered_device.clone()));
    
    // Select the device first
    state.update(Message::SelectDevice(device_address.clone()));
    
    // Verify device was selected
    assert_eq!(state.selected_device, Some(device_address.clone()), "Device should be selected");
    
    // Simulate connection state change
    state.update(Message::ConnectionStateChanged(ConnectionState::Connected));
    
    // Add a disconnection event
    state.update(Message::DeviceDisconnected);
    
    // Reconnection with the device
    state.update(Message::DeviceReconnected(device));
}

/// Test exit message
#[test]
fn test_exit_message() {
    // Note: We can't actually test the exit behavior as it calls process::exit(0)
    // This test is therefore just a stub that verifies we can create such a message
    let _ = Message::Exit;
}

/// Test helper to create a test AirPods device
fn create_test_airpods() -> DetectedAirPods {
    DetectedAirPods::new(
        BDAddr::from([0x11, 0x22, 0x33, 0x44, 0x55, 0x66]),
        Some("Test AirPods".to_string()),
        Some(-60),
        AirPodsType::AirPodsPro,
        Some(AirPodsBattery {
            left: Some(70),
            right: Some(70),
            case: None,
            charging: Some(AirPodsChargingState::NotCharging),
        }),
        false
    )
} 