//! Tests for the system tray integration with app state
//! These tests verify the functionality of the system tray

use std::sync::Arc;

use rustpods::ui::test_helpers::{
    MockSystemTray, create_test_battery, create_test_state_manager, create_test_config
};
use rustpods::ui::Message;
use rustpods::config::AppConfig;
use rustpods::bluetooth::AirPodsBatteryStatus;

/// Test system tray update with battery status
#[test]
fn test_system_tray_battery_update() {
    // Create test components
    let mut tray = MockSystemTray::new();
    
    // Create a test battery status
    let battery = create_test_battery();
    
    // Update the tray with the battery status
    tray.update_battery(battery.clone());
    
    // Verify the tray was updated correctly
    assert!(tray.battery_status.is_some());
    assert_eq!(tray.battery_status.as_ref().unwrap().battery.left, battery.battery.left);
    assert_eq!(tray.battery_status.as_ref().unwrap().battery.right, battery.battery.right);
    assert_eq!(tray.battery_status.as_ref().unwrap().battery.case, battery.battery.case);
    
    // Verify the action was recorded
    assert!(tray.actions.contains(&"update_battery".to_string()));
}

/// Test system tray menu item clicks
#[test]
fn test_system_tray_menu_clicks() {
    // Create test components
    let mut tray = MockSystemTray::new();
    
    // Test the "Show" menu item
    let message = tray.process_click("Show");
    assert!(message.is_some());
    assert!(matches!(message.unwrap(), Message::ShowWindow));
    
    // Test the "Settings" menu item
    let message = tray.process_click("Settings");
    assert!(message.is_some());
    assert!(matches!(message.unwrap(), Message::ShowSettings));
    
    // Test the "Exit" menu item
    let message = tray.process_click("Exit");
    assert!(message.is_some());
    assert!(matches!(message.unwrap(), Message::Exit));
    
    // Test an unknown menu item
    let message = tray.process_click("Unknown");
    assert!(message.is_none());
    
    // Verify actions were recorded
    assert_eq!(tray.actions.len(), 4);
    assert_eq!(tray.actions[0], "click: Show");
    assert_eq!(tray.actions[1], "click: Settings");
    assert_eq!(tray.actions[2], "click: Exit");
    assert_eq!(tray.actions[3], "click: Unknown");
}

/// Test system tray connection status updates
#[test]
fn test_system_tray_connection_update() {
    // Create test components
    let mut tray = MockSystemTray::new();
    
    // Verify initial state
    assert!(!tray.is_connected);
    
    // Update connection status to true
    tray.update_connection(true);
    assert!(tray.is_connected);
    
    // Update connection status to false
    tray.update_connection(false);
    assert!(!tray.is_connected);
    
    // Verify actions were recorded
    assert_eq!(tray.actions.len(), 2);
    assert_eq!(tray.actions[0], "update_connection: true");
    assert_eq!(tray.actions[1], "update_connection: false");
} 