//! Mock implementations for Bluetooth components (post-refactor)
//!
//! This module provides mock implementations of Bluetooth device polling, paired device management,
//! AirPods battery info, and event/message handling for use in headless testing environments
//! without requiring actual hardware. BLE scanning and scan logic are no longer supported or mocked.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use btleplug::api::BDAddr;

use rustpods::airpods::AirPodsType;
use rustpods::bluetooth::{AirPodsBatteryStatus, DiscoveredDevice};

/// Mock device that stores additional testing details
#[derive(Debug, Clone)]
pub struct MockDevice {
    /// Device address
    pub address: String,
    /// Device name
    pub name: Option<String>,
    /// Signal strength
    pub rssi: Option<i16>,
    /// Manufacturer data
    pub manufacturer_data: HashMap<u16, Vec<u8>>,
    /// Is the device connected
    pub is_connected: bool,
    /// Is the device an AirPods
    pub is_airpods: bool,
    /// AirPods battery status if applicable
    pub battery_status: Option<AirPodsBatteryStatus>,
    /// AirPods type if applicable
    pub airpods_type: Option<AirPodsType>,
    /// Last seen time
    pub last_seen: Instant,
    /// Connection attempts
    pub connection_attempts: usize,
}

/// Mock Bluetooth device poller for the new architecture
#[derive(Debug, Clone)]
pub struct MockDevicePoller {
    /// Known devices
    pub devices: Arc<Mutex<HashMap<String, MockDevice>>>,
}

impl Default for MockDevicePoller {
    fn default() -> Self {
        Self::new()
    }
}

impl MockDevicePoller {
    /// Create a new mock device poller
    pub fn new() -> Self {
        Self {
            devices: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Add a test device
    pub fn add_device(&self, device: MockDevice) {
        let mut devices = self.devices.lock().unwrap();
        devices.insert(device.address.clone(), device);
    }

    /// Poll for paired devices (simulates helper script output)
    pub fn poll_paired_devices(&self) -> Vec<DiscoveredDevice> {
        let devices = self.devices.lock().unwrap();
        devices
            .values()
            .map(|mock| DiscoveredDevice {
                address: BDAddr::from_str_hex(&mock.address).unwrap_or(BDAddr::default()),
                name: mock.name.clone(),
                rssi: mock.rssi,
                manufacturer_data: mock.manufacturer_data.clone(),
                is_potential_airpods: mock.is_airpods,
                last_seen: mock.last_seen,
                is_connected: mock.is_connected,
                service_data: HashMap::new(),
                services: Vec::new(),
                tx_power_level: None,
            })
            .collect()
    }

    /// Get AirPods battery info for a device
    pub fn get_airpods_battery(&self, address: &str) -> Option<AirPodsBatteryStatus> {
        let devices = self.devices.lock().unwrap();
        devices
            .get(address)
            .and_then(|mock| mock.battery_status.clone())
    }
}

/// Helper trait for BDAddr conversion
trait BDAddrExt {
    fn from_str_hex(s: &str) -> Option<Self>
    where
        Self: Sized;
}

impl BDAddrExt for BDAddr {
    fn from_str_hex(s: &str) -> Option<Self> {
        let parts: Vec<_> = s.split(':').collect();
        if parts.len() != 6 {
            return None;
        }
        let mut bytes = [0u8; 6];
        for (i, part) in parts.iter().enumerate() {
            bytes[i] = u8::from_str_radix(part, 16).ok()?;
        }
        Some(BDAddr::from(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rustpods::airpods::{AirPodsBattery, AirPodsChargingState};

    #[test]
    fn test_mock_device_poller_paired_devices() {
        let poller = MockDevicePoller::new();
        let device = MockDevice {
            address: "11:22:33:44:55:66".to_string(),
            name: Some("Test AirPods".to_string()),
            rssi: Some(-60),
            manufacturer_data: HashMap::new(),
            is_connected: true,
            is_airpods: true,
            battery_status: Some(AirPodsBatteryStatus {
                battery: AirPodsBattery {
                    left: Some(80),
                    right: Some(90),
                    case: Some(100),
                    charging: Some(AirPodsChargingState::CaseCharging),
                },
                last_updated: Instant::now(),
            }),
            airpods_type: None,
            last_seen: Instant::now(),
            connection_attempts: 0,
        };
        poller.add_device(device);
        let paired = poller.poll_paired_devices();
        assert_eq!(paired.len(), 1);
        assert_eq!(paired[0].name.as_deref(), Some("Test AirPods"));
    }

    #[test]
    fn test_mock_device_poller_battery_info() {
        let poller = MockDevicePoller::new();
        let device = MockDevice {
            address: "AA:BB:CC:DD:EE:FF".to_string(),
            name: Some("AirPods Pro".to_string()),
            rssi: Some(-55),
            manufacturer_data: HashMap::new(),
            is_connected: true,
            is_airpods: true,
            battery_status: Some(AirPodsBatteryStatus {
                battery: AirPodsBattery {
                    left: Some(100),
                    right: Some(100),
                    case: Some(90),
                    charging: Some(AirPodsChargingState::CaseCharging),
                },
                last_updated: Instant::now(),
            }),
            airpods_type: None,
            last_seen: Instant::now(),
            connection_attempts: 0,
        };
        poller.add_device(device);
        let battery = poller.get_airpods_battery("AA:BB:CC:DD:EE:FF");
        assert!(battery.is_some());
        assert_eq!(battery.unwrap().battery.left, Some(100));
    }
}
