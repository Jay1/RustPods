use std::collections::HashMap;
use std::time::Instant;
use btleplug::api::{BDAddr, Central, CentralEvent};

// Import from rustpods rather than crate
use rustpods::bluetooth::{DiscoveredDevice, ScanConfig};
use rustpods::BleScanner;

/// Helper to create a test device with manufacturer data
pub fn create_test_device_with_data(
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
    }
}

/// Helper to create a simple test device without manufacturer data
pub fn create_test_device(
    address: [u8; 6],
    name: Option<&str>,
    rssi: Option<i16>,
) -> DiscoveredDevice {
    create_test_device_with_data(
        address,
        name,
        rssi,
        HashMap::new(),
    )
}

/// Helper to create test AirPods manufacturer data with the correct format
pub fn create_airpods_data(
    prefix: &[u8],
    left_battery: u8,
    right_battery: u8,
    case_battery: u8,
    charging_status: u8
) -> Vec<u8> {
    // The data structure must exactly match what's expected in parse_airpods_data
    let mut data = Vec::with_capacity(27);
    
    // AirPods model prefix (first two bytes)
    data.push(prefix[0]);
    data.push(prefix[1]);
    
    // Add 10 bytes of dummy data
    data.extend_from_slice(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A]);
    
    // Add battery and status data
    data.push(left_battery);
    data.push(right_battery);
    data.push(charging_status);
    data.push(case_battery);
    
    // Add padding
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    
    data
}

// Constants used in tests
pub const APPLE_COMPANY_ID: u16 = 0x004C;
pub const AIRPODS_PRO_PREFIX: [u8; 2] = [0x0E, 0x20];
pub const AIRPODS_GEN2_PREFIX: [u8; 2] = [0x0F, 0x20];
pub const AIRPODS_MAX_PREFIX: [u8; 2] = [0x0A, 0x20]; 