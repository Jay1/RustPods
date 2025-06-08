//! Common test utilities for Bluetooth tests

use btleplug::api::BDAddr;
use std::collections::HashMap;
use std::time::Instant;

use rustpods::airpods::{
    AirPodsBattery, AirPodsChargingState, AirPodsType, DetectedAirPods, APPLE_COMPANY_ID,
};
use rustpods::bluetooth::DiscoveredDevice;

/// Create a test device with basic properties
pub fn create_test_device(
    address: [u8; 6],
    name: Option<&str>,
    rssi: Option<i16>,
    is_airpods: bool,
    prefix: Option<&[u8]>,
    has_battery: bool,
) -> DiscoveredDevice {
    let mut manufacturer_data = HashMap::new();

    if is_airpods {
        // Use the prefix to determine the type of AirPods
        let airpods_data = if let Some(prefix) = prefix {
            // Create mock AirPods data with the given prefix and battery info if needed
            let mut data = Vec::new();
            data.extend_from_slice(prefix);

            // Add dummy data for the middle section
            data.extend_from_slice(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A]);

            // Add battery data if requested
            if has_battery {
                data.extend_from_slice(&[0x08, 0x06, 0x05, 0x07]); // Left, Right, Status, Case
            } else {
                data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // No battery info
            }

            // Add padding
            data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

            data
        } else {
            // Generic AirPods data
            vec![
                0x07, 0x19, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C,
                0x0D, 0x0E,
            ]
        };

        manufacturer_data.insert(APPLE_COMPANY_ID, airpods_data);
    }

    DiscoveredDevice {
        address: BDAddr::from(address),
        name: name.map(|s| s.to_string()),
        rssi,
        manufacturer_data,
        is_potential_airpods: is_airpods,
        last_seen: Instant::now(),
        is_connected: false,
        service_data: HashMap::new(),
        services: Vec::new(),
        tx_power_level: None,
    }
}

/// Create a test AirPods instance with configurable properties
pub fn create_test_airpods(
    address: [u8; 6],
    name: Option<&str>,
    device_type: AirPodsType,
    left_battery: Option<u8>,
    right_battery: Option<u8>,
    case_battery: Option<u8>,
    charging: Option<AirPodsChargingState>,
) -> DetectedAirPods {
    DetectedAirPods {
        address: BDAddr::from(address),
        name: name.map(|s| s.to_string()),
        device_type,
        battery: Some(AirPodsBattery {
            left: left_battery,
            right: right_battery,
            case: case_battery,
            charging,
        }),
        rssi: Some(-60),
        last_seen: Instant::now(),
        is_connected: false,
    }
}

/// Create AirPods manufacturer data with specific battery levels and charging status
pub fn create_airpods_manufacturer_data(
    model_prefix: &[u8; 2],
    left_battery: u8,
    right_battery: u8,
    case_battery: u8,
    charging_flags: u8,
) -> HashMap<u16, Vec<u8>> {
    let mut data = Vec::with_capacity(27);

    // Add model prefix (first two bytes)
    data.push(model_prefix[0]);
    data.push(model_prefix[1]);

    // Add dummy data (10 bytes)
    data.extend_from_slice(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A]);

    // Add battery and charging status
    data.push(left_battery);
    data.push(right_battery);
    data.push(charging_flags);
    data.push(case_battery);

    // Add padding
    data.extend_from_slice(&[0; 11]);

    let mut result = HashMap::new();
    result.insert(APPLE_COMPANY_ID, data); // Apple company ID
    result
}

/// Check if the current test environment can run Bluetooth tests
pub fn should_skip_bluetooth_test() -> bool {
    // Skip if running in CI
    if std::env::var("CI").is_ok() {
        return true;
    }

    // Skip if explicitly requested
    if std::env::var("SKIP_BLUETOOTH_TESTS").is_ok() {
        return true;
    }

    false
}

// Constants used in tests
pub const AIRPODS_PRO_PREFIX: [u8; 2] = [0x0E, 0x20];
pub const AIRPODS_GEN2_PREFIX: [u8; 2] = [0x0F, 0x20];
pub const AIRPODS_MAX_PREFIX: [u8; 2] = [0x0A, 0x20];
