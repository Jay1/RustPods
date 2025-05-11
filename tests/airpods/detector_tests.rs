//! Integration tests for the AirPods detection module

use rustpods::airpods::{DetectedAirPods, AirPodsType, AirPodsBattery, detect_airpods, identify_airpods_type};
use rustpods::bluetooth::DiscoveredDevice;
use std::collections::HashMap;
use btleplug::api::BDAddr;

/// Test helper to create a discovered device with manufacturer data
fn create_device_with_data(address: &str, name: Option<&str>, data: Vec<u8>) -> DiscoveredDevice {
    let mut manufacturer_data = HashMap::new();
    manufacturer_data.insert(76, data); // 76 is Apple's manufacturer ID
    
    DiscoveredDevice {
        address: address.parse().unwrap(),
        name: name.map(String::from),
        rssi: Some(-60),
        manufacturer_data,
        services: vec![],
    }
}

/// Test AirPods detection from valid manufacturer data
#[test]
fn test_detect_airpods_valid_data() {
    // Example AirPods Pro 2nd Gen manufacturer data
    // This is sample data - actual values would need to be based on real device captures
    let manufacturer_data = vec![
        0x07, 0x19, 0x01, 0x0E, 0x2A, 0x00, 0x00, 0x00, 
        0x45, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0xb0, 0x00 // Battery info
    ];
    
    let device = create_device_with_data(
        "11:22:33:44:55:66", 
        Some("AirPods Pro"), 
        manufacturer_data.clone()
    );
    
    // Detect AirPods
    let result = detect_airpods(&device);
    
    // Verify detection succeeded
    assert!(result.is_some(), "AirPods detection should succeed with valid data");
    
    if let Some(airpods) = result {
        // Verify basic info
        assert_eq!(airpods.address.to_string(), "11:22:33:44:55:66");
        assert_eq!(airpods.name, Some("AirPods Pro".to_string()));
        assert_eq!(airpods.rssi, Some(-60));
        assert_eq!(airpods.raw_data, manufacturer_data);
        
        // Verify it was properly identified as an AirPods device
        assert!(matches!(airpods.device_type, AirPodsType::AirPodsPro | AirPodsType::AirPodsPro2));
    }
}

/// Test AirPods detection with invalid/non-Apple manufacturer data
#[test]
fn test_detect_airpods_invalid_data() {
    // Create a non-Apple device with random manufacturer data
    let mut manufacturer_data = HashMap::new();
    manufacturer_data.insert(0x01, vec![0x01, 0x02, 0x03]); // Non-Apple ID (0x01)
    
    let device = DiscoveredDevice {
        address: "11:22:33:44:55:66".parse().unwrap(),
        name: Some("Generic BT Device".to_string()),
        rssi: Some(-70),
        manufacturer_data,
        services: vec![],
    };
    
    // Attempt to detect as AirPods
    let result = detect_airpods(&device);
    
    // Verify detection failed
    assert!(result.is_none(), "Non-Apple device should not be detected as AirPods");
}

/// Test AirPods detection with no manufacturer data
#[test]
fn test_detect_airpods_no_data() {
    // Create a device with empty manufacturer data
    let device = DiscoveredDevice {
        address: "11:22:33:44:55:66".parse().unwrap(),
        name: Some("Unknown Device".to_string()),
        rssi: Some(-80),
        manufacturer_data: HashMap::new(), // Empty
        services: vec![],
    };
    
    // Attempt to detect as AirPods
    let result = detect_airpods(&device);
    
    // Verify detection failed
    assert!(result.is_none(), "Device with no manufacturer data should not be detected as AirPods");
}

/// Test different AirPods model identification
#[test]
fn test_identify_airpods_types() {
    // Test data for different AirPods models
    // These are sample patterns - actual values should be based on real captures
    let test_cases = [
        (vec![0x01, 0x19, 0x01, 0x00, 0x00], AirPodsType::AirPods1),
        (vec![0x02, 0x19, 0x01, 0x00, 0x00], AirPodsType::AirPods2),
        (vec![0x03, 0x19, 0x01, 0x00, 0x00], AirPodsType::AirPodsPro),
        (vec![0x04, 0x19, 0x01, 0x00, 0x00], AirPodsType::AirPodsMax),
        (vec![0x05, 0x19, 0x01, 0x00, 0x00], AirPodsType::AirPods3),
        (vec![0x06, 0x19, 0x01, 0x00, 0x00], AirPodsType::AirPodsPro2),
        (vec![0xFF, 0x19, 0x01, 0x00, 0x00], AirPodsType::Unknown),
    ];
    
    for (data, expected_type) in test_cases {
        let identified_type = identify_airpods_type(&data);
        assert_eq!(identified_type, expected_type, 
                  "AirPods type mismatch for data: {:?}", data);
    }
}

/// Test battery level extraction
#[test]
fn test_battery_extraction() {
    // Different battery level values in manufacturer data
    // These are example values - actual battery encoding would need to be verified
    let test_cases = [
        // Left, Right, Case, Charging status expected
        (vec![0x07, 0x19, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xb0], 
         Some(70), Some(70), None, false),
        
        (vec![0x07, 0x19, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc9], 
         Some(70), Some(90), None, true),
         
        (vec![0x07, 0x19, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3c], 
         Some(60), None, None, false),
    ];
    
    for (data, left_expected, right_expected, case_expected, charging_expected) in test_cases {
        let device = create_device_with_data("11:22:33:44:55:66", Some("AirPods"), data);
        
        if let Some(airpods) = detect_airpods(&device) {
            assert_eq!(airpods.battery.left, left_expected, 
                       "Left battery level mismatch");
            assert_eq!(airpods.battery.right, right_expected, 
                       "Right battery level mismatch");
            assert_eq!(airpods.battery.case, case_expected, 
                       "Case battery level mismatch");
            assert_eq!(airpods.battery.charging, charging_expected, 
                       "Charging status mismatch");
        } else {
            panic!("Failed to detect AirPods for battery test");
        }
    }
}

/// Test creating a DetectedAirPods manually
#[test]
fn test_detected_airpods_creation() {
    // Create a DetectedAirPods instance manually
    let airpods = DetectedAirPods {
        address: BDAddr::from([0x11, 0x22, 0x33, 0x44, 0x55, 0x66]),
        name: Some("My AirPods".to_string()),
        device_type: AirPodsType::AirPodsPro,
        battery: AirPodsBattery {
            left: Some(85),
            right: Some(90),
            case: Some(60),
            charging: true,
        },
        rssi: Some(-55),
        raw_data: vec![0x01, 0x02, 0x03],
    };
    
    // Verify properties
    assert_eq!(airpods.address, BDAddr::from([0x11, 0x22, 0x33, 0x44, 0x55, 0x66]));
    assert_eq!(airpods.name, Some("My AirPods".to_string()));
    assert_eq!(airpods.device_type, AirPodsType::AirPodsPro);
    assert_eq!(airpods.battery.left, Some(85));
    assert_eq!(airpods.battery.right, Some(90));
    assert_eq!(airpods.battery.case, Some(60));
    assert!(airpods.battery.charging);
    assert_eq!(airpods.rssi, Some(-55));
    assert_eq!(airpods.raw_data, vec![0x01, 0x02, 0x03]);
    
    // Test display formatting - the exact format would depend on implementation
    let display_string = format!("{}", airpods);
    assert!(display_string.contains("AirPods Pro"), 
            "Display string should contain device type");
}

/// Test default AirPods battery implementation
#[test]
fn test_airpods_battery_default() {
    let battery = AirPodsBattery::default();
    
    // Verify defaults
    assert_eq!(battery.left, None);
    assert_eq!(battery.right, None);
    assert_eq!(battery.case, None);
    assert_eq!(battery.charging, false);
    
    // Test display
    let display = format!("{}", battery);
    assert!(display.contains("N/A") || display.contains("Unknown"), 
            "Display for default battery should indicate unknown values");
}

/// Test battery display with various values
#[test]
fn test_battery_display() {
    let test_cases = [
        (AirPodsBattery { left: Some(80), right: Some(85), case: Some(90), charging: false },
         "L:80% R:85% C:90%"),
        (AirPodsBattery { left: Some(30), right: None, case: None, charging: true },
         "L:30% (charging)"),
        (AirPodsBattery { left: None, right: Some(45), case: None, charging: false },
         "R:45%"),
    ];
    
    for (battery, expected_substring) in test_cases {
        let display = format!("{}", battery);
        assert!(display.contains(expected_substring), 
                "Battery display '{}' doesn't contain expected text '{}'", 
                display, expected_substring);
    }
} 