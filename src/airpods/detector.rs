use btleplug::api::BDAddr;
use std::default::Default;
use std::collections::HashMap;

use crate::bluetooth::DiscoveredDevice;
use super::{AirPodsType, AirPodsBattery, parse_airpods_data};

/// Constants for AirPods detection
pub const APPLE_COMPANY_ID: u16 = 0x004C;
const AIRPODS_DATA_LENGTH: usize = 27;
const AIRPODS_1_2_PREFIX: &[u8] = &[0x07, 0x19];
const AIRPODS_PRO_PREFIX: &[u8] = &[0x0E, 0x19];
const AIRPODS_PRO_2_PREFIX: &[u8] = &[0x0F, 0x19];
const AIRPODS_3_PREFIX: &[u8] = &[0x13, 0x19];
const AIRPODS_MAX_PREFIX: &[u8] = &[0x0A, 0x19];

/// Offset positions for AirPods device flags
#[allow(dead_code)]
pub const FLIP_STATUS_OFFSET: usize = 11;
#[allow(dead_code)]
pub const LEFT_BATTERY_OFFSET: usize = 12;
#[allow(dead_code)]
pub const RIGHT_BATTERY_OFFSET: usize = 13;
#[allow(dead_code)]
pub const CASE_BATTERY_OFFSET: usize = 15;
#[allow(dead_code)]
pub const CHARGING_STATUS_OFFSET: usize = 14;

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

/// AirPods detector for identifying and parsing AirPods data
#[derive(Clone, Debug, Default)]
pub struct AirPodsDetector {
    /// Known AirPods devices
    known_devices: HashMap<String, AirPodsType>,
}

impl AirPodsDetector {
    /// Create a new AirPods detector
    pub fn new() -> Self {
        Self {
            known_devices: HashMap::new(),
        }
    }
    
    /// Check if a device is an AirPods based on device information
    pub fn is_airpods(&self, device: &crate::bluetooth::scanner::DiscoveredDevice) -> bool {
        // Check if name contains "AirPods"
        if let Some(name) = &device.name {
            if name.contains("AirPods") {
                return true;
            }
        }
        
        // Check for Apple manufacturer data with the right structure
        if device.manufacturer_data.contains_key(&APPLE_COMPANY_ID) {
            let data = &device.manufacturer_data[&APPLE_COMPANY_ID];
            // AirPods advertisement data is typically 27 bytes
            if data.len() >= 16 {
                // Check for common AirPods data patterns
                // This is simplified - real implementation would check specific byte patterns
                return true;
            }
        }
        
        false
    }
    
    /// Remember a device as AirPods
    pub fn remember_device(&mut self, address: &str, device_type: AirPodsType) {
        self.known_devices.insert(address.to_string(), device_type);
    }
    
    /// Check if a device is a known AirPods device
    pub fn is_known_airpods(&self, address: &str) -> bool {
        self.known_devices.contains_key(address)
    }
    
    /// Get the device type for a known AirPods device
    pub fn get_device_type(&self, address: &str) -> Option<AirPodsType> {
        self.known_devices.get(address).cloned()
    }
}

/// Detects if a discovered device is an AirPods device and extracts its information
pub fn detect_airpods(device: &DiscoveredDevice) -> Option<DetectedAirPods> {
    // Only process potential AirPods devices
    if !device.is_potential_airpods {
        return None;
    }
    
    // Check for Apple manufacturer data
    if !device.manufacturer_data.contains_key(&APPLE_COMPANY_ID) {
        return None;
    }
    
    // Get the manufacturer data
    let data = &device.manufacturer_data[&APPLE_COMPANY_ID];
    
    // Parse battery data (if available)
    let battery = parse_airpods_data(data).unwrap_or_default();
    
    // Determine device type
    let device_type = identify_airpods_type(&device.name, data);
    
    // Create DetectedAirPods
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

/// Identify the specific type of AirPods based on the device name and raw manufacturer data
/// The manufacturer data contains specific bytes that identify the model
pub fn identify_airpods_type(name: &Option<String>, data: &[u8]) -> AirPodsType {
    // Ensure data is at least 3 bytes
    if data.len() < 3 {
        return AirPodsType::Unknown;
    }
    
    // Check if bytes match a known pattern
    match data[0..3] {
        [0x07, 0x19, 0x01] => {
            // Could be either AirPods 1 or AirPods 2, try to distinguish by name
            if let Some(n) = name {
                if n.contains("AirPods 1") {
                    return AirPodsType::AirPods1;
                } else if n.contains("AirPods 2") {
                    return AirPodsType::AirPods2;
                }
            }
            // Default to AirPods 1 if we can't tell
            AirPodsType::AirPods1
        },
        [0x0E, 0x19, 0x01] => AirPodsType::AirPodsPro,
        [0x0F, 0x19, 0x01] => AirPodsType::AirPodsPro2,
        [0x13, 0x19, 0x01] => AirPodsType::AirPods3,
        [0x0A, 0x19, 0x01] => AirPodsType::AirPodsMax,
        _ => AirPodsType::Unknown,
    }
}

/// Creates a filter function for use with the BleScanner
pub fn create_airpods_filter() -> crate::airpods::AirPodsFilter {
    Box::new(|device: &DiscoveredDevice| {
        // Check if name contains "AirPods"
        if let Some(name) = &device.name {
            if name.contains("AirPods") {
                return true;
            }
        }
        
        // Check for Apple manufacturer data
        if device.manufacturer_data.contains_key(&APPLE_COMPANY_ID) {
            let data = &device.manufacturer_data[&APPLE_COMPANY_ID];
            // AirPods advertisement data is typically at least 16 bytes
            if data.len() >= 16 {
                return true;
            }
        }
        
        false
    })
}

/// Creates a custom filter function that combines the basic AirPods detection with additional criteria
pub fn create_custom_airpods_filter(
    min_rssi: Option<i16>,
    model_type: Option<AirPodsType>,
) -> crate::airpods::AirPodsFilter {
    let base_filter = create_airpods_filter();
    Box::new(move |device: &DiscoveredDevice| {
        // Apply RSSI filter if specified
        if let Some(min_rssi_value) = min_rssi {
            if let Some(rssi) = device.rssi {
                if rssi < min_rssi_value {
                    return false;
                }
            }
        }
        
        // Check basic AirPods criteria
        let is_airpods = base_filter(device);
        if !is_airpods {
            return false;
        }
        
        // If model type specified, check for that specific type
        if let Some(model) = &model_type {
            if let Some(detected) = detect_airpods(device) {
                return detected.device_type == *model;
            }
            return false;
        }
        
        true
    })
}

/// Helper to extract and interpret AirPods battery values
#[allow(dead_code)]
pub fn extract_battery_level(value: u8) -> Option<u8> {
    // In AirPods protocol, 0xFF is used for unknown battery level
    if value == 0xFF {
        None
    } else {
        // Battery levels in AirPods protocol go from 0 to 10
        // We convert to percentage (0-100)
        Some(value.min(10) * 10)
    }
}

/// Filter function type for AirPods detection
pub type AirPodsFilter = Box<dyn Fn(&DiscoveredDevice) -> bool + Send + Sync>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::time::Instant;
    
    fn create_test_device(manufacturer_data: HashMap<u16, Vec<u8>>, is_potential_airpods: bool) -> DiscoveredDevice {
        DiscoveredDevice {
            address: BDAddr::default(),
            name: None,
            rssi: None,
            manufacturer_data,
            is_potential_airpods,
            last_seen: std::time::Instant::now(),
            is_connected: false,
            service_data: HashMap::new(),
            services: Vec::new(),
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
        assert!(matches!(identify_airpods_type(&Some("AirPods 1".to_string()), &[0x07, 0x19, 0x01]), AirPodsType::AirPods1 | AirPodsType::AirPods2));
        assert_eq!(identify_airpods_type(&Some("AirPods Pro".to_string()), &[0x0E, 0x19, 0x01]), AirPodsType::AirPodsPro);
        assert_eq!(identify_airpods_type(&Some("AirPods Pro 2".to_string()), &[0x0F, 0x19, 0x01]), AirPodsType::AirPodsPro2);
        assert_eq!(identify_airpods_type(&Some("AirPods 3".to_string()), &[0x13, 0x19, 0x01]), AirPodsType::AirPods3);
        assert_eq!(identify_airpods_type(&Some("AirPods Max".to_string()), &[0x0A, 0x19, 0x01]), AirPodsType::AirPodsMax);
        assert_eq!(identify_airpods_type(&Some("Unknown Device".to_string()), &[0xFF, 0xFF, 0x01]), AirPodsType::Unknown);
        
        // Test with data that's too short
        assert_eq!(identify_airpods_type(&Some("".to_string()), &[0x01]), AirPodsType::Unknown);
        assert_eq!(identify_airpods_type(&None, &[]), AirPodsType::Unknown);
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
        let custom_filter = create_custom_airpods_filter(Some(-55), None);
        
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