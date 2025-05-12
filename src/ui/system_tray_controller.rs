//! Controller for system tray that integrates with app state
//! This module connects the system tray with the state management system

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use tokio::task::JoinHandle;
use tokio::sync::mpsc as tokio_mpsc;

use crate::ui::SystemTray;
use crate::ui::state_manager::StateManager;
use crate::ui::Message;
use crate::config::AppConfig;

/// Controls system tray updates based on application state
pub struct SystemTrayController {
    /// System tray instance
    system_tray: Arc<Mutex<SystemTray>>,
    /// State manager reference
    state_manager: Arc<StateManager>,
    /// Update thread handle
    update_thread: Option<thread::JoinHandle<()>>,
    /// Whether the controller is running
    running: Arc<Mutex<bool>>,
    /// Tokio task handle for battery status notifications
    notification_task: Option<JoinHandle<()>>,
}

impl SystemTrayController {
    /// Create a new system tray controller
    pub fn new(
        tx: mpsc::Sender<Message>, 
        config: AppConfig, 
        state_manager: Arc<StateManager>
    ) -> Result<Self, String> {
        // Create system tray
        let system_tray = match SystemTray::new(tx, config.clone()) {
            Ok(tray) => tray,
            Err(e) => return Err(format!("Failed to create system tray: {}", e)),
        };
        
        // Connect state manager
        let mut system_tray_ref = Arc::new(Mutex::new(system_tray));
        if let Ok(mut guard) = system_tray_ref.lock() {
            if let Err(e) = guard.connect_state_manager(Arc::clone(&state_manager)) {
                return Err(format!("Failed to connect state manager: {}", e));
            }
        } else {
            return Err("Failed to lock system tray".to_string());
        }
        
        Ok(Self {
            system_tray: system_tray_ref,
            state_manager,
            update_thread: None,
            running: Arc::new(Mutex::new(false)),
            notification_task: None,
        })
    }
    
    /// Start the controller
    pub fn start(&mut self) -> Result<(), String> {
        // Mark as running
        if let Ok(mut running) = self.running.lock() {
            *running = true;
        } else {
            return Err("Failed to lock running state".to_string());
        }
        
        // Create a clone for the thread
        let system_tray = Arc::clone(&self.system_tray);
        let running = Arc::clone(&self.running);
        
        // Start update thread
        let handle = thread::spawn(move || {
            // Update every second
            loop {
                // Check if we should stop
                let should_run = match running.lock() {
                    Ok(guard) => *guard,
                    Err(_) => false, // Stop if we can't lock
                };
                
                if !should_run {
                    break;
                }
                
                // Update the system tray
                if let Ok(mut guard) = system_tray.lock() {
                    if let Err(e) = guard.process_state_update() {
                        log::error!("Error updating system tray: {}", e);
                    }
                }
                
                // Sleep for a bit to avoid excessive updates
                thread::sleep(Duration::from_millis(500));
            }
        });
        
        self.update_thread = Some(handle);
        
        Ok(())
    }
    
    /// Start the notification task for battery status updates
    pub fn start_notification_task(
        &mut self, 
        ui_sender: tokio_mpsc::UnboundedSender<Message>
    ) -> Result<(), String> {
        // Get clones of necessary data
        let state_manager = Arc::clone(&self.state_manager);
        let system_tray = Arc::clone(&self.system_tray);
        
        // Create task
        let task = tokio::spawn(async move {
            // Poll every 5 seconds
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            
            loop {
                interval.tick().await;
                
                // Get current battery status
                let device_state = state_manager.get_device_state();
                if let Some(battery_status) = device_state.battery_status {
                    // Only process if we have a connected device
                    if device_state.selected_device.is_some() {
                        // Create a clone for the notification
                        let status_clone = battery_status.clone();
                        
                        // Process in system tray
                        if let Ok(mut guard) = system_tray.lock() {
                            if let Err(e) = guard.handle_battery_update(status_clone) {
                                log::error!("Error updating system tray battery: {}", e);
                            }
                        }
                        
                        // Send update to UI as well
                        let _ = ui_sender.send(Message::BatteryStatusUpdated(battery_status));
                    }
                }
            }
        });
        
        self.notification_task = Some(task);
        
        Ok(())
    }
    
    /// Stop the controller
    pub fn stop(&mut self) -> Result<(), String> {
        // Mark as not running
        if let Ok(mut running) = self.running.lock() {
            *running = false;
        } else {
            return Err("Failed to lock running state".to_string());
        }
        
        // Wait for thread to finish
        if let Some(handle) = self.update_thread.take() {
            if let Err(e) = handle.join() {
                return Err(format!("Failed to join update thread: {:?}", e));
            }
        }
        
        // Cancel notification task
        if let Some(task) = self.notification_task.take() {
            task.abort();
        }
        
        Ok(())
    }
    
    /// Update the system tray with new config
    pub fn update_config(&mut self, config: AppConfig) -> Result<(), String> {
        if let Ok(mut guard) = self.system_tray.lock() {
            guard.update_config(config)
                .map_err(|e| format!("Failed to update system tray config: {}", e))
        } else {
            Err("Failed to lock system tray".to_string())
        }
    }
}

impl Drop for SystemTrayController {
    fn drop(&mut self) {
        // Ensure we stop the thread
        if let Err(e) = self.stop() {
            log::error!("Error stopping system tray controller: {}", e);
        }
    }
} 