use crate::bluetooth::DiscoveredDevice;
use crate::bluetooth::AirPodsBatteryStatus;
use crate::airpods::DetectedAirPods;
use crate::config::AppConfig;
use crate::ui::components::{BluetoothSetting, UiSetting, SystemSetting};
use crate::ui::settings_window::SettingsTab;
use crate::ui::state_manager::ConnectionState;
use iced::Point;

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
    
    /// Device disconnected
    DeviceDisconnected,
    
    /// Device reconnected after temporary disconnect
    DeviceReconnected(DetectedAirPods),
    
    /// Connection state changed
    ConnectionStateChanged(ConnectionState),
    
    /// Connection error occurred
    ConnectionError(String),
    
    /// Bluetooth related error
    BluetoothError(String),
    
    /// Battery status updated
    BatteryStatusUpdated(AirPodsBatteryStatus),
    
    /// Battery update failed
    BatteryUpdateFailed(String),
    
    /// Low battery warning for a component
    LowBatteryWarning(String, u8),
    
    /// Generic error message
    Error(String),
    
    /// Clear the current error message
    ClearError,
    
    /// Status message for information (non-error)
    Status(String),
    
    /// Clear the current status message
    ClearStatus,
    
    /// Retry connection to a device
    RetryConnection,
    
    /// Bluetooth adapter changed
    AdapterChanged(String),
    
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
    
    /// Animation tick specifically for battery animations
    BatteryAnimationTick,
    
    /// Move the window to a new position
    WindowMove(Point),
    
    /// Form validation error
    FormValidationError { field: String, message: String },
    
    /// Right-click for context menu
    ContextMenu(Point),
    
    /// Context menu item selected
    ContextMenuItemSelected(String),
    
    /// Double-click event
    DoubleClick,
    
    /// Drag-and-drop operation started
    DragStarted { id: String, position: Point },
    
    /// Drag-and-drop operation ongoing
    DragMoved { id: String, position: Point },
    
    /// Drag-and-drop operation ended
    DragEnded { id: String, position: Point },
    
    /// Raw event from the iced event system
    RawEvent(iced::Event),
    
    /// Window focus gained
    WindowFocused,
    
    /// Window focus lost
    WindowBlurred,
    
    /// Window close requested
    WindowCloseRequested,
    
    /// Initialize system tray with message sender
    InitializeSystemTray(std::sync::mpsc::Sender<Message>),
    
    /// Show window
    ShowWindow,
    
    /// Hide window
    HideWindow,
    
    /// Window needs updating
    WindowUpdate,
    
    /// Toggle between simple and advanced display mode
    ToggleDisplayMode,
    
    /// System sleep event
    SystemSleep,
    
    /// System wake event
    SystemWake,
    
    /// Application is starting up
    AppStarting,
    
    /// Application initialization has completed
    AppInitialized,
    
    /// Application going to background mode
    AppBackground,
    
    /// Application coming to foreground
    AppForeground,
    
    /// Force save application state
    SaveState,
    
    /// Load application state from disk
    LoadState,
    
    /// Restore previous device connection
    RestoreConnection(String),
}

impl Message {
    /// Create a new error message
    pub fn error<S: Into<String>>(message: S) -> Self {
        Self::Error(message.into())
    }
    
    /// Create a new connection error message
    pub fn connection_error<S: Into<String>>(message: S) -> Self {
        Self::ConnectionError(message.into())
    }
    
    /// Create a new bluetooth error message
    pub fn bluetooth_error<S: Into<String>>(message: S) -> Self {
        Self::BluetoothError(message.into())
    }
    
    /// Create a new low battery warning message
    pub fn low_battery<S: Into<String>>(component: S, level: u8) -> Self {
        Self::LowBatteryWarning(component.into(), level)
    }
    
    /// Create a new form validation error
    pub fn validation_error<S: Into<String>, T: Into<String>>(field: S, message: T) -> Self {
        Self::FormValidationError {
            field: field.into(),
            message: message.into(),
        }
    }
    
    /// Check if this message is an error message
    pub fn is_error(&self) -> bool {
        matches!(self, 
            Self::Error(_) | 
            Self::ConnectionError(_) | 
            Self::BluetoothError(_) |
            Self::BatteryUpdateFailed(_) |
            Self::FormValidationError { .. }
        )
    }
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Standard variants that can be directly compared
            (Self::ToggleVisibility, Self::ToggleVisibility) => true,
            (Self::Exit, Self::Exit) => true,
            (Self::DeviceDiscovered(a), Self::DeviceDiscovered(b)) => a == b,
            (Self::DeviceUpdated(a), Self::DeviceUpdated(b)) => a == b,
            (Self::SelectDevice(a), Self::SelectDevice(b)) => a == b,
            (Self::StartScan, Self::StartScan) => true,
            (Self::StopScan, Self::StopScan) => true,
            (Self::ScanCompleted, Self::ScanCompleted) => true,
            (Self::ScanStarted, Self::ScanStarted) => true,
            (Self::ScanStopped, Self::ScanStopped) => true,
            (Self::ScanProgress(a), Self::ScanProgress(b)) => a == b,
            (Self::ToggleAutoScan(a), Self::ToggleAutoScan(b)) => a == b,
            (Self::Tick, Self::Tick) => true,
            (Self::AnimationTick, Self::AnimationTick) => true,
            (Self::AnimationProgress(a), Self::AnimationProgress(b)) => a == b,
            (Self::AirPodsConnected(a), Self::AirPodsConnected(b)) => a == b,
            (Self::DeviceDisconnected, Self::DeviceDisconnected) => true,
            (Self::DeviceReconnected(a), Self::DeviceReconnected(b)) => a == b,
            (Self::ConnectionStateChanged(a), Self::ConnectionStateChanged(b)) => a == b,
            (Self::ConnectionError(a), Self::ConnectionError(b)) => a == b,
            (Self::BluetoothError(a), Self::BluetoothError(b)) => a == b,
            (Self::BatteryStatusUpdated(a), Self::BatteryStatusUpdated(b)) => a == b,
            (Self::BatteryUpdateFailed(a), Self::BatteryUpdateFailed(b)) => a == b,
            (Self::LowBatteryWarning(a, c), Self::LowBatteryWarning(b, d)) => a == b && c == d,
            (Self::Error(a), Self::Error(b)) => a == b,
            (Self::ClearError, Self::ClearError) => true,
            (Self::Status(a), Self::Status(b)) => a == b,
            (Self::ClearStatus, Self::ClearStatus) => true,
            (Self::RetryConnection, Self::RetryConnection) => true,
            (Self::AdapterChanged(a), Self::AdapterChanged(b)) => a == b,
            (Self::UpdateBluetoothSetting(a), Self::UpdateBluetoothSetting(b)) => a == b,
            (Self::UpdateUiSetting(a), Self::UpdateUiSetting(b)) => a == b,
            (Self::UpdateSystemSetting(a), Self::UpdateSystemSetting(b)) => a == b,
            (Self::SettingsChanged(a), Self::SettingsChanged(b)) => a == b,
            (Self::OpenSettings, Self::OpenSettings) => true,
            (Self::SaveSettings, Self::SaveSettings) => true,
            (Self::ResetSettings, Self::ResetSettings) => true,
            (Self::CloseSettings, Self::CloseSettings) => true,
            (Self::SelectSettingsTab(a), Self::SelectSettingsTab(b)) => a == b,
            (Self::BatteryAnimationTick, Self::BatteryAnimationTick) => true,
            (Self::WindowMove(a), Self::WindowMove(b)) => a == b,
            (Self::FormValidationError { field: a, message: c }, 
             Self::FormValidationError { field: b, message: d }) => a == b && c == d,
            (Self::ContextMenu(a), Self::ContextMenu(b)) => a == b,
            (Self::ContextMenuItemSelected(a), Self::ContextMenuItemSelected(b)) => a == b,
            (Self::DoubleClick, Self::DoubleClick) => true,
            (Self::DragStarted { id: a, position: c }, 
             Self::DragStarted { id: b, position: d }) => a == b && c == d,
            (Self::DragMoved { id: a, position: c }, 
             Self::DragMoved { id: b, position: d }) => a == b && c == d,
            (Self::DragEnded { id: a, position: c }, 
             Self::DragEnded { id: b, position: d }) => a == b && c == d,
            (Self::RawEvent(a), Self::RawEvent(b)) => a == b,
            (Self::WindowFocused, Self::WindowFocused) => true,
            (Self::WindowBlurred, Self::WindowBlurred) => true,
            (Self::WindowCloseRequested, Self::WindowCloseRequested) => true,
            
            // Special case for InitializeSystemTray - just check variant, ignore field
            (Self::InitializeSystemTray(_), Self::InitializeSystemTray(_)) => true,
            
            (Self::ShowWindow, Self::ShowWindow) => true,
            (Self::HideWindow, Self::HideWindow) => true,
            (Self::WindowUpdate, Self::WindowUpdate) => true,
            (Self::ToggleDisplayMode, Self::ToggleDisplayMode) => true,
            (Self::SystemSleep, Self::SystemSleep) => true,
            (Self::SystemWake, Self::SystemWake) => true,
            (Self::AppStarting, Self::AppStarting) => true,
            (Self::AppInitialized, Self::AppInitialized) => true,
            (Self::AppBackground, Self::AppBackground) => true,
            (Self::AppForeground, Self::AppForeground) => true,
            (Self::SaveState, Self::SaveState) => true,
            (Self::LoadState, Self::LoadState) => true,
            (Self::RestoreConnection(a), Self::RestoreConnection(b)) => a == b,
            // Different variants are not equal
            _ => false,
        }
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
} 