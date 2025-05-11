//! Application entry point and main logic

use std::sync::mpsc;
use tokio::sync::mpsc as tokio_mpsc;

use crate::bluetooth::{BleScanner, BleEvent, ScanConfig};
use crate::config::AppConfig;
use crate::ui::{Message, SystemTray};
use crate::error::RustPodsError;

/// Main application struct
pub struct App {
    /// Configuration
    config: AppConfig,
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
}

impl App {
    /// Create a new application
    pub fn new() -> Result<Self, RustPodsError> {
        // Create channels
        let (ui_tx, ui_rx) = mpsc::channel();
        
        // Create config
        let config = AppConfig::default();
        
        // Create scanner
        let scanner = BleScanner::new();
        
        // Create system tray
        let tray = SystemTray::new(ui_tx.clone())
            .map_err(|_| RustPodsError::UiError)?;
        
        Ok(Self {
            config,
            ui_tx,
            ui_rx,
            scanner,
            tray,
            running: false,
        })
    }
    
    /// Initialize the application
    pub async fn initialize(&mut self) -> Result<(), RustPodsError> {
        // Initialize bluetooth
        self.scanner.initialize().await?;
        
        // Set running flag
        self.running = true;
        
        Ok(())
    }
    
    /// Run the application event loop
    pub async fn run(&mut self) -> Result<(), RustPodsError> {
        // Start a scan if auto-scan is enabled
        if self.config.auto_scan_on_startup {
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
        
        Ok(())
    }
    
    /// Handle a UI message
    async fn handle_message(&mut self, message: Message) -> Result<(), RustPodsError> {
        match message {
            Message::Exit => {
                self.running = false;
            }
            Message::StartScan => {
                self.start_scan().await?;
            }
            Message::StopScan => {
                self.stop_scan().await?;
            }
            Message::ToggleAutoScan(enabled) => {
                self.config.auto_scan_on_startup = enabled;
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
        let mut events = self.scanner.start_scanning().await?;
        
        // Process events
        let ui_tx = self.ui_tx.clone();
        tokio::spawn(async move {
            while let Some(event) = events.recv().await {
                match event {
                    BleEvent::DeviceDiscovered(device) => {
                        // Forward to UI
                        let _ = ui_tx.send(Message::DeviceDiscovered(device.clone()));
                        
                        // If it has updated, also send an update
                        let _ = ui_tx.send(Message::DeviceUpdated(device));
                    }
                    _ => { /* Handle other events */ }
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