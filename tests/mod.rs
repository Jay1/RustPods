//! Main test module for RustPods
//!
//! This file organizes the integration tests into subdirectories, each
//! with its own mod.rs file. Each subdirectory contains a set of related tests.
//!
//! This module structure allows for better organization of tests and shared 
//! test utilities across related test cases.

// Re-export common test helpers for use in all test modules
pub mod common_test_helpers;

// Domain-specific test modules
// Each of these represents a subdirectory with multiple test files
pub mod bluetooth;
pub mod event_system;
pub mod ui;
pub mod airpods;

// Standalone test modules for top-level components
pub mod app_config_tests;
pub mod system_tray_tests;
pub mod app_battery_monitoring_tests;
pub mod config_tests;
pub mod settings_ui_tests;

// Note: In Rust's test system, each .rs file in the tests directory 
// is compiled as a separate test binary by default, even when there's a mod.rs file.
// The mod.rs file imports allow us to share utilities and helpers across test binaries. 

//! RustPods Test Suite
//! This file organizes all test modules

// Include common test helpers
mod test_helpers;

// Test modules
mod form_validation_tests;
mod window_visibility_tests;
mod system_tray_tests;
mod system_tray_integration_tests;
mod settings_window_tests;
mod state_manager_tests;
mod app_config_tests;
mod app_battery_monitoring_tests;
mod error_handling_tests;
mod settings_ui_tests;

#[cfg(test)]
mod tests {
    // Global test setup if needed
    #[test]
    fn test_suite_setup() {
        // Verify test suite is set up correctly
        assert!(true);
    }
} 