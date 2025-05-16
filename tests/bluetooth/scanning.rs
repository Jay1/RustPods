//! Tests for the BleScanner implementation
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use btleplug::api::BDAddr;

use rustpods::bluetooth::events::BleEvent;
use rustpods::bluetooth::{
    BleScanner, ScanConfig, EventFilter, 
    AdapterManager, DiscoveredDevice, events::EventType
};
use rustpods::airpods::{
    airpods_all_models_filter,
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
        // Include valid battery information in bytes 12-15
        // Format: [prefix, data, left battery, right battery, charging status, case battery, ...]
        manufacturer_data.insert(0x004C, vec![
            0x07, 0x19,  // AirPods identifier
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A,  // Device data
            0x08,        // Left battery (80%)
            0x07,        // Right battery (70%)
            0x01,        // Charging status (left earbud charging)
            0x06,        // Case battery (60%)
            0x0D, 0x0E   // Additional data
        ]);
    }
    
    DiscoveredDevice {
        address: BDAddr::from(address),
        name: name.map(|s| s.to_string()),
        rssi,
        manufacturer_data,
        is_potential_airpods: is_airpods,
        last_seen: Instant::now(),
        is_connected: false,
        service_data: HashMap::new(),
        services: Vec::new(),
        tx_power_level: None,
    }
}

/// Test basic ScanConfig functionality 
#[tokio::test]
async fn test_scan_config() {
    // Check the default config - fix the values to match ScanConfig implementation
    let config = ScanConfig::default();
    assert_eq!(config.scan_duration, Duration::from_secs(10)); // Default is 10 seconds, not 3
    assert_eq!(config.interval_between_scans, Duration::from_secs(20)); // Default is 20 seconds, not 2
    
    // Check the airpods optimized config
    let airpods_config = ScanConfig::airpods_optimized();
    assert_eq!(airpods_config.scan_duration, Duration::from_secs(5)); // This is now correct
    
    // Create a simple scanner
    let scanner = BleScanner::new();
    
    // Get scanner config - check it using default values
    let scanner_config = scanner.get_config();
    assert_eq!(scanner_config.scan_duration, Duration::from_secs(10)); // Default scanner is 10 seconds
    
    // Test config has been applied
    let scanner = BleScanner::with_config(ScanConfig::airpods_optimized());
    assert_eq!(scanner.get_config().scan_duration, Duration::from_secs(5));
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
    // Create a scanner
    println!("Creating BLE scanner...");
    let mut scanner = BleScanner::new();
    println!("✅ Scanner created successfully");
    
    // Start scanning
    println!("Starting scanner...");
    match scanner.start_scanning().await {
        Ok(_) => println!("✅ Scanner started successfully"),
        Err(e) => {
            println!("❌ Failed to start scanner: {:?}", e);
            panic!("Failed to start scanner: {:?}", e);
        }
    }
    
    // Verify scanner is running
    assert!(scanner.is_scanning(), "Scanner should be scanning");
    println!("✅ Scanner is running");
    
    // Wait for a short period
    println!("Waiting for scan to run...");
    tokio::time::sleep(Duration::from_millis(1000)).await;
    
    // Stop scanning
    println!("Stopping scanner...");
    let stop_result = timeout(Duration::from_millis(5000), scanner.stop_scanning()).await;
    match stop_result {
        Ok(result) => match result {
            Ok(_) => println!("✅ Scanner stopped successfully"),
            Err(e) => {
                println!("❌ Failed to stop scanner: {:?}", e);
                panic!("Failed to stop scanner: {:?}", e);
            }
        },
        Err(e) => {
            println!("❌ Timeout stopping scanner: {:?}", e);
            panic!("Scanner did not stop scanning within timeout period");
        }
    }
    
    // Verify scanner stopped
    assert!(!scanner.is_scanning(), "Scanner should not be scanning after stop");
    println!("✅ Scanner is no longer running");
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
    let filter_fn = &filter;
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
    let near_filter = airpods_nearby_filter(-70);
    let near_filter_fn = &near_filter;
    assert!(near_filter_fn(&near_device));
    assert!(!near_filter_fn(&far_device));
    
    // Test battery info filter
    let battery_filter = airpods_with_battery_filter();
    let battery_filter_fn = &battery_filter;
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