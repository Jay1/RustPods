//! Application lifecycle management
//! 
//! This module handles application lifecycle events including startup, shutdown, 
//! and system events like sleep/wake.

use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;

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
    /// System event task handle (std::thread::JoinHandle on Windows)
    #[cfg(target_os = "windows")]
    system_event_task: Option<std::thread::JoinHandle<()>>,
    /// System event task handle (tokio::task::JoinHandle on non-Windows)
    #[cfg(not(target_os = "windows"))]
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
    
    /// Start synchronous initialization (without async tasks)
    pub fn start(&mut self) -> Result<(), String> {
        log::info!("Starting lifecycle manager (sync phase)");
        
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
        
        // Set state to running
        let mut state = self.state.lock().unwrap();
        *state = LifecycleState::Running;
        
        log::info!("Lifecycle manager sync phase completed");
        Ok(())
    }
    
    /// Start async tasks (call after tokio runtime is available)
    pub fn start_async_tasks(&mut self) -> Result<(), String> {
        log::info!("Starting lifecycle manager async tasks");
        
        // Only start if we haven't already started async tasks
        if self.auto_save_task.is_some() || self.crash_recovery_task.is_some() {
            log::warn!("Async tasks already started, skipping");
            return Ok(());
        }
        
        // Start periodic state saving for crash recovery
        self.start_crash_recovery_task();
        
        // Start auto-save task
        self.start_auto_save();
        
        // Register for system events
        self.register_system_events();
        
        log::info!("Lifecycle manager async tasks started");
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
        let _state = Arc::clone(&self.state);
        let _ui_sender = self.ui_sender.clone();
        let _state_manager = Arc::clone(&self.state_manager);
        let _persistence_manager = self.persistence_manager.clone();
        
        // Create the system event task
        // Win32 system event monitoring is currently disabled due to removal of unsupported Win32 imports.
        // See docs/windows-ble-airpods.md for details. Only platform-agnostic or WinRT-relevant code should remain here.
        // No 'task' is created for Windows at this time.
        #[cfg(not(target_os = "windows"))]
        let task = tokio::spawn(async move {
            log::info!("Starting generic system event monitor");
            
            // For non-Windows platforms, we'll use a simple timer-based approach
            // In a real implementation, this would use platform-specific mechanisms
            let mut interval = tokio::time::interval(Duration::from_secs(2));
            
            loop {
                interval.tick().await;
                
                // Check for platform-specific signals
                // This is a placeholder for platform-specific implementations
                
                // For demonstration only: occasionally check system status
                // In reality, this would hook into OS-specific signals
                #[cfg(target_os = "macos")]
                {
                    // On macOS, we might use IOKit notifications
                    // This is a placeholder
                }
                
                #[cfg(target_os = "linux")]
                {
                    // On Linux, we might use D-Bus signals
                    // This is a placeholder
                }
            }
        });
        
        #[cfg(not(target_os = "windows"))]
        {
            self.system_event_task = Some(task);
        }
        #[cfg(target_os = "windows")]
        {
            self.system_event_task = None;
        }
    }
    
    /// Handle shutdown request
    pub fn shutdown(&mut self) -> Result<(), String> {
        log::info!("Shutting down lifecycle manager");
        
        // Set state to shutting down
        {
            let mut state = self.state.lock().unwrap();
            if *state == LifecycleState::ShuttingDown {
                log::warn!("Lifecycle manager is already shutting down");
                return Ok(());
            }
            *state = LifecycleState::ShuttingDown;
        }
        
        // Force save state
        self.force_save()?;
        
        // Cancel auto-save task
        if let Some(task) = self.auto_save_task.take() {
            log::debug!("Cancelling auto-save task");
            task.abort();
        }
        
        // Cancel system event task
        #[cfg(target_os = "windows")]
        {
            if let Some(_task) = self.system_event_task.take() {
                log::debug!("Dropping system event thread handle (Windows)");
                // No abort for std::thread::JoinHandle; dropping is sufficient
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            if let Some(task) = self.system_event_task.take() {
                log::debug!("Cancelling system event task (tokio)");
                task.abort();
            }
        }
        
        // Cancel crash recovery task
        if let Some(task) = self.crash_recovery_task.take() {
            log::debug!("Cancelling crash recovery task");
            task.abort();
        }
        
        // Release resources in state manager
        let shutdown_action = crate::ui::state_manager::Action::Shutdown;
        self.state_manager.dispatch(shutdown_action);
        
        // Save final recovery point and mark as clean shutdown
        if let Some(persistence_manager) = &mut self.persistence_manager {
            // Save final state
            if let Err(e) = persistence_manager.save_state() {
                log::warn!("Failed to save final state during shutdown: {}", e);
            }
            
            // Create a marker file to indicate clean shutdown
            let clean_shutdown_path = get_recovery_file_path()
                .map_err(|e| format!("Failed to get recovery path: {}", e))?
                .with_file_name("clean_shutdown");
                
            std::fs::write(&clean_shutdown_path, "")
                .map_err(|e| format!("Failed to write clean shutdown marker: {}", e))?;
                
            log::debug!("Created clean shutdown marker at {:?}", clean_shutdown_path);
        }
        
        log::info!("Lifecycle manager shutdown complete");
        Ok(())
    }
    
    /// Handle system sleep event
    pub fn handle_sleep(&mut self) {
        log::info!("Handling system sleep event");
        
        // Set state to sleep
        {
            let mut state = self.state.lock().unwrap();
            *state = LifecycleState::Sleep;
        }
        
        // Save state before sleep
        if let Err(e) = self.force_save() {
            log::warn!("Failed to save state before sleep: {}", e);
        }
        
        // Notify state manager
        let sleep_action = crate::ui::state_manager::Action::SystemSleep;
        self.state_manager.dispatch(sleep_action);
    }
    
    /// Handle system wake event
    pub fn handle_wake(&mut self) {
        log::info!("Handling system wake event");
        
        // Set state to running
        {
            let mut state = self.state.lock().unwrap();
            *state = LifecycleState::Running;
        }
        
        // Notify state manager
        let wake_action = crate::ui::state_manager::Action::SystemWake;
        self.state_manager.dispatch(wake_action);
    }
    
    /// Force save all state immediately
    pub fn force_save(&mut self) -> Result<(), String> {
        log::info!("Forcing immediate state save");
        
        // Save config
        let config = self.state_manager.get_config();
        if let Err(e) = config.save() {
            log::error!("Failed to save config: {}", e);
            return Err(format!("Failed to save config: {}", e));
        }
        
        // Save persistent state
        if let Some(persistence_manager) = &mut self.persistence_manager {
            if let Err(e) = persistence_manager.save_state() {
                log::error!("Failed to save persistent state: {}", e);
                return Err(format!("Failed to save persistent state: {}", e));
            }
        }
        
        // Save recovery state
        if let Err(e) = self.save_recovery_state() {
            log::warn!("Failed to save recovery state: {}", e);
            // Don't return error for recovery state failure
        }
        
        // Update last save time
        {
            let mut last_save_time = self.last_save.lock().unwrap();
            *last_save_time = Instant::now();
        }
        
        log::info!("Force save completed successfully");
        Ok(())
    }
    
    /// Save crash recovery state
    pub fn save_recovery_state(&self) -> Result<(), String> {
        // Get necessary state for recovery
        let device_state = self.state_manager.get_device_state();
        let _config = self.state_manager.get_config();
        
        // Create recovery data
        let recovery_data = RecoveryData {
            timestamp: chrono::Utc::now().to_rfc3339(),
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
        
        log::info!("Successfully recovered state from previous session");
        Ok(())
    }
    
    /// Start crash recovery task
    fn start_crash_recovery_task(&mut self) {
        // Get necessary clones for the task
        let state_manager = Arc::clone(&self.state_manager);
        
        // Create the crash recovery task
        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30)); // Save recovery data every 30 seconds
            
            loop {
                interval.tick().await;
                
                // Capture current state for recovery
                let device_state = state_manager.get_device_state();
                let _config = state_manager.get_config();
                
                // Create recovery data
                let recovery_data = RecoveryData {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    selected_device: device_state.selected_device.clone(),
                    connection_state: format!("{:?}", device_state.connection_state),
                    is_scanning: device_state.is_scanning,
                    auto_scan: device_state.auto_scan,
                    last_error: device_state.last_error.clone(),
                };
                
                // Save recovery data
                if let Err(e) = save_recovery_data(&recovery_data) {
                    log::warn!("Failed to save recovery data: {}", e);
                }
                
                // Perform periodic cleanup of old recovery files
                if let Err(e) = cleanup_old_recovery_files() {
                    log::warn!("Failed to clean up old recovery files: {}", e);
                }
            }
        });
        
        self.crash_recovery_task = Some(task);
    }
}

impl Drop for LifecycleManager {
    fn drop(&mut self) {
        log::info!("LifecycleManager being dropped, ensuring clean shutdown");
        
        // Get current state
        let state = {
            let state_guard = self.state.lock().unwrap();
            (*state_guard).clone()
        };
        
        // Only run shutdown if we're not already in shutdown process
        if state != LifecycleState::ShuttingDown {
            // Try to do a clean shutdown
            if let Err(e) = self.shutdown() {
                log::error!("Error during LifecycleManager drop shutdown: {}", e);
            }
        }
        
        // Abort any remaining tasks
        if let Some(task) = self.auto_save_task.take() {
            task.abort();
        }
        
        #[cfg(target_os = "windows")]
        {
            if let Some(_task) = self.system_event_task.take() {
                log::debug!("Dropping system event thread handle (Windows)");
                // No abort for std::thread::JoinHandle; dropping is sufficient
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            if let Some(task) = self.system_event_task.take() {
                log::debug!("Cancelling system event task (tokio)");
                task.abort();
            }
        }
        
        if let Some(task) = self.crash_recovery_task.take() {
            task.abort();
        }
        
        log::info!("LifecycleManager cleanup complete");
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

/// Save recovery data to disk
fn save_recovery_data(data: &RecoveryData) -> Result<(), String> {
    // Get recovery file path
    let path = get_recovery_file_path()?;
    
    // Create directory if it doesn't exist
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create recovery directory: {}", e))?;
        }
    }
    
    // Serialize data to JSON and write to file
    let json = serde_json::to_string_pretty(data)
        .map_err(|e| format!("Failed to serialize recovery data: {}", e))?;
    
    std::fs::write(&path, json)
        .map_err(|e| format!("Failed to write recovery file: {}", e))?;
    
    Ok(())
}

/// Cleanup old recovery files
fn cleanup_old_recovery_files() -> Result<(), String> {
    // Get recovery directory
    let path = get_recovery_file_path()?;
    let parent = path.parent()
        .ok_or_else(|| "Recovery file has no parent directory".to_string())?;
    
    // Check if directory exists
    if !parent.exists() {
        return Ok(());
    }
    
    // Get current time
    let _now = chrono::Utc::now();
    
    // Read all files in the directory
    let entries = std::fs::read_dir(parent)
        .map_err(|e| format!("Failed to read recovery directory: {}", e))?;
    
    // Find and delete old recovery files
    // Use flatten to handle Results in entries directly
    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(extension) = path.extension() {
            if extension == "recovery" {
                // Get file metadata
                if let Ok(metadata) = entry.metadata() {
                    // Get file creation time
                    if let Ok(created) = metadata.created() {
                        // Get elapsed time since creation
                        if let Ok(duration) = created.elapsed() {
                            // If the file is older than RECOVERY_FILE_MAX_AGE, delete it
                            if duration > Duration::from_secs(7 * 24 * 60 * 60) {
                                log::debug!("Deleting old recovery file: {:?}", path);
                                if let Err(e) = std::fs::remove_file(&path) {
                                    log::warn!("Failed to delete recovery file: {}", e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
} 