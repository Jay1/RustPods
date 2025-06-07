//! Application state controller that manages Bluetooth connections, UI state,
//! and lifecycle events within RustPods

use std::sync::Arc;
use tokio::sync::mpsc;
use log::{error, info, warn};
use std::time::Duration;

use crate::bluetooth::adapter::BluetoothAdapter;
use crate::bluetooth::scanner::BleScanner;
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
    #[allow(dead_code)]
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
        
        info!("AppStateController initialized successfully");
        
        Ok(())
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
        
        // Only attempt to select the device if it's present
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