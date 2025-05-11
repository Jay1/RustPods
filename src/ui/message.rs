use crate::bluetooth::DiscoveredDevice;

/// Messages that can be sent to update the UI state
#[derive(Debug, Clone)]
pub enum Message {
    /// Toggle the application window visibility
    ToggleVisibility,
    
    /// Exit the application
    Exit,
    
    /// New Bluetooth device discovered
    DeviceDiscovered(DiscoveredDevice),
    
    /// Device updated (battery level, etc.)
    DeviceUpdated(DiscoveredDevice),
    
    /// Select a device to connect to
    SelectDevice(String),
    
    /// Start scanning for devices
    StartScan,
    
    /// Stop scanning for devices
    StopScan,
    
    /// Toggle automatic scanning
    ToggleAutoScan(bool),
    
    /// Tick event for periodic updates
    Tick,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bluetooth::DiscoveredDevice;
    use btleplug::api::BDAddr;
    use std::collections::HashMap;
    use std::time::Instant;
    
    #[test]
    fn test_message_creation() {
        // Test creating different message types
        let toggle_msg = Message::ToggleVisibility;
        let exit_msg = Message::Exit;
        let tick_msg = Message::Tick;
        
        // Check they can be created without errors
        assert!(matches!(toggle_msg, Message::ToggleVisibility));
        assert!(matches!(exit_msg, Message::Exit));
        assert!(matches!(tick_msg, Message::Tick));
    }
    
    #[test]
    fn test_device_messages() {
        // Create a mock device
        let addr = BDAddr::from([1, 2, 3, 4, 5, 6]);
        let device = DiscoveredDevice {
            address: addr,
            name: Some("Test Device".to_string()),
            rssi: Some(-60),
            manufacturer_data: HashMap::new(),
            is_potential_airpods: false,
            last_seen: Instant::now(),
        };
        
        // Create messages with the device
        let discovered_msg = Message::DeviceDiscovered(device.clone());
        let updated_msg = Message::DeviceUpdated(device);
        
        // Check correct message types
        assert!(matches!(discovered_msg, Message::DeviceDiscovered(_)));
        assert!(matches!(updated_msg, Message::DeviceUpdated(_)));
    }
} 