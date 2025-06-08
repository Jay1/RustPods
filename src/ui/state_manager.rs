//! State management system for RustPods
//!
//! This module provides a centralized state management architecture for
//! managing application data flow and UI updates.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::bluetooth::AirPodsBatteryStatus;
use crate::bluetooth::DiscoveredDevice;
use crate::config::{AppConfig, ConfigManager};
use crate::ui::Message;
use tokio::sync::mpsc;

/// Represents an action that can be dispatched to update state
#[derive(Debug, Clone)]
pub enum Action {
    /// Toggle application visibility
    ToggleVisibility,

    /// Show window
    ShowWindow,

    /// Hide window
    HideWindow,

    /// Update device list with a new or updated device
    UpdateDevice(DiscoveredDevice),

    /// Remove a device from the list
    RemoveDevice(String),

    /// Select a device
    SelectDevice(String),

    /// Update battery status
    UpdateBatteryStatus(AirPodsBatteryStatus),

    /// Update animation progress (0.0-1.0)
    UpdateAnimationProgress(f32),

    /// Update application settings
    UpdateSettings(AppConfig),

    /// Show settings view
    ShowSettings,

    /// Hide settings view
    HideSettings,

    /// Set an error message
    SetError(String),

    /// Clear error message
    ClearError,

    /// Set connection state
    SetConnectionState(ConnectionState),

    /// System entering sleep mode
    SystemSleep,

    /// System waking from sleep mode
    SystemWake,

    /// Shutdown the application
    Shutdown,

    /// Restore connection to previously connected device
    RestorePreviousConnection(String),

    /// Set advanced display mode
    SetAdvancedDisplayMode(bool),

    /// Save persistent state
    SavePersistentState,

    /// Load persistent state
    LoadPersistentState,

    /// Toggle auto scan
    ToggleAutoScan(bool),

    /// Start scanning for devices
    StartScanning,

    /// Stop scanning for devices  
    StopScanning,
}

/// Represents a connection state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    /// Not connected
    Disconnected,
    /// Attempting to connect
    Connecting,
    /// Connected
    Connected,
    /// Connection failed
    Failed(String),
    /// Reconnecting after temporary disconnection
    Reconnecting,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::Disconnected
    }
}

/// Application state slice for device management
#[derive(Debug, Clone)]
pub struct DeviceState {
    /// Discovered Bluetooth devices
    pub devices: HashMap<String, DiscoveredDevice>,

    /// Currently selected device
    pub selected_device: Option<String>,

    /// Whether we're currently scanning for devices
    pub is_scanning: bool,

    /// Whether automatic scanning is enabled
    pub auto_scan: bool,

    /// Timestamp when the current device was connected
    pub connection_timestamp: Option<Instant>,

    /// Last known battery status
    pub battery_status: Option<AirPodsBatteryStatus>,

    /// Current connection state
    pub connection_state: ConnectionState,

    /// Last error message (if any)
    pub last_error: Option<String>,

    /// Connection retry count
    pub connection_retries: usize,
}

impl Default for DeviceState {
    fn default() -> Self {
        Self {
            devices: HashMap::new(),
            selected_device: None,
            is_scanning: false,
            auto_scan: true,
            connection_timestamp: None,
            battery_status: None,
            connection_state: ConnectionState::Disconnected,
            last_error: None,
            connection_retries: 0,
        }
    }
}

/// Application state slice for UI related state
#[derive(Debug, Clone)]
pub struct UiState {
    /// Whether the application window is visible
    pub visible: bool,

    /// Whether settings view is open
    pub show_settings: bool,

    /// Animation progress for refresh button (0.0-1.0)
    pub animation_progress: f32,

    /// Current error message (if any)
    pub error_message: Option<String>,

    /// Whether an error notification is visible
    pub show_error: bool,

    /// Current informational message (if any)
    pub info_message: Option<String>,

    /// Whether an info notification is visible
    pub show_info: bool,

    /// Current settings error message (if any)
    pub settings_error: Option<String>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            visible: true,
            show_settings: false,
            animation_progress: 0.0,
            error_message: None,
            show_error: false,
            info_message: None,
            show_info: false,
            settings_error: None,
        }
    }
}

/// Centralized state manager for the application
#[derive(Debug)]
pub struct StateManager {
    /// Device-related state
    device_state: Arc<Mutex<DeviceState>>,

    /// UI-related state
    ui_state: Arc<Mutex<UiState>>,

    /// Application configuration
    config: Arc<Mutex<AppConfig>>,

    /// Configuration manager
    config_manager: Arc<Mutex<ConfigManager>>,

    /// Channel to send messages to the UI
    ui_sender: mpsc::UnboundedSender<Message>,
}

impl StateManager {
    /// Create a new state manager
    pub fn new(ui_sender: mpsc::UnboundedSender<Message>) -> Self {
        // Load config or use default
        let config = AppConfig::load().unwrap_or_default();

        Self {
            device_state: Arc::new(Mutex::new(DeviceState::default())),
            ui_state: Arc::new(Mutex::new(UiState::default())),
            config: Arc::new(Mutex::new(config)),
            config_manager: Arc::new(Mutex::new(ConfigManager::create_default())),
            ui_sender,
        }
    }

    /// Check if advanced display mode is enabled
    pub fn is_advanced_display_mode(&self) -> bool {
        // This would be stored in the state in a real implementation
        // For now, we'll return a default value
        false
    }

    /// Dispatch an action to update the state
    pub fn dispatch(&self, action: Action) {
        match action {
            Action::ToggleVisibility => {
                let mut ui_state = self.ui_state.lock().unwrap();
                ui_state.visible = !ui_state.visible;
            }
            Action::UpdateDevice(device) => {
                let mut device_state = self.device_state.lock().unwrap();
                match device_state.devices.entry(device.address.to_string()) {
                    std::collections::hash_map::Entry::Occupied(mut e) => {
                        e.insert(device.clone());
                        self.notify_ui(Message::DeviceUpdated(device));
                    }
                    std::collections::hash_map::Entry::Vacant(e) => {
                        e.insert(device.clone());
                        self.notify_ui(Message::DeviceDiscovered(device));
                    }
                }
            }
            Action::RemoveDevice(address) => {
                let mut device_state = self.device_state.lock().unwrap();
                device_state.devices.remove(&address);
                if let Some(selected_device) = &device_state.selected_device {
                    if selected_device == &address {
                        device_state.selected_device = None;
                        device_state.connection_state = ConnectionState::Disconnected;
                        device_state.battery_status = None;
                        self.notify_ui(Message::DeviceDisconnected);
                    }
                }
            }
            Action::SelectDevice(address) => {
                let mut device_state = self.device_state.lock().unwrap();
                device_state.selected_device = Some(address.clone());
                device_state.connection_state = ConnectionState::Connected;
                device_state.connection_timestamp = Some(std::time::Instant::now());
                if let Some(device) = device_state.devices.get(&address) {
                    if let Some(name) = &device.name {
                        if name.contains("AirPods") {
                            let airpods = crate::airpods::DetectedAirPods {
                                address: device.address,
                                name: Some(name.clone()),
                                device_type: crate::airpods::AirPodsType::detect_from_name(name),
                                battery: Some(crate::airpods::AirPodsBattery::default()),
                                rssi: device.rssi,
                                is_connected: false,
                                last_seen: std::time::Instant::now(),
                            };
                            self.notify_ui(Message::AirPodsConnected(airpods));
                        } else {
                            self.notify_ui(Message::SelectDevice(address));
                        }
                    }
                }
            }
            Action::UpdateBatteryStatus(status) => {
                let mut device_state = self.device_state.lock().unwrap();
                device_state.battery_status = Some(status.clone());
                self.notify_ui(Message::BatteryStatusUpdated(status));
            }
            Action::UpdateAnimationProgress(progress) => {
                let mut ui_state = self.ui_state.lock().unwrap();
                ui_state.animation_progress = progress;
                self.notify_ui(Message::AnimationProgress(progress));
            }
            Action::UpdateSettings(new_config) => {
                let mut config = self.config.lock().unwrap();
                *config = new_config.clone();
                self.notify_ui(Message::SettingsChanged(new_config));
            }
            Action::ShowSettings => {
                let mut ui_state = self.ui_state.lock().unwrap();
                ui_state.show_settings = true;
                self.notify_ui(Message::OpenSettings);
            }
            Action::HideSettings => {
                let mut ui_state = self.ui_state.lock().unwrap();
                ui_state.show_settings = false;
                self.notify_ui(Message::CloseSettings);
            }
            Action::ShowWindow => {
                let mut ui_state = self.ui_state.lock().unwrap();
                ui_state.visible = true;
                self.notify_ui(Message::ShowWindow);
            }
            Action::HideWindow => {
                let mut ui_state = self.ui_state.lock().unwrap();
                ui_state.visible = false;
                self.notify_ui(Message::HideWindow);
            }
            Action::ToggleAutoScan(enabled) => {
                let mut device_state = self.device_state.lock().unwrap();
                device_state.auto_scan = enabled;
            }
            Action::StartScanning => {
                let mut device_state = self.device_state.lock().unwrap();
                device_state.is_scanning = true;
                self.notify_ui(Message::StartScan);
            }
            Action::StopScanning => {
                let mut device_state = self.device_state.lock().unwrap();
                device_state.is_scanning = false;
                self.notify_ui(Message::StopScan);
            }
            _ => {
                // Other actions are not handled in the UI
            }
        }
    }

    /// Send a message to the UI
    fn notify_ui(&self, message: Message) {
        let _ = self.ui_sender.send(message);
    }

    /// Get the current device state
    pub fn get_device_state(&self) -> DeviceState {
        self.device_state.lock().unwrap().clone()
    }

    /// Get the current UI state
    pub fn get_ui_state(&self) -> UiState {
        self.ui_state.lock().unwrap().clone()
    }

    /// Get the current configuration
    pub fn get_config(&self) -> AppConfig {
        self.config.lock().unwrap().clone()
    }

    /// Get all state components for use by other components
    #[allow(clippy::type_complexity)]
    pub fn get_state_components(
        &self,
    ) -> (
        Arc<Mutex<DeviceState>>,
        Arc<Mutex<UiState>>,
        Arc<Mutex<AppConfig>>,
        Arc<Mutex<ConfigManager>>,
    ) {
        (
            Arc::clone(&self.device_state),
            Arc::clone(&self.ui_state),
            Arc::clone(&self.config),
            Arc::clone(&self.config_manager),
        )
    }

    /// Check if currently connected to a device
    pub fn is_connected(&self) -> bool {
        let device_state = self.device_state.lock().unwrap();
        device_state.connection_state == ConnectionState::Connected
            && device_state.selected_device.is_some()
    }

    /// Get the current error message if any
    pub fn get_error(&self) -> Option<String> {
        let device_state = self.device_state.lock().unwrap();
        device_state.last_error.clone()
    }

    /// Check if currently trying to connect
    pub fn is_connecting(&self) -> bool {
        let device_state = self.device_state.lock().unwrap();
        device_state.connection_state == ConnectionState::Connecting
    }

    /// Check if currently trying to reconnect
    pub fn is_reconnecting(&self) -> bool {
        let device_state = self.device_state.lock().unwrap();
        matches!(device_state.connection_state, ConnectionState::Reconnecting)
    }

    /// Get the current animation progress
    pub fn get_animation_progress(&self) -> f32 {
        let ui_state = self.ui_state.lock().unwrap();
        ui_state.animation_progress
    }

    /// Set the animation progress (0.0-1.0)
    pub fn set_animation_progress(&self, progress: f32) {
        let mut ui_state = self.ui_state.lock().unwrap();
        ui_state.animation_progress = progress;
    }
}
