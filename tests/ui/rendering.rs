//! Tests for UI rendering and component behavior (post-refactor)
//! Updated for native C++ AirPods battery helper and new state/message model

use std::collections::HashMap;
use std::time::Instant;

use btleplug::api::BDAddr;
use iced::Application;

use rustpods::airpods::{AirPodsBattery, AirPodsType, DetectedAirPods};
use rustpods::bluetooth::DiscoveredDevice;
use rustpods::ui::components::{view_circular_battery_widget, battery_icon_display};
use rustpods::ui::state::AppState;
// UI rendering tests

/// Test that the circular battery widget renders correctly with different levels
#[test]
fn test_circular_battery_widget_rendering() {
    let _widget_75 = view_circular_battery_widget(75.0, false);
    let _widget_empty = view_circular_battery_widget(0.0, false);
    let _widget_full = view_circular_battery_widget(100.0, true);
    let _widget_charging = view_circular_battery_widget(50.0, true);
}

/// Test that battery icon display renders correctly
#[test]
fn test_battery_icon_display_rendering() {
    let _icon_display = battery_icon_display(Some(75), false, 80.0, 0.0);
    let _icon_charging = battery_icon_display(Some(50), true, 100.0, 20.0);
    let _icon_low = battery_icon_display(Some(10), false, 60.0, 0.0);
    let _icon_full = battery_icon_display(Some(100), false, 80.0, 0.0);
}

/// Test device filtering for paired devices only
#[test]
fn test_device_filtering_paired_only() {
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
    let paired_devices: Vec<_> = devices.into_iter().filter(|d| d.is_connected).collect();
    
    // Only paired devices should be included
    assert_eq!(paired_devices.len(), 1);
    assert_eq!(paired_devices[0].address, device1.address);
}

/// Create test AirPods device with battery info
fn create_test_airpods(
    address: [u8; 6],
    name: Option<&str>,
    left: Option<u8>,
    right: Option<u8>,
    case: Option<u8>,
) -> DetectedAirPods {
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
fn test_airpods_battery_widget_display() {
    let airpods = create_test_airpods(
        [1, 2, 3, 4, 5, 6],
        Some("My AirPods Pro 2"),
        Some(88),
        Some(92),
        Some(75),
    );
    
    if let Some(battery) = &airpods.battery {
        if let Some(left) = battery.left {
            let _left_widget = view_circular_battery_widget(left as f32, false);
        }
        if let Some(right) = battery.right {
            let _right_widget = view_circular_battery_widget(right as f32, false);
        }
        if let Some(case) = battery.case {
            let _case_widget = view_circular_battery_widget(case as f32, true);
        }
    }
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
    assert!(state.devices.len() > initial_device_count);
    state.select_device(device.address.to_string());
    assert_eq!(state.selected_device, Some(device.address.to_string()));
}

/// Test circular widget rendering with various battery levels
#[test]
fn test_circular_widget_battery_levels() {
    let levels = [0, 25, 50, 75, 100];
    
    for &level in &levels {
        let _widget_normal = view_circular_battery_widget(level as f32, false);
        let _widget_charging = view_circular_battery_widget(level as f32, true);
    }
}

/// Test battery icon rendering with various dimensions
#[test]
fn test_battery_icon_dimensions() {
    let dimensions = [(50.0, 20.0), (80.0, 30.0), (100.0, 40.0)];
    
    for &(width, height) in &dimensions {
        let _icon = battery_icon_display(Some(75), false, width, height);
    }
}

/// Test widget edge cases
#[test]
fn test_widget_edge_cases() {
    // Test zero battery
    let _zero_widget = view_circular_battery_widget(0.0, false);
    let _zero_charging = view_circular_battery_widget(0.0, true);
    
    // Test full battery
    let _full_widget = view_circular_battery_widget(100.0, false);
    let _full_charging = view_circular_battery_widget(100.0, true);
    
    // Test battery icon with zero dimensions
    let _zero_width = battery_icon_display(Some(50), false, 0.0, 20.0);
    let _zero_height = battery_icon_display(Some(50), false, 20.0, 0.0);
}
