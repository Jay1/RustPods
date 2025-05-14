use std::collections::HashMap;
use btleplug::api::BDAddr;
use std::str::FromStr;
use std::time::Instant;

use rustpods::bluetooth::scanner::DiscoveredDevice;
use rustpods::airpods::{
    detector::{detect_airpods, identify_airpods_type, APPLE_COMPANY_ID},
    AirPodsType, Result as AirPodsResult
};
use rustpods::error::{AirPodsError, ErrorContext, ErrorManager, RecoveryAction, RustPodsError};

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
    }
}

#[test]
fn test_error_propagation() {
    // Test with invalid data (too short)
    let device = create_test_device(
        Some("AirPods"),
        Some(vec![0x07]), // Too short for identification
        true
    );
    
    let result = detect_airpods(&device);
    assert!(result.is_err());
    
    match result {
        Err(AirPodsError::InvalidData(_)) => {
            // Expected error, test passes
        },
        _ => panic!("Expected InvalidData error but got {:?}", result),
    }
}

#[test]
fn test_graceful_degradation() {
    // Test with valid type data but missing battery data
    let device = create_test_device(
        Some("AirPods Pro"),
        Some(vec![0x0E, 0x19, 0x01]), // Valid type prefix but no battery data
        true
    );
    
    let result = detect_airpods(&device);
    assert!(result.is_ok());
    
    let airpods = result.unwrap();
    assert!(airpods.is_some());
    
    let detected = airpods.unwrap();
    assert_eq!(detected.device_type, AirPodsType::AirPodsPro);
    assert!(detected.battery.is_none(), "Battery should be None with missing data");
}

#[test]
fn test_conversion_to_rustpods_error() {
    // Test that AirPodsError properly converts to RustPodsError
    let airpods_error = AirPodsError::ParseError("Test parse error".to_string());
    
    let rustpods_error: RustPodsError = airpods_error.into();
    
    match rustpods_error {
        RustPodsError::AirPods(msg) => {
            assert!(msg.contains("parse error"), "Error message should contain 'parse error'");
        },
        _ => panic!("Expected RustPodsError::AirPods variant but got {:?}", rustpods_error),
    }
}

#[test]
fn test_error_manager_integration() {
    // Test that errors can be properly recorded with the error manager
    let mut error_manager = ErrorManager::new();
    let context = ErrorContext::new("AirPodsTest", "error_test")
        .with_metadata("device_address", "00:11:22:33:44:55");
    
    let airpods_error = AirPodsError::InvalidData("Test invalid data".to_string());
    
    error_manager.record_error_with_context(
        airpods_error.into(),
        context,
        RecoveryAction::Retry
    );
    
    // Verify the error was recorded
    let history = error_manager.get_error_history();
    assert_eq!(history.len(), 1, "Error should be recorded in history");
    
    let error_entry = &history[0];
    match &error_entry.error {
        RustPodsError::AirPods(msg) => {
            assert!(msg.contains("invalid data"), "Error message should contain 'invalid data'");
        },
        _ => panic!("Expected AirPods error variant but got something else"),
    }
}

#[test]
fn test_identify_airpods_error_handling() {
    // Test error handling in identify_airpods_type
    let result: AirPodsResult<AirPodsType> = identify_airpods_type(&None, &[]);
    
    assert!(result.is_err());
    assert!(matches!(result, Err(AirPodsError::InvalidData(_))));
    
    // Test with just enough data but invalid prefix
    let result = identify_airpods_type(&None, &[0xFF, 0xFF]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), AirPodsType::Unknown);
}

#[test]
fn test_different_airpods_error_variants() {
    // Create different error types
    let parse_error = AirPodsError::ParseError("Parse failed".to_string());
    let invalid_data = AirPodsError::InvalidData("Invalid data".to_string());
    let manufacturer_missing = AirPodsError::ManufacturerDataMissing;
    let detection_failed = AirPodsError::DetectionFailed("Detection failed".to_string());
    
    // Convert to RustPodsError
    let parse_error: RustPodsError = parse_error.into();
    let invalid_data: RustPodsError = invalid_data.into();
    let manufacturer_missing: RustPodsError = manufacturer_missing.into();
    let detection_failed: RustPodsError = detection_failed.into();
    
    // Verify all are mapped to RustPodsError::AirPods
    assert!(matches!(parse_error, RustPodsError::AirPods(_)));
    assert!(matches!(invalid_data, RustPodsError::AirPods(_)));
    assert!(matches!(manufacturer_missing, RustPodsError::AirPods(_)));
    assert!(matches!(detection_failed, RustPodsError::AirPods(_)));
} 