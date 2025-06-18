//! Tests for the AirPodsPopup component
//!
//! This module contains tests for the AirPodsPopup component, which displays
//! battery information for AirPods devices in a popup window.

use iced::Element;
use rustpods::ui::components::AirPodsPopup;
use rustpods::ui::state::{DeviceType, MergedBluetoothDevice};
use rustpods::ui::theme::Theme;
use rustpods::ui::Message;
use rustpods::ui::UiComponent;
use std::time::SystemTime;

#[test]
fn test_airpods_popup_construction() {
    // Create a simple MergedBluetoothDevice for testing
    let device = MergedBluetoothDevice {
        address: "00:11:22:33:44:55".to_string(),
        name: "AirPods Pro".to_string(),
        rssi: Some(-60),
        connected: true,
        left_battery: Some(80),
        right_battery: Some(75),
        case_battery: Some(90),
        left_battery_fractional: Some(0.8),
        right_battery_fractional: Some(0.75),
        case_battery_fractional: Some(0.9),
        paired: true,
        device_type: DeviceType::AirPods,
        battery: Some(80),
        device_subtype: None,
        left_in_ear: None,
        right_in_ear: None,
        case_lid_open: None,
        side: None,
        both_in_case: None,
        color: None,
        switch_count: None,
        is_connected: true,
        last_seen: SystemTime::now(),
        manufacturer_data: Vec::new(),
    };

    // Create the component
    let popup = AirPodsPopup::new(device);

    // Basic assertions
    assert_eq!(popup.device.name, "AirPods Pro");
    assert_eq!(popup.device.left_battery, Some(80));
    assert_eq!(popup.device.right_battery, Some(75));
    assert_eq!(popup.device.case_battery, Some(90));
}

#[test]
fn test_airpods_popup_view_rendering() {
    // Create a simple MergedBluetoothDevice for testing
    let device = MergedBluetoothDevice {
        address: "00:11:22:33:44:55".to_string(),
        name: "AirPods Pro".to_string(),
        rssi: Some(-60),
        connected: true,
        left_battery: Some(80),
        right_battery: Some(75),
        case_battery: Some(90),
        left_battery_fractional: Some(0.8),
        right_battery_fractional: Some(0.75),
        case_battery_fractional: Some(0.9),
        paired: true,
        device_type: DeviceType::AirPods,
        battery: Some(80),
        device_subtype: None,
        left_in_ear: None,
        right_in_ear: None,
        case_lid_open: None,
        side: None,
        both_in_case: None,
        color: None,
        switch_count: None,
        is_connected: true,
        last_seen: SystemTime::now(),
        manufacturer_data: Vec::new(),
    };

    // Create the component
    let popup = AirPodsPopup::new(device);

    // Verify the view method returns an Element
    let _element: Element<Message, iced::Renderer<Theme>> = popup.view();

    // We can't easily test the actual rendering, but we can verify it doesn't panic
    assert!(true);
}

#[test]
fn test_airpods_popup_with_missing_battery_info() {
    // Create a device with missing battery information
    let device = MergedBluetoothDevice {
        address: "00:11:22:33:44:55".to_string(),
        name: "AirPods Pro".to_string(),
        rssi: Some(-60),
        connected: true,
        left_battery: None,
        right_battery: None,
        case_battery: None,
        left_battery_fractional: None,
        right_battery_fractional: None,
        case_battery_fractional: None,
        paired: true,
        device_type: DeviceType::AirPods,
        battery: None,
        device_subtype: None,
        left_in_ear: None,
        right_in_ear: None,
        case_lid_open: None,
        side: None,
        both_in_case: None,
        color: None,
        switch_count: None,
        is_connected: true,
        last_seen: SystemTime::now(),
        manufacturer_data: Vec::new(),
    };

    // Create the component
    let popup = AirPodsPopup::new(device);

    // Verify the view method returns an Element even with missing battery info
    let _element: Element<Message, iced::Renderer<Theme>> = popup.view();

    // We can't easily test the actual rendering, but we can verify it doesn't panic
    assert!(true);
}
