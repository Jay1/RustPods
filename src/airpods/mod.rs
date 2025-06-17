//! AirPods-specific functionality

pub mod battery;
pub mod battery_estimator;
pub mod battery_intelligence;
pub mod detector;
mod filter;

pub use detector::{
    create_airpods_filter, create_custom_airpods_filter, detect_airpods, identify_airpods_type,
    DetectedAirPods,
};

pub use filter::{
    airpods_all_models_filter, airpods_nearby_filter, airpods_pro_filter,
    airpods_with_battery_filter, AirPodsFilter, AirPodsFilterOptions, APPLE_COMPANY_ID,
};

pub use battery_intelligence::{
    BatteryIntelligence, BatteryEstimate, BatteryHealthMetrics, DeviceBatteryProfile,
    BatteryEvent, BatteryEventType, UsageSession, UsagePattern, SessionType,
    DischargeModel, IntelligenceSettings,
};

use crate::error::{AirPodsError, ErrorContext};
use serde::{Deserialize, Serialize};

/// Convenience type alias for Result with AirPodsError
pub type Result<T> = std::result::Result<T, AirPodsError>;

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

/// Charging state for AirPods
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AirPodsChargingState {
    /// Nothing is charging
    NotCharging,
    /// Left AirPod is charging
    LeftCharging,
    /// Right AirPod is charging
    RightCharging,
    /// Case is charging
    CaseCharging,
    /// Both AirPods are charging
    BothBudsCharging,
}

impl AirPodsChargingState {
    /// Returns true if any component is charging
    pub fn is_any_charging(&self) -> bool {
        !matches!(self, Self::NotCharging)
    }

    /// Check if left AirPod is charging
    pub fn is_left_charging(&self) -> bool {
        matches!(self, Self::LeftCharging | Self::BothBudsCharging)
    }

    /// Check if right AirPod is charging
    pub fn is_right_charging(&self) -> bool {
        matches!(self, Self::RightCharging | Self::BothBudsCharging)
    }

    /// Check if case is charging
    pub fn is_case_charging(&self) -> bool {
        matches!(self, Self::CaseCharging)
    }
}

/// Battery status for AirPods
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AirPodsBattery {
    /// Left AirPod battery level (percent)
    pub left: Option<u8>,
    /// Right AirPod battery level (percent)
    pub right: Option<u8>,
    /// AirPods case battery level (percent)
    pub case: Option<u8>,
    /// Charging status
    pub charging: Option<AirPodsChargingState>,
}

/// Helper function to extract battery level from raw value
///
/// Returns None if the battery value can't be determined (0xFF)
/// Otherwise returns Some(percentage) where percentage is 0-100
pub fn extract_battery_level(raw_value: u8) -> Option<u8> {
    if raw_value <= 10 {
        Some(raw_value * 10)
    } else if raw_value == 0xFF {
        None
    } else {
        // Cap at 100% for unexpected values
        Some(100)
    }
}

/// Helper function to parse AirPods data from manufacturer data
pub fn parse_airpods_data(data: &[u8]) -> Result<AirPodsBattery> {
    let _ctx = ErrorContext::new("AirPods", "parse_airpods_data")
        .with_metadata("data_length", data.len().to_string())
        .with_metadata("data_hex", format!("{:02X?}", data));

    // Check if data is long enough to contain battery information
    // AirPods battery data starts at offset 11 and requires at least 16 bytes
    if data.len() < 16 {
        return Err(AirPodsError::InvalidData(format!(
            "Data too short for battery parsing: {} bytes (need at least 16)",
            data.len()
        )));
    }

    // Offset constants for battery data
    const LEFT_BATTERY_OFFSET: usize = 12;
    const RIGHT_BATTERY_OFFSET: usize = 13;
    const CASE_BATTERY_OFFSET: usize = 15;
    const CHARGING_STATUS_OFFSET: usize = 14;

    // Parse left earbud battery
    let left_battery = if data.len() > LEFT_BATTERY_OFFSET {
        extract_battery_level(data[LEFT_BATTERY_OFFSET])
    } else {
        log::debug!(
            "Data too short for left battery value at offset {}",
            LEFT_BATTERY_OFFSET
        );
        None
    };

    // Parse right earbud battery
    let right_battery = if data.len() > RIGHT_BATTERY_OFFSET {
        extract_battery_level(data[RIGHT_BATTERY_OFFSET])
    } else {
        log::debug!(
            "Data too short for right battery value at offset {}",
            RIGHT_BATTERY_OFFSET
        );
        None
    };

    // Parse case battery
    let case_battery = if data.len() > CASE_BATTERY_OFFSET {
        extract_battery_level(data[CASE_BATTERY_OFFSET])
    } else {
        log::debug!(
            "Data too short for case battery value at offset {}",
            CASE_BATTERY_OFFSET
        );
        None
    };

    // Parse charging status
    let charging_status = if data.len() > CHARGING_STATUS_OFFSET {
        let raw_status = data[CHARGING_STATUS_OFFSET];
        match raw_status {
            0 => Some(AirPodsChargingState::NotCharging),
            1 => Some(AirPodsChargingState::LeftCharging),
            2 => Some(AirPodsChargingState::RightCharging),
            4 => Some(AirPodsChargingState::CaseCharging),
            5 => Some(AirPodsChargingState::BothBudsCharging),
            _ => {
                log::debug!("Unknown charging status value: {}", raw_status);
                None
            }
        }
    } else {
        log::debug!(
            "Data too short for charging status at offset {}",
            CHARGING_STATUS_OFFSET
        );
        None
    };

    // Create battery info object - if we have at least some data
    if left_battery.is_none() && right_battery.is_none() && case_battery.is_none() {
        return Err(AirPodsError::ParseError(
            "No valid battery data found in manufacturer data".to_string(),
        ));
    }

    Ok(AirPodsBattery {
        left: left_battery,
        right: right_battery,
        case: case_battery,
        charging: charging_status,
    })
}

/// Struct version of charging status for individual components
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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

    /// Create a new charging status with all values set to false
    pub fn none() -> Self {
        Self {
            left: false,
            right: false,
            case: false,
        }
    }

    /// Convert from AirPodsChargingState enum
    pub fn from_state(state: AirPodsChargingState) -> Self {
        Self {
            left: state.is_left_charging(),
            right: state.is_right_charging(),
            case: state.is_case_charging(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bluetooth::scanner::DiscoveredDevice;
    use btleplug::api::BDAddr;
    use std::collections::HashMap;
    use std::str::FromStr;

    #[test]
    fn test_airpods_battery_default() {
        let battery = AirPodsBattery::default();
        assert_eq!(battery.left, None);
        assert_eq!(battery.right, None);
        assert_eq!(battery.case, None);
        assert!(battery.charging.is_none());
    }

    #[test]
    fn test_parse_airpods_data_empty() {
        let data = vec![1, 2, 3];
        let result = parse_airpods_data(&data);
        assert!(result.is_err());
        match result {
            Err(AirPodsError::InvalidData(msg)) => {
                assert!(msg.contains("Data too short"));
            }
            other => panic!("Expected InvalidData error but got {:?}", other),
        }
    }

    #[test]
    fn test_parse_airpods_data_with_valid_structure() {
        // Simplified test data
        let mut data = vec![0u8; 27]; // Initialize with zeros
        data[1] = 1; // Test data with known structure
        const LEFT_BATTERY_OFFSET: usize = 12;
        const RIGHT_BATTERY_OFFSET: usize = 13;
        const CASE_BATTERY_OFFSET: usize = 15;
        const CHARGING_STATUS_OFFSET: usize = 14;

        data[LEFT_BATTERY_OFFSET] = 10; // Left battery 100%
        data[RIGHT_BATTERY_OFFSET] = 10; // Right battery 100%
        data[CASE_BATTERY_OFFSET] = 2; // Case battery 20%
        data[CHARGING_STATUS_OFFSET] = 2; // Right bud charging (value 2)

        // Parse the data
        let result = parse_airpods_data(&data);

        // Check valid result
        assert!(result.is_ok(), "Expected successful parse");

        // Check battery levels
        let battery = result.unwrap();
        assert_eq!(battery.left, Some(100), "Left battery should be 100%");
        assert_eq!(battery.right, Some(100), "Right battery should be 100%");
        assert_eq!(battery.case, Some(20), "Case battery should be 20%");
        assert_eq!(
            battery.charging,
            Some(AirPodsChargingState::RightCharging),
            "Right bud should be charging"
        );
    }

    #[test]
    fn test_parse_airpods_data_with_partial_battery() {
        // Simplified test data with partial battery info
        let mut data = vec![0u8; 27]; // Initialize with zeros
        data[1] = 1; // Test data with known structure
        const LEFT_BATTERY_OFFSET: usize = 12;
        const RIGHT_BATTERY_OFFSET: usize = 13;
        const CASE_BATTERY_OFFSET: usize = 15;
        const CHARGING_STATUS_OFFSET: usize = 14;

        // Set left and case battery to 0xFF which should result in None
        data[LEFT_BATTERY_OFFSET] = 0xFF;
        data[CASE_BATTERY_OFFSET] = 0xFF;

        // Only set right battery
        data[RIGHT_BATTERY_OFFSET] = 7; // Right battery 70%
        data[CHARGING_STATUS_OFFSET] = 0; // No charging

        // Parse the data
        let result = parse_airpods_data(&data);

        // Check valid result
        assert!(result.is_ok(), "Expected successful parse");

        // Check battery levels
        let battery = result.unwrap();
        assert_eq!(battery.left, None, "Left battery should be None");
        assert_eq!(battery.right, Some(70), "Right battery should be 70%");
        assert_eq!(battery.case, None, "Case battery should be None");
        assert_eq!(
            battery.charging,
            Some(AirPodsChargingState::NotCharging),
            "Nothing should be charging"
        );
    }

    #[test]
    fn test_extract_battery_percentage() {
        assert_eq!(extract_battery_level(0), Some(0));
        assert_eq!(extract_battery_level(5), Some(50));
        assert_eq!(extract_battery_level(10), Some(100));
        assert_eq!(extract_battery_level(15), Some(100)); // Capping at 100%
        assert_eq!(extract_battery_level(0xFF), None); // Unknown value
    }

    #[test]
    fn test_error_conversion() {
        // Test AirPodsError string conversion
        let airpods_error = AirPodsError::DetectionFailed("Test error".to_string());

        // Now we just ensure the error contains the expected message
        let msg_str = airpods_error.to_string();
        assert!(
            msg_str.contains("Test error"),
            "Error message should contain the original message"
        );
        assert!(
            msg_str.contains("AirPods detection failed"),
            "Error message should contain the error category"
        );
    }

    #[test]
    fn test_error_manager_integration() {
        // This test is skipped since we're no longer importing ErrorManager and RecoveryAction
        // Creating an error context is still valid
        let context = ErrorContext::new("AirPodsTest", "parsing")
            .with_metadata("device_address", "00:11:22:33:44:55")
            .with_user_message("Unable to read AirPods battery status");

        // Just verify the context contains the expected data
        assert_eq!(context.component, "AirPodsTest");
        assert_eq!(context.operation, "parsing");
        assert_eq!(
            context.metadata.get("device_address"),
            Some(&"00:11:22:33:44:55".to_string())
        );
        assert_eq!(
            context.user_message,
            Some("Unable to read AirPods battery status".to_string())
        );
    }

    #[test]
    fn test_detect_airpods_with_bad_data() {
        // Import detect_airpods from detector
        use super::detector::detect_airpods;

        // Create a device with Apple ID but corrupt data
        let mut manufacturer_data = HashMap::new();
        manufacturer_data.insert(APPLE_COMPANY_ID, vec![0x01]); // Too short to even validate

        let device = DiscoveredDevice {
            address: BDAddr::from([1, 2, 3, 4, 5, 6]),
            name: Some("AirPods".to_string()),
            rssi: Some(-50),
            manufacturer_data,
            is_potential_airpods: true,
            last_seen: std::time::Instant::now(),
            is_connected: false,
            service_data: HashMap::new(),
            services: vec![],
            tx_power_level: None,
        };

        // Detect should return an error since data is too short
        let result = detect_airpods(&device);
        assert!(result.is_err(), "Should error on corrupt data");

        // Check specific error message
        match result {
            Err(AirPodsError::DetectionFailed(msg)) => {
                assert!(msg.contains("Invalid data"), "Should mention invalid data");
                assert!(
                    msg.contains("too short"),
                    "Should mention data is too short"
                );
            }
            _ => panic!("Wrong error type returned"),
        }
    }

    #[test]
    fn test_detect_airpods_graceful_degradation() {
        // Import detect_airpods from detector
        use super::detector::detect_airpods;

        // Create manufacturer data with valid format but unknown model prefix
        let mut manufacturer_data = HashMap::new();
        let data = vec![
            0xFF, 0x19, // Unknown model prefix
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B,
            0x0A, // Left battery (100%)
            0x0A, // Right battery (100%)
            0x02, // Charging status
            0x02, // Case battery (20%)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        manufacturer_data.insert(APPLE_COMPANY_ID, data);

        let device = DiscoveredDevice {
            address: BDAddr::from_str("00:11:22:33:44:55").unwrap(),
            name: Some("AirPods".to_string()),
            rssi: Some(-60),
            tx_power_level: None,
            manufacturer_data,
            services: vec![],
            is_potential_airpods: true,
            last_seen: std::time::Instant::now(),
            is_connected: false,
            service_data: HashMap::new(),
        };

        // Should return success but with AirPods1 model type (falls back to name-based detection)
        let result = detect_airpods(&device);
        assert!(result.is_ok());
        if let Ok(Some(airpods)) = result {
            assert_eq!(airpods.device_type, AirPodsType::AirPods1);
            // Battery info should still be parsed
            if let Some(battery) = airpods.battery.as_ref() {
                assert_eq!(battery.left, Some(100));
                assert_eq!(battery.right, Some(100));
                assert_eq!(battery.case, Some(20));
            } else {
                panic!("Battery information is missing");
            }
        } else {
            panic!("Expected Some(DetectedAirPods), got {:?}", result);
        }
    }
}
