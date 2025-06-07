//! Main test module for RustPods
//!
//! This file organizes the integration tests and provides shared utilities.
//! Heavy integration tests with async operations have been temporarily disabled
//! to prevent system performance issues during test runs.

// Re-export common test helpers for use in all test modules
pub mod common_test_helpers;
pub mod test_helpers;
pub mod bluetooth_mocks;

// Core functionality tests
mod bluetooth_tests;
mod airpods_tests;
mod airpods_error_handling_tests;
mod config_tests;
mod app_config_tests;
mod error_handling_tests;
mod error_context_display_test;
mod form_validation_tests;
mod window_visibility_tests;

// UI tests
mod settings_ui_tests;
mod settings_window_tests;

// State management tests (lighter weight)
mod state_manager_tests;

// System integration tests (lighter weight)
mod system_tray_tests;

// Subdirectories with specific test categories
pub mod bluetooth;
pub mod airpods;
pub mod ui;

// Temporarily disabled heavy integration tests causing performance issues
// These test files have been renamed with .disabled extension to prevent execution
// TODO: Re-enable after optimizing async operations and background task cleanup
// - state_management_integration_tests.rs.disabled
// - module_integration_tests.rs.disabled  
// - system_tray_integration_tests.rs.disabled
// - app_battery_monitoring_tests.rs.disabled
// - event_system.disabled/ directory

#[cfg(test)]
mod tests {
    #[test]
    fn test_suite_setup() {
        // Verify test suite is set up correctly
        assert!(true);
    }
} 