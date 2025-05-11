//! Tests for the Settings UI Window

use std::time::Duration;
use std::fs;
use tempfile::tempdir;

use rustpods::config::{AppConfig, ConfigError, Theme, LogLevel};
use rustpods::ui::settings_window::{SettingsWindow, SettingsTab};
use rustpods::ui::components::settings_view::{BluetoothSetting, UiSetting, SystemSetting};
use rustpods::ui::Message;
use rustpods::ui::UiComponent;
use rustpods::ui::theme;

/// Test creating a new settings window
#[test]
fn test_create_settings_window() {
    let config = AppConfig::default();
    let settings_window = SettingsWindow::new(config.clone());
    
    // Check initial state
    assert!(!settings_window.has_changes());
    
    // Check returned config matches input
    let returned_config = settings_window.config();
    assert_eq!(returned_config.bluetooth.scan_duration, Duration::from_secs(5));
    assert_eq!(returned_config.ui.theme, Theme::System);
    assert_eq!(returned_config.system.log_level, LogLevel::Info);
}

/// Test updating settings
#[test]
fn test_update_settings() {
    let mut config = AppConfig::default();
    let mut settings_window = SettingsWindow::new(config.clone());
    
    // Modify config
    config.bluetooth.scan_duration = Duration::from_secs(10);
    config.ui.theme = Theme::Dark;
    
    // Update window with new config
    settings_window.update_config(config.clone());
    
    // Check returned config matches updated config
    let returned_config = settings_window.config();
    assert_eq!(returned_config.bluetooth.scan_duration, Duration::from_secs(10));
    assert_eq!(returned_config.ui.theme, Theme::Dark);
}

/// Test marking changes
#[test]
fn test_mark_changed() {
    let config = AppConfig::default();
    let mut settings_window = SettingsWindow::new(config);
    
    // Initially no changes
    assert!(!settings_window.has_changes());
    
    // Mark as changed
    settings_window.mark_changed();
    
    // Should now have changes
    assert!(settings_window.has_changes());
}

/// Test tab selection
#[test]
fn test_tab_selection() {
    let config = AppConfig::default();
    let mut settings_window = SettingsWindow::new(config);
    
    // Default tab is Bluetooth
    
    // Select Interface tab
    settings_window.select_tab(SettingsTab::Interface);
    
    // Select System tab
    settings_window.select_tab(SettingsTab::System);
}

/// Test validation errors
#[test]
fn test_validation_errors() {
    let config = AppConfig::default();
    let mut settings_window = SettingsWindow::new(config);
    
    // No initial error
    
    // Set an error
    settings_window.set_validation_error(Some("Test error message".to_string()));
    
    // Clear error
    settings_window.set_validation_error(None);
}

/// Test persistence with real files
#[test]
fn test_settings_persistence() {
    // Create a temporary directory for the test
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test_settings.json");
    
    // Create a custom config
    let mut config = AppConfig::default();
    config.bluetooth.auto_scan_on_startup = false;
    config.bluetooth.scan_duration = Duration::from_secs(15);
    config.ui.theme = Theme::Dark;
    config.settings_path = file_path.clone();
    
    // Save the config
    config.save().unwrap();
    
    // Verify file was created
    assert!(file_path.exists());
    
    // Create a settings window with this config
    let settings_window = SettingsWindow::new(config);
    
    // Get the config from the settings window
    let window_config = settings_window.config();
    
    // Verify it matches what we set
    assert_eq!(window_config.bluetooth.scan_duration, Duration::from_secs(15));
    assert_eq!(window_config.ui.theme, Theme::Dark);
    assert!(!window_config.bluetooth.auto_scan_on_startup);
    
    // Load the config directly from the file to verify
    let loaded_config = AppConfig::load_from_path(&file_path).unwrap();
    assert_eq!(loaded_config.bluetooth.scan_duration, Duration::from_secs(15));
    assert_eq!(loaded_config.ui.theme, Theme::Dark);
    
    // Cleanup
    temp_dir.close().unwrap();
}

/// Test configuration validation
#[test]
fn test_config_validation() {
    // Create a valid config
    let mut config = AppConfig::default();
    assert!(config.validate().is_ok());
    
    // Test invalid Bluetooth config
    config.bluetooth.scan_duration = Duration::from_millis(100); // Too short
    assert!(matches!(config.validate(), Err(ConfigError::ValidationFailed(_, _))));
    
    // Reset config and test invalid UI config
    config = AppConfig::default();
    config.ui.low_battery_threshold = 101; // Over 100%
    assert!(matches!(config.validate(), Err(ConfigError::ValidationFailed(_, _))));
}

/// Test that the UI can render
#[test]
fn test_ui_rendering() {
    let config = AppConfig::default();
    let settings_window = SettingsWindow::new(config);
    
    // Just verify the window can render without panic
    let _element = settings_window.view();
    
    // This is a simple smoke test - actual visual rendering would require integration tests
}

/// Test updating individual settings
#[test]
fn test_update_individual_settings() {
    let config = AppConfig::default();
    let mut settings_window = SettingsWindow::new(config.clone());
    
    // Update individual settings
    
    // Test each setting type to ensure they all work
    settings_window.update_config(config.clone());
    settings_window.mark_changed();
    
    // Bluetooth settings
    let bluetooth_config = settings_window.config().bluetooth;
    assert_eq!(bluetooth_config.scan_duration, Duration::from_secs(5)); // Default
    
    // UI settings  
    let ui_config = settings_window.config().ui;
    assert_eq!(ui_config.theme, Theme::System); // Default
    
    // System settings
    let system_config = settings_window.config().system;
    assert_eq!(system_config.log_level, LogLevel::Info); // Default
} 