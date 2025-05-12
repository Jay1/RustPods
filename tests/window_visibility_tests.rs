//! Tests for window visibility management
//! These tests verify the functionality of window visibility components

use std::time::Duration;
use std::sync::mpsc;
use std::sync::Arc;
use iced::{Point, Size, Rectangle};

use rustpods::ui::window_visibility::{WindowVisibilityManager, WindowPosition};
use rustpods::ui::state_manager::StateManager;
use rustpods::ui::Message;
use rustpods::config::AppConfig;
use crate::test_helpers::{MockWindowVisibilityManager, create_test_config};

// Helper function to create test dependencies
fn setup_test_env() -> (Arc<StateManager>, AppConfig) {
    let (tokio_tx, _) = tokio::sync::mpsc::unbounded_channel();
    let state_manager = Arc::new(StateManager::new(tokio_tx));
    let config = create_test_config();
    
    (state_manager, config)
}

#[test]
fn test_window_position_conversion() {
    // Create a test Rectangle
    let rect = Rectangle::new(
        Point::new(100.0, 200.0),
        Size::new(800.0, 600.0),
    );
    
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

#[test]
fn test_window_visibility_manager_creation() {
    let (state_manager, config) = setup_test_env();
    
    // Test with start_minimized = true
    let mut config_minimized = config.clone();
    config_minimized.ui.start_minimized = true;
    
    let manager = WindowVisibilityManager::new(config_minimized);
    assert_eq!(manager.is_visible(), false);
    
    // Test with start_minimized = false
    let mut config_visible = config.clone();
    config_visible.ui.start_minimized = false;
    
    let manager = WindowVisibilityManager::new(config_visible);
    assert_eq!(manager.is_visible(), true);
}

#[test]
fn test_window_visibility_manager_toggle() {
    let (state_manager, config) = setup_test_env();
    let mut manager = WindowVisibilityManager::new(config);
    
    // Get initial state
    let initial_visible = manager.is_visible();
    
    // Create a dummy rectangle for testing
    let rect = Rectangle::new(
        Point::new(100.0, 200.0),
        Size::new(800.0, 600.0),
    );
    
    // Toggle visibility
    manager.toggle(rect);
    
    // Verify it toggled
    assert_eq!(manager.is_visible(), !initial_visible);
    
    // If it was hidden, verify position was saved
    if !initial_visible {
        assert!(manager.last_position().is_some());
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
    assert_eq!(manager.is_visible(), initial_visible);
}

#[test]
fn test_window_visibility_manager_auto_hide() {
    let (state_manager, config) = setup_test_env();
    
    // Create manager with auto-hide
    let mut manager = WindowVisibilityManager::new(config)
        .with_auto_hide_timeout(Duration::from_millis(10));
    
    // Make sure window is visible and focused
    let rect = Rectangle::new(
        Point::new(100.0, 200.0),
        Size::new(800.0, 600.0),
    );
    
    // Show window and focus it
    manager.show();
    manager.handle_focus();
    
    // Should not have an auto-hide command yet
    assert!(manager.update(rect).is_none());
    
    // Now blur the window
    manager.handle_blur();
    
    // Wait for timeout
    std::thread::sleep(Duration::from_millis(20));
    
    // Should get a command to hide now
    assert!(manager.update(rect).is_some());
    
    // Window should be hidden
    assert_eq!(manager.is_visible(), false);
}

#[test]
fn test_window_visibility_manager_position() {
    let (state_manager, config) = setup_test_env();
    let mut manager = WindowVisibilityManager::new(config);
    
    // Initially no position
    assert!(manager.last_position().is_none());
    
    // Set a position
    let position = WindowPosition {
        x: 100.0,
        y: 200.0,
        width: 800.0,
        height: 600.0,
    };
    
    manager.set_position(position);
    
    // Verify position was set
    assert!(manager.last_position().is_some());
    if let Some(pos) = manager.last_position() {
        assert_eq!(pos.x, 100.0);
        assert_eq!(pos.y, 200.0);
        assert_eq!(pos.width, 800.0);
        assert_eq!(pos.height, 600.0);
    }
}

/// Test window visibility toggling
#[test]
fn test_window_visibility_toggle() {
    // Create a mock window visibility manager
    let mut manager = MockWindowVisibilityManager::new();
    
    // Window should start as visible
    assert!(manager.visible);
    
    // Toggle visibility
    manager.toggle();
    assert!(!manager.visible);
    
    // Toggle back
    manager.toggle();
    assert!(manager.visible);
}

/// Test window show/hide
#[test]
fn test_window_visibility_show_hide() {
    // Create a mock window visibility manager
    let mut manager = MockWindowVisibilityManager::new();
    
    // Hide the window
    manager.hide();
    assert!(!manager.visible);
    
    // Show the window
    manager.show();
    assert!(manager.visible);
    
    // Hide again
    manager.hide();
    assert!(!manager.visible);
} 