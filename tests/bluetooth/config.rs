//! Integration tests for Bluetooth scanner configuration

use std::time::Duration;
use rustpods::bluetooth::ScanConfig;

#[test]
fn test_scan_config_default() {
    let config = ScanConfig::default();
    
    assert_eq!(config.scan_duration, Duration::from_secs(10));
    assert_eq!(config.interval_between_scans, Duration::from_secs(20));
    assert!(config.auto_stop_scan);
    assert_eq!(config.max_scan_cycles, None);
    assert!(config.maintain_device_history);
    assert!(config.active_scanning);
    assert_eq!(config.min_rssi, None);
}

#[test]
fn test_scan_config_continuous() {
    let config = ScanConfig::continuous();
    
    assert_eq!(config.scan_duration, Duration::from_secs(5));
    assert_eq!(config.interval_between_scans, Duration::from_secs(2));
    assert!(config.auto_stop_scan);
    assert_eq!(config.max_scan_cycles, None);
    assert!(config.maintain_device_history);
    assert!(config.active_scanning);
    assert_eq!(config.min_rssi, None);
}

#[test]
fn test_scan_config_power_efficient() {
    let config = ScanConfig::power_efficient();
    
    assert_eq!(config.scan_duration, Duration::from_secs(3));
    assert_eq!(config.interval_between_scans, Duration::from_secs(60));
    assert!(config.auto_stop_scan);
    assert_eq!(config.max_scan_cycles, None);
    assert!(config.maintain_device_history);
    assert!(!config.active_scanning);
    assert_eq!(config.min_rssi, Some(-80));
}

#[test]
fn test_scan_config_airpods_optimized() {
    let config = ScanConfig::airpods_optimized();
    
    assert_eq!(config.scan_duration, Duration::from_secs(5));
    assert_eq!(config.interval_between_scans, Duration::from_secs(10));
    assert!(config.auto_stop_scan);
    assert_eq!(config.max_scan_cycles, None);
    assert!(config.maintain_device_history);
    assert!(config.active_scanning);
    assert_eq!(config.min_rssi, Some(-70));
}

#[test]
fn test_scan_config_one_time_scan() {
    let duration = Duration::from_secs(15);
    let config = ScanConfig::one_time_scan(duration);
    
    assert_eq!(config.scan_duration, duration);
    assert_eq!(config.interval_between_scans, Duration::from_secs(0));
    assert!(config.auto_stop_scan);
    assert_eq!(config.max_scan_cycles, Some(1));
    assert!(!config.maintain_device_history);
    assert!(config.active_scanning);
    assert_eq!(config.min_rssi, None);
}

#[test]
fn test_scan_config_builder_pattern() {
    let config = ScanConfig::new()
        .with_scan_duration(Duration::from_secs(30))
        .with_interval(Duration::from_secs(45))
        .with_auto_stop(false)
        .with_max_cycles(Some(5))
        .with_history(false)
        .with_active_scanning(false)
        .with_min_rssi(Some(-65));
    
    assert_eq!(config.scan_duration, Duration::from_secs(30));
    assert_eq!(config.interval_between_scans, Duration::from_secs(45));
    assert!(!config.auto_stop_scan);
    assert_eq!(config.max_scan_cycles, Some(5));
    assert!(!config.maintain_device_history);
    assert!(!config.active_scanning);
    assert_eq!(config.min_rssi, Some(-65));
}

#[test]
fn test_scan_config_chained_builders() {
    // Test that we can combine different builders
    let config = ScanConfig::airpods_optimized()
        .with_max_cycles(Some(3))
        .with_history(false);
    
    // Should keep airpods_optimized values except for the ones we changed
    assert_eq!(config.scan_duration, Duration::from_secs(5));
    assert_eq!(config.interval_between_scans, Duration::from_secs(10));
    assert!(config.auto_stop_scan);
    assert_eq!(config.max_scan_cycles, Some(3)); // Changed
    assert!(!config.maintain_device_history); // Changed
    assert!(config.active_scanning);
    assert_eq!(config.min_rssi, Some(-70));
}

#[test]
fn test_scan_config_conversions() {
    // Test that we can convert to and from other types if needed
    // For example, test conversion from app config (if applicable)
    
    // Create a config with specific values
    let config = ScanConfig::new()
        .with_scan_duration(Duration::from_secs(10))
        .with_min_rssi(Some(-75))
        .with_active_scanning(true);
    
    // Verify all values
    assert_eq!(config.scan_duration, Duration::from_secs(10));
    assert_eq!(config.min_rssi, Some(-75));
    assert!(config.active_scanning);
} 