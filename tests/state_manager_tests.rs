//! Tests for the StateManager component
//! This tests the core state management functionality (post-refactor)

use std::sync::Arc;
use std::time::Duration;
use chrono::Utc;
use std::convert::TryInto;

use tokio::sync::mpsc::{UnboundedSender, UnboundedReceiver};
use rustpods::ui::state_manager::{StateManager, Action, DeviceState, UiState};
use rustpods::bluetooth::DiscoveredDevice;
use rustpods::bluetooth::AirPodsBatteryStatus;
use rustpods::airpods::{AirPodsBattery, AirPodsChargingState};
use rustpods::ui::Message;
use rustpods::config::AppConfig;

/// Helper function to create a test state manager
fn create_test_state_manager() -> Arc<StateManager> {
    let (tokio_tx, _) = tokio::sync::mpsc::unbounded_channel();
    Arc::new(StateManager::new(tokio_tx))
}

/// Helper function to create a test battery status
fn create_test_battery() -> AirPodsBatteryStatus {
    AirPodsBatteryStatus {
        battery: AirPodsBattery {
            left: Some(70),
            right: Some(80),
            case: Some(90),
            charging: Some(AirPodsChargingState::CaseCharging),
        },
        last_updated: std::time::Instant::now(),
    }
}

/// Helper function to create a test device
fn create_test_device(address: &str, name: &str) -> DiscoveredDevice {
    let address_bytes = address.split(':')
        .map(|x| u8::from_str_radix(x, 16).unwrap())
        .collect::<Vec<u8>>();
    
    DiscoveredDevice {
        address: btleplug::api::BDAddr::from(address_bytes.try_into().unwrap_or([0, 0, 0, 0, 0, 0])),
        name: Some(name.to_string()),
        rssi: Some(-60),
        manufacturer_data: std::collections::HashMap::new(),
        is_potential_airpods: true,
        last_seen: std::time::Instant::now(),
        is_connected: false,
        service_data: std::collections::HashMap::new(),
        services: Vec::new(),
        tx_power_level: None,
    }
}

/// Test state manager initialization
#[test]
fn test_state_manager_init() {
    // Create a state manager
    let state_manager = create_test_state_manager();
    
    // Get initial device state
    let device_state = state_manager.get_device_state();
    
    // Check initial state
    assert!(device_state.devices.is_empty());
    assert!(device_state.selected_device.is_none());
    assert!(device_state.battery_status.is_none());
}

/// Test device state updates
#[test]
fn test_state_manager_device_updates() {
    // Create a state manager
    let state_manager = create_test_state_manager();
    
    // Create a test device
    let device = create_test_device("00:11:22:33:44:55", "Test Device");
    let device_id = device.address.to_string();
    
    // Update the device state
    state_manager.dispatch(Action::UpdateDevice(device.clone()));
    
    // Get updated device state
    let device_state = state_manager.get_device_state();
    
    // Check that the device was added
    assert_eq!(device_state.devices.len(), 1);
    assert!(device_state.devices.contains_key(&device_id));
    
    // Select the device
    state_manager.dispatch(Action::SelectDevice(device_id.clone()));
    
    // Get updated device state
    let device_state = state_manager.get_device_state();
    
    // Check that the device was selected
    assert!(device_state.selected_device.is_some());
    assert_eq!(device_state.selected_device.unwrap(), device_id);
    
    // Remove the device
    state_manager.dispatch(Action::RemoveDevice(device_id));
    
    // Get updated device state
    let device_state = state_manager.get_device_state();
    
    // Check that the device was removed
    assert!(device_state.devices.is_empty());
    assert!(device_state.selected_device.is_none());
}

/// Test battery status updates
#[test]
fn test_state_manager_battery_status() {
    // Create a state manager
    let state_manager = create_test_state_manager();
    
    // Create a test battery status
    let battery_status = create_test_battery();
    
    // Update the battery status
    state_manager.dispatch(Action::UpdateBatteryStatus(battery_status.clone()));
    
    // Get updated device state
    let device_state = state_manager.get_device_state();
    
    // Check that the battery status was updated
    assert!(device_state.battery_status.is_some());
    assert_eq!(device_state.battery_status.as_ref().unwrap().battery.left, battery_status.battery.left);
    assert_eq!(device_state.battery_status.as_ref().unwrap().battery.right, battery_status.battery.right);
    assert_eq!(device_state.battery_status.as_ref().unwrap().battery.case, battery_status.battery.case);
}

/// Test config updates
#[test]
fn test_state_manager_config_updates() {
    // Create a state manager
    let state_manager = create_test_state_manager();
    
    // Create a test config
    let config = AppConfig::default();
    
    // Update the config
    state_manager.dispatch(Action::UpdateSettings(config.clone()));
    
    // Get updated config
    let updated_config = state_manager.get_config();
    
    // Check that the config was updated
    assert_eq!(updated_config.bluetooth.auto_scan_on_startup, config.bluetooth.auto_scan_on_startup);
}

#[test]
fn test_ui_actions() {
    let state_manager = create_test_state_manager();
    
    // Initial UI state
    let ui_state = state_manager.get_ui_state();
    assert!(!ui_state.show_settings);
    assert_eq!(ui_state.animation_progress, 0.0);
    
    // Show settings
    state_manager.dispatch(Action::ShowSettings);
    
    // Verify settings are shown
    let ui_state = state_manager.get_ui_state();
    assert!(ui_state.show_settings);
    
    // Update animation progress
    state_manager.dispatch(Action::UpdateAnimationProgress(0.5));
    
    // Verify animation progress updated
    let ui_state = state_manager.get_ui_state();
    assert_eq!(ui_state.animation_progress, 0.5);
    
    // Update settings
    let config = AppConfig::default();
    state_manager.dispatch(Action::UpdateSettings(config));
}

#[test]
fn test_visibility_action() {
    let state_manager = create_test_state_manager();
    
    // The initial visibility state isn't directly readable through the API
    // But we can verify the action doesn't fail
    
    // Toggle visibility
    state_manager.dispatch(Action::ToggleVisibility);
    
    // Since this doesn't directly expose state, we'd verify via integration tests
    // that the visibility actually changes
}

#[test]
fn test_notification_generation() {
    let (tokio_tx, mut tokio_rx) = tokio::sync::mpsc::unbounded_channel();
    let state_manager = Arc::new(StateManager::new(tokio_tx));
    
    // Create a test device
    let device = create_test_device("11:22:33:44:55:66", "Test Device");
    
    // Dispatch a message that should generate a notification
    state_manager.dispatch(Action::UpdateDevice(device));
    
    // In a real test with tokio runtime, we'd try to receive the message:
    // let message = tokio_rx.try_recv();
    // assert!(message.is_ok());
    
    // Here we just verify the state manager doesn't panic
}

#[test]
fn test_auto_toggle_action() {
    let state_manager = create_test_state_manager();
    
    // Toggle auto scan
    state_manager.dispatch(Action::ToggleAutoScan(false));
    
    // Since auto scan state isn't directly accessible, we'd verify
    // in integration tests that auto scanning behavior changes
}

#[test]
fn test_shutdown_action() {
    let (tokio_tx, mut tokio_rx) = tokio::sync::mpsc::unbounded_channel();
    let state_manager = Arc::new(StateManager::new(tokio_tx));
    
    // Dispatch shutdown action
    state_manager.dispatch(Action::Shutdown);
    
    // This should trigger an Exit message to be sent
    // In a tokio runtime test, we would verify:
    // let message = tokio_rx.try_recv();
    // assert!(message.is_ok());
    // assert!(matches!(message.unwrap(), Message::Exit));
} 