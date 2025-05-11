//! Integration tests for AirPods filtering capabilities

use std::collections::HashMap;
use std::time::Instant;

use btleplug::api::BDAddr;

use rustpods::airpods::{
    airpods_all_models_filter, airpods_pro_filter, 
    airpods_with_battery_filter, airpods_nearby_filter,
    APPLE_COMPANY_ID, parse_airpods_data
};
use rustpods::bluetooth::DiscoveredDevice;

/// Helper to create a test device with specified properties
fn create_test_device(
    address: [u8; 6],
    name: Option<&str>,
    rssi: Option<i16>,
    is_airpods: bool,
    prefix: Option<&[u8]>,
    has_battery: bool
) -> DiscoveredDevice {
    let mut manufacturer_data = HashMap::new();
    
    if is_airpods {
        // Use the prefix to determine the type of AirPods
        let airpods_data = if let Some(prefix) = prefix {
            // Create mock AirPods data with the given prefix and battery info if needed
            let mut data = Vec::new();
            data.extend_from_slice(prefix);
            
            // Add dummy data for the middle section
            data.extend_from_slice(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A]);
            
            // Add battery data if requested
            if has_battery {
                data.extend_from_slice(&[0x08, 0x06, 0x05, 0x07]); // Left, Right, Status, Case
            } else {
                data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // No battery info
            }
            
            // Add padding
            data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
            
            data
        } else {
            // Generic AirPods data
            vec![0x07, 0x19, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E]
        };
        
        manufacturer_data.insert(APPLE_COMPANY_ID, airpods_data);
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

#[test]
fn test_all_models_filter() {
    // Create test devices
    let airpods_device = create_test_device(
        [0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
        Some("AirPods"),
        Some(-60),
        true,
        Some(&[0x07, 0x19]), // Regular AirPods
        true
    );
    
    let regular_device = create_test_device(
        [0x06, 0x05, 0x04, 0x03, 0x02, 0x01],
        Some("Bluetooth Speaker"),
        Some(-70),
        false,
        None,
        false
    );
    
    // Create the filter and get filter function
    let filter = airpods_all_models_filter();
    let filter_fn = filter.create_filter_function();
    
    // Test the filter
    assert!(filter_fn(&airpods_device), "AirPods should match the all models filter");
    assert!(!filter_fn(&regular_device), "Non-AirPods device should not match");
}

#[test]
fn test_pro_filter() {
    // Create test devices
    let airpods_regular = create_test_device(
        [0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
        Some("AirPods"),
        Some(-60),
        true,
        Some(&[0x07, 0x19]), // Regular AirPods
        true
    );
    
    let airpods_pro = create_test_device(
        [0x02, 0x03, 0x04, 0x05, 0x06, 0x07],
        Some("AirPods Pro"),
        Some(-60),
        true,
        Some(&[0x0E, 0x19]), // AirPods Pro prefix
        true
    );
    
    let airpods_pro_2 = create_test_device(
        [0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
        Some("AirPods Pro 2"),
        Some(-60),
        true,
        Some(&[0x0F, 0x19]), // AirPods Pro 2 prefix
        true
    );
    
    // Create the filter and get filter function
    let filter = airpods_pro_filter();
    let filter_fn = filter.create_filter_function();
    
    // Test the filter
    assert!(!filter_fn(&airpods_regular), "Regular AirPods should not match");
    assert!(filter_fn(&airpods_pro), "AirPods Pro should match");
    assert!(filter_fn(&airpods_pro_2), "AirPods Pro 2 should match");
}

#[test]
fn test_battery_filter() {
    // Create test devices with proper battery data
    let airpods_with_battery = create_test_device(
        [0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
        Some("AirPods"),
        Some(-60),
        true,
        Some(&[0x07, 0x19]), // Regular AirPods
        true // Has battery info
    );
    
    // For AirPods without battery, we need to manually modify the manufacturer data
    // to ensure parse_airpods_data returns None
    let mut no_battery_device = create_test_device(
        [0x02, 0x03, 0x04, 0x05, 0x06, 0x07],
        Some("AirPods Pro"),
        Some(-60),
        true,
        Some(&[0x0E, 0x19]), // AirPods Pro prefix
        false // No battery info
    );
    
    // The filter checks the result of parse_airpods_data, so we need to make sure
    // the manufacturer data is either too short or has 0xFF values for battery
    if let Some(data) = no_battery_device.manufacturer_data.get_mut(&APPLE_COMPANY_ID) {
        // Replace with invalid/unknown battery data (all 0xFF)
        if data.len() >= 16 {
            data[12] = 0xFF; // Left battery 
            data[13] = 0xFF; // Right battery
            data[15] = 0xFF; // Case battery
        }
    }
    
    // Create the filter and get filter function
    let filter = airpods_with_battery_filter();
    let filter_fn = filter.create_filter_function();
    
    // Test the filter
    assert!(filter_fn(&airpods_with_battery), "AirPods with battery info should match");
    // For additional validation, verify that our test device actually has battery info
    let with_battery_data = airpods_with_battery.manufacturer_data.get(&APPLE_COMPANY_ID).unwrap();
    assert!(parse_airpods_data(with_battery_data).is_some(), "Validation failed: Test device should have battery info");
    
    // For the no-battery device, verify it doesn't match
    let no_battery_data = no_battery_device.manufacturer_data.get(&APPLE_COMPANY_ID).unwrap();
    assert!(parse_airpods_data(no_battery_data).is_none(), "Validation failed: Test device should not have battery info");
    assert!(!filter_fn(&no_battery_device), "AirPods without battery info should not match");
}

#[test]
fn test_nearby_filter() {
    // Create test devices
    let nearby_airpods = create_test_device(
        [0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
        Some("AirPods"),
        Some(-50), // Strong signal
        true,
        Some(&[0x07, 0x19]),
        true
    );
    
    let distant_airpods = create_test_device(
        [0x02, 0x03, 0x04, 0x05, 0x06, 0x07],
        Some("AirPods Pro"),
        Some(-80), // Weak signal
        true,
        Some(&[0x0E, 0x19]),
        true
    );
    
    let airpods_no_rssi = create_test_device(
        [0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
        Some("AirPods Max"),
        None, // No RSSI
        true,
        Some(&[0x0A, 0x19]),
        true
    );
    
    // Create the filter and get filter function
    let filter = airpods_nearby_filter();
    let filter_fn = filter.create_filter_function();
    
    // Test the filter
    assert!(filter_fn(&nearby_airpods), "Nearby AirPods should match");
    assert!(!filter_fn(&distant_airpods), "Distant AirPods should not match");
    assert!(!filter_fn(&airpods_no_rssi), "AirPods without RSSI should not match");
}

#[test]
fn test_combined_filters() {
    // Create a combined filter function for Pro AirPods with battery info and nearby
    let pro_filter = airpods_pro_filter();
    let battery_filter = airpods_with_battery_filter();
    let nearby_filter = airpods_nearby_filter();
    
    let combined_filter = |device: &DiscoveredDevice| {
        let filter1 = pro_filter.create_filter_function();
        let filter2 = battery_filter.create_filter_function();
        let filter3 = nearby_filter.create_filter_function();
        
        filter1(device) && filter2(device) && filter3(device)
    };
    
    // Test device that should pass all filters
    let passing_device = create_test_device(
        [0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
        Some("AirPods Pro"),
        Some(-50), // Strong signal
        true,
        Some(&[0x0E, 0x19]), // Pro model
        true // Has battery
    );
    
    // Test device that should fail some filters
    let failing_device = create_test_device(
        [0x06, 0x05, 0x04, 0x03, 0x02, 0x01],
        Some("AirPods Pro"),
        Some(-80), // Weak signal
        true,
        Some(&[0x0E, 0x19]), // Pro model
        true // Has battery
    );
    
    // Test the combined filter
    assert!(combined_filter(&passing_device), "Device passing all filters should match");
    assert!(!combined_filter(&failing_device), "Device failing some filters should not match");
} 