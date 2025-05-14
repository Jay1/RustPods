use std::collections::HashMap;
use btleplug::api::BDAddr;
use std::str::FromStr;
use std::time::Instant;

use rustpods::bluetooth::scanner::DiscoveredDevice;
use rustpods::airpods::{
    detector::{detect_airpods, identify_airpods_type, APPLE_COMPANY_ID},
    AirPodsType, AirPodsBattery, parse_airpods_data, Result as AirPodsResult
};
use rustpods::error::{AirPodsError, ErrorContext, RecoveryAction, RustPodsError};

// Helper function to create test devices
fn create_test_device(
    name: Option<&str>,
    data: Option<Vec<u8>>,
    is_potential_airpods: bool,
) -> DiscoveredDevice {
    let mut manufacturer_data = HashMap::new();
    
    if let Some(bytes) = data {
        manufacturer_data.insert(APPLE_COMPANY_ID, bytes);
    }
    
    DiscoveredDevice {
        address: BDAddr::from_str("00:11:22:33:44:55").unwrap_or_default(),
        name: name.map(String::from),
        rssi: Some(-60),
        manufacturer_data,
        services: vec![],
        is_potential_airpods: is_potential_airpods,
        last_seen: Instant::now(),
        is_connected: false,
        service_data: HashMap::new(),
        tx_power_level: None,
    }
}

#[test]
fn test_manufacturer_data_missing_error() {
    // Test with a device flagged as potential AirPods but no Apple manufacturer data
    let device = create_test_device(
        Some("AirPods-like"),
        None, // No manufacturer data
        true  // But flagged as potential AirPods
    );
    
    let result = detect_airpods(&device);
    // Now we expect Ok(None) instead of an error when manufacturer data is missing
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), None);
}

#[test]
fn test_detection_failed_error() {
    // Test with invalid manufacturer data that causes identification to fail
    let device = create_test_device(
        Some("AirPods"),
        Some(vec![0x00]), // Too short for identification
        true
    );
    
    let result = detect_airpods(&device);
    // We should get an error because the manufacturer data is too short
    assert!(result.is_err());
    
    match result {
        Err(AirPodsError::InvalidData(_)) => {
            // Expected error, test passes
            // The error message should contain information about the cause
        },
        Err(e) => panic!("Expected InvalidData error but got {:?}", e),
        _ => panic!("Expected error but got Ok result"),
    }
}

#[test]
fn test_parse_airpods_data_with_invalid_data() {
    // Test with data that's too short
    let data = vec![0x01, 0x02, 0x03]; // Too short
    let result = parse_airpods_data(&data);
    
    assert!(result.is_err());
    match result {
        Err(AirPodsError::InvalidData(msg)) => {
            assert!(msg.contains("Data too short"), "Error message should mention data length");
        },
        _ => panic!("Expected InvalidData error but got {:?}", result),
    }
} 