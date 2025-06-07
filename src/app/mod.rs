//! Application entry point and main logic

use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use log::{info, error, debug, warn};

// Main app imports
// Temporarily disable system tray
// use crate::ui::{Message, SystemTray};
use crate::ui::Message;
use crate::error::RustPodsError;
use crate::airpods::{DetectedAirPods, detect_airpods};
use crate::bluetooth::{BleScanner, AirPodsBatteryStatus};
// Remove unused ScanConfig import
use crate::config::{AppConfig, ConfigManager};
// Remove duplicate and unused imports
use tracing;

/// Main application struct
pub struct App {
    /// Configuration
    config: AppConfig,
    /// Configuration manager
    config_manager: ConfigManager,
    /// Channel for UI messages
    ui_tx: mpsc::Sender<Message>,
    /// Channel for UI messages (receiver)
    ui_rx: mpsc::Receiver<Message>,
    /// Bluetooth scanner
    scanner: BleScanner,
    /// System tray
    #[allow(dead_code)]
    // tray: SystemTray, // Temporarily disabled
    /// Running flag
    running: bool,
    /// Current AirPods device
    current_airpods: Arc<Mutex<Option<DetectedAirPods>>>,
    /// Current battery status
    battery_status: Arc<Mutex<AirPodsBatteryStatus>>,
    /// Battery monitoring task handle
    battery_monitor_task: Option<tokio::task::JoinHandle<()>>,
}

impl App {
    /// Create a new application
    pub fn new() -> Result<Self, RustPodsError> {
        // Create channels
        let (ui_tx, ui_rx) = mpsc::channel(100);
        
        // Create config manager and load config
        let config_manager = ConfigManager::default();
        config_manager.load().map_err(|e| RustPodsError::ConfigError(e.to_string()))?;
        let config = config_manager.get_config();
        
        // TODO: Initialize adapter and provider for production
        // let provider = Arc::new(RealAdapterEventsProvider { adapter });
        // let scanner = BleScanner::new(provider, ScanConfig::default());
        // panic!("BleScanner::new now requires a provider and config. Update this code to provide them.");
        // unreachable!();
        // For now, create a dummy scanner to allow the app to run
        let scanner = BleScanner::dummy();
        // Temporarily disable system tray
        // let tray = SystemTray::new(ui_tx.clone(), config.clone())
        //     .map_err(|_| RustPodsError::UiError)?;
        Ok(Self {
            config,
            config_manager,
            ui_tx,
            ui_rx,
            scanner,
            // tray, // Temporarily disabled
            running: false,
            current_airpods: Arc::new(Mutex::new(None)),
            battery_status: Arc::new(Mutex::new(AirPodsBatteryStatus::default())),
            battery_monitor_task: None,
        })
    }
    
    /// Initialize the application
    pub async fn initialize(&mut self) -> Result<(), RustPodsError> {
        // Initialize bluetooth
        self.scanner.initialize().await?;
        
        // Apply configuration
        self.apply_config();
        
        // Set running flag
        self.running = true;
        
        Ok(())
    }
    
    /// Apply configuration to components
    fn apply_config(&mut self) {
        // Configure scanner - as_configurable returns a reference directly
        let scanner = self.scanner.as_configurable();
        scanner.apply_config(&self.config);
        
        // Configure other components as needed
    }
    
    /// Save configuration
    fn save_config(&self) -> Result<(), RustPodsError> {
        self.config_manager.update(|c| {
            *c = self.config.clone();
        }).map_err(|e| RustPodsError::ConfigError(e.to_string()))?;
        
        self.config_manager.save()
            .map_err(|e| RustPodsError::ConfigError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Run the application event loop
    pub async fn run(&mut self) -> Result<(), RustPodsError> {
        println!("[DEBUG] Entered App::run");
        tracing::info!("App::run: auto_scan_on_startup = {}", self.config.bluetooth.auto_scan_on_startup);

        // TEMP: Debug WinRT device query
        println!("[DEBUG] About to call get_connected_bluetooth_devices()");
        // WinRT async device enumeration is currently disabled due to interop issues.
        // Use the Win32 SetupAPI fallback or comment out this block for now.
        println!("[DEBUG] Finished call to get_connected_bluetooth_devices()");

        // TEMP: Query and print connected AirPods using WinRT
        // WinRT async AirPods enumeration is currently disabled due to interop issues.
        // Use the Win32 SetupAPI fallback or comment out this block for now.

        // Start a scan if auto-scan is enabled
        // if self.config.bluetooth.auto_scan_on_startup {
        //     tracing::info!("App::run: Starting scan at startup");
        //     if let Err(e) = self.start_scan().await {
        //         tracing::error!("App::run: Failed to start scan: {}", e);
        //     }
        // }
        
        // Event loop
        while self.running {
            // Check for UI messages
            if let Ok(message) = self.ui_rx.try_recv() {
                self.handle_message(message).await?;
            }
            
            // Sleep to avoid busy waiting
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        
        // Save configuration on exit
        if let Err(e) = self.save_config() {
            log::error!("Failed to save configuration: {}", e);
        }
        
        // Clean up battery monitoring task
        if let Some(task) = self.battery_monitor_task.take() {
            task.abort();
        }
        
        Ok(())
    }
    
    /// Handle a UI message
    async fn handle_message(&mut self, message: Message) -> Result<(), RustPodsError> {
        match message {
            Message::Exit => {
                self.running = false;
            }
            Message::SettingsChanged(new_config) => {
                // Update configuration
                self.config = new_config;
                // Apply the new configuration
                self.apply_config();
            }
            Message::SaveSettings => {
                let _ = self.save_config();
            }
            Message::DeviceDiscovered(device) if device.is_potential_airpods => {
                // If this is a potential AirPods device
                // Extract AirPods specific details if available
                if let Ok(Some(airpods)) = detect_airpods(&device) {
                    // Store the detected airpods in our state
                    if let Ok(mut current) = self.current_airpods.lock() {
                        *current = Some(airpods.clone());
                    }
                    // Notify UI about the detected device
                    let _ = self.ui_tx.send(Message::AirPodsConnected(airpods));
                }
            }
            Message::BatteryStatusUpdated(status) => {
                // Update battery status
                *self.battery_status.lock().unwrap() = status.clone();
                // Forward to UI
                let _ = self.ui_tx.send(Message::BatteryStatusUpdated(status));
            }
            _ => { /* Other messages are handled by the UI */ }
        }
        
        Ok(())
    }
    
    /// Start monitoring battery status for detected AirPods without providing the device
    pub async fn start_battery_monitoring(&mut self) -> Result<(), RustPodsError> {
        // Get the current AirPods device if available
        if let Some(airpods) = self.get_airpods() {
            // Use the existing method that takes an AirPods parameter
            self.start_battery_monitoring_for_device(&airpods).await
        } else {
            // No AirPods device found
            Err(RustPodsError::DeviceNotFound)
        }
    }
    
    /// Start monitoring battery status for a specific AirPods device
    pub async fn start_battery_monitoring_for_device(&mut self, airpods: &DetectedAirPods) -> Result<(), RustPodsError> {
        // Cancel any existing battery monitoring task
        if let Some(task) = self.battery_monitor_task.take() {
            task.abort();
        }
        
        // Get the peripheral device associated with the AirPods
        let peripheral = match self.get_peripheral_for_airpods(airpods).await {
            Some(peripheral) => peripheral,
            None => return Err(RustPodsError::DeviceNotFound),
        };
        
        // Get the refresh interval from config
        let refresh_interval = self.config.bluetooth.battery_refresh_interval;
        
        // Create the callback for battery updates
        let ui_tx = self.ui_tx.clone();
        let battery_status = self.battery_status.clone();
        let error_tx = self.ui_tx.clone();
        
        let callback = move |status: AirPodsBatteryStatus| {
            // Check if we got valid battery information
            if !status.has_battery_info() {
                // No battery info available, might indicate connection issue
                let _ = error_tx.send(Message::ShowToast("Reconnection attempt".to_string()));
                return;
            }
            
            // Update the battery status
            *battery_status.lock().unwrap() = status.clone();
            
            // Send battery update to UI
            let _ = ui_tx.send(Message::BatteryStatusUpdated(status));
        };
        
        // Start the battery monitoring with error handling
        let handle = match crate::bluetooth::start_battery_monitoring(
            &peripheral,
            callback,
            refresh_interval
        ).await {
            Ok(handle) => handle,
            Err(e) => return Err(RustPodsError::BatteryMonitorError(format!("{}", e))),
        };
        
        // Store the task handle
        self.battery_monitor_task = Some(handle);
        
        Ok(())
    }
    
    /// Get the peripheral device for the given AirPods
    async fn get_peripheral_for_airpods(&self, airpods: &DetectedAirPods) -> Option<btleplug::platform::Peripheral> {
        // Get all peripherals from the scanner
        match self.scanner.get_peripherals_by_address(&airpods.address).await {
            Ok(peripherals) if !peripherals.is_empty() => Some(peripherals[0].clone()),
            _ => None,
        }
    }
    
    /// Get the current battery status
    pub fn get_battery_status(&self) -> AirPodsBatteryStatus {
        self.battery_status.lock().unwrap().clone()
    }
    
    /// Get the current AirPods device
    pub fn get_airpods(&self) -> Option<DetectedAirPods> {
        self.current_airpods.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    // Note: Most of these tests are commented out because they depend on BLE hardware
    
    #[test]
    fn test_app_creation() {
        // This test doesn't actually test anything yet
        // TODO: Add meaningful tests when mocking infrastructure is in place
    }
} 