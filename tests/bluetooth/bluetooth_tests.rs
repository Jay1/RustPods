//! Tests for AirPods device and battery info model (post-refactor)
//! Updated for native C++ AirPods battery helper and new state/message model

use btleplug::api::BDAddr;
use rustpods::airpods::{AirPodsBattery, AirPodsChargingState};
use rustpods::bluetooth::DiscoveredDevice;
use std::collections::HashMap;
use std::time::Instant;

/// Test helper to create a sample discovered device
fn create_test_device(
    address: [u8; 6],
    name: Option<&str>,
    rssi: Option<i16>,
    is_airpods: bool,
) -> DiscoveredDevice {
    let manufacturer_data = HashMap::new();
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

#[test]
fn test_airpods_battery_struct() {
    let battery = AirPodsBattery {
        left: Some(80),
        right: Some(70),
        case: Some(60),
        charging: Some(AirPodsChargingState::LeftCharging),
    };
    assert_eq!(battery.left, Some(80));
    assert_eq!(battery.right, Some(70));
    assert_eq!(battery.case, Some(60));
    assert_eq!(battery.charging, Some(AirPodsChargingState::LeftCharging));
}

#[test]
fn test_discovered_device_fields() {
    let device = create_test_device([1, 2, 3, 4, 5, 6], Some("AirPods Pro"), Some(-55), true);
    assert_eq!(device.address, BDAddr::from([1, 2, 3, 4, 5, 6]));
    assert_eq!(device.name.as_deref(), Some("AirPods Pro"));
    assert_eq!(device.rssi, Some(-55));
    assert!(device.is_potential_airpods);
}
