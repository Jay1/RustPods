//! Tests for window visibility management
//! These tests verify the functionality of window visibility components

use std::time::Duration;
use std::sync::mpsc;
use std::sync::Arc;
use iced::{Point, Size, Rectangle, Event, window, Command};

use rustpods::ui::window_visibility::{WindowVisibilityManager, WindowPosition};
use rustpods::ui::state_manager::{StateManager, Action};
use rustpods::ui::Message;
use rustpods::config::AppConfig;
use rustpods::ui::system_tray::{SystemTray, SystemTrayError};
use crate::test_helpers::{MockWindowVisibilityManager, MockSystemTray, create_test_config};

// SECTION: Helper Functions and Test Setup

/// Helper function to create test dependencies
fn setup_test_env() -> (Arc<StateManager>, AppConfig) {
    let (tokio_tx, _) = tokio::sync::mpsc::unbounded_channel();
    let state_manager = Arc::new(StateManager::new(tokio_tx));
    let config = create_test_config();
    
    (state_manager, config)
}

/// Helper function to create a test rectangle
fn create_test_rectangle() -> Rectangle {
    Rectangle::new(
        Point::new(100.0, 200.0),
        Size::new(800.0, 600.0),
    )
}

// SECTION: Basic Window Position Tests

#[test]
fn test_window_position_conversion() {
    // Create a test Rectangle
    let rect = create_test_rectangle();
    
    // Convert to WindowPosition
    let position = WindowPosition::from(rect);
    
    // Verify values
    assert_eq!(position.x, 100.0);
    assert_eq!(position.y, 200.0);
    assert_eq!(position.width, 800.0);
    assert_eq!(position.height, 600.0);
    
    // Convert back to Rectangle
    let rect2: Rectangle = position.into();
    
    // Verify back conversion
    assert_eq!(rect2.x, 100.0);
    assert_eq!(rect2.y, 200.0);
    assert_eq!(rect2.width, 800.0);
    assert_eq!(rect2.height, 600.0);
}

// SECTION: Window Visibility Manager Tests

#[test]
fn test_window_visibility_manager_creation() {
    let (state_manager, config) = setup_test_env();
    
    // Test with start_minimized = true
    let mut config_minimized = config.clone();
    config_minimized.ui.start_minimized = true;
    
    let manager = WindowVisibilityManager::new(config_minimized);
    assert_eq!(manager.is_visible(), false, "Window should start hidden when start_minimized is true");
    
    // Test with start_minimized = false
    let mut config_visible = config.clone();
    config_visible.ui.start_minimized = false;
    
    let manager = WindowVisibilityManager::new(config_visible);
    assert_eq!(manager.is_visible(), true, "Window should start visible when start_minimized is false");
}

#[test]
fn test_window_visibility_manager_toggle() {
    let (state_manager, config) = setup_test_env();
    let mut manager = WindowVisibilityManager::new(config);
    
    // Get initial state
    let initial_visible = manager.is_visible();
    
    // Create a dummy rectangle for testing
    let rect = create_test_rectangle();
    
    // Toggle visibility
    manager.toggle(rect);
    
    // Verify it toggled
    assert_eq!(manager.is_visible(), !initial_visible, "Window visibility should be toggled");
    
    // If it was hidden, verify position was saved
    if !initial_visible {
        assert!(manager.last_position().is_some(), "Position should be saved when showing window");
        if let Some(pos) = manager.last_position() {
            assert_eq!(pos.x, 100.0);
            assert_eq!(pos.y, 200.0);
            assert_eq!(pos.width, 800.0);
            assert_eq!(pos.height, 600.0);
        }
    }
    
    // Toggle back
    manager.toggle(rect);
    
    // Verify it toggled back
    assert_eq!(manager.is_visible(), initial_visible, "Window visibility should be toggled back");
}

// SECTION: Window Position Memory Tests

#[test]
fn test_window_position_memory() {
    let (state_manager, config) = setup_test_env();
    let mut manager = WindowVisibilityManager::new(config);
    
    // Initially no position is stored
    assert!(manager.last_position().is_none(), "Initially no position should be stored");
    
    // Create dummy rectangles for position testing
    let position1 = WindowPosition {
        x: 100.0,
        y: 200.0,
        width: 800.0,
        height: 600.0,
    };
    
    let position2 = WindowPosition {
        x: 200.0,
        y: 300.0,
        width: 900.0,
        height: 700.0,
    };
    
    // Set position and verify it's saved
    manager.set_position(position1);
    assert!(manager.last_position().is_some(), "Position should be saved");
    assert_eq!(
        manager.last_position().unwrap(),
        position1,
        "Saved position should match the set position"
    );
    
    // Hide window with a different position
    let rect2 = Rectangle::new(
        Point::new(position2.x, position2.y),
        Size::new(position2.width, position2.height),
    );
    
    manager.hide(rect2);
    
    // Verify position is updated
    assert!(manager.last_position().is_some(), "Position should be saved after hiding");
    assert_eq!(
        manager.last_position().unwrap(),
        position2,
        "Position should be updated after hiding with new position"
    );
    
    // Show window (would use last position in real app)
    manager.show();
    
    // Verify still visible and position retained
    assert!(manager.is_visible(), "Window should be visible after show");
    assert_eq!(
        manager.last_position().unwrap(),
        position2,
        "Last position should be retained after showing"
    );
}

// SECTION: Focus and Blur Event Tests

#[test]
fn test_focus_blur_events() {
    let (state_manager, config) = setup_test_env();
    let mut manager = WindowVisibilityManager::new(config);
    
    // Initially window should not be focused
    assert!(!manager.is_focused(), "Window should not be initially focused");
    
    // Handle focus event
    manager.handle_focus();
    assert!(manager.is_focused(), "Window should be focused after focus event");
    
    // Handle blur event
    manager.handle_blur();
    assert!(!manager.is_focused(), "Window should not be focused after blur event");
}

// SECTION: Auto-hide Timeout Tests

#[test]
fn test_window_visibility_manager_auto_hide() {
    let (state_manager, config) = setup_test_env();
    
    // Create manager with auto-hide
    let mut manager = WindowVisibilityManager::new(config)
        .with_auto_hide_timeout(Duration::from_millis(10));
    
    // Make sure window is visible and focused
    let rect = create_test_rectangle();
    
    // Show window and focus it
    manager.show();
    manager.handle_focus();
    
    // Should not have an auto-hide command yet
    assert!(manager.update(rect).is_none(), "Should not auto-hide while focused");
    
    // Now blur the window
    manager.handle_blur();
    
    // Wait for timeout
    std::thread::sleep(Duration::from_millis(20));
    
    // Should get a command to hide now
    let command = manager.update(rect);
    assert!(command.is_some(), "Should get auto-hide command after blur and timeout");
    
    // Window should be hidden
    assert_eq!(manager.is_visible(), false, "Window should be hidden after auto-hide timeout");
}

// SECTION: Window Close Event Tests

#[test]
fn test_window_close_handling() {
    let (state_manager, config) = setup_test_env();
    let mut manager = WindowVisibilityManager::new(config);
    
    // Ensure window is visible initially
    manager.show();
    assert!(manager.is_visible(), "Window should be visible initially");
    
    // Get a test rectangle
    let rect = create_test_rectangle();
    
    // Handle close event (should hide to tray, not really close)
    let _command = manager.handle_close_requested(rect);
    
    // Window should be hidden, not closed
    assert!(!manager.is_visible(), "Window should be hidden after close event");
    assert!(manager.last_position().is_some(), "Position should be saved on close");
}

// SECTION: Config Update Tests

#[test]
fn test_config_update() {
    let (state_manager, config) = setup_test_env();
    let mut manager = WindowVisibilityManager::new(config.clone());
    
    // Start with visible window
    manager.show();
    assert!(manager.is_visible(), "Window should be visible initially");
    
    // Update config to prefer minimized
    let mut new_config = config.clone();
    new_config.ui.start_minimized = true;
    manager.update_config(new_config);
    
    // Settings shouldn't immediately change visibility
    assert!(manager.is_visible(), "Updating config shouldn't change current visibility");
    
    // But should affect next startup (which we can't test here directly)
}

// SECTION: System Tray Integration Tests

#[test]
fn test_mock_system_tray_window_control() {
    // Create test components
    let mut tray = MockSystemTray::new();
    
    // Process show window menu click
    let message = tray.process_click("Show");
    assert!(message.is_some(), "Show menu click should produce a message");
    assert!(matches!(message.unwrap(), Message::ShowWindow), "Should be ShowWindow message");
    
    // Process hide window menu click
    let message = tray.process_click("Hide");
    assert!(message.is_some(), "Hide menu click should produce a message");
    assert!(matches!(message.unwrap(), Message::HideWindow), "Should be HideWindow message");
}

#[test]
fn test_integration_with_state_manager() {
    // Set up environment
    let (state_manager, config) = setup_test_env();
    let state_manager_clone = Arc::clone(&state_manager);
    
    // Create visibility manager with state manager
    let mut manager = WindowVisibilityManager::new(config.clone())
        .with_state_manager(state_manager_clone);
    
    // Create a test rectangle
    let rect = create_test_rectangle();
    
    // Toggle visibility (should trigger Action::ToggleVisibility)
    manager.toggle(rect);
    
    // In a real integration test, we'd verify that the state was updated
    // and the appropriate message was sent to the UI, but this is hard to test
    // without a running iced application
}

#[test]
fn test_minimize_to_tray_operations() {
    // Set up environment
    let (state_manager, config) = setup_test_env();
    
    // Create visibility manager
    let mut manager = WindowVisibilityManager::new(config.clone());
    
    // Start with visible window
    manager.show();
    assert!(manager.is_visible(), "Window should be visible initially");
    
    // Create a test rectangle
    let rect = create_test_rectangle();
    
    // Minimize to tray (hide)
    manager.hide(rect);
    
    // Verify state
    assert!(!manager.is_visible(), "Window should be hidden after minimize to tray");
    assert!(manager.last_position().is_some(), "Position should be saved on minimize to tray");
    
    // Restore from tray (show)
    manager.show();
    
    // Verify state
    assert!(manager.is_visible(), "Window should be visible after restore from tray");
}

// SECTION: Window Event Handling Tests

#[test]
fn test_mock_window_visibility_toggle() {
    // Create a mock window visibility manager
    let mut manager = MockWindowVisibilityManager::new();
    
    // Window should start as visible
    assert!(manager.visible, "Mock window should start visible");
    
    // Toggle visibility
    manager.toggle();
    assert!(!manager.visible, "Mock window should be hidden after toggle");
    
    // Toggle back
    manager.toggle();
    assert!(manager.visible, "Mock window should be visible after second toggle");
}

#[test]
fn test_mock_window_position_handling() {
    // Create a mock window visibility manager
    let mut manager = MockWindowVisibilityManager::new();
    
    // Initially no position
    assert!(manager.get_position().is_none(), "Initially should have no position");
    
    // Set a position
    let position = WindowPosition {
        x: 100.0,
        y: 200.0,
        width: 800.0,
        height: 600.0,
    };
    
    manager.set_position(position);
    
    // Verify position was set
    assert!(manager.get_position().is_some(), "Position should be saved");
    let saved_position = manager.get_position().unwrap();
    assert_eq!(saved_position.x, 100.0, "X coordinate should match");
    assert_eq!(saved_position.y, 200.0, "Y coordinate should match");
    assert_eq!(saved_position.width, 800.0, "Width should match");
    assert_eq!(saved_position.height, 600.0, "Height should match");
} 