//! AirPods-specific functionality

pub mod detector;
mod filter;

pub use detector::{
    DetectedAirPods, detect_airpods, 
    identify_airpods_type, create_airpods_filter,
    create_custom_airpods_filter
};

pub use filter::{
    AirPodsFilter, 
    airpods_all_models_filter,
    airpods_pro_filter,
    airpods_nearby_filter,
    airpods_with_battery_filter,
    APPLE_COMPANY_ID
};

use serde::{Serialize, Deserialize};

/// AirPods device types
#[derive(Debug, Clone, PartialEq)]
pub enum AirPodsType {
    /// Original AirPods
    AirPods1,
    /// AirPods 2nd generation
    AirPods2,
    /// AirPods 3rd generation
    AirPods3,
    /// AirPods Pro
    AirPodsPro,
    /// AirPods Pro 2nd generation
    AirPodsPro2,
    /// AirPods Max
    AirPodsMax,
    /// Unknown AirPods type
    Unknown,
}

impl AirPodsType {
    /// Detect AirPods model based on device name
    pub fn detect_from_name(name: &str) -> Self {
        let name_lower = name.to_lowercase();
        
        if name_lower.contains("airpods pro") {
            if name_lower.contains("2") || name_lower.contains("second") {
                Self::AirPodsPro2
            } else {
                Self::AirPodsPro
            }
        } else if name_lower.contains("airpods max") {
            Self::AirPodsMax
        } else if name_lower.contains("airpods") {
            if name_lower.contains("3") || name_lower.contains("third") {
                Self::AirPods3
            } else if name_lower.contains("2") || name_lower.contains("second") {
                Self::AirPods2
            } else {
                Self::AirPods1
            }
        } else {
            Self::Unknown
        }
    }
}

/// AirPods battery status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AirPodsBattery {
    /// Left earbud battery level (0-100)
    pub left: Option<u8>,
    /// Right earbud battery level (0-100)
    pub right: Option<u8>,
    /// Case battery level (0-100)
    pub case: Option<u8>,
    /// Charging status
    pub charging: ChargingStatus,
}

/// Charging status for AirPods
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChargingStatus {
    /// Left earbud charging status
    pub left: bool,
    /// Right earbud charging status
    pub right: bool,
    /// Case charging status
    pub case: bool,
}

impl ChargingStatus {
    /// Check if any component is charging
    pub fn is_any_charging(&self) -> bool {
        self.left || self.right || self.case
    }
}

impl Default for AirPodsBattery {
    fn default() -> Self {
        Self {
            left: None,
            right: None,
            case: None,
            charging: ChargingStatus {
                left: false,
                right: false,
                case: false,
            },
        }
    }
}

/// Parse AirPods data from manufacturer data bytes
pub fn parse_airpods_data(data: &[u8]) -> Option<AirPodsBattery> {
    // Minimum data length check
    if data.len() < 16 {
        return None;
    }
    
    // Extract battery levels
    // AirPods protocol: Each battery level is 0-10 (multiply by 10 for percentage)
    // 0xFF means unknown/unavailable
    let left_battery = extract_battery_level(data[12]);
    let right_battery = extract_battery_level(data[13]);
    let case_battery = extract_battery_level(data[15]);
    
    // Extract charging status from byte 14
    // Bit 0 (LSB): Case charging
    // Bit 1: Right AirPod charging
    // Bit 2: Left AirPod charging
    let charging_flags = data[14];
    let charging = ChargingStatus {
        left: (charging_flags & 0b100) != 0,
        right: (charging_flags & 0b010) != 0,
        case: (charging_flags & 0b001) != 0,
    };
    
    Some(AirPodsBattery {
        left: left_battery,
        right: right_battery,
        case: case_battery,
        charging,
    })
}

/// Extract battery level from AirPods data
fn extract_battery_level(value: u8) -> Option<u8> {
    // 0xFF = unknown/unavailable
    if value == 0xFF {
        None
    } else {
        // Convert 0-10 scale to 0-100 percentage
        Some(value.min(10) * 10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_airpods_battery_default() {
        let battery = AirPodsBattery::default();
        assert_eq!(battery.left, None);
        assert_eq!(battery.right, None);
        assert_eq!(battery.case, None);
        assert!(!battery.charging.left);
        assert!(!battery.charging.right);
        assert!(!battery.charging.case);
    }
    
    #[test]
    fn test_parse_airpods_data_empty() {
        let data = vec![1, 2, 3];
        let result = parse_airpods_data(&data);
        assert!(result.is_none());
    }
    
    #[test]
    fn test_parse_airpods_data_with_valid_structure() {
        // Mock AirPods data with valid structure
        let data = vec![
            0x07, 0x19, // AirPods identifier
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B,
            0x0A, // Left battery (100%)
            0x0A, // Right battery (100%)
            0x02, // Charging status: 0x02 = 0b0010 (right charging)
            0x02, // Case battery (20%)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ];
        
        let result = parse_airpods_data(&data);
        assert!(result.is_some());
        
        if let Some(battery) = result {
            println!("Left battery: {:?}", battery.left);
            println!("Right battery: {:?}", battery.right);
            println!("Case battery: {:?}", battery.case);
            println!("Left charging: {}", battery.charging.left);
            println!("Right charging: {}", battery.charging.right);
            println!("Case charging: {}", battery.charging.case);
            
            assert_eq!(battery.left, Some(100), "Left battery should be 100%");
            assert_eq!(battery.right, Some(100), "Right battery should be 100%");
            assert_eq!(battery.case, Some(20), "Case battery should be 20%");
            assert!(!battery.charging.left, "Left pod should not be charging");
            assert!(battery.charging.right, "Right pod should be charging");
            assert!(!battery.charging.case, "Case should not be charging");
        }
    }
    
    #[test]
    fn test_extract_battery_percentage() {
        assert_eq!(extract_battery_level(0), Some(0));
        assert_eq!(extract_battery_level(5), Some(50));
        assert_eq!(extract_battery_level(10), Some(100));
        assert_eq!(extract_battery_level(15), Some(100)); // Capping at 100%
        assert_eq!(extract_battery_level(0xFF), None);    // Unknown value
    }
} 