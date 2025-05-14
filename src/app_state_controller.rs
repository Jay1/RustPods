//! Application state controller that manages Bluetooth connections, UI state,
//! and lifecycle events within RustPods

use std::sync::Arc;
use tokio::sync::mpsc;
use log::{debug, error, info, warn};
use std::time::Duration;

use crate::bluetooth::adapter::BluetoothAdapter;
use crate::bluetooth::scanner::{BleScanner, BleScannerConfig, DiscoveredDevice};
use crate::bluetooth::events::BleEvent;
use crate::airpods::detector::AirPodsDetector;
use crate::ui::Message;
use crate::ui::state_manager::{StateManager, Action, ConnectionState};
use crate::lifecycle_manager::LifecycleManager;
use crate::state_persistence::StatePersistenceManager;

/// Controller that manages application state and mediates between UI and Bluetooth components
pub struct AppStateController {
    /// Bluetooth adapter
    adapter: Option<BluetoothAdapter>,
    
    /// BLE scanner
    scanner: Option<BleScanner>,
    
    /// Message sender for UI updates
    ui_sender: mpsc::UnboundedSender<Message>,
    
    /// State manager
    state_manager: Arc<StateManager>,
    
    /// AirPods detector
    airpods_detector: AirPodsDetector,
    
    /// Lifecycle manager
    lifecycle_manager: Option<LifecycleManager>,
    
    /// State persistence manager
    persistence_manager: Option<StatePersistenceManager>,
}

impl AppStateController {
    /// Create a new app state controller
    pub fn new(ui_sender: mpsc::UnboundedSender<Message>) -> Self {
        let state_manager = Arc::new(StateManager::new(ui_sender.clone()));
        
        Self {
            adapter: None,
            scanner: None,
            ui_sender: ui_sender.clone(),
            state_manager,
            airpods_detector: AirPodsDetector::new(),
            lifecycle_manager: None,
            persistence_manager: None,
        }
    }
    
    /// Initialize the controller
    pub async fn initialize(&mut self) -> Result<(), String> {
        info!("Initializing AppStateController");
        
        // Notify application starting
        self.ui_sender.send(Message::AppStarting).map_err(|e| e.to_string())?;
        
        // Create lifecycle manager
        let lifecycle_manager = LifecycleManager::new(
            Arc::clone(&self.state_manager),
            self.ui_sender.clone()
        ).with_auto_save_interval(Duration::from_secs(300)); // 5 minutes
        
        self.lifecycle_manager = Some(lifecycle_manager);
        
        // Create state persistence manager
        let persistence_manager = StatePersistenceManager::new(Arc::clone(&self.state_manager));
        self.persistence_manager = Some(persistence_manager);
        
        // Initialize Bluetooth adapter
        match BluetoothAdapter::new().await {
            Ok(adapter) => {
                self.adapter = Some(adapter);
                info!("Bluetooth adapter initialized successfully");
            },
            Err(e) => {
                error!("Failed to initialize Bluetooth adapter: {}", e);
                return Err(format!("Failed to initialize Bluetooth adapter: {}", e));
            }
        }
        
        // Initialize lifecycle manager
        if let Some(lifecycle_manager) = &mut self.lifecycle_manager {
            if let Err(e) = lifecycle_manager.start() {
                error!("Failed to start lifecycle manager: {}", e);
                // Continue despite error - app can still work without lifecycle management
            }
        }
        
        // Notify application initialized
        self.ui_sender.send(Message::AppInitialized).map_err(|e| e.to_string())?;
        info!("AppStateController initialized successfully");
        
        Ok(())
    }
    
    /// Start Bluetooth scanning
    pub async fn start_scanning(&mut self) -> Result<(), String> {
        if self.scanner.is_some() {
            warn!("Scanner is already active, ignoring start_scanning request");
            return Ok(());
        }
        
        let adapter = match &self.adapter {
            Some(adapter) => adapter,
            None => {
                error!("Cannot start scanning - Bluetooth adapter not initialized");
                return Err("Bluetooth adapter not initialized".to_string());
            }
        };
        
        // Get config from state manager
        let config = self.state_manager.get_config().bluetooth.clone();
        
        // Create scanner config
        let scanner_config = BleScannerConfig {
            scan_duration: config.scan_duration,
            interval_between_scans: config.scan_interval,
            filter_known_devices: false, // Default value - not in config
            update_rssi_only: false, // Default value
            update_interval: config.scan_interval, // Use scan_interval as update_interval
            scan_timeout: None, // No timeout
            max_retries: 3, // Default value
            retry_delay: std::time::Duration::from_millis(500), // Default value
        };
        
        // Create and start scanner
        info!("Creating BLE scanner with config: {:?}", scanner_config);
        
        // Get adapter from BluetoothAdapter
        let adapter = adapter.get_adapter();
        
        // Create scanner with adapter and config
        let mut scanner = BleScanner::with_adapter_config(adapter, scanner_config);
        
        // Listen for scanner events
        let event_rx = match scanner.start_scanning().await {
            Ok(rx) => rx,
            Err(e) => {
                error!("Failed to start BLE scanner: {}", e);
                return Err(format!("Failed to start scanning: {}", e));
            }
        };
        
        self.scanner = Some(scanner);
        
        // Start a task to process scanner events
        let ui_sender = self.ui_sender.clone();
        let state_manager = Arc::clone(&self.state_manager);
        let airpods_detector = self.airpods_detector.clone();
        
        tokio::spawn(async move {
            Self::process_scanner_events(event_rx, ui_sender, state_manager, airpods_detector).await;
        });
        
        // Update state
        let action = Action::StartScanning;
        self.state_manager.dispatch(action);
        
        info!("Started Bluetooth scanning");
        Ok(())
    }
    
    /// Stop Bluetooth scanning
    pub async fn stop_scanning(&mut self) -> Result<(), String> {
        if let Some(mut scanner) = self.scanner.take() {
            info!("Stopping Bluetooth scanning");
            
            if let Err(e) = scanner.stop_scanning().await {
                error!("Error stopping scanner: {}", e);
                return Err(format!("Error stopping scanner: {}", e));
            }
            
            // Update state
            let action = Action::StopScanning;
            self.state_manager.dispatch(action);
            
            info!("Bluetooth scanning stopped");
        } else {
            warn!("Cannot stop scanning - scanner not active");
        }
        
        Ok(())
    }
    
    /// Process scanner events
    async fn process_scanner_events(
        mut event_rx: mpsc::Receiver<BleEvent>,
        ui_sender: mpsc::UnboundedSender<Message>,
        state_manager: Arc<StateManager>,
        airpods_detector: AirPodsDetector,
    ) {
        while let Some(event) = event_rx.recv().await {
            match event {
                BleEvent::DeviceDiscovered(device) => {
                    // Handle both discovered and updated devices with the same variant
                    Self::handle_device_discovered(
                        device.clone(),
                        &ui_sender,
                        &state_manager,
                        &airpods_detector,
                    );
                    
                    // Also call the update handler for existing devices
                    Self::handle_device_updated(
                        device,
                        &ui_sender,
                        &state_manager,
                        &airpods_detector,
                    );
                },
                BleEvent::Error(error) => {
                    error!("BLE error: {}", error);
                    
                    // Notify UI of error
                    if let Err(e) = ui_sender.send(Message::BluetoothError(error)) {
                        error!("Failed to send error message: {}", e);
                    }
                },
                BleEvent::AirPodsDetected(airpods) => {
                    debug!("AirPods detected: {:?}", airpods);
                    
                    // Send to UI
                    if let Err(e) = ui_sender.send(Message::AirPodsConnected(airpods)) {
                        error!("Failed to send AirPods detected event to UI: {}", e);
                    }
                },
                BleEvent::DeviceLost(addr) => {
                    debug!("Device lost: {}", addr);
                    
                    // Notify UI of device lost
                    if let Err(e) = ui_sender.send(Message::DeviceDisconnected) {
                        error!("Failed to send device lost message: {}", e);
                    }
                    
                    // Update state
                    state_manager.dispatch(Action::RemoveDevice(addr.to_string()));
                    
                    // Handle device lost if needed
                },
                BleEvent::ScanCycleCompleted { devices_found } => {
                    debug!("Scan cycle completed with {} devices found", devices_found);
                    
                    // Notify UI of scan progress
                    if let Err(e) = ui_sender.send(Message::ScanProgress(devices_found)) {
                        error!("Failed to send scan progress message: {}", e);
                    }
                    
                    // Update state (scanning is still in progress)
                    state_manager.dispatch(Action::StartScanning);
                },
                BleEvent::ScanningCompleted => {
                    debug!("Scanning completed");
                    
                    // Notify UI of scan completion
                    if let Err(e) = ui_sender.send(Message::ScanCompleted) {
                        error!("Failed to send scan completed message: {}", e);
                    }
                    
                    // Update state 
                    state_manager.dispatch(Action::StopScanning);
                },
                _ => {
                    // Handle any other events
                }
            }
        }
        
        debug!("Scanner event channel closed");
    }
    
    /// Handle device discovered event
    fn handle_device_discovered(
        device: DiscoveredDevice,
        ui_sender: &mpsc::UnboundedSender<Message>,
        state_manager: &Arc<StateManager>,
        airpods_detector: &AirPodsDetector,
    ) {
        let device_addr = device.address.to_string();
        let is_airpods = device.is_potential_airpods || 
                         airpods_detector.is_airpods(&device);
        
        let device_name = device.name.clone().unwrap_or_else(|| device_addr.clone());
        debug!("Device discovered: {} ({}), is_airpods: {}", device_name, device_addr, is_airpods);
        
        // Update device with airpods detection result
        let mut device = device.clone();
        device.is_potential_airpods = is_airpods;
        
        // Send to UI
        if let Err(e) = ui_sender.send(Message::DeviceDiscovered(device.clone())) {
            error!("Failed to send DeviceDiscovered message: {}", e);
        }
        
        // Update state
        let action = Action::UpdateDevice(device);
        state_manager.dispatch(action);
    }
    
    /// Handle device updated event
    fn handle_device_updated(
        device: DiscoveredDevice,
        ui_sender: &mpsc::UnboundedSender<Message>,
        state_manager: &Arc<StateManager>,
        airpods_detector: &AirPodsDetector,
    ) {
        let device_addr = device.address.to_string();
        let is_airpods = device.is_potential_airpods || 
                         airpods_detector.is_airpods(&device);
        
        // Update device with airpods detection result
        let mut device = device.clone();
        device.is_potential_airpods = is_airpods;
        
        // Send to UI
        if let Err(e) = ui_sender.send(Message::DeviceUpdated(device.clone())) {
            error!("Failed to send DeviceUpdated message: {}", e);
        }
        
        // Update state
        let action = Action::UpdateDevice(device);
        state_manager.dispatch(action);
    }
    
    /// Select a device for connection
    pub async fn select_device(&mut self, device_address: String) -> Result<(), String> {
        info!("Selecting device: {}", device_address);
        
        // Get device from state manager
        let device_state = self.state_manager.get_device_state();
        let device = match device_state.devices.get(&device_address) {
            Some(device) => device.clone(),
            None => {
                error!("Device not found: {}", device_address);
                return Err(format!("Device not found: {}", device_address));
            }
        };
        
        // Only connect if it's an AirPods device
        if !device.is_potential_airpods {
            warn!("Device is not recognized as AirPods: {}", device_address);
            return Err("Device is not recognized as AirPods".to_string());
        }
        
        // Update connection state
        let action = Action::SelectDevice(device_address.clone());
        self.state_manager.dispatch(action);
        
        // Update connection state in UI
        let action = Action::SetConnectionState(ConnectionState::Connecting);
        self.state_manager.dispatch(action);
        
        info!("Device selected: {}", device_address);
        Ok(())
    }
    
    /// Handle system sleep event
    pub fn handle_sleep(&mut self) -> Result<(), String> {
        info!("Handling system sleep event");
        
        // Stop scanning if active
        if self.scanner.is_some() {
            // Create clones for the async task
            let mut scanner_clone = self.scanner.clone();
            
            // Spawn task to stop scanning without blocking
            tokio::spawn(async move {
                if let Some(mut scanner) = scanner_clone.take() {
                    if let Err(e) = scanner.stop_scanning().await {
                        error!("Failed to stop scanning on sleep: {}", e);
                    }
                }
            });
        }
        
        // Let lifecycle manager handle sleep
        if let Some(lifecycle_manager) = &mut self.lifecycle_manager {
            lifecycle_manager.handle_sleep();
        }
        
        // Save state before sleep
        if let Some(persistence_manager) = &mut self.persistence_manager {
            if let Err(e) = persistence_manager.save_state() {
                error!("Failed to save state before sleep: {}", e);
                // Continue despite error - not critical
            }
        }
        
        Ok(())
    }
    
    /// Handle system wake event
    pub fn handle_wake(&mut self) -> Result<(), String> {
        info!("Handling system wake event");
        
        // Let lifecycle manager handle wake
        if let Some(lifecycle_manager) = &mut self.lifecycle_manager {
            lifecycle_manager.handle_wake();
        }
        
        // Restart scanning if auto-scan is enabled
        let device_state = self.state_manager.get_device_state();
        if device_state.auto_scan {
            // Clone necessary components
            let ui_sender_clone = self.ui_sender.clone();
            let state_manager_clone = Arc::clone(&self.state_manager);
            
            // Spawn a new task for starting scanning
            tokio::spawn(async move {
                // Create a new controller instance for the async task
                let mut temp_controller = AppStateController::new(ui_sender_clone);
                temp_controller.state_manager = state_manager_clone;
                
                // Initialize and start scanning
                if let Err(e) = temp_controller.initialize().await {
                    error!("Failed to initialize controller on wake: {}", e);
                    return;
                }
                
                if let Err(e) = temp_controller.start_scanning().await {
                    error!("Failed to restart scanning on wake: {}", e);
                }
            });
        }
        
        Ok(())
    }
    
    /// Force save application state
    pub fn force_save(&mut self) -> Result<(), String> {
        info!("Force saving application state");
        
        // Let lifecycle manager handle save
        if let Some(lifecycle_manager) = &mut self.lifecycle_manager {
            lifecycle_manager.force_save()?;
            info!("Application state saved successfully");
        } else {
            // Try to use persistence manager directly if lifecycle manager is not available
            if let Some(persistence_manager) = &mut self.persistence_manager {
                persistence_manager.save_state()?;
                info!("Application state saved successfully via persistence manager");
            } else {
                warn!("Cannot save state - neither lifecycle manager nor persistence manager is initialized");
                return Err("State persistence not initialized".to_string());
            }
        }
        
        Ok(())
    }
    
    /// Handle application shutdown
    pub async fn shutdown(&mut self) -> Result<(), String> {
        info!("Shutting down AppStateController");
        
        // Stop scanning if active
        if self.scanner.is_some() {
            if let Err(e) = self.stop_scanning().await {
                error!("Error stopping scanner during shutdown: {}", e);
                // Continue with shutdown despite errors
            }
        }
        
        // Save state before shutdown
        if let Some(persistence_manager) = &mut self.persistence_manager {
            if let Err(e) = persistence_manager.save_state() {
                error!("Failed to save state during shutdown: {}", e);
                // Continue with shutdown despite errors
            }
        }
        
        // Shut down lifecycle manager
        if let Some(lifecycle_manager) = &mut self.lifecycle_manager {
            if let Err(e) = lifecycle_manager.shutdown() {
                error!("Error shutting down lifecycle manager: {}", e);
                // Continue with shutdown despite errors
            }
        }
        
        info!("AppStateController shutdown complete");
        Ok(())
    }
    
    /// Restore a previous connection when the app starts
    pub async fn restore_previous_connection(&mut self, device_address: String) -> Result<(), String> {
        info!("Attempting to restore previous connection to: {}", device_address);
        
        // Start scanning to discover the device first
        if self.scanner.is_none() {
            if let Err(e) = self.start_scanning().await {
                error!("Failed to start scanning for previous device: {}", e);
                return Err(e);
            }
        }
        
        // Give some time for the device to be discovered
        tokio::time::sleep(Duration::from_secs(3)).await;
        
        // Try to select the device if it's been discovered
        if self.state_manager.get_device_state().devices.contains_key(&device_address) {
            self.select_device(device_address).await?;
            info!("Successfully restored previous connection");
            Ok(())
        } else {
            // Set connection state to reconnecting and try again later
            let action = Action::SetConnectionState(ConnectionState::Reconnecting);
            self.state_manager.dispatch(action);
            
            warn!("Device not found during reconnection, setting state to reconnecting");
            Err("Device not found, will try to reconnect later".to_string())
        }
    }
    
    /// Get reference to state manager
    pub fn get_state_manager(&self) -> Arc<StateManager> {
        Arc::clone(&self.state_manager)
    }
    
    /// Toggle the application display mode (simple/advanced)
    pub fn toggle_display_mode(&mut self) -> Result<(), String> {
        info!("Toggling display mode");
        
        // Get current mode (this would be stored in state in a real implementation)
        let is_advanced = self.state_manager.is_advanced_display_mode();
        
        // Update state
        let action = Action::SetAdvancedDisplayMode(!is_advanced);
        self.state_manager.dispatch(action);
        
        // Save state after changing display mode
        self.force_save()?;
        
        Ok(())
    }
    
    /// Load saved state
    pub fn load_state(&mut self) -> Result<(), String> {
        info!("Loading saved application state");
        
        if let Some(persistence_manager) = &self.persistence_manager {
            if persistence_manager.state_exists() {
                match persistence_manager.load_state() {
                    Ok(state) => {
                        // Apply the state
                        if let Err(e) = persistence_manager.apply_state(state) {
                            error!("Failed to apply persistent state: {}", e);
                            return Err(e);
                        }
                        
                        info!("Successfully loaded and applied persistent state");
                        Ok(())
                    },
                    Err(e) => {
                        error!("Failed to load persistent state: {}", e);
                        Err(e)
                    }
                }
            } else {
                info!("No saved state exists");
                Ok(())
            }
        } else {
            warn!("Cannot load state - persistence manager is not initialized");
            Err("State persistence not initialized".to_string())
        }
    }
    
    /// Start the controller
    pub async fn start(&mut self) -> Result<(), String> {
        info!("Starting AppStateController");
        
        // Initialize the controller
        if let Err(e) = self.initialize().await {
            error!("Failed to initialize controller: {}", e);
            return Err(format!("Failed to initialize controller: {}", e));
        }
        
        // Load saved state
        if let Err(e) = self.load_state() {
            warn!("Failed to load persistent state: {}", e);
            // Continue despite error - app can work with default state
        }
        
        // Start scanning if auto-scan is enabled
        let device_state = self.state_manager.get_device_state();
        if device_state.auto_scan {
            if let Err(e) = self.start_scanning().await {
                error!("Failed to start automatic scanning: {}", e);
                // Continue despite error - user can start scanning manually
            }
        }
        
        // Restore previous connection if enabled and available
        let settings = self.state_manager.get_config();
        if settings.bluetooth.auto_reconnect {
            if let Some(last_device) = self.state_manager.get_device_state().selected_device.clone() {
                // Get copies of needed resources for the async task
                let ui_sender_clone = self.ui_sender.clone();
                let last_device_clone = last_device.clone();
                
                // Spawn task to handle reconnection without blocking main flow
                tokio::spawn(async move {
                    // Small delay to allow scanning to start
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    
                    // Create a new controller for the async task
                    let mut reconnect_controller = AppStateController::new(ui_sender_clone);
                    if let Err(e) = reconnect_controller.initialize().await {
                        warn!("Failed to initialize controller for auto-reconnect: {}", e);
                        return;
                    }
                    
                    // Try to reconnect
                    if let Err(e) = reconnect_controller.restore_previous_connection(last_device_clone).await {
                        warn!("Auto-reconnect failed: {}", e);
                        // This is expected if the device is not available
                    }
                });
            }
        }
        
        info!("AppStateController started successfully");
        Ok(())
    }
} 