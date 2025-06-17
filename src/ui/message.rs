use std::fmt::Debug;

use crate::airpods::{battery::AirPodsBatteryInfo, DetectedAirPods};
use crate::bluetooth::AirPodsBatteryStatus;
use crate::bluetooth::DiscoveredDevice;
use crate::config::AppConfig;
use crate::ui::components::{BluetoothSetting, SystemSetting, UiSetting};
use crate::ui::state::MergedBluetoothDevice;
use crate::ui::state_manager::ConnectionState;
use iced::Point;

/// Messages that can be sent to update the UI state
#[derive(Debug, Clone)]
pub enum Message {
    /// Toggle the application window visibility
    ToggleVisibility,

    /// Toggle the application window (show/hide)
    ToggleWindow,

    /// Exit the application
    Exit,

    /// Force quit the application (ignores minimize to tray setting)
    ForceQuit,

    /// No operation - used internally for subscription management
    NoOp,

    /// New Bluetooth device discovered
    DeviceDiscovered(DiscoveredDevice),

    /// Existing device was updated
    DeviceUpdated(DiscoveredDevice),

    /// Select a device to connect to
    SelectDevice(String),

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

    /// Show a toast/notification message
    ShowToast(String),

    /// Merged scan result (paired + BLE)
    MergedScanResult(Vec<MergedBluetoothDevice>),

    /// Settings changed
    SettingsChanged(AppConfig),

    /// Update a Bluetooth setting
    UpdateBluetoothSetting(BluetoothSetting),

    /// Update a UI setting
    UpdateUiSetting(UiSetting),

    /// Update a system setting
    UpdateSystemSetting(SystemSetting),

    /// Open the settings window
    OpenSettings,

    /// Close the settings window
    CloseSettings,

    /// Save settings
    SaveSettings,

    /// Window drag started
    WindowDragStart(Point),

    /// Window drag ended
    WindowDragEnd,

    /// Window drag moved
    WindowDragMove(Point),

    /// Window position changed
    WindowPositionChanged(Point),

    /// Window bounds changed
    WindowBoundsChanged(iced::Rectangle),

    /// Window close requested
    WindowCloseRequested,

    /// Window minimized
    WindowMinimized,

    /// Window restored
    WindowRestored,

    /// Window maximized
    WindowMaximized,

    /// Window unmaximized
    WindowUnmaximized,

    /// Window focused
    WindowFocused,

    /// Window unfocused
    WindowUnfocused,

    /// Show the application window
    ShowWindow,

    /// Hide the application window
    HideWindow,

    /// Start scanning for devices
    StartScan,

    /// Stop scanning for devices
    StopScan,

    /// Battery update failed with error message
    BatteryUpdateFailed(String),

    /// Toggle auto scan setting
    ToggleAutoScan(bool),

    /// Unpair the current device
    UnpairDevice,

    /// AirPods data loaded from CLI scanner (async)
    AirPodsDataLoaded(Vec<AirPodsBatteryInfo>),

    /// Close popup window
    ClosePopup,

    /// Connect to a device
    ConnectDevice,

    /// Disconnect from a device
    DisconnectDevice,

    /// Device detection state changed
    DeviceDetectionStateChanged(crate::ui::state::DeviceDetectionState),



    /// Device scan completed
    ScanCompleted,

    /// Device scan failed
    ScanFailed(String),

    /// Set custom device name
    SetDeviceName(String),

    /// Open battery intelligence profile folder
    OpenProfileFolder,

    /// Purge all battery intelligence profiles (reset)
    PurgeProfiles,
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ToggleVisibility, Self::ToggleVisibility) => true,
            (Self::ToggleWindow, Self::ToggleWindow) => true,
            (Self::Exit, Self::Exit) => true,
            (Self::ForceQuit, Self::ForceQuit) => true,
            (Self::NoOp, Self::NoOp) => true,
            (Self::DeviceDiscovered(a), Self::DeviceDiscovered(b)) => a == b,
            (Self::DeviceUpdated(a), Self::DeviceUpdated(b)) => a == b,
            (Self::SelectDevice(a), Self::SelectDevice(b)) => a == b,
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
            (Self::ShowToast(a), Self::ShowToast(b)) => a == b,
            (Self::MergedScanResult(a), Self::MergedScanResult(b)) => a.len() == b.len(),
            (Self::SettingsChanged(a), Self::SettingsChanged(b)) => a == b,
            (Self::UpdateBluetoothSetting(a), Self::UpdateBluetoothSetting(b)) => a == b,
            (Self::UpdateUiSetting(a), Self::UpdateUiSetting(b)) => a == b,
            (Self::UpdateSystemSetting(a), Self::UpdateSystemSetting(b)) => a == b,
            (Self::OpenSettings, Self::OpenSettings) => true,
            (Self::SaveSettings, Self::SaveSettings) => true,
            (Self::CloseSettings, Self::CloseSettings) => true,
            (Self::WindowDragStart(a), Self::WindowDragStart(b)) => a == b,
            (Self::WindowDragEnd, Self::WindowDragEnd) => true,
            (Self::WindowDragMove(a), Self::WindowDragMove(b)) => a == b,
            (Self::WindowPositionChanged(a), Self::WindowPositionChanged(b)) => a == b,
            (Self::WindowBoundsChanged(a), Self::WindowBoundsChanged(b)) => a == b,
            (Self::WindowCloseRequested, Self::WindowCloseRequested) => true,
            (Self::WindowMinimized, Self::WindowMinimized) => true,
            (Self::WindowRestored, Self::WindowRestored) => true,
            (Self::WindowMaximized, Self::WindowMaximized) => true,
            (Self::WindowUnmaximized, Self::WindowUnmaximized) => true,
            (Self::WindowFocused, Self::WindowFocused) => true,
            (Self::WindowUnfocused, Self::WindowUnfocused) => true,
            (Self::ShowWindow, Self::ShowWindow) => true,
            (Self::HideWindow, Self::HideWindow) => true,
            (Self::StartScan, Self::StartScan) => true,
            (Self::StopScan, Self::StopScan) => true,
            (Self::BatteryUpdateFailed(a), Self::BatteryUpdateFailed(b)) => a == b,
            (Self::ToggleAutoScan(a), Self::ToggleAutoScan(b)) => a == b,
            (Self::UnpairDevice, Self::UnpairDevice) => true,
            (Self::AirPodsDataLoaded(a), Self::AirPodsDataLoaded(b)) => a.len() == b.len(),
            (Self::ClosePopup, Self::ClosePopup) => true,
            (Self::ConnectDevice, Self::ConnectDevice) => true,
            (Self::DisconnectDevice, Self::DisconnectDevice) => true,
            (Self::DeviceDetectionStateChanged(a), Self::DeviceDetectionStateChanged(b)) => a == b,

            (Self::ScanCompleted, Self::ScanCompleted) => true,
            (Self::ScanFailed(a), Self::ScanFailed(b)) => a == b,
            _ => false,
        }
    }
}
