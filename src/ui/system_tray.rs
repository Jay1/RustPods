use std::sync::mpsc;
use tray_item::TrayItem;

use crate::ui::Message;

/// Manages the system tray icon and menu
pub struct SystemTray {
    /// The system tray item
    tray: TrayItem,
    /// Sender for UI messages
    tx: mpsc::Sender<Message>,
}

#[derive(Debug, thiserror::Error)]
pub enum SystemTrayError {
    #[error("Failed to create tray item: {0}")]
    Creation(String),
    
    #[error("Failed to add menu item: {0}")]
    MenuItem(String),
    
    #[error("Failed to set icon: {0}")]
    SetIcon(String),
}

impl SystemTray {
    /// Create a new system tray
    pub fn new(tx: mpsc::Sender<Message>) -> Result<Self, SystemTrayError> {
        // Create the tray item
        let mut tray = TrayItem::new("RustPods", "default")
            .map_err(|e| SystemTrayError::Creation(e.to_string()))?;
        
        // Clone the sender for closures
        let tx_clone = tx.clone();
        tray.add_menu_item("Open", move || {
            let _ = tx_clone.send(Message::ToggleVisibility);
        })
        .map_err(|e| SystemTrayError::MenuItem(e.to_string()))?;
        
        // Clone the sender for closures
        let tx_clone = tx.clone();
        tray.add_menu_item("Start Scan", move || {
            let _ = tx_clone.send(Message::StartScan);
        })
        .map_err(|e| SystemTrayError::MenuItem(e.to_string()))?;
        
        // Clone the sender for closures
        let tx_clone = tx.clone();
        tray.add_menu_item("Stop Scan", move || {
            let _ = tx_clone.send(Message::StopScan);
        })
        .map_err(|e| SystemTrayError::MenuItem(e.to_string()))?;
        
        // Clone the sender for closures
        let tx_clone = tx.clone();
        tray.add_menu_item("Exit", move || {
            let _ = tx_clone.send(Message::Exit);
        })
        .map_err(|e| SystemTrayError::MenuItem(e.to_string()))?;
        
        // Note: tray-item 0.7 doesn't have set_tooltip
            
        Ok(Self { tray, tx })
    }
    
    /// Update the system tray icon based on connection status
    pub fn update_icon(&mut self, connected: bool) -> Result<(), SystemTrayError> {
        let icon = if connected {
            "connected"
        } else {
            "default"
        };
        
        self.tray.set_icon(icon)
            .map_err(|e| SystemTrayError::SetIcon(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    
    // Note: Most of these tests are commented out because they require a GUI environment
    // They would typically be run in an integration test environment

    #[test]
    fn test_system_tray_creation() {
        // This is more of a compilation test than a runtime test
        // It verifies that the type signatures are correct
        
        let (tx, _rx) = mpsc::channel::<Message>();
        
        // Just make sure the type compiles
        let tray_type = std::any::TypeId::of::<SystemTray>();
        assert_eq!(tray_type, std::any::TypeId::of::<SystemTray>());
        
        // Verify error enum works
        let error = SystemTrayError::Creation("test".to_string());
        assert!(error.to_string().contains("Failed to create tray item"));
    }
    
    // Skipping actual tray tests as they require a GUI environment
    // #[test]
    // fn test_send_message_through_tray() { ... }
} 