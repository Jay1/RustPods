//! Application lifecycle management
//! 
//! This module handles application lifecycle events including startup, shutdown, 
//! and system events like sleep/wake.

use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;

use crate::config::AppConfig;
use crate::ui::Message;
use crate::ui::state_manager::StateManager;
use crate::state_persistence::StatePersistenceManager;

/// Application lifecycle states
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleState {
    /// Application is starting up
    Starting,
    /// Application is running normally
    Running,
    /// Application is in background (minimized)
    Background,
    /// System is in sleep or hibernate mode
    Sleep,
    /// Application is shutting down
    ShuttingDown,
}

/// Application lifecycle manager
pub struct LifecycleManager {
    /// Current lifecycle state
    state: Arc<Mutex<LifecycleState>>,
    /// State manager
    state_manager: Arc<StateManager>,
    /// Message sender to UI
    ui_sender: mpsc::UnboundedSender<Message>,
    /// Last save timestamp
    last_save: Arc<Mutex<Instant>>,
    /// Auto-save interval in seconds
    auto_save_interval: Duration,
    /// Auto-save task handle
    auto_save_task: Option<JoinHandle<()>>,
    /// System event task handle
    system_event_task: Option<JoinHandle<()>>,
    /// Crash recovery task handle
    crash_recovery_task: Option<JoinHandle<()>>,
    /// State persistence manager
    persistence_manager: Option<StatePersistenceManager>,
}

impl Clone for LifecycleManager {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            state_manager: Arc::clone(&self.state_manager),
            ui_sender: self.ui_sender.clone(),
            last_save: Arc::clone(&self.last_save),
            auto_save_interval: self.auto_save_interval,
            auto_save_task: None,
            system_event_task: None,
            crash_recovery_task: None,
            persistence_manager: None,
        }
    }
}

impl LifecycleManager {
    /// Create a new lifecycle manager
    pub fn new(state_manager: Arc<StateManager>, ui_sender: mpsc::UnboundedSender<Message>) -> Self {
        Self {
            state: Arc::new(Mutex::new(LifecycleState::Starting)),
            state_manager,
            ui_sender,
            last_save: Arc::new(Mutex::new(Instant::now())),
            auto_save_interval: Duration::from_secs(300), // 5 minutes default
            auto_save_task: None,
            system_event_task: None,
            crash_recovery_task: None,
            persistence_manager: None,
        }
    }
    
    /// Set the auto-save interval
    pub fn with_auto_save_interval(mut self, interval: Duration) -> Self {
        self.auto_save_interval = interval;
        self
    }
    
    /// Start the lifecycle manager
    pub fn start(&mut self) -> Result<(), String> {
        // Initialize state persistence manager
        self.persistence_manager = Some(StatePersistenceManager::new(Arc::clone(&self.state_manager)));
        
        // Try to load persistent state
        if let Some(persistence_manager) = &self.persistence_manager {
            if persistence_manager.state_exists() {
                match persistence_manager.load_state() {
                    Ok(state) => {
                        // Apply the state
                        if let Err(e) = persistence_manager.apply_state(state) {
                            log::warn!("Failed to apply persistent state: {}", e);
                        } else {
                            log::info!("Successfully loaded and applied persistent state");
                        }
                    },
                    Err(e) => {
                        log::warn!("Failed to load persistent state: {}", e);
                    }
                }
            }
        }
        
        // Check for crash recovery
        match self.check_for_recovery_state() {
            Ok(true) => {
                log::info!("Successfully recovered from previous session state");
            },
            Ok(false) => {
                log::info!("No recovery state found or recovery state is too old");
            },
            Err(e) => {
                log::warn!("Failed to check recovery state: {}", e);
            }
        }
        
        // Start periodic state saving for crash recovery
        self.start_crash_recovery_task();
        
        // Start auto-save task
        self.start_auto_save();
        
        // Register for system events
        self.register_system_events();
        
        // Set state to running
        let mut state = self.state.lock().unwrap();
        *state = LifecycleState::Running;
        
        Ok(())
    }
    
    /// Start auto-save task
    fn start_auto_save(&mut self) {
        // Get necessary clones for the task
        let state_manager = Arc::clone(&self.state_manager);
        let last_save = Arc::clone(&self.last_save);
        let auto_save_interval = self.auto_save_interval;
        let persistence_manager = self.persistence_manager.clone();
        
        // Create the auto-save task
        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30)); // Check every 30 seconds
            
            loop {
                interval.tick().await;
                
                // Check if we need to save
                let duration_since_save = {
                    let last_save_time = *last_save.lock().unwrap();
                    last_save_time.elapsed()
                };
                
                if duration_since_save >= auto_save_interval {
                    // Save config
                    let config = state_manager.get_config();
                    if let Err(e) = config.save() {
                        log::error!("Failed to auto-save config: {}", e);
                    } else {
                        log::info!("Auto-saved configuration");
                    }
                    
                    // Save persistent state
                    if let Some(mut persistence_manager) = persistence_manager.clone() {
                        if let Err(e) = persistence_manager.save_state() {
                            log::error!("Failed to auto-save persistent state: {}", e);
                        } else {
                            log::info!("Auto-saved persistent state");
                            
                            // Update last save time
                            let mut last_save_time = last_save.lock().unwrap();
                            *last_save_time = Instant::now();
                        }
                    }
                }
            }
        });
        
        self.auto_save_task = Some(task);
    }
    
    /// Register for system events
    fn register_system_events(&mut self) {
        // Get necessary clones for the task
        let state = Arc::clone(&self.state);
        let ui_sender = self.ui_sender.clone();
        let state_manager = Arc::clone(&self.state_manager);
        let persistence_manager = self.persistence_manager.clone();
        
        // Create the system event task
        #[cfg(target_os = "windows")]
        let task = tokio::spawn(async move {
            // On Windows, we would normally use the Power APIs to listen for sleep/wake events
            // But for this implementation, we'll use a timer-based approach to simulate these events
            let mut interval = tokio::time::interval(Duration::from_secs(2));
            
            loop {
                interval.tick().await;
                
                // Simulate receiving a power event
                // In a real implementation, this would come from Windows message loop
                
                // Check current state
                let current_state = {
                    let state_guard = state.lock().unwrap();
                    (*state_guard).clone()
                };
                
                // Handle based on current state
                match current_state {
                    LifecycleState::Sleep => {
                        if rand::random::<f32>() < 0.1 {
                            *state.lock().unwrap() = LifecycleState::Running;
                            ui_sender.send(Message::SystemWake).ok();
                            let action = crate::ui::state_manager::Action::SystemWake;
                            state_manager.dispatch(action);
                            log::info!("System woke from sleep");
                        }
                    },
                    LifecycleState::Running => {
                        if rand::random::<f32>() < 0.01 {
                            *state.lock().unwrap() = LifecycleState::Sleep;
                            
                            // Save state before sleep
                            if let Some(mut persistence_manager) = persistence_manager.clone() {
                                if let Err(e) = persistence_manager.save_state() {
                                    log::error!("Failed to save state before sleep: {}", e);
                                }
                            }
                            
                            ui_sender.send(Message::SystemSleep).ok();
                            let config = state_manager.get_config();
                            if let Err(e) = config.save() {
                                log::error!("Failed to save config before sleep: {}", e);
                            }
                            log::info!("System entering sleep mode");
                        }
                    },
                    _ => {}
                }
            }
        });
        
        #[cfg(not(target_os = "windows"))]
        let task = tokio::spawn(async move {
            // For non-Windows platforms, implement similar logic
            // but using platform-specific APIs
            
            let mut interval = tokio::time::interval(Duration::from_secs(2));
            
            loop {
                interval.tick().await;
                
                // Simplified logic similar to Windows version
                // Would be replaced with proper OS-specific event handling
                
                let current_state = {
                    let state_guard = state.lock().unwrap();
                    (*state_guard).clone()
                };
                
                // Simple state machine similar to Windows version
                // but without the Windows-specific API calls
                
                match current_state {
                    LifecycleState::Sleep => {
                        if rand::random::<f32>() < 0.1 {
                            *state.lock().unwrap() = LifecycleState::Running;
                            ui_sender.send(Message::SystemWake).ok();
                            let action = crate::ui::state_manager::Action::SystemWake;
                            state_manager.dispatch(action);
                            log::info!("System woke from sleep");
                        }
                    },
                    LifecycleState::Running => {
                        if rand::random::<f32>() < 0.01 {
                            *state.lock().unwrap() = LifecycleState::Sleep;
                            
                            // Save state before sleep
                            if let Some(mut persistence_manager) = persistence_manager.clone() {
                                if let Err(e) = persistence_manager.save_state() {
                                    log::error!("Failed to save state before sleep: {}", e);
                                }
                            }
                            
                            ui_sender.send(Message::SystemSleep).ok();
                            let config = state_manager.get_config();
                            if let Err(e) = config.save() {
                                log::error!("Failed to save config before sleep: {}", e);
                            }
                            log::info!("System entering sleep mode");
                        }
                    },
                    _ => {}
                }
            }
        });
        
        self.system_event_task = Some(task);
    }
    
    /// Handle shutdown request
    pub fn shutdown(&mut self) -> Result<(), String> {
        // Set state to shutting down
        let mut state = self.state.lock().unwrap();
        *state = LifecycleState::ShuttingDown;
        
        // Cancel auto-save task
        if let Some(task) = self.auto_save_task.take() {
            task.abort();
        }
        
        // Cancel system event task
        if let Some(task) = self.system_event_task.take() {
            task.abort();
        }
        
        // Save persistent state
        if let Some(ref mut persistence_manager) = self.persistence_manager {
            if let Err(e) = persistence_manager.save_state() {
                log::error!("Failed to save persistent state on shutdown: {}", e);
            }
        }
        
        // Save config before exit
        let config = self.state_manager.get_config();
        if let Err(e) = config.save() {
            log::error!("Failed to save config on shutdown: {}", e);
        }
        
        Ok(())
    }
    
    /// Handle system sleep event
    pub fn handle_sleep(&mut self) {
        // Set state to sleep
        let mut state = self.state.lock().unwrap();
        *state = LifecycleState::Sleep;
        
        // Save persistent state
        if let Some(ref mut persistence_manager) = self.persistence_manager {
            if let Err(e) = persistence_manager.save_state() {
                log::error!("Failed to save persistent state before sleep: {}", e);
            }
        }
        
        // Save config before sleep
        let config = self.state_manager.get_config();
        if let Err(e) = config.save() {
            log::error!("Failed to save config before sleep: {}", e);
        }
        
        // Disconnect from devices to prevent issues during sleep
        let action = crate::ui::state_manager::Action::SystemSleep;
        self.state_manager.dispatch(action);
    }
    
    /// Handle system wake event
    pub fn handle_wake(&mut self) {
        // Set state to running
        let mut state = self.state.lock().unwrap();
        *state = LifecycleState::Running;
        
        // Reconnect to devices if needed
        let action = crate::ui::state_manager::Action::SystemWake;
        self.state_manager.dispatch(action);
    }
    
    /// Force save the current state
    pub fn force_save(&mut self) -> Result<(), String> {
        // Get the persistence manager
        let mut persistence_manager = match &mut self.persistence_manager {
            Some(manager) => manager.clone(),
            None => return Err("Persistence manager not initialized".to_string()),
        };
        
        // Save the state
        let result = persistence_manager.save_state();
        
        // Check for errors
        match result {
            Ok(()) => Ok(()),
            Err(e) => Err(format!("Failed to save state: {}", e)),
        }
    }
    
    /// Save crash recovery state
    pub fn save_recovery_state(&self) -> Result<(), String> {
        // Get necessary state for recovery
        let device_state = self.state_manager.get_device_state();
        let config = self.state_manager.get_config();
        
        // Create recovery data
        let recovery_data = RecoveryData {
            timestamp: chrono::Utc::now().to_string(),
            selected_device: device_state.selected_device.clone(),
            connection_state: format!("{:?}", device_state.connection_state),
            is_scanning: device_state.is_scanning,
            auto_scan: device_state.auto_scan,
            last_error: device_state.last_error.clone(),
        };
        
        // Serialize to JSON
        let json = serde_json::to_string_pretty(&recovery_data)
            .map_err(|e| format!("Failed to serialize recovery data: {}", e))?;
        
        // Get recovery file path
        let recovery_path = get_recovery_file_path()?;
        
        // Write to file
        std::fs::write(recovery_path, json)
            .map_err(|e| format!("Failed to write recovery file: {}", e))?;
        
        log::info!("Saved recovery state successfully");
        Ok(())
    }
    
    /// Check for and load recovery state if it exists
    pub fn check_for_recovery_state(&self) -> Result<bool, String> {
        // Get recovery file path
        let recovery_path = get_recovery_file_path()?;
        
        // Check if recovery file exists
        if !recovery_path.exists() {
            return Ok(false);
        }
        
        // Read recovery file
        let json = std::fs::read_to_string(recovery_path.clone())
            .map_err(|e| format!("Failed to read recovery file: {}", e))?;
        
        // Parse JSON
        let recovery_data: RecoveryData = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse recovery data: {}", e))?;
        
        // Check if recovery data is recent (within 30 minutes)
        if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(&recovery_data.timestamp) {
            let now = chrono::Utc::now();
            let duration = now.signed_duration_since(timestamp);
            
            if duration.num_minutes() > 30 {
                // Recovery data is too old, delete it
                let _ = std::fs::remove_file(recovery_path);
                return Ok(false);
            }
            
            // Recovery data is recent, apply it
            self.apply_recovery_data(recovery_data)?;
            
            // Delete recovery file after successful recovery
            let _ = std::fs::remove_file(recovery_path);
            
            return Ok(true);
        }
        
        // Couldn't parse timestamp, skip recovery
        Ok(false)
    }
    
    /// Apply recovery data to restore state
    fn apply_recovery_data(&self, data: RecoveryData) -> Result<(), String> {
        log::info!("Recovering from previous session state");
        
        // Restore auto-scan setting
        let action = crate::ui::state_manager::Action::ToggleAutoScan(data.auto_scan);
        self.state_manager.dispatch(action);
        
        // Restore error message if any
        if let Some(error) = data.last_error {
            let action = crate::ui::state_manager::Action::SetError(error);
            self.state_manager.dispatch(action);
        }
        
        // If we had a selected device, try to reconnect
        if let Some(device) = data.selected_device {
            log::info!("Attempting to reconnect to device from previous session: {}", device);
            
            // Use RestorePreviousConnection action instead of sending message directly
            let action = crate::ui::state_manager::Action::RestorePreviousConnection(device);
            self.state_manager.dispatch(action);
        }
        
        // If we were scanning, restart scanning
        if data.is_scanning {
            log::info!("Restarting scanning from previous session");
            
            // Start scanning
            let action = crate::ui::state_manager::Action::StartScanning;
            self.state_manager.dispatch(action);
        }
        
        log::info!("Successfully recovered state from previous session");
        Ok(())
    }
    
    /// Start crash recovery task
    fn start_crash_recovery_task(&mut self) {
        // Get necessary clones for the task
        let lifecycle_manager = Arc::new(self.clone());
        
        // Create the crash recovery task
        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60)); // Save state every minute
            
            loop {
                interval.tick().await;
                
                // Save recovery state
                if let Err(e) = lifecycle_manager.save_recovery_state() {
                    log::error!("Failed to save recovery state: {}", e);
                }
                
                // Also save persistent state occasionally
                if let Some(mut persistence_manager) = lifecycle_manager.persistence_manager.clone() {
                    // Only save if enough time has passed since last save
                    if persistence_manager.time_since_last_save().num_seconds() > 300 {
                        if let Err(e) = persistence_manager.save_state() {
                            log::error!("Failed to save persistent state in recovery task: {}", e);
                        }
                    }
                }
            }
        });
        
        // Store task
        self.crash_recovery_task = Some(task);
    }
}

impl Drop for LifecycleManager {
    fn drop(&mut self) {
        // Clean up tasks on drop
        if let Some(task) = self.auto_save_task.take() {
            task.abort();
        }
        
        if let Some(task) = self.system_event_task.take() {
            task.abort();
        }
        
        if let Some(task) = self.crash_recovery_task.take() {
            task.abort();
        }
        
        // Try to save config one last time
        let config = self.state_manager.get_config();
        let _ = config.save();
    }
}

/// Recovery data structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct RecoveryData {
    /// Timestamp when recovery data was created
    timestamp: String,
    /// Selected device address
    selected_device: Option<String>,
    /// Connection state when recovery data was created
    connection_state: String,
    /// Whether Bluetooth scanning was active
    is_scanning: bool,
    /// Whether auto-scan was enabled
    auto_scan: bool,
    /// Last error message if any
    last_error: Option<String>,
}

/// Get the recovery file path
fn get_recovery_file_path() -> Result<std::path::PathBuf, String> {
    let data_dir = dirs::data_local_dir()
        .ok_or_else(|| "Could not determine local data directory".to_string())?;
    
    let app_dir = data_dir.join("RustPods");
    
    // Create directory if it doesn't exist
    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir)
            .map_err(|e| format!("Failed to create app data directory: {}", e))?;
    }
    
    let recovery_path = app_dir.join("recovery.json");
    Ok(recovery_path)
} 