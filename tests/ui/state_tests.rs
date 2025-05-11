//! Integration tests for UI state management

use rustpods::ui::state::{AppState, Message};
use rustpods::ui::theme::Theme;
use rustpods::bluetooth::BleEvent;
use rustpods::airpods::{DetectedAirPods, AirPodsType, AirPodsBattery};
use btleplug::api::BDAddr;
use iced::Command;

/// Test default AppState initialization
#[test]
fn test_app_state_default() {
    let state = AppState::default();
    
    // Verify default values
    assert!(!state.visible, "Default state should not be visible");
    assert!(!state.is_scanning, "Default state should not be scanning");
    assert!(state.auto_scan, "Default state should have auto_scan enabled");
    assert!(state.devices.is_empty(), "Default state should have no devices");
    assert_eq!(state.selected_device, None, "Default state should have no selected device");
    assert!(!state.show_settings, "Default state should not be showing settings");
    assert_eq!(state.theme(), Theme::default(), "Default state should use default theme");
}

/// Test state visibility toggle
#[test]
fn test_app_state_visibility_toggle() {
    let mut state = AppState::default();
    
    // Toggle visibility
    let result = state.update(Message::ToggleVisibility);
    
    // Check state was updated
    assert!(state.visible, "Visibility should be toggled to true");
    
    // Check that no command was returned
    match result {
        Command::none() => {}, // Expected
        _ => panic!("ToggleVisibility should return Command::none()"),
    }
    
    // Toggle again
    state.update(Message::ToggleVisibility);
    
    // Check toggled back
    assert!(!state.visible, "Visibility should be toggled back to false");
}

/// Test scanning state management
#[test]
fn test_scanning_state() {
    let mut state = AppState::default();
    
    // Start scanning
    state.update(Message::StartScanning);
    assert!(state.is_scanning, "State should reflect scanning in progress");
    
    // Process scan started event
    state.update(Message::BluetoothEvent(BleEvent::ScanStarted));
    assert!(state.is_scanning, "State should still be scanning after ScanStarted event");
    
    // Stop scanning
    state.update(Message::StopScanning);
    // Note: is_scanning should only be set to false after receiving ScanStopped event
    
    // Process scan stopped event
    state.update(Message::BluetoothEvent(BleEvent::ScanStopped));
    assert!(!state.is_scanning, "State should not be scanning after ScanStopped event");
}

/// Test device discovery and selection
#[test]
fn test_device_discovery_and_selection() {
    let mut state = AppState::default();
    
    // Create a test device
    let device = create_test_airpods();
    let device_address = device.address;
    
    // Process device discovered event
    state.update(Message::BluetoothEvent(BleEvent::DeviceDiscovered(device.clone())));
    
    // Verify device was added
    assert_eq!(state.devices.len(), 1, "State should contain one device");
    assert!(state.devices.contains_key(&device_address), "Device should be in the state's devices map");
    
    // Select device
    state.update(Message::SelectDevice(Some(device_address)));
    
    // Verify selection
    assert_eq!(state.selected_device, Some(device_address), "Device should be selected");
    
    // Deselect
    state.update(Message::SelectDevice(None));
    
    // Verify deselection
    assert_eq!(state.selected_device, None, "No device should be selected");
}

/// Test battery update handling
#[test]
fn test_battery_update_handling() {
    let mut state = AppState::default();
    
    // Create a test device
    let mut device = create_test_airpods();
    let device_address = device.address;
    
    // Add device to state
    state.update(Message::BluetoothEvent(BleEvent::DeviceDiscovered(device.clone())));
    
    // Update the device battery
    device.battery.left = Some(80);
    device.battery.right = Some(85);
    device.battery.case = Some(95);
    
    // Process battery update
    state.update(Message::UpdateDevice(device.clone()));
    
    // Verify device was updated
    if let Some(updated_device) = state.devices.get(&device_address) {
        assert_eq!(updated_device.battery.left, Some(80), "Left battery should be updated");
        assert_eq!(updated_device.battery.right, Some(85), "Right battery should be updated");
        assert_eq!(updated_device.battery.case, Some(95), "Case battery should be updated");
    } else {
        panic!("Device should exist in state");
    }
}

/// Test settings visibility toggle
#[test]
fn test_settings_visibility() {
    let mut state = AppState::default();
    
    // Initially settings should be hidden
    assert!(!state.show_settings, "Settings should be hidden by default");
    
    // Show settings
    state.update(Message::ToggleSettings);
    assert!(state.show_settings, "Settings should be visible after toggle");
    
    // Hide settings
    state.update(Message::ToggleSettings);
    assert!(!state.show_settings, "Settings should be hidden after second toggle");
}

/// Test theme change
#[test]
fn test_theme_change() {
    let mut state = AppState::default();
    let initial_theme = state.theme();
    
    // Find a different theme
    let new_theme = if initial_theme == Theme::CatppuccinMocha {
        Theme::TokyoNight
    } else {
        Theme::CatppuccinMocha
    };
    
    // Change theme
    state.update(Message::ChangeTheme(new_theme));
    
    // Verify theme changed
    assert_eq!(state.theme(), new_theme, "Theme should be updated");
    assert_ne!(state.theme(), initial_theme, "Theme should be different from initial");
}

/// Test device connection state
#[test]
fn test_device_connection() {
    let mut state = AppState::default();
    
    // Create a test device
    let device = create_test_airpods();
    let device_address = device.address;
    
    // Add device to state
    state.update(Message::BluetoothEvent(BleEvent::DeviceDiscovered(device.clone())));
    
    // Connect to device
    state.update(Message::ConnectToDevice(device_address));
    
    // Process connection event
    state.update(Message::BluetoothEvent(BleEvent::DeviceConnected(device_address)));
    
    // Verify connection state
    if let Some(updated_device) = state.devices.get(&device_address) {
        // The actual state depends on how you're tracking connection status
        // This is a placeholder assertion
        assert!(updated_device.rssi.is_some(), "Device should have RSSI information");
    } else {
        panic!("Device should exist in state");
    }
    
    // Disconnect
    state.update(Message::DisconnectFromDevice(device_address));
    
    // Process disconnection event
    state.update(Message::BluetoothEvent(BleEvent::DeviceDisconnected(device_address)));
}

/// Test exit message
#[test]
fn test_exit_message() {
    let mut state = AppState::default();
    
    // Verify initial state
    assert!(!state.should_exit, "Application should not be exiting initially");
    
    // Request exit
    state.update(Message::Exit);
    
    // Verify exit state
    assert!(state.should_exit, "Application should be exiting after Exit message");
}

/// Test helper to create a test AirPods device
fn create_test_airpods() -> DetectedAirPods {
    DetectedAirPods {
        address: BDAddr::from([0x11, 0x22, 0x33, 0x44, 0x55, 0x66]),
        name: Some("Test AirPods".to_string()),
        device_type: AirPodsType::AirPodsPro,
        battery: AirPodsBattery {
            left: Some(70),
            right: Some(70),
            case: None,
            charging: false,
        },
        rssi: Some(-60),
        raw_data: vec![0x01, 0x02, 0x03],
    }
} 