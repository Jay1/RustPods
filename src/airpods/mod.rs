//! AirPods-specific functionality

mod detector;
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

/// AirPods battery status
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
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

/// Parse AirPods manufacturer data to extract battery and charging information
pub fn parse_airpods_data(data: &[u8]) -> Option<AirPodsBattery> {
    // Ensure we have enough data for parsing
    if data.len() < 16 {
        return None;
    }
    
    // For most AirPods, battery info is in these offsets
    // However, different models might have slight variations
    
    // The status byte containing charging information
    // Bit 0: case charging, Bit 1: right pod charging, Bit 2: left pod charging
    let charging_flags = if data.len() > 14 { data[14] } else { 0 };
    
    // Extract charging status information
    let charging = ChargingStatus {
        left: (charging_flags & 0x04) != 0,
        right: (charging_flags & 0x02) != 0,
        case: (charging_flags & 0x01) != 0,
    };
    
    // Extract battery levels
    // AirPods typically report battery levels from 0-10
    // We convert to percentage by multiplying by 10
    let left_battery = if data.len() > 12 {
        extract_battery_percentage(data[12])
    } else {
        None
    };
    
    let right_battery = if data.len() > 13 {
        extract_battery_percentage(data[13])
    } else {
        None
    };
    
    let case_battery = if data.len() > 15 {
        extract_battery_percentage(data[15])
    } else {
        None
    };
    
    Some(AirPodsBattery {
        left: left_battery,
        right: right_battery,
        case: case_battery,
        charging,
    })
}

/// Helper function to convert AirPods battery value to percentage
fn extract_battery_percentage(value: u8) -> Option<u8> {
    if value == 0xFF {
        None  // 0xFF means unknown/unavailable
    } else {
        // Cap at 10 (which is 100%) in case we get invalid values
        Some((value.min(10) * 10) as u8)
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
        assert_eq!(extract_battery_percentage(0), Some(0));
        assert_eq!(extract_battery_percentage(5), Some(50));
        assert_eq!(extract_battery_percentage(10), Some(100));
        assert_eq!(extract_battery_percentage(15), Some(100)); // Capping at 100%
        assert_eq!(extract_battery_percentage(0xFF), None);    // Unknown value
    }
} 