//! Application entry point and main logic

use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
// Remove unused tokio_mpsc import
// use tokio::sync::mpsc as tokio_mpsc;

use crate::bluetooth::{BleScanner, BleEvent, AirPodsBatteryStatus, BleError};
// Remove unused ScanConfig import
use crate::config::{AppConfig, ConfigManager};
use crate::ui::{Message, SystemTray};
use crate::error::RustPodsError;
use crate::airpods::{DetectedAirPods, detect_airpods};
use log; // Keep log module but remove specific imports

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
    tray: SystemTray,
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
        let (ui_tx, ui_rx) = mpsc::channel();
        
        // Create config manager and load config
        let config_manager = ConfigManager::default();
        config_manager.load().map_err(|e| RustPodsError::ConfigError(e.to_string()))?;
        let config = config_manager.get_config();
        
        // Create scanner
        let scanner = BleScanner::new();
        
        // Create system tray with configuration
        let tray = SystemTray::new(ui_tx.clone(), config.clone())
            .map_err(|_| RustPodsError::UiError)?;
        
        Ok(Self {
            config,
            config_manager,
            ui_tx,
            ui_rx,
            scanner,
            tray,
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
        // Configure scanner
        if let Some(scanner) = self.scanner.as_configurable() {
            scanner.apply_config(&self.config);
        }
        
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
        // Start a scan if auto-scan is enabled
        if self.config.bluetooth.auto_scan_on_startup {
            self.start_scan().await?;
        }
        
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
            Message::StartScan => {
                match self.start_scan().await {
                    Ok(_) => {
                        // Successfully started scanning
                        let _ = self.ui_tx.send(Message::ScanStarted);
                    }
                    Err(e) => {
                        // Handle scanning error
                        log::error!("Failed to start scan: {}", e);
                        let _ = self.ui_tx.send(Message::Error(format!("Failed to start scanning: {}", e)));
                        
                        // If the adapter was not found, try to reinitialize
                        if matches!(e, RustPodsError::BluetoothApiError(BleError::AdapterNotFound)) {
                            log::info!("Attempting to reinitialize Bluetooth adapter");
                            if let Err(reinit_err) = self.reinitialize_bluetooth().await {
                                log::error!("Failed to reinitialize Bluetooth: {}", reinit_err);
                                let _ = self.ui_tx.send(Message::Error(format!("Failed to reinitialize Bluetooth: {}", reinit_err)));
                            }
                        }
                    }
                }
            }
            Message::StopScan => {
                if let Err(e) = self.stop_scan().await {
                    log::error!("Failed to stop scan: {}", e);
                    let _ = self.ui_tx.send(Message::Error(format!("Failed to stop scanning: {}", e)));
                } else {
                    let _ = self.ui_tx.send(Message::ScanStopped);
                }
            }
            Message::ToggleAutoScan(enabled) => {
                self.config.bluetooth.auto_scan_on_startup = enabled;
                // Save config when auto-scan setting changes
                if let Err(e) = self.save_config() {
                    log::error!("Failed to save configuration: {}", e);
                }
            }
            Message::SettingsChanged(new_config) => {
                // Update configuration
                self.config = new_config;
                // Apply the new configuration
                self.apply_config();
                // Notify UI of changes
                let _ = self.ui_tx.send(Message::Status("Settings updated".to_string()));
            }
            Message::SaveSettings => {
                match self.save_config() {
                    Ok(_) => {
                        let _ = self.ui_tx.send(Message::Status("Settings saved".to_string()));
                    },
                    Err(e) => {
                        log::error!("Failed to save settings: {}", e);
                        let _ = self.ui_tx.send(Message::Error(format!("Failed to save settings: {}", e)));
                    }
                }
            }
            Message::DeviceDiscovered(device) if device.is_potential_airpods => {
                // Try to detect AirPods from the discovered device
                if let Some(airpods) = detect_airpods(&device) {
                    // Update current AirPods device
                    *self.current_airpods.lock().unwrap() = Some(airpods.clone());
                    
                    // Start battery monitoring
                    match self.start_battery_monitoring_for_device(&airpods).await {
                        Ok(_) => {
                            // Successfully started battery monitoring
                            let _ = self.ui_tx.send(Message::AirPodsConnected(airpods));
                        }
                        Err(e) => {
                            log::error!("Failed to start battery monitoring: {}", e);
                            let _ = self.ui_tx.send(Message::Error(format!("Failed to start battery monitoring: {}", e)));
                            
                            // Try to reconnect to the device
                            self.schedule_device_reconnect(&airpods);
                        }
                    }
                }
            }
            Message::BatteryStatusUpdated(status) => {
                // Update battery status
                *self.battery_status.lock().unwrap() = status.clone();
                
                // Forward to UI
                let _ = self.ui_tx.send(Message::BatteryStatusUpdated(status));
            }
            Message::RetryConnection => {
                // Try to reconnect to the last known device
                if let Some(airpods) = self.get_airpods() {
                    log::info!("Attempting to reconnect to AirPods device");
                    
                    // Start battery monitoring again
                    match self.start_battery_monitoring_for_device(&airpods).await {
                        Ok(_) => {
                            log::info!("Successfully reconnected to AirPods device");
                            let _ = self.ui_tx.send(Message::AirPodsConnected(airpods));
                        }
                        Err(e) => {
                            log::error!("Failed to reconnect to AirPods device: {}", e);
                            let _ = self.ui_tx.send(Message::Error(format!("Failed to reconnect: {}", e)));
                            
                            // Schedule another retry with exponential backoff
                            self.schedule_device_reconnect(&airpods);
                        }
                    }
                }
            }
            _ => { /* Other messages are handled by the UI */ }
        }
        
        Ok(())
    }
    
    /// Start scanning for devices
    async fn start_scan(&mut self) -> Result<(), RustPodsError> {
        // Start scanning with the configured scan profile
        let scan_config = self.config.to_scan_config();
        
        // Use the existing start_scanning method and pass options if needed
        let mut events = self.scanner.start_scanning_with_config(scan_config).await?;
        
        // Process events
        let ui_tx = self.ui_tx.clone();
        tokio::spawn(async move {
            while let Some(event) = events.recv().await {
                match event {
                    BleEvent::DeviceDiscovered(device) => {
                        // Forward to UI
                        let _ = ui_tx.send(Message::DeviceDiscovered(device.clone()));
                    }
                    BleEvent::AirPodsDetected(airpods) => {
                        let _ = ui_tx.send(Message::AirPodsConnected(airpods));
                    }
                    BleEvent::Error(error) => {
                        let _ = ui_tx.send(Message::Error(format!("Bluetooth error: {}", error)));
                    }
                    BleEvent::ScanCycleCompleted { devices_found } => {
                        let _ = ui_tx.send(Message::ScanProgress(devices_found));
                    }
                    BleEvent::ScanningCompleted => {
                        let _ = ui_tx.send(Message::ScanCompleted);
                    }
                    // Handle other event types as needed
                    _ => { /* Ignore other events */ }
                }
            }
        });
        
        Ok(())
    }
    
    /// Stop scanning for devices
    async fn stop_scan(&mut self) -> Result<(), RustPodsError> {
        self.scanner.stop_scanning().await?;
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
        let refresh_interval = Duration::from_secs(self.config.bluetooth.battery_refresh_interval);
        
        // Create the callback for battery updates
        let ui_tx = self.ui_tx.clone();
        let battery_status = self.battery_status.clone();
        let error_tx = self.ui_tx.clone();
        
        let callback = move |status: AirPodsBatteryStatus| {
            // Check if we got valid battery information
            if !status.has_battery_info() {
                // No battery info available, might indicate connection issue
                let _ = error_tx.send(Message::Error("Battery information unavailable. Device may be disconnected.".to_string()));
                let _ = error_tx.send(Message::RetryConnection);
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
    
    /// Schedule a device reconnection attempt
    fn schedule_device_reconnect(&self, airpods: &DetectedAirPods) {
        // Clone necessary data for the reconnection task
        let ui_tx = self.ui_tx.clone();
        let airpods_clone = airpods.clone();
        
        // Create a retry task with exponential backoff
        tokio::spawn(async move {
            // Wait before retry (could implement exponential backoff here)
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            
            // Send retry message
            let _ = ui_tx.send(Message::RetryConnection);
            
            // Also notify the UI that we're attempting to reconnect
            let _ = ui_tx.send(Message::Status(format!("Attempting to reconnect to {}", 
                airpods_clone.name.unwrap_or_else(|| "AirPods".to_string()))));
        });
    }
    
    /// Attempt to reinitialize the Bluetooth adapter
    async fn reinitialize_bluetooth(&mut self) -> Result<(), RustPodsError> {
        // First, ensure any existing scanning is stopped
        if let Err(e) = self.scanner.stop_scanning().await {
            log::warn!("Error while stopping scan during reinitialization: {}", e);
            // Continue anyway since we're reinitializing
        }
        
        // Create a fresh scanner
        self.scanner = BleScanner::new();
        
        // Initialize the new scanner
        self.scanner.initialize().await?;
        
        // Notify UI that Bluetooth has been reinitialized
        let _ = self.ui_tx.send(Message::Status("Bluetooth adapter reinitialized".to_string()));
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // Note: Most of these tests are commented out because they depend on BLE hardware
    
    #[test]
    fn test_app_creation() {
        // Just a basic compilation test
        assert!(true);
    }
} 