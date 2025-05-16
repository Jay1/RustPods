//! Simplified test for the AppState component to demonstrate fixed initialization

use std::sync::Arc;
use iced::Application;

use rustpods::ui::state::AppState;
use rustpods::ui::Message;
use rustpods::ui::theme::Theme;
use rustpods::ui::state_manager::StateManager;

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
    
    // Check theme is set correctly via the theme() method
    assert_eq!(state.theme(), Theme::CatppuccinMocha);
}

/// Test state visibility toggle
#[test]
fn test_app_state_visibility_toggle() {
    // Create a test state manager
    let state_manager = test_helpers::create_test_state_manager();
    
    // Initialize AppState
    let (mut state, _) = AppState::new(state_manager);
    
    // Toggle visibility
    state.update(Message::ToggleVisibility);
    assert!(!state.visible, "Visibility should be toggled to false");
    
    // Toggle again
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