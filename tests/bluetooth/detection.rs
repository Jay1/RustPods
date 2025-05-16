//! Integration tests for AirPods detection functionality

use std::collections::HashMap;
use std::time::Instant;

use btleplug::api::BDAddr;

use rustpods::airpods::{
    detect_airpods, identify_airpods_type, create_airpods_filter,
    AirPodsType, AirPodsBattery, DetectedAirPods,
    APPLE_COMPANY_ID
};
use rustpods::bluetooth::DiscoveredDevice;

/// Helper to create a test device with manufacturer data
fn create_test_device_with_data(
    address: [u8; 6],
    name: Option<&str>,
    rssi: Option<i16>,
    manufacturer_data: HashMap<u16, Vec<u8>>
) -> DiscoveredDevice {
    DiscoveredDevice {
        address: BDAddr::from(address),
        name: name.map(|s| s.to_string()),
        rssi,
        manufacturer_data,
        is_potential_airpods: false,
        last_seen: Instant::now(),
        is_connected: false,
        service_data: HashMap::new(),
        services: Vec::new(),
        tx_power_level: None,
    }
}

/// Helper to create test AirPods manufacturer data with the correct format
fn create_airpods_data(
    prefix: &[u8],
    left_battery: u8,
    right_battery: u8,
    case_battery: u8,
    charging_status: u8
) -> Vec<u8> {
    // The data structure must exactly match what's expected in parse_airpods_data
    // Left battery is at index 12, right at 13, charging at 14, case at 15
    
    let mut data = Vec::with_capacity(27);
    
    // AirPods model prefix (first two bytes)
    data.push(prefix[0]);
    data.push(prefix[1]);
    
    // Add 10 bytes of dummy data
    data.extend_from_slice(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A]);
    
    // Add battery and status data at the correct positions
    // Index 12: Left battery
    data.push(left_battery);
    // Index 13: Right battery
    data.push(right_battery);
    // Index 14: Charging status
    data.push(charging_status);
    // Index 15: Case battery
    data.push(case_battery);
    
    // Add padding to ensure enough length
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    
    data
}

/// Constant AirPods model prefixes
const AIRPODS_1_2_PREFIX: &[u8] = &[0x07, 0x19];
const AIRPODS_PRO_PREFIX: &[u8] = &[0x0E, 0x19];
const AIRPODS_PRO_2_PREFIX: &[u8] = &[0x0F, 0x19];
const AIRPODS_3_PREFIX: &[u8] = &[0x13, 0x19];
const AIRPODS_MAX_PREFIX: &[u8] = &[0x0A, 0x19];

#[test]
fn test_detect_airpods_regular() {
    // Create manufacturer data for regular AirPods
    let mut manufacturer_data = HashMap::new();
    manufacturer_data.insert(
        APPLE_COMPANY_ID,
        create_airpods_data(AIRPODS_1_2_PREFIX, 8, 7, 6, 0b00000001) // 80%, 70%, 60%, case charging
    );
    let device = create_test_device_with_data(
        [0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
        Some("AirPods"),
        Some(-60),
        manufacturer_data
    );
    // Test detection
    let result = detect_airpods(&device).unwrap();
    assert!(result.is_some(), "Should detect AirPods device");
    let airpods = result.unwrap();
    assert!(matches!(airpods.device_type, AirPodsType::AirPods1 | AirPodsType::AirPods2),
            "Should detect as regular AirPods");
    assert_eq!(airpods.battery.as_ref().unwrap().left, Some(80), "Left battery should be 80%");
    assert_eq!(airpods.battery.as_ref().unwrap().right, Some(70), "Right battery should be 70%");
    assert_eq!(airpods.battery.as_ref().unwrap().case, Some(60), "Case battery should be 60%");
    match airpods.battery.as_ref().unwrap().charging {
        Some(state) => {
            assert!(!state.is_left_charging(), "Left should not be charging");
            assert!(!state.is_right_charging(), "Right should not be charging");
            assert!(state.is_case_charging(), "Case should be charging");
        },
        None => (),
    }
}

#[test]
fn test_detect_airpods_pro() {
    // Create manufacturer data for AirPods Pro
    let mut manufacturer_data = HashMap::new();
    manufacturer_data.insert(
        APPLE_COMPANY_ID,
        create_airpods_data(AIRPODS_PRO_PREFIX, 9, 9, 5, 0b00000110) // 90%, 90%, 50%, left+right charging
    );
    let device = create_test_device_with_data(
        [0x02, 0x03, 0x04, 0x05, 0x06, 0x07],
        Some("AirPods Pro"),
        Some(-65),
        manufacturer_data
    );
    // Test detection
    let result = detect_airpods(&device).unwrap();
    assert!(result.is_some(), "Should detect AirPods Pro device");
    let airpods = result.unwrap();
    assert_eq!(airpods.device_type, AirPodsType::AirPodsPro, "Should detect as AirPods Pro");
    assert_eq!(airpods.battery.as_ref().unwrap().left, Some(90), "Left battery should be 90%");
    assert_eq!(airpods.battery.as_ref().unwrap().right, Some(90), "Right battery should be 90%");
    assert_eq!(airpods.battery.as_ref().unwrap().case, Some(50), "Case battery should be 50%");
    match airpods.battery.as_ref().unwrap().charging {
        Some(state) => {
            assert!(state.is_left_charging(), "Left should be charging");
            assert!(state.is_right_charging(), "Right should be charging");
            assert!(!state.is_case_charging(), "Case should not be charging");
        },
        None => (),
    }
}

#[test]
fn test_detect_airpods_pro_2() {
    // Create manufacturer data for AirPods Pro 2
    let mut manufacturer_data = HashMap::new();
    manufacturer_data.insert(
        APPLE_COMPANY_ID,
        create_airpods_data(AIRPODS_PRO_2_PREFIX, 10, 10, 10, 0b00000001) // 100%, 100%, 100%, case charging
    );
    let device = create_test_device_with_data(
        [0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
        Some("AirPods Pro 2"),
        Some(-70),
        manufacturer_data
    );
    // Test detection
    let result = detect_airpods(&device).unwrap();
    assert!(result.is_some(), "Should detect AirPods Pro 2 device");
    let airpods = result.unwrap();
    assert_eq!(airpods.device_type, AirPodsType::AirPodsPro2, "Should detect as AirPods Pro 2");
    assert_eq!(airpods.battery.as_ref().unwrap().left, Some(100), "Left battery should be 100%");
    assert_eq!(airpods.battery.as_ref().unwrap().right, Some(100), "Right battery should be 100%");
    assert_eq!(airpods.battery.as_ref().unwrap().case, Some(100), "Case battery should be 100%");
    match airpods.battery.as_ref().unwrap().charging {
        Some(state) => {
            assert!(!state.is_left_charging(), "Left should not be charging");
            assert!(!state.is_right_charging(), "Right should not be charging");
            assert!(state.is_case_charging(), "Case should be charging");
        },
        None => (),
    }
}

#[test]
fn test_detect_airpods_max() {
    // Create manufacturer data for AirPods Max
    let mut manufacturer_data = HashMap::new();
    manufacturer_data.insert(
        APPLE_COMPANY_ID,
        create_airpods_data(AIRPODS_MAX_PREFIX, 6, 6, 0, 0b00000000) // 60%, 60%, N/A, none charging
    );
    let device = create_test_device_with_data(
        [0x04, 0x05, 0x06, 0x07, 0x08, 0x09],
        Some("AirPods Max"),
        Some(-55),
        manufacturer_data
    );
    // Test detection
    let result = detect_airpods(&device).unwrap();
    assert!(result.is_some(), "Should detect AirPods Max device");
    let airpods = result.unwrap();
    assert_eq!(airpods.device_type, AirPodsType::AirPodsMax, "Should detect as AirPods Max");
    assert_eq!(airpods.battery.as_ref().unwrap().left, Some(60), "Left battery should be 60%");
    assert_eq!(airpods.battery.as_ref().unwrap().right, Some(60), "Right battery should be 60%");
    assert_eq!(airpods.battery.as_ref().unwrap().case, Some(0), "Case battery should be 0%");
    match airpods.battery.as_ref().unwrap().charging {
        Some(state) => {
            assert!(!state.is_left_charging(), "Left should not be charging");
            assert!(!state.is_right_charging(), "Right should not be charging");
            assert!(!state.is_case_charging(), "Case should not be charging");
        },
        None => (),
    }
}

#[test]
fn test_detect_airpods_3() {
    // Create manufacturer data for AirPods 3
    let mut manufacturer_data = HashMap::new();
    manufacturer_data.insert(
        APPLE_COMPANY_ID,
        create_airpods_data(AIRPODS_3_PREFIX, 8, 8, 7, 0b00000000) // 80%, 80%, 70%, none charging
    );
    let device = create_test_device_with_data(
        [0x05, 0x06, 0x07, 0x08, 0x09, 0x0A],
        Some("AirPods 3"),
        Some(-65),
        manufacturer_data
    );
    // Test detection
    let result = detect_airpods(&device).unwrap();
    assert!(result.is_some(), "Should detect AirPods 3 device");
    let airpods = result.unwrap();
    assert_eq!(airpods.device_type, AirPodsType::AirPods3, "Should detect as AirPods 3");
    assert_eq!(airpods.battery.as_ref().unwrap().left, Some(80), "Left battery should be 80%");
    assert_eq!(airpods.battery.as_ref().unwrap().right, Some(80), "Right battery should be 80%");
    assert_eq!(airpods.battery.as_ref().unwrap().case, Some(70), "Case battery should be 70%");
    // Charging state
    match airpods.battery.as_ref().unwrap().charging {
        Some(state) => {
            assert!(!state.is_left_charging(), "Left should not be charging");
            assert!(!state.is_right_charging(), "Right should not be charging");
            assert!(!state.is_case_charging(), "Case should not be charging");
        },
        None => (),
    }
}

#[test]
fn test_detect_non_airpods_apple_device() {
    // Create Apple manufacturer data that isn't AirPods
    let mut manufacturer_data = HashMap::new();
    manufacturer_data.insert(
        APPLE_COMPANY_ID,
        vec![0x01, 0x02, 0x03, 0x04, 0x05] // Some random data
    );
    
    let device = create_test_device_with_data(
        [0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F],
        Some("Apple Device"),
        Some(-65),
        manufacturer_data
    );
    
    // Test detection
    let result = detect_airpods(&device).unwrap();
    assert!(result.is_none(), "Should not detect as AirPods");
}

#[test]
fn test_detect_non_apple_device() {
    // Create non-Apple manufacturer data
    let mut manufacturer_data = HashMap::new();
    manufacturer_data.insert(
        0x0081, // Sony Company ID
        vec![0x01, 0x02, 0x03, 0x04, 0x05]
    );
    
    let device = create_test_device_with_data(
        [0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F],
        Some("Sony Headphones"),
        Some(-65),
        manufacturer_data
    );
    
    // Test detection
    let result = detect_airpods(&device).unwrap();
    assert!(result.is_none(), "Should not detect as AirPods");
}

#[test]
fn test_detect_airpods_empty_battery() {
    // Create manufacturer data for AirPods with unknown battery levels
    let mut manufacturer_data = HashMap::new();
    manufacturer_data.insert(
        APPLE_COMPANY_ID,
        create_airpods_data(AIRPODS_1_2_PREFIX, 0xFF, 0xFF, 0xFF, 0b00000000) // Unknown battery levels
    );
    
    let device = create_test_device_with_data(
        [0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
        Some("AirPods"),
        Some(-60),
        manufacturer_data
    );
    
    // Test detection
    let result = detect_airpods(&device).unwrap();
    assert!(result.is_some(), "Should detect AirPods device even with unknown battery");
    let airpods = result.unwrap();
    assert!(matches!(airpods.device_type, AirPodsType::AirPods1 | AirPodsType::AirPods2));
    assert_eq!(airpods.battery.as_ref().unwrap().left, None, "Left battery should be None");
    assert_eq!(airpods.battery.as_ref().unwrap().right, None, "Right battery should be None");
    assert_eq!(airpods.battery.as_ref().unwrap().case, None, "Case battery should be None");
}

#[test]
fn test_identify_airpods_type() {
    // Test detection of each model
    assert_eq!(
        identify_airpods_type(&None, AIRPODS_1_2_PREFIX).unwrap(),
        AirPodsType::AirPods2,
        "Should identify AirPods 1/2"
    );
    assert_eq!(
        identify_airpods_type(&None, AIRPODS_PRO_PREFIX).unwrap(),
        AirPodsType::AirPodsPro,
        "Should identify AirPods Pro"
    );
    assert_eq!(
        identify_airpods_type(&None, AIRPODS_PRO_2_PREFIX).unwrap(),
        AirPodsType::AirPodsPro2,
        "Should identify AirPods Pro 2"
    );
    assert_eq!(
        identify_airpods_type(&None, AIRPODS_3_PREFIX).unwrap(),
        AirPodsType::AirPods3,
        "Should identify AirPods 3"
    );
    assert_eq!(
        identify_airpods_type(&None, AIRPODS_MAX_PREFIX).unwrap(),
        AirPodsType::AirPodsMax,
        "Should identify AirPods Max"
    );
    // Test unknown prefix
    assert_eq!(
        identify_airpods_type(&None, &[0x00, 0x00]).unwrap(),
        AirPodsType::Unknown,
        "Should return Unknown for unknown prefix"
    );
    // Test with insufficient data
    assert!(
        identify_airpods_type(&None, &[0x07]).is_err(),
        "Should return error for insufficient data"
    );
}

#[test]
fn test_airpods_filter() {
    // Create the filter
    let filter = create_airpods_filter();
    
    // Create test devices
    let mut airpods_data = HashMap::new();
    airpods_data.insert(
        APPLE_COMPANY_ID,
        create_airpods_data(AIRPODS_1_2_PREFIX, 8, 7, 6, 0)
    );
    
    let airpods_device = create_test_device_with_data(
        [0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
        Some("AirPods"),
        Some(-60),
        airpods_data
    );
    
    let mut other_apple_data = HashMap::new();
    other_apple_data.insert(
        APPLE_COMPANY_ID,
        vec![0x01, 0x02, 0x03, 0x04, 0x05] // Some random Apple data
    );
    
    let other_apple_device = create_test_device_with_data(
        [0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F],
        Some("Apple Device"),
        Some(-65),
        other_apple_data
    );
    
    let mut non_apple_data = HashMap::new();
    non_apple_data.insert(
        0x0081, // Sony
        vec![0x01, 0x02, 0x03, 0x04, 0x05]
    );
    
    let non_apple_device = create_test_device_with_data(
        [0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F],
        Some("Sony Headphones"),
        Some(-65),
        non_apple_data
    );
    
    // Test the filter
    assert!(filter(&airpods_device), "Should identify AirPods device");
    assert!(!filter(&other_apple_device), "Should not identify other Apple devices");
    assert!(!filter(&non_apple_device), "Should not identify non-Apple devices");
}

#[test]
fn test_airpods_default_trait_implementations() {
    // Test that default traits work correctly
    let default_airpods = DetectedAirPods::default();
    assert_eq!(default_airpods.device_type, AirPodsType::Unknown);
    assert!(default_airpods.battery.is_none());
    // Test that default battery creation works
    let default_battery = AirPodsBattery::default();
    assert_eq!(default_battery.left, None);
    assert_eq!(default_battery.right, None);
    assert_eq!(default_battery.case, None);
    assert!(default_battery.charging.is_none());
} 