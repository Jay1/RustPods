use crate::bluetooth::DiscoveredDevice;
use crate::bluetooth::AirPodsBatteryStatus;
use crate::airpods::DetectedAirPods;
use crate::config::AppConfig;
use crate::ui::components::{BluetoothSetting, UiSetting, SystemSetting};
use crate::ui::settings_window::SettingsTab;

/// Messages that can be sent to update the UI state
#[derive(Debug, Clone)]
pub enum Message {
    /// Toggle the application window visibility
    ToggleVisibility,
    
    /// Exit the application
    Exit,
    
    /// New Bluetooth device discovered
    DeviceDiscovered(DiscoveredDevice),
    
    /// Existing device was updated
    DeviceUpdated(DiscoveredDevice),
    
    /// Select a device to connect to
    SelectDevice(String),
    
    /// Start scanning for devices
    StartScan,
    
    /// Stop scanning for devices
    StopScan,
    
    /// Scan completed
    ScanCompleted,
    
    /// Scan started
    ScanStarted,
    
    /// Scan stopped
    ScanStopped,
    
    /// Scan progress update
    ScanProgress(usize),
    
    /// Toggle automatic scanning
    ToggleAutoScan(bool),
    
    /// Tick event for periodic updates
    Tick,
    
    /// Raw animation tick event
    AnimationTick,
    
    /// Animation progress update (0.0-1.0)
    AnimationProgress(f32),
    
    /// AirPods device connected
    AirPodsConnected(DetectedAirPods),
    
    /// Battery status updated
    BatteryStatusUpdated(AirPodsBatteryStatus),
    
    /// Battery update failed
    BatteryUpdateFailed(String),
    
    /// Generic error message
    Error(String),
    
    /// Status message for information (non-error)
    Status(String),
    
    /// Retry connection to a device
    RetryConnection,
    
    /// Update a Bluetooth setting
    UpdateBluetoothSetting(BluetoothSetting),
    
    /// Update a UI setting
    UpdateUiSetting(UiSetting),
    
    /// Update a system setting
    UpdateSystemSetting(SystemSetting),
    
    /// Settings have been changed
    SettingsChanged(AppConfig),
    
    /// Open the settings view
    OpenSettings,
    
    /// Save the current settings
    SaveSettings,
    
    /// Reset settings to defaults
    ResetSettings,
    
    /// Close the settings view
    CloseSettings,
    
    /// Select a settings tab
    SelectSettingsTab(SettingsTab),
}

impl Message {
    /// Create a new error message
    pub fn error<S: Into<String>>(message: S) -> Self {
        Self::Error(message.into())
    }
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
} 