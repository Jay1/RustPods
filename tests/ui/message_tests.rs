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
            is_connected: false,
            service_data: HashMap::new(),
            services: Vec::new(),
            tx_power_level: None,
        };
        // Create messages with the device
        let discovered_msg = Message::DeviceDiscovered(device.clone());
        // Check correct message types
        assert!(matches!(discovered_msg, Message::DeviceDiscovered(_)));
    }

    #[test]
    fn test_error_message() {
        // Test the error message constructor
        let error_message = "Test error message";
        let msg = Message::error(error_message);
        assert!(matches!(msg, Message::Error(_)));
        if let Message::Error(text) = msg {
            assert_eq!(text, error_message);
        } else {
            panic!("Expected Error message");
        }
    }

    #[test]
    fn test_validation_error() {
        // Test creating a validation error
        let field = "username";
        let message = "Username must be at least 3 characters";
        let msg = Message::validation_error(field, message);
        if let Message::FormValidationError { field: f, message: m } = msg {
            assert_eq!(f, field);
            assert_eq!(m, message);
        } else {
            panic!("Expected FormValidationError message");
        }
    }

    #[test]
    fn test_is_error() {
        // Test the is_error method
        let error_msg = Message::Error("Test error".to_string());
        let conn_error_msg = Message::ConnectionError("Connection failed".to_string());
        let bt_error_msg = Message::BluetoothError("Bluetooth error".to_string());
        let status_msg = Message::Status("Not an error".to_string());
        assert!(error_msg.is_error());
        assert!(conn_error_msg.is_error());
        assert!(bt_error_msg.is_error());
        assert!(!status_msg.is_error());
    }

    #[test]
    fn test_bluetooth_error_toast() {
        use rustpods::ui::Message;
        let error_message = "Bluetooth connection failed".to_string();
        let msg = Message::Error(error_message.clone());
        // TODO: Simulate message handling and assert that a toast is triggered
        // For now, just check the message variant
        if let Message::Error(e) = msg {
            assert_eq!(e, error_message);
        } else {
            panic!("Expected Message::Error");
        }
    }
} 
