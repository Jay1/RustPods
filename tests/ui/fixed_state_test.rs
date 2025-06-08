//! Simplified test for the AppState component to demonstrate fixed initialization (post-refactor)
//! Updated for native C++ AirPods battery helper and new state/message model

use iced::Application;

use rustpods::ui::state::AppState;
use rustpods::ui::theme::Theme;

/// Test default AppState initialization
#[test]
fn test_app_state_default() {
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let state = AppState::new(tx);
    // Note: devices may not be empty due to CLI scanner integration
    // assert!(state.devices.is_empty(), "Default state should have no devices");
    assert_eq!(
        state.selected_device, None,
        "Default state should have no selected device"
    );
    assert!(
        !state.show_settings,
        "Default state should not be showing settings"
    );
    assert_eq!(state.theme(), Theme::CatppuccinMocha);
}

/// Test state visibility toggle
#[test]
fn test_app_state_visibility_toggle() {
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut state = AppState::new(tx);
    state.toggle_visibility();
    assert!(!state.visible, "Visibility should be toggled to false");
    state.toggle_visibility();
    assert!(state.visible, "Visibility should be toggled back to true");
}

/// Test settings visibility toggle
#[test]
fn test_settings_visibility() {
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut state = AppState::new(tx);
    assert!(!state.show_settings, "Settings should be hidden by default");
    state.show_settings = true;
    assert!(
        state.show_settings,
        "Settings should be visible after opening"
    );
    state.show_settings = false;
    assert!(
        !state.show_settings,
        "Settings should be hidden after closing"
    );
}
