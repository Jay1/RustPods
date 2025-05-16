#[cfg(test)]
mod tests {
    use super::*;
    use btleplug::api::BDAddr;
    use std::collections::HashMap;
    use std::time::Instant;
    use crate::ui::components::device_list::DeviceList;
    use crate::bluetooth::DiscoveredDevice;

    #[test]
    fn test_device_list_creation() {
        // Create some test devices
        let device1 = DiscoveredDevice {
            address: BDAddr::from([0x11, 0x22, 0x33, 0x44, 0x55, 0x66]),
            name: Some("AirPods Pro".to_string()),
            rssi: Some(-45),
            manufacturer_data: {
                let mut map = HashMap::new();
                map.insert(76, vec![1, 2, 3, 4, 5]); // Apple manufacturer ID with some data
                map
            },
            is_potential_airpods: true,
            last_seen: Instant::now(),
            is_connected: false,
            service_data: HashMap::new(),
            services: Vec::new(),
            tx_power_level: None,
        };

        let device2 = DiscoveredDevice {
            address: BDAddr::from([0x22, 0x33, 0x44, 0x55, 0x66, 0x77]),
            name: Some("Bluetooth Speaker".to_string()),
            rssi: Some(-60),
            manufacturer_data: HashMap::new(),
            is_potential_airpods: false,
            last_seen: Instant::now(),
            is_connected: false,
            service_data: HashMap::new(),
            services: Vec::new(),
            tx_power_level: None,
        };

        let devices = vec![device1, device2];
        let selected = Some(BDAddr::from([6, 5, 4, 3, 2, 1]).to_string());

        // Create the device list component
        let device_list = DeviceList::new(devices, selected.clone());

        // Verify device count
        assert_eq!(device_list.devices.len(), 2);
        
        // Verify selected device
        assert_eq!(device_list.selected, selected);
    }
} 