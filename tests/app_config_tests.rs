//! Integration tests for the AppConfig functionality

use std::path::PathBuf;
use std::time::Duration;

use rustpods::config::AppConfig;
use rustpods::config::Theme;
use rustpods::bluetooth::ScanConfig;

/// Test default AppConfig settings
#[test]
fn test_default_config() {
    let config = AppConfig::default();
    
    // Check default values
    assert!(config.auto_scan_on_startup);
    assert_eq!(config.scan_duration, Duration::from_secs(5));
    assert_eq!(config.scan_interval, Duration::from_secs(30));
    assert_eq!(config.min_rssi, Some(-70));
    assert!(config.show_notifications);
    assert!(config.start_minimized);
    
    // Verify the path points to a valid location
    let path = &config.settings_path;
    assert!(path.to_str().is_some(), "Path should be convertible to a string");
    
    // For Windows, expect a path that includes AppData
    #[cfg(target_os = "windows")]
    assert!(path.to_str().unwrap().contains("config") || 
            path.to_str().unwrap().contains("Config") ||
            path.to_str().unwrap().contains("AppData"),
            "Windows path should contain config or AppData directory");
    
    // For Unix systems, should include .config
    #[cfg(target_family = "unix")]
    assert!(path.to_str().unwrap().contains(".config"), 
            "Unix path should contain .config directory");
}

/// Test conversion to ScanConfig
#[test]
fn test_to_scan_config() {
    // Test with default settings
    let config = AppConfig::default();
    let scan_config = config.to_scan_config();
    
    assert_eq!(scan_config.scan_duration, config.scan_duration);
    assert_eq!(scan_config.interval_between_scans, config.scan_interval);
    assert!(scan_config.active_scanning); // Should be true by default
    assert_eq!(scan_config.min_rssi, config.min_rssi);
    
    // Test with custom settings
    let custom_config = AppConfig {
        scan_duration: Duration::from_secs(10),
        scan_interval: Duration::from_secs(60),
        min_rssi: Some(-60),
        ..AppConfig::default()
    };
    
    let custom_scan_config = custom_config.to_scan_config();
    
    assert_eq!(custom_scan_config.scan_duration, Duration::from_secs(10));
    assert_eq!(custom_scan_config.interval_between_scans, Duration::from_secs(60));
    assert_eq!(custom_scan_config.min_rssi, Some(-60));
}

/// Test the load and save operations (without actually touching the filesystem)
#[test]
fn test_load_save_operations() {
    // Test load (currently returns default)
    let loaded_config = AppConfig::load().expect("Load should succeed");
    assert_eq!(loaded_config.scan_duration, Duration::from_secs(5));
    
    // Test save (currently no-op but should return Ok)
    let config = AppConfig::default();
    let save_result = config.save();
    assert!(save_result.is_ok(), "Save should succeed");
}

/// Test creating custom configs
#[test]
fn test_custom_config() {
    // Create a fully custom config
    let custom_config = AppConfig {
        auto_scan_on_startup: false,
        scan_duration: Duration::from_secs(15),
        scan_interval: Duration::from_secs(45),
        min_rssi: Some(-55),
        show_notifications: false,
        start_minimized: false,
        theme: Theme::Dark,
        settings_path: PathBuf::from("/custom/path/settings.json"),
    };
    
    // Verify all custom values
    assert!(!custom_config.auto_scan_on_startup);
    assert_eq!(custom_config.scan_duration, Duration::from_secs(15));
    assert_eq!(custom_config.scan_interval, Duration::from_secs(45));
    assert_eq!(custom_config.min_rssi, Some(-55));
    assert!(!custom_config.show_notifications);
    assert!(!custom_config.start_minimized);
    assert_eq!(custom_config.theme, Theme::Dark);
    assert_eq!(custom_config.settings_path, PathBuf::from("/custom/path/settings.json"));
    
    // Test converting to ScanConfig
    let scan_config = custom_config.to_scan_config();
    assert_eq!(scan_config.scan_duration, Duration::from_secs(15));
} 