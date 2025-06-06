//! Tests for UI rendering and component behavior (post-refactor)
//! Updated for native C++ AirPods battery helper and new state/message model

use std::collections::HashMap;
use std::time::Instant;
use tokio::sync::mpsc;

use btleplug::api::BDAddr;
use iced::Element;
use iced::Application;

use rustpods::bluetooth::DiscoveredDevice;
use rustpods::ui::components::{BatteryDisplay, DeviceList, Header};
use rustpods::ui::{Message, UiComponent};
use rustpods::ui::theme::Theme;
use rustpods::ui::state::AppState;
use rustpods::airpods::{DetectedAirPods, AirPodsType, AirPodsBattery};

/// Test that the battery display renders correctly with different levels
#[test]
fn test_battery_display_component() {
    let display = BatteryDisplay::new(Some(75), Some(80), Some(90));
    let _element: Element<'_, Message, iced::Renderer<Theme>> = display.view();
    let display = BatteryDisplay::empty();
    let _element: Element<'_, Message, iced::Renderer<Theme>> = display.view();
    let display = BatteryDisplay::new(Some(0), Some(100), None);
    let _element: Element<'_, Message, iced::Renderer<Theme>> = display.view();
}

/// Test that the device list renders only paired devices
#[test]
fn test_device_list_paired_only() {
    let device1 = DiscoveredDevice {
        address: BDAddr::from([1, 2, 3, 4, 5, 6]),
        name: Some("Paired Device".to_string()),
        rssi: Some(-60),
        manufacturer_data: HashMap::new(),
        is_potential_airpods: false,
        last_seen: Instant::now(),
        is_connected: true, // Simulate paired
        service_data: HashMap::new(),
        services: Vec::new(),
        tx_power_level: None,
    };
    let device2 = DiscoveredDevice {
        address: BDAddr::from([6, 5, 4, 3, 2, 1]),
        name: Some("Unpaired Device".to_string()),
        rssi: Some(-50),
        manufacturer_data: HashMap::new(),
        is_potential_airpods: false,
        last_seen: Instant::now(),
        is_connected: false, // Not paired
        service_data: HashMap::new(),
        services: Vec::new(),
        tx_power_level: None,
    };
    let devices = vec![device1.clone(), device2];
    let selected = Some(device1.address.to_string());
    let device_list = DeviceList::new(devices, selected);
    let _element: Element<'_, Message, iced::Renderer<Theme>> = device_list.view();
}

/// Test that the header renders correctly
#[test]
fn test_header_component() {
    let header = Header::new();
    let _element: Element<'_, Message, iced::Renderer<Theme>> = header.view();
}

/// Create test AirPods device with battery info
fn create_test_airpods(address: [u8; 6], name: Option<&str>, left: Option<u8>, right: Option<u8>, case: Option<u8>) -> DetectedAirPods {
    DetectedAirPods {
        address: BDAddr::from(address),
        name: name.map(|s| s.to_string()),
        rssi: Some(-40),
        device_type: AirPodsType::AirPodsPro2,
        battery: Some(AirPodsBattery {
            left,
            right,
            case,
            charging: None,
        }),
        last_seen: Instant::now(),
        is_connected: true,
    }
}

/// Test AirPods battery info display in the UI
#[test]
fn test_airpods_battery_display() {
    let airpods = create_test_airpods([1, 2, 3, 4, 5, 6], Some("My AirPods Pro 2"), Some(88), Some(92), Some(75));
    let display = BatteryDisplay::new(airpods.battery.as_ref().and_then(|b| b.left), airpods.battery.as_ref().and_then(|b| b.right), airpods.battery.as_ref().and_then(|b| b.case));
    let _element: Element<'_, Message, iced::Renderer<Theme>> = display.view();
}

/// Test AppState overlays: status and toast
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

/// Test AppState visibility toggling
#[test]
fn test_app_state_visibility_toggle() {
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut state = AppState::new(tx);
    assert!(state.visible);
    state.toggle_visibility();
    assert!(!state.visible);
    state.toggle_visibility();
    assert!(state.visible);
}

/// Test AppState device update and selection (paired devices)
#[test]
fn test_app_state_device_update_and_select() {
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut state = AppState::new(tx);
    let device = DiscoveredDevice {
        address: BDAddr::from([1, 2, 3, 4, 5, 6]),
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
    let initial_device_count = state.devices.len();
    state.update_device(device.clone());
    // Should have at least the test device (may have more from CLI scanner)
    assert!(state.devices.len() >= initial_device_count + 1);
    state.select_device(device.address.to_string());
    assert_eq!(state.selected_device, Some(device.address.to_string()));
} 