use std::collections::HashMap;
#[cfg(test)]
use std::time::Instant;
use iced::{Subscription, Application, Command};
use std::convert::TryInto;
use iced::executor;
use std::sync::mpsc;

use crate::bluetooth::DiscoveredDevice;
use crate::config::{AppConfig, ConfigManager, ConfigError};
use crate::ui::Message;
use crate::ui::components::{BluetoothSetting, UiSetting, SystemSetting};
use crate::ui::{MainWindow, SettingsWindow};
use crate::ui::SystemTray;
use crate::ui::system_tray::SystemTrayError;
use crate::ui::state_manager::StateManager;

/// Main application state
#[derive(Debug, Clone)]
pub struct AppState {
    /// Whether the application window is visible
    pub visible: bool,
    
    /// Whether we're currently scanning for devices
    pub is_scanning: bool,
    
    /// Whether automatic scanning is enabled
    pub auto_scan: bool,
    
    /// Discovered Bluetooth devices
    pub devices: HashMap<String, DiscoveredDevice>,
    
    /// Currently selected device
    pub selected_device: Option<String>,
    
    /// Timestamp when the current device was connected
    pub connection_timestamp: Option<std::time::Instant>,
    
    /// Animation progress for refresh button (0.0-1.0)
    pub animation_progress: f32,
    
    /// Last known battery status
    pub battery_status: Option<crate::bluetooth::AirPodsBatteryStatus>,
    
    /// Application configuration
    pub config: AppConfig,
    
    /// Configuration manager
    pub config_manager: Option<ConfigManager>,
    
    /// Whether settings view is open
    pub show_settings: bool,
    
    /// Main window component
    pub main_window: MainWindow,
    
    /// Settings window component
    pub settings_window: SettingsWindow,
    
    /// Current settings error message (if any)
    pub settings_error: Option<String>,
    
    /// System tray component
    pub system_tray: Option<SystemTray>,
}

impl Default for AppState {
    fn default() -> Self {
        // Load config or use default
        let config = AppConfig::load().unwrap_or_default();
        
        // Create config manager
        let config_manager = Some(ConfigManager::default());
        
        // Create an empty main window
        let main_window = MainWindow::empty();
        
        // Create settings window with the config
        let settings_window = SettingsWindow::new(config.clone());
        
        Self {
            // Always start with visible = true to ensure the UI shows up
            visible: true,
            is_scanning: false,
            auto_scan: true,
            devices: HashMap::new(),
            selected_device: None,
            connection_timestamp: None,
            animation_progress: 0.0,
            battery_status: None,
            config,
            config_manager,
            show_settings: false,
            main_window,
            settings_window,
            settings_error: None,
            system_tray: None,
        }
    }
}

impl Application for AppState {
    type Message = Message;
    type Theme = crate::ui::theme::Theme;
    type Executor = executor::Default;
    type Flags = std::sync::Arc<StateManager>;

    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        let state = Self::default();
        
        // Store the state manager reference in some way
        // This is a placeholder - ideally we'd have a way to store and use the state manager
        // For now, we'll just use the default state
        
        (state, Command::none())
    }
    
    fn title(&self) -> String {
        String::from("RustPods - AirPods Battery Monitor")
    }

    fn theme(&self) -> Self::Theme {
        crate::ui::theme::Theme::CatppuccinMocha
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ToggleVisibility => {
                self.toggle_visibility();
            }
            Message::Exit => {
                // Clean up resources before exit
                #[cfg(target_os = "windows")]
                if let Some(tray) = &mut self.system_tray {
                    if let Err(e) = tray.cleanup() {
                        log::error!("Failed to clean up system tray: {}", e);
                    } else {
                        log::info!("System tray resources cleaned up");
                    }
                }
                
                // Save settings before exit
                if let Err(e) = self.config.save() {
                    log::error!("Failed to save settings on exit: {}", e);
                }
                
                // Exit will be handled by the AppController
                std::process::exit(0);
            }
            Message::DeviceDiscovered(device) => {
                self.update_device(device);
            }
            Message::DeviceUpdated(device) => {
                self.update_device(device);
            }
            Message::SelectDevice(address) => {
                self.select_device(address);
            }
            Message::StartScan => {
                self.is_scanning = true;
                // Reset animation progress
                self.animation_progress = 0.0;
                // AppController will handle the actual scanning
            }
            Message::StopScan => {
                self.is_scanning = false;
                // AppController will handle the actual scanning
            }
            Message::ScanStarted => {
                self.is_scanning = true;
            }
            Message::ScanStopped => {
                self.is_scanning = false;
            }
            Message::ScanCompleted => {
                self.is_scanning = false;
            }
            Message::ScanProgress(_progress) => {
                // We don't need to update state for scan progress
            }
            Message::AnimationTick => {
                // Update animation progress
                self.animation_progress = (self.animation_progress + 0.01) % 1.0;
                
                // Update main window with current animation progress
                if let Some(_device) = self.get_selected_device() {
                    self.main_window = MainWindow::new()
                        .with_animation_progress(self.animation_progress);
                }
            }
            Message::AnimationProgress(progress) => {
                self.animation_progress = progress;
            }
            Message::ToggleAutoScan(enabled) => {
                self.auto_scan = enabled;
            }
            Message::BatteryStatusUpdated(status) => {
                // Update battery status
                let status_clone = status.clone();
                self.battery_status = Some(status);
                
                // Update main window with new battery status
                if let Some(_device) = self.get_selected_device() {
                    self.main_window = MainWindow::new()
                        .with_animation_progress(self.animation_progress);
                }
                
                // Update system tray with battery status if available
                if let Some(tray) = &mut self.system_tray {
                    // Update connection status in tray icon
                    if let Err(e) = tray.update_icon(true) {
                        log::warn!("Failed to update tray icon: {}", e);
                    }
                    
                    // Update tooltip with battery information
                    if let Err(e) = tray.update_tooltip_with_battery(
                        status_clone.battery.left,
                        status_clone.battery.right,
                        status_clone.battery.case
                    ) {
                        log::warn!("Failed to update tray tooltip: {}", e);
                    }
                }
            }
            Message::AirPodsConnected(airpods) => {
                // We've connected to AirPods
                println!("Connected to AirPods: {:?}", airpods);
                // Update the corresponding device if we have it
                if let Some(address) = self.selected_device.as_ref() {
                    if let Some(device) = self.devices.get_mut(address) {
                        device.is_potential_airpods = true;
                    }
                }
            }
            Message::BatteryUpdateFailed(error) => {
                eprintln!("Battery update failed: {}", error);
            }
            Message::Error(error) => {
                eprintln!("Error: {}", error);
            }
            Message::Status(status) => {
                println!("Status: {}", status);
            }
            Message::RetryConnection => {
                // Retry connection with selected device
                if let Some(address) = self.selected_device.clone() {
                    // Simply reselect the device to trigger reconnection
                    self.select_device(address);
                }
            }
            Message::Tick => {
                // Periodic update, nothing to do for now
            }
            Message::UpdateBluetoothSetting(setting) => {
                // Update settings window
                self.settings_window.mark_changed();
                
                // Update application state
                self.update_bluetooth_setting(setting);
            }
            Message::UpdateUiSetting(setting) => {
                // Update settings window
                self.settings_window.mark_changed();
                
                // Update application state
                self.update_ui_setting(setting);
            }
            Message::UpdateSystemSetting(setting) => {
                // Update settings window
                self.settings_window.mark_changed();
                
                // Update application state
                self.update_system_setting(setting);
            }
            Message::OpenSettings => {
                // Reset any validation errors
                self.settings_window.set_validation_error(None);
                
                // Show the settings window
                self.settings_window.update_config(self.config.clone());
                self.show_settings = true;
            }
            Message::CloseSettings => {
                // Close the settings window
                self.show_settings = false;
                
                // Discard any changes by updating the settings window with current config
                self.settings_window.update_config(self.config.clone());
            }
            Message::SaveSettings => {
                // Get the updated config from settings window
                let updated_config = self.settings_window.config();
                
                // Validate the config
                if let Err(e) = updated_config.validate() {
                    // Set validation error
                    self.settings_window.set_validation_error(Some(e.to_string()));
                    
                    // Log the error
                    log::error!("Settings validation failed: {}", e);
                    
                    return Command::none();
                }
                
                // Update application config
                self.config = updated_config.clone();
                
                // Save settings to disk
                if let Err(e) = self.config.save() {
                    // Set save error
                    self.settings_window.set_validation_error(Some(format!("Failed to save: {}", e)));
                    
                    // Log the error
                    log::error!("Settings save failed: {}", e);
                    
                    return Command::none();
                }
                
                // Apply the settings
                self.apply_settings();
                
                // Close settings window
                self.show_settings = false;
            }
            Message::ResetSettings => {
                // Reset to default settings
                self.reset_settings();
                
                // Update settings window with default config
                self.settings_window.update_config(self.config.clone());
            }
            Message::SelectSettingsTab(tab) => {
                // Update selected tab in settings window
                self.settings_window.select_tab(tab);
            }
            Message::SettingsChanged(config) => {
                // Update config
                self.config = config.clone();
                
                // Update settings window
                self.settings_window.update_config(config);
            }
            // Add wildcard pattern to handle all other Message variants
            _ => {
                // Other message types can be ignored or logged
                log::debug!("Unhandled message in state.rs");
            }
        }
        
        Command::none()
    }

    fn view(&self) -> iced::Element<'_, Message, iced::Renderer<crate::ui::theme::Theme>> {
        crate::ui::app::view(self)
    }
    
    fn subscription(&self) -> Subscription<Message> {
        // Combine regular subscription with window events to handle close
        Subscription::batch(vec![
            crate::ui::app::subscription(self),
            iced::subscription::events_with(|event, _status| {
                if let iced::Event::Window(iced::window::Event::CloseRequested) = event {
                    Some(Message::Exit)
                } else {
                    None
                }
            })
        ])
    }
}

impl AppState {
    /// Toggle the visibility of the application
    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }
    
    /// Initialize the system tray component
    pub fn initialize_system_tray(&mut self, tx: mpsc::Sender<Message>) -> Result<(), SystemTrayError> {
        // Create the system tray
        let tray = SystemTray::new(tx, self.config.clone())?;
        self.system_tray = Some(tray);
        Ok(())
    }
    
    /// Update a device in the devices list
    pub fn update_device(&mut self, device: DiscoveredDevice) {
        let address = device.address.to_string();
        self.devices.insert(address, device);
    }

    /// Remove a device and clear selection if it was selected
    pub fn remove_device(&mut self, address: &str) {
        self.devices.remove(address);
        self.check_selected_device();
    }
    
    /// Select a device by address
    pub fn select_device(&mut self, address: String) {
        // Only set if it exists
        if self.devices.contains_key(&address) {
            self.selected_device = Some(address);
            // Set the connection timestamp to now
            self.connection_timestamp = Some(std::time::Instant::now());
        }
    }
    
    /// Check if selected device exists and clear if not
    pub fn check_selected_device(&mut self) {
        if let Some(selected) = &self.selected_device {
            if !self.devices.contains_key(selected) {
                self.selected_device = None;
            }
        }
    }
    
    /// Get the currently selected device
    pub fn get_selected_device(&self) -> Option<&DiscoveredDevice> {
        self.selected_device.as_ref().and_then(|addr| self.devices.get(addr))
    }
    
    /// Save the current settings
    fn save_settings(&mut self) {
        // Validate settings first
        match self.config.validate() {
            Ok(_) => {
                // Save to file
                match self.config.save() {
                    Ok(_) => {
                        // Clear any previous error
                        self.settings_error = None;
                        
                        // Log success
                        log::info!("Settings saved successfully");
                    },
                    Err(e) => {
                        // Set error message
                        self.settings_error = Some(format!("Failed to save settings: {}", e));
                        
                        // Log error
                        log::error!("Failed to save settings: {}", e);
                    }
                }
            },
            Err(e) => {
                // Set validation error message
                self.settings_error = Some(format!("Invalid settings: {}", e));
                
                // Log error
                log::error!("Settings validation failed: {}", e);
            }
        }
    }
    
    /// Reset settings to defaults
    fn reset_settings(&mut self) {
        // Create a new default config but preserve the settings path
        let settings_path = self.config.settings_path.clone();
        self.config = AppConfig::default();
        self.config.settings_path = settings_path;
        
        // Clear any error
        self.settings_error = None;
        
        log::info!("Settings reset to defaults");
    }
    
    /// Load settings from disk
    fn load_settings(&mut self) -> Result<(), ConfigError> {
        match AppConfig::load() {
            Ok(config) => {
                self.config = config;
                Ok(())
            },
            Err(e) => {
                self.settings_error = Some(format!("Failed to load settings: {}", e));
                log::error!("Failed to load settings: {}", e);
                Err(e)
            }
        }
    }
    
    /// Apply settings to the application
    fn apply_settings(&mut self) {
        // Update all components with new settings
        
        // Update main window theme
        // (This would be implemented in a real app by applying theme settings
        // to all UI components)
        
        // Update system tray configuration
        if let Some(system_tray) = &mut self.system_tray {
            if let Err(e) = system_tray.update_config(self.config.clone()) {
                log::error!("Failed to update system tray config: {}", e);
            } else {
                log::debug!("System tray configuration updated");
            }
            
            // Special handling for startup registration on Windows
            #[cfg(target_os = "windows")]
            {
                // Ensure startup registration matches config
                let should_register = self.config.system.launch_at_startup;
                let is_registered = system_tray.is_startup_registered();
                
                if should_register != is_registered {
                    if let Err(e) = system_tray.set_startup_enabled(should_register) {
                        log::error!("Failed to update startup registration: {}", e);
                    } else {
                        log::info!("Updated startup registration to: {}", should_register);
                    }
                }
            }
        }
        
        // Check if auto-scan setting changed
        if self.config.bluetooth.auto_scan_on_startup && !self.is_scanning && self.auto_scan {
            // We should be scanning but aren't - start scanning
            // In real code, this would send a message to start scanning
        } else if !self.config.bluetooth.auto_scan_on_startup && self.is_scanning && self.auto_scan {
            // We shouldn't be scanning but are - stop scanning
            // In real code, this would send a message to stop scanning
        }
        
        // Update the auto_scan flag to match config
        self.auto_scan = self.config.bluetooth.auto_scan_on_startup;
        
        log::info!("Settings applied");
    }
    
    /// Update a Bluetooth setting
    fn update_bluetooth_setting(&mut self, setting: BluetoothSetting) {
        match setting {
            BluetoothSetting::AutoScanOnStartup(value) => {
                self.config.bluetooth.auto_scan_on_startup = value;
            }
            BluetoothSetting::ScanDuration(value) => {
                self.config.bluetooth.scan_duration = std::time::Duration::from_secs(value as u64);
            }
            BluetoothSetting::ScanInterval(value) => {
                self.config.bluetooth.scan_interval = std::time::Duration::from_secs(value as u64);
            }
            BluetoothSetting::MinRssi(value) => {
                self.config.bluetooth.min_rssi = Some(value.try_into().unwrap_or(-70));
            }
            BluetoothSetting::BatteryRefreshInterval(value) => {
                self.config.bluetooth.battery_refresh_interval = value as u64;
            }
            BluetoothSetting::AutoReconnect(value) => {
                self.config.bluetooth.auto_reconnect = value;
            }
            BluetoothSetting::ReconnectAttempts(value) => {
                self.config.bluetooth.reconnect_attempts = value.try_into().unwrap_or(3);
            }
        }
    }
    
    /// Update a UI setting
    fn update_ui_setting(&mut self, setting: UiSetting) {
        match setting {
            UiSetting::ShowNotifications(value) => {
                self.config.ui.show_notifications = value;
            }
            UiSetting::StartMinimized(value) => {
                self.config.ui.start_minimized = value;
            }
            UiSetting::Theme(value) => {
                self.config.ui.theme = value;
            }
            UiSetting::ShowPercentageInTray(value) => {
                self.config.ui.show_percentage_in_tray = value;
            }
            UiSetting::ShowLowBatteryWarning(value) => {
                self.config.ui.show_low_battery_warning = value;
            }
            UiSetting::LowBatteryThreshold(value) => {
                self.config.ui.low_battery_threshold = value;
            }
        }
    }
    
    /// Update a system setting
    fn update_system_setting(&mut self, setting: SystemSetting) {
        match setting {
            SystemSetting::StartOnBoot(value) => {
                self.config.system.launch_at_startup = value;
            }
            SystemSetting::StartMinimized(value) => {
                self.config.ui.start_minimized = value;
            }
            SystemSetting::LogLevel(value) => {
                self.config.system.log_level = value;
            }
            SystemSetting::EnableTelemetry(value) => {
                self.config.system.enable_telemetry = value;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use btleplug::api::BDAddr;
    
    #[test]
    fn test_default_state() {
        let state = AppState::default();
        // Since this is not dependent on config in the default constructor,
        // visibility starts as true by default
        assert_eq!(state.visible, true);
        assert!(!state.is_scanning);
        assert!(state.auto_scan);
        assert!(state.devices.is_empty());
        assert_eq!(state.selected_device, None);
    }
    
    #[test]
    fn test_toggle_visibility() {
        let mut state = AppState::default();
        // Default visibility is true
        assert_eq!(state.visible, true);
        
        // Toggle should flip the visibility
        state.toggle_visibility();
        assert_eq!(state.visible, false);
        
        // Toggle again should restore original visibility
        state.toggle_visibility();
        assert_eq!(state.visible, true);
    }
    
    #[test]
    fn test_update_device() {
        let mut state = AppState::default();
        assert!(state.devices.is_empty());
        
        let addr = BDAddr::from([1, 2, 3, 4, 5, 6]);
        let addr_str = addr.to_string();
        
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
        };
        
        state.update_device(device.clone());
        assert_eq!(state.devices.len(), 1);
        assert!(state.devices.contains_key(&addr_str));
        
        // Update existing device
        let updated_device = DiscoveredDevice {
            rssi: Some(-50),
            ..device
        };
        
        state.update_device(updated_device);
        assert_eq!(state.devices.len(), 1);
        assert_eq!(state.devices.get(&addr_str).unwrap().rssi, Some(-50));
    }
    
    #[test]
    fn test_select_device() {
        let mut state = AppState::default();
        let addr = BDAddr::from([1, 2, 3, 4, 5, 6]);
        let addr_str = addr.to_string();
        
        // Add the device first
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
        };
        
        state.update_device(device);
        
        // Now select it - should work because it exists
        state.select_device(addr_str.clone());
        assert_eq!(state.selected_device, Some(addr_str.clone()));
        
        // Try to select a non-existent device
        let non_existent = "non:ex:is:te:nt:00";
        state.select_device(non_existent.to_string());
        // Selection should not change to the non-existent device
        assert_eq!(state.selected_device, Some(addr_str));
    }
    
    #[test]
    fn test_get_selected_device() {
        let mut state = AppState::default();
        let addr = BDAddr::from([1, 2, 3, 4, 5, 6]);
        let addr_str = addr.to_string();
        
        // No selected device
        assert!(state.get_selected_device().is_none());
        
        // Add a device
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
        };
        
        state.update_device(device.clone());
        state.select_device(addr_str);
        
        // Get the selected device
        let selected = state.get_selected_device();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, Some("Test Device".to_string()));
    }
} 