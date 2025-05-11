// Re-export common test helpers for use in all test modules
pub mod common_test_helpers;

// Domain-specific test modules
pub mod bluetooth;
pub mod event_system;
pub mod ui;

// Standalone test modules for top-level components
pub mod app_config_tests;
pub mod system_tray_tests; 