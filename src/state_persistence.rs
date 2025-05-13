//! State persistence module for saving and restoring application state between sessions
//! 
//! This module handles persistent storage of application state, including device connections,
//! settings, and UI state. It works alongside the LifecycleManager to ensure state is
//! preserved across application restarts.

use std::sync::Arc;
use std::path::PathBuf;
use std::fs::{self};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::sync::Mutex;

use crate::bluetooth::AirPodsBatteryStatus;
use crate::config::AppConfig;
use crate::ui::state_manager::StateManager;
use crate::bluetooth::scanner::DiscoveredDevice;

/// Persistent state data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentState {
    /// Application version when state was saved
    pub app_version: String,
    
    /// When the state was saved
    pub last_saved: chrono::DateTime<chrono::Utc>,
    
    /// Connected devices' addresses
    pub connected_devices: Vec<String>,
    
    /// Main window visibility
    pub window_visible: bool,
    
    /// UI theme
    pub theme: String,
    
    /// Auto-scan setting
    pub auto_scan: bool,
    
    /// Advanced display mode
    pub advanced_display_mode: bool,
    
    /// Known devices
    pub known_devices: Vec<DiscoveredDevice>,
    
    /// Application configuration
    pub config: AppConfig,
    
    /// Battery status
    pub battery_status: Option<AirPodsBatteryStatus>,
    
    /// Selected device
    pub selected_device: Option<String>,
    
    /// Connection state
    pub connection_state: String,
}

/// Result type for persistence operations
pub type Result<T> = std::result::Result<T, String>;

/// Manager for saving and loading persistent state
#[derive(Clone)]
pub struct StatePersistenceManager {
    /// State manager reference
    state_manager: Arc<StateManager>,
    
    /// Last save timestamp
    last_save: Arc<Mutex<DateTime<Utc>>>,
    
    /// Path to state file
    state_path: PathBuf,
}

impl StatePersistenceManager {
    /// Create a new state persistence manager
    pub fn new(state_manager: Arc<StateManager>) -> Self {
        let state_path = Self::get_state_file_path()
            .unwrap_or_else(|_| PathBuf::from("rustpods_state.json"));
        
        Self {
            state_manager,
            last_save: Arc::new(Mutex::new(Utc::now())),
            state_path,
        }
    }
    
    /// Check if a previous state file exists
    pub fn state_exists(&self) -> bool {
        self.state_path.exists()
    }
    
    /// Save the current state to disk
    pub fn save_state(&mut self) -> Result<()> {
        log::info!("Saving persistent state to disk");
        
        // Get the current state from state manager
        let device_state = self.state_manager.get_device_state();
        let config = self.state_manager.get_config();
        let ui_state = self.state_manager.get_ui_state();
        
        // Create persistent state object
        let state = PersistentState {
            app_version: "1.0.0".to_string(),
            last_saved: Utc::now(),
            connected_devices: vec![device_state.selected_device.clone().unwrap_or_default()],
            window_visible: ui_state.visible,
            theme: format!("{:?}", config.ui.theme),
            auto_scan: device_state.auto_scan,
            advanced_display_mode: false, // Default value since it's not in UiState
            known_devices: device_state.devices.values().cloned().collect(),
            config: config.clone(),
            battery_status: device_state.battery_status.clone(),
            selected_device: device_state.selected_device.clone(),
            connection_state: format!("{:?}", device_state.connection_state),
        };
        
        // Serialize to JSON
        let json = serde_json::to_string_pretty(&state)
            .map_err(|e| format!("Failed to serialize state: {}", e))?;
        
        // Write to file
        fs::write(&self.state_path, json)
            .map_err(|e| format!("Failed to write state file: {}", e))?;
        
        // Update last save timestamp
        *self.last_save.lock().unwrap() = Utc::now();
        
        log::info!("Successfully saved persistent state");
        Ok(())
    }
    
    /// Load the persistent state from disk
    pub fn load_state(&self) -> Result<PersistentState> {
        log::info!("Loading persistent state from disk");
        
        // Check if file exists
        if !self.state_path.exists() {
            return Err("State file does not exist".to_string());
        }
        
        // Read file content
        let json = fs::read_to_string(&self.state_path)
            .map_err(|e| format!("Failed to read state file: {}", e))?;
        
        // Deserialize JSON
        let state: PersistentState = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse state file: {}", e))?;
        
        log::info!("Successfully loaded persistent state");
        Ok(state)
    }
    
    /// Apply persistent state to the application
    pub fn apply_state(&self, state: PersistentState) -> Result<()> {
        log::info!("Applying persistent state to application");
        
        // Apply auto-scan setting
        let action = crate::ui::state_manager::Action::ToggleAutoScan(state.auto_scan);
        self.state_manager.dispatch(action);
        
        // Apply advanced display mode setting
        let action = crate::ui::state_manager::Action::SetAdvancedDisplayMode(state.advanced_display_mode);
        self.state_manager.dispatch(action);
        
        // Restore known devices
        for device in state.known_devices {
            let action = crate::ui::state_manager::Action::UpdateDevice(device);
            self.state_manager.dispatch(action);
        }
        
        // Apply config settings
        let action = crate::ui::state_manager::Action::UpdateSettings(state.config);
        self.state_manager.dispatch(action);
        
        // Restore battery status if available
        if let Some(battery_status) = state.battery_status {
            let action = crate::ui::state_manager::Action::UpdateBatteryStatus(battery_status);
            self.state_manager.dispatch(action);
        }
        
        // Reconnect to selected device if available
        if let Some(device_address) = state.selected_device {
            // Only reconnect if we were connected before
            if state.connection_state == "Connected" || state.connection_state == "Reconnecting" {
                log::info!("Restoring connection to device: {}", device_address);
                
                let action = crate::ui::state_manager::Action::RestorePreviousConnection(device_address);
                self.state_manager.dispatch(action);
            }
        }
        
        log::info!("Successfully applied persistent state");
        Ok(())
    }
    
    /// Delete persistent state file
    pub fn delete_state(&self) -> Result<()> {
        if self.state_path.exists() {
            fs::remove_file(&self.state_path)
                .map_err(|e| format!("Failed to delete state file: {}", e))?;
            
            log::info!("Deleted persistent state file");
        }
        
        Ok(())
    }
    
    /// Get the time since the last save
    pub fn time_since_last_save(&self) -> chrono::Duration {
        let last_save = *self.last_save.lock().unwrap();
        Utc::now() - last_save
    }
    
    /// Get the path to the state file
    fn get_state_file_path() -> Result<PathBuf> {
        let data_dir = dirs::data_local_dir()
            .ok_or_else(|| "Could not determine local data directory".to_string())?;
        
        let app_dir = data_dir.join("RustPods");
        
        // Create directory if it doesn't exist
        if !app_dir.exists() {
            fs::create_dir_all(&app_dir)
                .map_err(|e| format!("Failed to create app data directory: {}", e))?;
        }
        
        Ok(app_dir.join("app_state.json"))
    }
} 