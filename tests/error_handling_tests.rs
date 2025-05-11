//! Integration tests for error handling across the application
//!
//! These tests verify that errors are properly propagated, handled, and displayed
//! in the user interface and that the application recovers gracefully from errors.

use rustpods::bluetooth::{BleScanner, BleError};
use rustpods::config::AppConfig;
use rustpods::ui::state::{AppState, Message};
use rustpods::ui::theme::Theme;
use std::path::PathBuf;
use tempfile::tempdir;
use std::fs::File;
use std::io::{self, Write};
use tokio::time::Duration;

/// Test that Bluetooth errors are properly propagated to UI
#[tokio::test]
async fn test_bluetooth_error_propagation() {
    // Create a test state
    let mut state = AppState::default();
    
    // Force scanning when there's no adapter available
    // This assumes BleScanner handles "no adapter" properly
    // If this test runs on a machine with Bluetooth, it might not trigger an error
    
    // The message should be processed without panicking
    state.update(Message::StartScanning);
    
    // Proper handling: state should not be left in an inconsistent state
    // Even if scanning failed due to lack of Bluetooth adapter
    
    // Check that a subsequent stop scanning message doesn't cause issues
    state.update(Message::StopScanning);
    
    // Application should remain functional
    assert!(!state.should_exit, "Application should not exit after scan error");
}

/// Test that configuration file errors are handled gracefully
#[test]
fn test_config_file_errors() {
    // Create a temporary directory for the test
    let temp_dir = tempdir().expect("Failed to create temp directory");
    
    // Test case 1: Invalid JSON in config file
    let invalid_config_path = temp_dir.path().join("invalid_config.json");
    {
        let mut file = File::create(&invalid_config_path).unwrap();
        // Write invalid JSON
        file.write_all(b"{this is not valid JSON}").unwrap();
    }
    
    // Loading the invalid config should not panic, should return an error or default
    let result = AppConfig::load(&invalid_config_path);
    assert!(result.is_err(), "Loading invalid JSON should return an error");
    
    // Test case 2: Load or default should return default for invalid files
    let config = AppConfig::load_or_default(&invalid_config_path);
    assert_eq!(config.ui.theme, AppConfig::default().ui.theme, 
               "Should return default config for invalid file");
    
    // Test case 3: Permission errors
    // (This test is OS-specific and may not work on all platforms)
    if cfg!(unix) {
        // Create a directory where a file is expected
        let config_dir_path = temp_dir.path().join("config_dir.json");
        std::fs::create_dir(&config_dir_path).unwrap_or_default();
        
        // Attempt to save to a path that can't be written to
        let config = AppConfig::default();
        let result = config.save(&config_dir_path);
        assert!(result.is_err(), "Saving to invalid path should return an error");
    }
}

/// Test Bluetooth scanner error handling
#[tokio::test]
async fn test_bluetooth_scanner_errors() {
    // Test for error when starting scanning twice (already running)
    match BleScanner::new().await {
        Ok(mut scanner) => {
            // Start scanning
            let _rx1 = scanner.start_scanning().await.unwrap();
            
            // Should return specific error when starting again
            let result = scanner.start_scanning().await;
            assert!(result.is_err(), "Starting scanner twice should error");
            
            // The error should be a specific type
            match result {
                Err(BleError::ScanInProgress) => {
                    // This is the expected error type
                },
                Err(e) => {
                    panic!("Wrong error type: {:?}, expected ScanInProgress", e);
                },
                Ok(_) => {
                    panic!("Should not succeed when starting scanner twice");
                }
            }
            
            // Clean up
            let _ = scanner.stop_scanning().await;
        },
        Err(_) => {
            // Skip test if no Bluetooth adapter (common in CI environments)
            println!("Skipping test_bluetooth_scanner_errors - no adapter");
        }
    }
}

/// Test error recovery during scanning
#[tokio::test]
async fn test_error_recovery_during_scanning() {
    // Create state with default settings
    let mut state = AppState::default();
    
    // Start scanning
    state.update(Message::StartScanning);
    assert!(state.is_scanning, "Scanning state should be true after StartScanning");
    
    // Simulate an error during scanning by sending scan stopped event
    // without actually stopping the scan properly
    state.update(Message::BluetoothEvent(rustpods::bluetooth::BleEvent::ScanStopped));
    
    // Verify state is recovered
    assert!(!state.is_scanning, "Scanning state should be reset after error");
    
    // Verify we can start scanning again
    state.update(Message::StartScanning);
    assert!(state.is_scanning, "Should be able to restart scanning after error recovery");
}

/// Test UI state error handling
#[test]
fn test_ui_state_error_handling() {
    // Create a test state
    let mut state = AppState::default();
    
    // Test invalid theme handling
    let invalid_theme = "NonExistentTheme";
    
    // Store current theme
    let original_theme = state.theme();
    
    // Try to set an invalid theme
    // This uses a Message that might not exist - adjust as needed
    // The app should internally call Theme::from_string which validates the theme
    state.update(Message::SetTheme(invalid_theme.to_string()));
    
    // Theme should either:
    // 1. Stay the same (if invalid themes are rejected)
    // 2. Be set to default (if invalid themes are normalized)
    // Either is valid behavior, but we shouldn't get an invalid theme
    let current_theme = state.theme();
    
    assert!(current_theme == original_theme || current_theme == Theme::default(),
            "Invalid theme should be handled gracefully");
}

/// Test error handling during save operations
#[test]
fn test_save_error_handling() {
    // Create a configuration
    let config = AppConfig::default();
    
    // Try to save to an invalid location
    let result = config.save(&PathBuf::from("/invalid/path/that/doesnt/exist/config.json"));
    
    // Should return an error, not panic
    assert!(result.is_err(), "Saving to invalid path should return error");
    
    // The error should be an IO error
    match result {
        Err(e) => {
            assert!(e.to_string().contains("No such file or directory") || 
                    e.to_string().contains("The system cannot find the path specified"),
                    "Error should be a file system error: {}", e);
        },
        Ok(_) => {
            panic!("Save to invalid path should not succeed");
        }
    }
} 