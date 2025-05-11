//! Integration tests for Bluetooth functionality

use std::collections::HashMap;
use std::time::{Duration, Instant};

use btleplug::api::BDAddr;
use tokio::time::timeout;
use futures::StreamExt;
use futures::pin_mut;

use rustpods::bluetooth::{
    BleScanner, ScanConfig, EventFilter, BleEvent, 
    receiver_to_stream, AdapterManager, DiscoveredDevice, events::EventType
};
use rustpods::airpods::{
    create_airpods_filter, create_custom_airpods_filter,
    airpods_all_models_filter, airpods_pro_filter,
    airpods_with_battery_filter, airpods_nearby_filter
};

/// Test helper to create a sample discovered device
fn create_test_device(
    address: [u8; 6], 
    name: Option<&str>, 
    rssi: Option<i16>, 
    is_airpods: bool
) -> DiscoveredDevice {
    let mut manufacturer_data = HashMap::new();
    if is_airpods {
        manufacturer_data.insert(0x004C, vec![
            0x07, 0x19, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C
        ]);
    }
    
    DiscoveredDevice {
        address: BDAddr::from(address),
        name: name.map(|s| s.to_string()),
        rssi,
        manufacturer_data,
        is_potential_airpods: is_airpods,
        last_seen: Instant::now(),
    }
}

/// Test ScanConfig creation and methods
#[tokio::test]
async fn test_scan_config() {
    // Test the default config
    let default_config = ScanConfig::default();
    assert_eq!(default_config.scan_duration, Duration::from_secs(3));
    assert_eq!(default_config.interval_between_scans, Duration::from_secs(2));
    assert!(default_config.auto_stop_scan);
    assert_eq!(default_config.max_scan_cycles, None);
    assert!(default_config.maintain_device_history);
    
    // Test the AirPods optimized config
    let airpods_config = ScanConfig::airpods_optimized();
    assert_eq!(airpods_config.scan_duration, Duration::from_secs(5));
    assert!(airpods_config.active_scanning);
    
    // Test custom configuration
    let custom_config = ScanConfig::new()
        .with_scan_duration(Duration::from_secs(10))
        .with_interval(Duration::from_secs(5))
        .with_max_cycles(Some(3))
        .with_active_scanning(false)
        .with_min_rssi(Some(-70));
    
    assert_eq!(custom_config.scan_duration, Duration::from_secs(10));
    assert_eq!(custom_config.interval_between_scans, Duration::from_secs(5));
    assert_eq!(custom_config.max_scan_cycles, Some(3));
    assert!(!custom_config.active_scanning);
    assert_eq!(custom_config.min_rssi, Some(-70));
}

/// Test scanner subscription methods
#[tokio::test]
async fn test_scanner_subscribers() {
    // Create a scanner with default config
    let mut scanner = BleScanner::new();
    
    // Test subscribing to all events
    let all_rx = scanner.subscribe_all();
    assert!(!all_rx.is_closed());
    
    // Test subscribing with a filter
    let filter = EventFilter::event_types(vec![EventType::DeviceDiscovered]);
    let filtered_rx = scanner.subscribe(filter);
    assert!(!filtered_rx.is_closed());
    
    // We don't actually test event delivery here since that would require
    // starting a scan, which might not be possible in all test environments
}

/// Test adapter discovery (only if Bluetooth is available)
#[tokio::test]
async fn test_adapter_discovery() {
    // Skip the actual Bluetooth operations if running in CI or without adapter
    if skip_bluetooth_test() {
        return;
    }
    
    let adapter_manager_result = AdapterManager::new().await;
    // We don't assert on success since the test might run in an environment without Bluetooth
    if let Ok(adapter_manager) = adapter_manager_result {
        let available_adapters = adapter_manager.get_available_adapters();
        println!("Found {} adapters", available_adapters.len());
        for adapter_info in available_adapters {
            println!("Adapter {}: {} {}", 
                     adapter_info.index, 
                     adapter_info.name, 
                     adapter_info.address.map_or("unknown".to_string(), |a| a.to_string()));
        }
    } else {
        println!("Failed to create adapter manager: {:?}", adapter_manager_result.err());
    }
}

/// Test scanner initialization (without starting scan)
#[tokio::test]
async fn test_scanner_initialization() {
    // Create a scanner
    let scanner = BleScanner::new();
    
    // The scanner should be properly initialized during creation
    // We can't actually test much more without real hardware
    assert!(!scanner.is_scanning());
}

/// Test scanning with timeout (if Bluetooth available)
#[tokio::test]
async fn test_scanner_start_stop() {
    // Skip the actual Bluetooth operations if running in CI or without adapter
    if skip_bluetooth_test() {
        return;
    }
    
    // Create a scanner with a short scan duration
    let config = ScanConfig::one_time_scan(Duration::from_secs(2));
    let mut scanner = BleScanner::with_config(config);
    
    // Try to initialize the scanner (might fail without hardware)
    if let Err(e) = scanner.initialize().await {
        println!("Scanner initialization failed (expected in CI): {:?}", e);
        return;
    }
    
    // Start scanning
    let scan_result = scanner.start_scanning().await;
    if let Err(e) = scan_result {
        println!("Failed to start scanning: {:?}", e);
        return;
    }
    
    // Wait for scan to complete automatically
    tokio::time::sleep(Duration::from_secs(3)).await;
    
    // Scanner should have stopped automatically after scan_duration
    assert!(!scanner.is_scanning());
}

/// Test AirPods filters
#[tokio::test]
async fn test_device_filtering() {
    // Create devices for testing
    let airpods_device = create_test_device(
        [0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
        Some("AirPods"),
        Some(-60),
        true
    );
    
    let regular_device = create_test_device(
        [0x06, 0x05, 0x04, 0x03, 0x02, 0x01],
        Some("Regular Headphones"),
        Some(-70),
        false
    );
    
    // Test the all models filter
    let filter = airpods_all_models_filter();
    let filter_fn = filter.create_filter_function();
    assert!(filter_fn(&airpods_device));
    assert!(!filter_fn(&regular_device));
    
    // Create test devices for signal strength filtering
    let near_device = create_test_device(
        [0x02, 0x03, 0x04, 0x05, 0x06, 0x07],
        Some("Near AirPods"),
        Some(-50),
        true
    );
    
    let far_device = create_test_device(
        [0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
        Some("Far AirPods"),
        Some(-80),
        true
    );
    
    // Test nearby filter
    let near_filter = airpods_nearby_filter();
    let near_filter_fn = near_filter.create_filter_function();
    assert!(near_filter_fn(&near_device));
    assert!(!near_filter_fn(&far_device));
    
    // Test battery info filter
    let battery_filter = airpods_with_battery_filter();
    let battery_filter_fn = battery_filter.create_filter_function();
    assert!(battery_filter_fn(&airpods_device));
}

/// Test event filtering
#[tokio::test]
async fn test_event_filtering() {
    // Test event type filtering
    let device_filter = EventFilter::event_types(vec![EventType::DeviceDiscovered]);
    
    let device_event = BleEvent::DeviceDiscovered(create_test_device(
        [0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
        Some("Test Device"),
        Some(-60),
        false
    ));
    
    let lost_event = BleEvent::DeviceLost(BDAddr::from([0x01, 0x02, 0x03, 0x04, 0x05, 0x06]));
    
    assert!(device_filter.matches(&device_event));
    assert!(!device_filter.matches(&lost_event));
    
    // Test device address filtering
    let target_address = BDAddr::from([0x01, 0x02, 0x03, 0x04, 0x05, 0x06]);
    let address_filter = EventFilter::devices(vec![target_address]);
    
    assert!(address_filter.matches(&device_event));
    assert!(address_filter.matches(&lost_event));
    
    let other_address_event = BleEvent::DeviceLost(BDAddr::from([0x06, 0x05, 0x04, 0x03, 0x02, 0x01]));
    assert!(!address_filter.matches(&other_address_event));
    
    // Test custom filter
    let strong_signal_filter = EventFilter::custom(|event| {
        match event {
            BleEvent::DeviceDiscovered(device) => {
                if let Some(rssi) = device.rssi {
                    rssi > -70
                } else {
                    false
                }
            },
            _ => false
        }
    });
    
    let strong_device_event = BleEvent::DeviceDiscovered(create_test_device(
        [0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
        Some("Strong Signal"),
        Some(-60),
        false
    ));
    
    let weak_device_event = BleEvent::DeviceDiscovered(create_test_device(
        [0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
        Some("Weak Signal"),
        Some(-80),
        false
    ));
    
    assert!(strong_signal_filter.matches(&strong_device_event));
    assert!(!strong_signal_filter.matches(&weak_device_event));
}

/// Helper function to determine if we should skip Bluetooth tests
fn skip_bluetooth_test() -> bool {
    // Skip if running in CI
    if std::env::var("CI").is_ok() {
        return true;
    }
    
    // Skip if explicitly requested
    if std::env::var("SKIP_BLUETOOTH_TESTS").is_ok() {
        return true;
    }
    
    false
} 