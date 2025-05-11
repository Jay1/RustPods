//! Integration tests for AirPods filtering capabilities

use std::collections::HashMap;
use std::time::Instant;

use btleplug::api::BDAddr;

use rustpods::airpods::{
    airpods_all_models_filter, airpods_pro_filter, 
    airpods_with_battery_filter, airpods_nearby_filter,
    APPLE_COMPANY_ID, detect_airpods
};
use rustpods::bluetooth::DiscoveredDevice;
use crate::bluetooth::common_utils::{create_test_device, create_airpods_manufacturer_data, AIRPODS_PRO_PREFIX};

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

#[tokio::test]
async fn test_battery_filter() {
    // Notice that our test AirPods Pro prefix is different from the actual implementation
    // In the test utilities it's [0x0E, 0x20] but in src/airpods/detector.rs it's [0x0E, 0x19]
    // We need to use the correct implementation value
    
    // Create test device with valid AirPods Pro data including battery information
    // using create_airpods_manufacturer_data directly since it gives us more control
    let mut device_with_battery = DiscoveredDevice {
        address: BDAddr::from([0x01, 0x02, 0x03, 0x04, 0x05, 0x06]),
        name: Some("AirPods Pro with battery".to_string()),
        rssi: Some(-60),
        manufacturer_data: create_airpods_manufacturer_data(
            &[0x0E, 0x19],  // Use detector.rs value
            8,               // Left battery (80%)
            7,               // Right battery (70%)
            6,               // Case battery (60%)
            0x01             // Charging flags: left charging
        ),
        is_potential_airpods: true,
        last_seen: Instant::now(),
    };

    // Create a device without battery (AirPods but empty manufacturer data)
    let mut device_without_battery = DiscoveredDevice {
        address: BDAddr::from([0x02, 0x03, 0x04, 0x05, 0x06, 0x07]),
        name: Some("AirPods without battery".to_string()),
        rssi: Some(-60),
        manufacturer_data: HashMap::new(),
        is_potential_airpods: true,
        last_seen: Instant::now(),
    };

    // Add Apple manufacturer data without valid battery info
    let mut invalid_battery_data = Vec::new();
    invalid_battery_data.extend_from_slice(&[0x0E, 0x19]); // AirPods Pro prefix
    invalid_battery_data.extend_from_slice(&[0x01, 0x02, 0x03]); // Too short to contain battery
    device_without_battery.manufacturer_data.insert(APPLE_COMPANY_ID, invalid_battery_data);
    
    // Create a regular BLE device (not AirPods)
    let non_airpods_device = create_test_device(
        [0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
        Some("Regular BLE device"),
        Some(-70),
        false,                            // Not AirPods
        None,                             // No prefix
        false                             // No battery
    );
    
    // For device with battery, verify the detected AirPods has battery information
    let airpods_with_battery = detect_airpods(&device_with_battery);
    assert!(airpods_with_battery.is_some(), "AirPods with battery should be detected");
    let detected = airpods_with_battery.unwrap();
    
    // Check battery fields directly
    assert!(detected.battery.left.is_some(), "Left battery should be present");
    assert!(detected.battery.right.is_some(), "Right battery should be present");
    assert!(detected.battery.case.is_some(), "Case battery should be present");
    
    // For device without battery, verify no battery info is detected
    let airpods_without_battery = detect_airpods(&device_without_battery);
    
    // Debug output to understand battery parsing
    println!("Device without battery manufacturer data: {:?}", device_without_battery.manufacturer_data);
    if airpods_without_battery.is_some() {
        println!("Detected AirPods: {:?}", airpods_without_battery.unwrap());
    } else {
        println!("No AirPods detected for device_without_battery");
    }
    
    // Create the battery filter
    let filter = airpods_with_battery_filter();
    
    // Test the filter by using its create_filter_function method
    let filter_fn = filter.create_filter_function();
    
    // Test on all device types and print debug info
    println!("\nFilter test results:");
    println!("Device with battery passes filter: {}", filter_fn(&device_with_battery));
    println!("Device without battery passes filter: {}", filter_fn(&device_without_battery));
    println!("Non-AirPods device passes filter: {}", filter_fn(&non_airpods_device));
    
    // Make assertions
    assert!(filter_fn(&device_with_battery), "Filter should match AirPods with battery");
    assert!(!filter_fn(&device_without_battery), "Filter should not match AirPods without battery");
    assert!(!filter_fn(&non_airpods_device), "Filter should not match non-AirPods device");
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
    let filter = airpods_nearby_filter(-60); // Filter for RSSI > -60
    let filter_fn = filter.create_filter_function();
    
    // Test the filter
    assert!(filter_fn(&nearby_airpods), "Nearby AirPods should match");
    assert!(!filter_fn(&distant_airpods), "Distant AirPods should not match");
    assert!(!filter_fn(&airpods_no_rssi), "AirPods with no RSSI should not match");
}

#[test]
fn test_combined_filters() {
    // Create a nearby AirPods Pro with battery
    let mut pro_nearby_with_battery = create_test_device(
        [0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
        Some("AirPods Pro"),
        Some(-50), // Strong signal
        true,
        Some(&[0x0E, 0x19]), // Pro prefix
        true
    );
    
    // Manually set manufacturer data
    pro_nearby_with_battery.manufacturer_data = create_airpods_manufacturer_data(
        &[0x0E, 0x19], // Pro prefix
        80, // Left battery
        75, // Right battery
        90, // Case battery
        0x01 // Charging flags
    );
    
    // Create a far away AirPods Pro with battery
    let mut pro_distant_with_battery = create_test_device(
        [0x02, 0x03, 0x04, 0x05, 0x06, 0x07],
        Some("AirPods Pro"),
        Some(-80), // Weak signal
        true,
        Some(&[0x0E, 0x19]), // Pro prefix
        true
    );
    
    // Manually set manufacturer data
    pro_distant_with_battery.manufacturer_data = create_airpods_manufacturer_data(
        &[0x0E, 0x19], // Pro prefix
        80, // Left battery
        75, // Right battery
        90, // Case battery
        0x01 // Charging flags
    );
    
    // Create a nearby regular AirPods with battery
    let mut regular_nearby_with_battery = create_test_device(
        [0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
        Some("AirPods"),
        Some(-50), // Strong signal
        true,
        Some(&[0x07, 0x19]), // Regular prefix
        true
    );
    
    // Manually set manufacturer data
    regular_nearby_with_battery.manufacturer_data = create_airpods_manufacturer_data(
        &[0x07, 0x19], // Regular prefix
        80, // Left battery
        75, // Right battery
        90, // Case battery
        0x01 // Charging flags
    );
    
    // Combining filters: Pro + Nearby + Battery
    let pro_filter = airpods_pro_filter();
    let nearby_filter = airpods_nearby_filter(-60);
    let battery_filter = airpods_with_battery_filter();
    
    let pro_filter_fn = pro_filter.create_filter_function();
    let nearby_filter_fn = nearby_filter.create_filter_function();
    let battery_filter_fn = battery_filter.create_filter_function();
    
    // Test combined filter functions
    let combined_filter = |device: &DiscoveredDevice| {
        pro_filter_fn(device) && nearby_filter_fn(device) && battery_filter_fn(device)
    };
    
    assert!(combined_filter(&pro_nearby_with_battery), "Nearby AirPods Pro with battery should match all filters");
    assert!(!combined_filter(&pro_distant_with_battery), "Distant AirPods Pro should not match nearby filter");
    assert!(!combined_filter(&regular_nearby_with_battery), "Regular AirPods should not match Pro filter");
} 