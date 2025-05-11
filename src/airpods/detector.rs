use btleplug::api::BDAddr;
use std::collections::HashMap;
use std::default::Default;

use crate::bluetooth::DiscoveredDevice;
use super::{AirPodsType, AirPodsBattery, parse_airpods_data, AirPodsFilter};

/// Constants for AirPods detection
const APPLE_COMPANY_ID: u16 = 0x004C;
const AIRPODS_DATA_LENGTH: usize = 27;
const AIRPODS_1_2_PREFIX: &[u8] = &[0x07, 0x19];
const AIRPODS_PRO_PREFIX: &[u8] = &[0x0E, 0x19];
const AIRPODS_PRO_2_PREFIX: &[u8] = &[0x0F, 0x19];
const AIRPODS_3_PREFIX: &[u8] = &[0x13, 0x19];
const AIRPODS_MAX_PREFIX: &[u8] = &[0x0A, 0x19];

/// Offset positions for AirPods device flags
const FLIP_STATUS_OFFSET: usize = 11;
const LEFT_BATTERY_OFFSET: usize = 12;
const RIGHT_BATTERY_OFFSET: usize = 13;
const CASE_BATTERY_OFFSET: usize = 15;
const CHARGING_STATUS_OFFSET: usize = 14;

/// Represents a detected AirPods device with all available information
#[derive(Debug, Clone)]
pub struct DetectedAirPods {
    /// Bluetooth address
    pub address: BDAddr,
    /// Device name if available
    pub name: Option<String>,
    /// Type of AirPods
    pub device_type: AirPodsType,
    /// Battery information
    pub battery: AirPodsBattery,
    /// Signal strength
    pub rssi: Option<i16>,
    /// Raw manufacturer data for debugging
    pub raw_data: Vec<u8>,
}

impl Default for DetectedAirPods {
    fn default() -> Self {
        Self {
            address: BDAddr::default(),
            name: None,
            device_type: AirPodsType::Unknown,
            battery: AirPodsBattery::default(),
            rssi: None,
            raw_data: Vec::new(),
        }
    }
}

/// Detects if a discovered device is an AirPods device and extracts its information
pub fn detect_airpods(device: &DiscoveredDevice) -> Option<DetectedAirPods> {
    // Check if the device has Apple's manufacturer data
    let data = device.manufacturer_data.get(&APPLE_COMPANY_ID)?;
    
    // Perform preliminary validation of data format
    if !is_valid_airpods_data(data) {
        return None;
    }
    
    // Determine AirPods type
    let device_type = identify_airpods_type(data);
    
    // If it's an unknown type, it's not AirPods
    if matches!(device_type, AirPodsType::Unknown) {
        return None;
    }
    
    // Parse battery levels and charging status
    let battery = parse_airpods_data(data).unwrap_or_default();
    
    Some(DetectedAirPods {
        address: device.address,
        name: device.name.clone(),
        device_type,
        battery,
        rssi: device.rssi,
        raw_data: data.clone(),
    })
}

/// Checks if the manufacturer data has a valid AirPods format
fn is_valid_airpods_data(data: &[u8]) -> bool {
    // Basic length check for AirPods data
    if data.len() < 20 {
        return false;
    }
    
    // Check for known AirPods data patterns
    // This is a simplified check; the actual implementation would be more robust
    if starts_with_any(data, &[
        AIRPODS_1_2_PREFIX,
        AIRPODS_PRO_PREFIX, 
        AIRPODS_PRO_2_PREFIX,
        AIRPODS_3_PREFIX,
        AIRPODS_MAX_PREFIX
    ]) {
        return true;
    }
    
    // Check if the data has the right structure based on known patterns
    // Apple-specific header check (simplified)
    data.len() >= AIRPODS_DATA_LENGTH && data[0] <= 0x20
}

/// Helper function to check if data starts with any of the provided prefixes
fn starts_with_any(data: &[u8], prefixes: &[&[u8]]) -> bool {
    for prefix in prefixes {
        if data.starts_with(prefix) {
            return true;
        }
    }
    false
}

/// Identifies the AirPods type from manufacturer data
pub fn identify_airpods_type(data: &[u8]) -> AirPodsType {
    if data.len() < 2 {
        return AirPodsType::Unknown;
    }
    
    // Match against known device identifiers in the manufacturer data
    match &data[0..2] {
        // These patterns are based on known AirPods identifiers
        // The actual implementation might need to be refined based on real-world testing
        d if d == AIRPODS_1_2_PREFIX => {
            // Further distinguish between AirPods 1 and 2 based on additional data
            // This is a simplified approach and might need refinement
            if data.len() > 4 && data[4] >= 0x10 {
                AirPodsType::AirPods2
            } else {
                AirPodsType::AirPods1
            }
        },
        d if d == AIRPODS_PRO_PREFIX => AirPodsType::AirPodsPro,
        d if d == AIRPODS_PRO_2_PREFIX => AirPodsType::AirPodsPro2,
        d if d == AIRPODS_3_PREFIX => AirPodsType::AirPods3,
        d if d == AIRPODS_MAX_PREFIX => AirPodsType::AirPodsMax,
        _ => AirPodsType::Unknown,
    }
}

/// Creates a filter function for use with the BleScanner
pub fn create_airpods_filter() -> impl Fn(&DiscoveredDevice) -> bool + Clone {
    // Create the filter as a static value so it lives long enough
    move |device| {
        // Basic Apple manufacturer data check - minimum requirement
        if !device.manufacturer_data.contains_key(&APPLE_COMPANY_ID) {
            return false;
        }
        
        // Apply checks similar to AirPodsFilter but directly in this function
        if let Some(data) = device.manufacturer_data.get(&APPLE_COMPANY_ID) {
            is_valid_airpods_data(data)
        } else {
            false
        }
    }
}

/// Creates a custom filter function that combines the basic AirPods detection with additional criteria
pub fn create_custom_airpods_filter<F>(custom_filter: F) -> impl Fn(&DiscoveredDevice) -> bool + Clone
where
    F: Fn(&DiscoveredDevice) -> bool + Clone + 'static,
{
    let base_filter = create_airpods_filter();
    move |device| {
        // First check if it's an AirPods device using the base filter
        if !base_filter(device) {
            return false;
        }
        
        // Then apply the custom filter
        custom_filter(device)
    }
}

/// Helper to extract and interpret AirPods battery values
pub fn extract_battery_level(value: u8) -> Option<u8> {
    // In AirPods protocol, 0xFF is used for unknown battery level
    if value == 0xFF {
        None
    } else {
        // Battery levels in AirPods protocol go from 0 to 10
        // We convert to percentage (0-100)
        Some((value as u8).min(10) * 10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::time::Instant;
    
    fn create_test_device(manufacturer_data: HashMap<u16, Vec<u8>>, is_potential_airpods: bool) -> DiscoveredDevice {
        DiscoveredDevice {
            address: BDAddr::default(),
            name: Some("Test Device".to_string()),
            rssi: Some(-60),
            manufacturer_data,
            is_potential_airpods,
            last_seen: Instant::now(),
        }
    }
    
    fn create_apple_data(prefix: &[u8], battery_left: u8, battery_right: u8, battery_case: u8, charging_status: u8) -> Vec<u8> {
        let mut data = vec![
            prefix[0], prefix[1], 
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A,
            battery_left, battery_right, charging_status, battery_case, // Battery levels and charging status
        ];
        
        // Extend to ensure minimum length
        data.extend_from_slice(&[0x00; 20]);
        data.truncate(AIRPODS_DATA_LENGTH);
        
        data
    }
    
    #[test]
    fn test_detect_airpods_no_manufacturer_data() {
        let device = create_test_device(HashMap::new(), false);
        
        let result = detect_airpods(&device);
        assert!(result.is_none());
        
        // Test with non-Apple manufacturer data
        let mut non_apple_data = HashMap::new();
        non_apple_data.insert(0x0123, vec![0x01, 0x02, 0x03]);
        let device = create_test_device(non_apple_data, false);
        
        let result = detect_airpods(&device);
        assert!(result.is_none());
    }
    
    #[test]
    fn test_detect_airpods_with_valid_data() {
        // Create sample AirPods data
        let mut manufacturer_data = HashMap::new();
        manufacturer_data.insert(APPLE_COMPANY_ID, create_apple_data(
            AIRPODS_1_2_PREFIX, 8, 7, 6, 1 // 80%, 70%, 60%, charging status
        ));
        
        let device = create_test_device(manufacturer_data, true);
        
        let result = detect_airpods(&device);
        assert!(result.is_some());
        if let Some(airpods) = result {
            assert!(matches!(airpods.device_type, AirPodsType::AirPods1 | AirPodsType::AirPods2));
            assert_eq!(airpods.battery.left, Some(80));
            assert_eq!(airpods.battery.right, Some(70));
            assert_eq!(airpods.battery.case, Some(60));
            assert!(!airpods.battery.charging.left); // Based on charging status 1
            assert!(!airpods.battery.charging.right);
            assert!(airpods.battery.charging.case); // Bit 0 is set
        }
    }
    
    #[test]
    fn test_detect_airpods_pro() {
        let mut manufacturer_data = HashMap::new();
        manufacturer_data.insert(APPLE_COMPANY_ID, create_apple_data(
            AIRPODS_PRO_PREFIX, 9, 9, 5, 0b00000110 // 90%, 90%, 50%, left and right charging
        ));
        
        let device = create_test_device(manufacturer_data, true);
        
        let result = detect_airpods(&device);
        assert!(result.is_some());
        if let Some(airpods) = result {
            assert_eq!(airpods.device_type, AirPodsType::AirPodsPro);
            assert_eq!(airpods.battery.left, Some(90));
            assert_eq!(airpods.battery.right, Some(90));
            assert_eq!(airpods.battery.case, Some(50));
            assert!(airpods.battery.charging.left); // Bit 2 is set
            assert!(airpods.battery.charging.right); // Bit 1 is set
            assert!(!airpods.battery.charging.case); // Bit 0 is not set
        }
    }
    
    #[test]
    fn test_identify_airpods_type() {
        // Test various AirPods type identification
        assert!(matches!(identify_airpods_type(&[0x07, 0x19, 0x01]), AirPodsType::AirPods1 | AirPodsType::AirPods2));
        assert_eq!(identify_airpods_type(&[0x0E, 0x19, 0x01]), AirPodsType::AirPodsPro);
        assert_eq!(identify_airpods_type(&[0x0F, 0x19, 0x01]), AirPodsType::AirPodsPro2);
        assert_eq!(identify_airpods_type(&[0x13, 0x19, 0x01]), AirPodsType::AirPods3);
        assert_eq!(identify_airpods_type(&[0x0A, 0x19, 0x01]), AirPodsType::AirPodsMax);
        assert_eq!(identify_airpods_type(&[0xFF, 0xFF, 0x01]), AirPodsType::Unknown);
        
        // Test with data that's too short
        assert_eq!(identify_airpods_type(&[0x01]), AirPodsType::Unknown);
        assert_eq!(identify_airpods_type(&[]), AirPodsType::Unknown);
    }
    
    #[test]
    fn test_extract_battery_level() {
        assert_eq!(extract_battery_level(0), Some(0));
        assert_eq!(extract_battery_level(5), Some(50));
        assert_eq!(extract_battery_level(10), Some(100));
        assert_eq!(extract_battery_level(0xFF), None); // Unknown level
        
        // Test with values exceeding normal range
        assert_eq!(extract_battery_level(11), Some(100)); // Should clamp to 100%
        assert_eq!(extract_battery_level(20), Some(100)); // Should clamp to 100%
    }
    
    #[test]
    fn test_is_valid_airpods_data() {
        // Test with valid data
        assert!(is_valid_airpods_data(&create_apple_data(AIRPODS_1_2_PREFIX, 5, 5, 5, 0)));
        assert!(is_valid_airpods_data(&create_apple_data(AIRPODS_PRO_PREFIX, 5, 5, 5, 0)));
        assert!(is_valid_airpods_data(&create_apple_data(AIRPODS_MAX_PREFIX, 5, 5, 5, 0)));
        
        // Test with invalid data
        assert!(!is_valid_airpods_data(&[0x01, 0x02])); // Too short
        assert!(!is_valid_airpods_data(&[0x30, 0x19, 0x01, 0x02])); // Invalid header
        
        // Create data with invalid prefix but valid length
        let mut invalid_data = vec![0x99, 0x99]; // Invalid prefix
        invalid_data.extend_from_slice(&[0x00; 25]);
        assert!(!is_valid_airpods_data(&invalid_data));
    }
    
    #[test]
    fn test_starts_with_any() {
        let data = [0x07, 0x19, 0x01, 0x02];
        
        assert!(starts_with_any(&data, &[
            &[0x07, 0x19],
            &[0x0E, 0x19],
        ]));
        
        assert!(!starts_with_any(&data, &[
            &[0x0E, 0x19],
            &[0x0F, 0x19],
        ]));
        
        // Test with empty data
        assert!(!starts_with_any(&[], &[
            &[0x07, 0x19],
        ]));
        
        // Test with empty prefixes list
        assert!(!starts_with_any(&data, &[]));
    }
    
    #[test]
    fn test_create_airpods_filter() {
        let filter = create_airpods_filter();
        
        // Create a valid AirPods device
        let mut apple_data = HashMap::new();
        apple_data.insert(APPLE_COMPANY_ID, create_apple_data(AIRPODS_1_2_PREFIX, 5, 5, 5, 0));
        let airpods_device = create_test_device(apple_data, true);
        
        // Create a non-AirPods device
        let mut non_apple_data = HashMap::new();
        non_apple_data.insert(0x0123, vec![0x01, 0x02, 0x03]);
        let non_airpods_device = create_test_device(non_apple_data, false);
        
        assert!(filter(&airpods_device));
        assert!(!filter(&non_airpods_device));
    }
    
    #[test]
    fn test_create_custom_airpods_filter() {
        // Create a custom filter that only accepts devices with RSSI > -55
        let custom_filter = create_custom_airpods_filter(|device| {
            device.rssi.unwrap_or(-100) > -55
        });
        
        // Create a valid AirPods device with strong signal
        let mut apple_data_strong = HashMap::new();
        apple_data_strong.insert(APPLE_COMPANY_ID, create_apple_data(AIRPODS_1_2_PREFIX, 5, 5, 5, 0));
        let mut strong_device = create_test_device(apple_data_strong, true);
        strong_device.rssi = Some(-50);
        
        // Create a valid AirPods device with weak signal
        let mut apple_data_weak = HashMap::new();
        apple_data_weak.insert(APPLE_COMPANY_ID, create_apple_data(AIRPODS_1_2_PREFIX, 5, 5, 5, 0));
        let mut weak_device = create_test_device(apple_data_weak, true);
        weak_device.rssi = Some(-60);
        
        assert!(custom_filter(&strong_device));
        assert!(!custom_filter(&weak_device));
    }
} 