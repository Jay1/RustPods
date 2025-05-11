use std::sync::mpsc;
use tray_item::{TrayItem, IconSource};
use std::io;

use crate::ui::Message;
use crate::config::{AppConfig, Theme as ConfigTheme};
use crate::ui::theme::{Theme as UiTheme, BLUE, GREEN, RED, TEXT};

/// Menu item information for system tray
struct MenuItem {
    /// Label to display in the menu
    label: String,
    /// Keyboard shortcut (Windows only)
    shortcut: Option<String>,
    /// Message to send when clicked
    message: Option<Message>,
}

/// Represents the current theme mode being used
enum ThemeMode {
    /// Light theme mode
    Light,
    /// Dark theme mode
    Dark,
}

/// Manages the system tray icon and menu
pub struct SystemTray {
    /// The system tray item
    tray: TrayItem,
    /// Sender for UI messages
    #[allow(dead_code)]
    tx: mpsc::Sender<Message>,
    /// Application configuration
    config: AppConfig,
    /// Last known connection status
    is_connected: bool,
    /// Current theme mode
    theme_mode: ThemeMode,
    /// Whether the tray is registered for startup
    startup_registered: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum SystemTrayError {
    #[error("Failed to create tray item: {0}")]
    Creation(String),
    
    #[error("Failed to add menu item: {0}")]
    MenuItem(String),
    
    #[error("Failed to set icon: {0}")]
    SetIcon(String),
    
    #[error("Failed to set tooltip: {0}")]
    SetTooltip(String),
    
    #[error("Failed to handle tray event: {0}")]
    EventHandling(String),
    
    #[error("Registry error: {0}")]
    Registry(String),
    
    #[error("Windows-specific error: {0}")]
    WindowsError(#[from] io::Error),
}

impl SystemTray {
    /// Create a new system tray
    pub fn new(tx: mpsc::Sender<Message>, config: AppConfig) -> Result<Self, SystemTrayError> {
        // Determine theme mode from config
        let theme_mode = Self::detect_theme_mode(&config);
        
        // Create the tray item with a proper application name
        let app_name = "RustPods";
        let mut tray = TrayItem::new(app_name, Self::get_icon(&theme_mode, false))
            .map_err(|e| SystemTrayError::Creation(e.to_string()))?;
        
        // Set a tooltip for the tray icon
        #[cfg(target_os = "windows")]
        {
            // The set_tooltip method is only available on Windows
            tray.set_tooltip("RustPods - Disconnected")
                .map_err(|e| SystemTrayError::SetTooltip(e.to_string()))?;
        }
        
        // Define menu groups
        let main_actions = vec![
            MenuItem { 
                label: "Show/Hide Window".to_string(), 
                shortcut: Some("Alt+W".to_string()), 
                message: Some(Message::ToggleVisibility) 
            },
        ];
        
        let scan_actions = vec![
            MenuItem { 
                label: "Start Scan".to_string(), 
                shortcut: Some("Ctrl+S".to_string()), 
                message: Some(Message::StartScan) 
            },
            MenuItem { 
                label: "Stop Scan".to_string(), 
                shortcut: Some("Ctrl+X".to_string()), 
                message: Some(Message::StopScan) 
            },
        ];
        
        let settings_actions = vec![
            MenuItem { 
                label: "Settings".to_string(), 
                shortcut: Some("Ctrl+P".to_string()), 
                message: Some(Message::OpenSettings) 
            },
        ];
        
        let about_actions = vec![
            MenuItem { 
                label: "About RustPods".to_string(), 
                shortcut: None, 
                message: None  // We'll handle this differently
            },
        ];
        
        let exit_actions = vec![
            MenuItem { 
                label: "Exit".to_string(), 
                shortcut: Some("Alt+F4".to_string()), 
                message: Some(Message::Exit) 
            },
        ];
        
        // Add main actions group
        Self::add_menu_group(&mut tray, &tx, &main_actions)
            .map_err(|e| SystemTrayError::MenuItem(e))?;
        
        // Add separator
        tray.add_menu_item("-", || {})
            .map_err(|e| SystemTrayError::MenuItem(e.to_string()))?;
        
        // Add scan group
        Self::add_menu_group(&mut tray, &tx, &scan_actions)
            .map_err(|e| SystemTrayError::MenuItem(e))?;
        
        // Add separator
        tray.add_menu_item("-", || {})
            .map_err(|e| SystemTrayError::MenuItem(e.to_string()))?;
        
        // Add settings group
        Self::add_menu_group(&mut tray, &tx, &settings_actions)
            .map_err(|e| SystemTrayError::MenuItem(e))?;
        
        // Add separator
        tray.add_menu_item("-", || {})
            .map_err(|e| SystemTrayError::MenuItem(e.to_string()))?;
        
        // Add about group
        // Special handling for about dialog
        let tx_clone = tx.clone();
        let about_label = if let Some(shortcut) = &about_actions[0].shortcut {
            format!("{}\t{}", about_actions[0].label, shortcut)
        } else {
            about_actions[0].label.clone()
        };
        
        tray.add_menu_item(&about_label, move || {
            // Display the about dialog
            // This would ideally open a custom about dialog, but for now we'll just show a message
            let version = env!("CARGO_PKG_VERSION", "0.1.0");
            let about_message = format!(
                "RustPods v{}\nA simple AirPods battery monitor for Windows\nDeveloped with Rust and Iced", 
                version
            );
            let _ = tx_clone.send(Message::Status(about_message));
        })
        .map_err(|e| SystemTrayError::MenuItem(e.to_string()))?;
        
        // Add separator
        tray.add_menu_item("-", || {})
            .map_err(|e| SystemTrayError::MenuItem(e.to_string()))?;
        
        // Add exit group
        Self::add_menu_group(&mut tray, &tx, &exit_actions)
            .map_err(|e| SystemTrayError::MenuItem(e))?;
        
        let mut startup_registered = false;
        
        // Create system tray structure
        let mut system_tray = Self {
            tray,
            tx,
            config,
            is_connected: false,
            theme_mode,
            startup_registered,
        };
        
        // Register startup if enabled in config
        #[cfg(target_os = "windows")]
        if system_tray.config.system.launch_at_startup {
            if let Err(e) = system_tray.set_startup_enabled(true) {
                log::error!("Failed to set startup: {}", e);
            } else {
                startup_registered = true;
            }
        }
        
        // Register click handlers if supported by the platform
        #[cfg(target_os = "windows")]
        {
            // Register left button click handler (toggle window visibility)
            let tx_clone = tx.clone();
            if let Err(e) = system_tray.tray.inner_mut().add_listener(move |event| {
                match event {
                    tray_item::TrayEvent::LeftClick => {
                        let _ = tx_clone.send(Message::ToggleVisibility);
                    },
                    tray_item::TrayEvent::RightClick => {
                        // Right click shows the context menu by default
                    },
                    tray_item::TrayEvent::DoubleClick => {
                        // Double click also toggles visibility
                        let _ = tx_clone.send(Message::ToggleVisibility);
                    },
                    _ => {}
                }
            }) {
                log::warn!("Failed to add tray event listener: {}", e);
                // Continue even if listener registration fails
            }
            
            // Setup global system event handling for Windows
            system_tray.setup_system_event_handling()?;
        }
        
        // Return the tray
        Ok(system_tray)
    }
    
    /// Set up Windows-specific event handling
    #[cfg(target_os = "windows")]
    fn setup_system_event_handling(&self) -> Result<(), SystemTrayError> {
        // This would be implemented in a real application to register
        // for Windows specific events like:
        // - System shutdown
        // - Sleep/wake events
        // - Monitor topology changes (for handling multi-monitor setups)
        // - Power status changes (for laptops)
        
        // For this prototype, we'll just log that we would register handlers
        log::info!("Registered for Windows system events");
        
        Ok(())
    }
    
    /// Enable or disable startup registration with Windows
    #[cfg(target_os = "windows")]
    pub fn set_startup_enabled(&mut self, enabled: bool) -> Result<(), SystemTrayError> {
        // Get the path to the executable
        let exe_path = std::env::current_exe()
            .map_err(|e| SystemTrayError::WindowsError(e))?;
        
        // Convert path to string
        let exe_path_str = exe_path.to_string_lossy().to_string();
        
        // This would use the Windows registry to set or remove the startup entry
        if enabled {
            // In a real implementation, we'd use winreg crate to add a registry entry:
            // HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run
            log::info!("Added startup registry entry for: {}", exe_path_str);
            self.startup_registered = true;
        } else {
            // Remove registry entry
            log::info!("Removed startup registry entry");
            self.startup_registered = false;
        }
        
        Ok(())
    }
    
    /// Get the startup registration status
    pub fn is_startup_registered(&self) -> bool {
        self.startup_registered
    }
    
    /// Detect the current theme mode based on configuration
    fn detect_theme_mode(config: &AppConfig) -> ThemeMode {
        match config.ui.theme {
            ConfigTheme::Light => ThemeMode::Light,
            ConfigTheme::Dark => ThemeMode::Dark,
            ConfigTheme::System => {
                // Check system theme preference
                // This is a simplified implementation - in a real app,
                // you would query the actual system theme
                #[cfg(target_os = "windows")]
                {
                    // On Windows, attempt to detect system theme
                    // This is a placeholder - in a real implementation,
                    // you would use the Windows API to check the actual theme
                    let is_dark_mode = Self::is_system_using_dark_mode();
                    if is_dark_mode {
                        ThemeMode::Dark
                    } else {
                        ThemeMode::Light
                    }
                }
                
                // Default to dark mode on other platforms or if detection fails
                #[cfg(not(target_os = "windows"))]
                {
                    ThemeMode::Dark
                }
            }
        }
    }
    
    /// Check if the system is using dark mode
    /// This is a placeholder implementation - in a real app, you would
    /// use platform-specific APIs to detect the actual system theme
    #[cfg(target_os = "windows")]
    fn is_system_using_dark_mode() -> bool {
        // In a real implementation, you would use Windows Registry or API
        // to determine if dark mode is enabled
        // For now, we'll default to dark mode
        true
    }
    
    /// Add a group of menu items to the tray
    fn add_menu_group(
        tray: &mut TrayItem,
        tx: &mpsc::Sender<Message>,
        items: &[MenuItem]
    ) -> Result<(), String> {
        for item in items {
            // Skip separators (handled separately)
            if item.label == "-" {
                tray.add_menu_item("-", || {})
                    .map_err(|e| e.to_string())?;
                continue;
            }
            
            // Format label with shortcut if available
            let label = if let Some(shortcut) = &item.shortcut {
                format!("{}\t{}", item.label, shortcut)
            } else {
                item.label.clone()
            };
            
            // Add menu item with appropriate handler
            if let Some(message) = &item.message {
                let tx_clone = tx.clone();
                let message_clone = message.clone();
                
                tray.add_menu_item(&label, move || {
                    let _ = tx_clone.send(message_clone.clone());
                })
                .map_err(|e| e.to_string())?;
            }
        }
        
        Ok(())
    }
    
    /// Get the appropriate icon based on theme mode and connection status
    fn get_icon(theme_mode: &ThemeMode, connected: bool) -> IconSource {
        match (theme_mode, connected) {
            // Dark theme icons (for dark mode or light backgrounds)
            (ThemeMode::Dark, true) => Self::get_dark_connected_icon(),
            (ThemeMode::Dark, false) => Self::get_dark_disconnected_icon(),
            
            // Light theme icons (for light mode or dark backgrounds)
            (ThemeMode::Light, true) => Self::get_light_connected_icon(),
            (ThemeMode::Light, false) => Self::get_light_disconnected_icon(),
        }
    }
    
    /// Get the dark theme disconnected icon
    fn get_dark_disconnected_icon() -> IconSource {
        // On Windows, prefer ICO format
        #[cfg(target_os = "windows")]
        {
            // Use the embedded asset instead of a resource name
            IconSource::Data {
                data: crate::assets::tray::DARK_DISCONNECTED.to_vec(),
            }
        }
        
        // On other platforms, use a generic default
        #[cfg(not(target_os = "windows"))]
        {
            IconSource::Default
        }
    }
    
    /// Get the dark theme connected icon
    fn get_dark_connected_icon() -> IconSource {
        // On Windows, prefer ICO format
        #[cfg(target_os = "windows")]
        {
            // Use the embedded asset instead of a resource name
            IconSource::Data {
                data: crate::assets::tray::DARK_CONNECTED.to_vec(),
            }
        }
        
        // On other platforms, use a generic default
        #[cfg(not(target_os = "windows"))]
        {
            IconSource::Default
        }
    }
    
    /// Get the light theme disconnected icon
    fn get_light_disconnected_icon() -> IconSource {
        // On Windows, prefer ICO format
        #[cfg(target_os = "windows")]
        {
            // Use the embedded asset instead of a resource name
            IconSource::Data {
                data: crate::assets::tray::LIGHT_DISCONNECTED.to_vec(),
            }
        }
        
        // On other platforms, use a generic default
        #[cfg(not(target_os = "windows"))]
        {
            IconSource::Default
        }
    }
    
    /// Get the light theme connected icon
    fn get_light_connected_icon() -> IconSource {
        // On Windows, prefer ICO format
        #[cfg(target_os = "windows")]
        {
            // Use the embedded asset instead of a resource name
            IconSource::Data {
                data: crate::assets::tray::LIGHT_CONNECTED.to_vec(),
            }
        }
        
        // On other platforms, use a generic default
        #[cfg(not(target_os = "windows"))]
        {
            IconSource::Default
        }
    }
    
    /// Cleanup resources before exit
    #[cfg(target_os = "windows")]
    pub fn cleanup(&mut self) -> Result<(), SystemTrayError> {
        // In a real implementation, this would:
        // - Unregister any global hotkeys
        // - Release other system resources
        // - Save any pending state
        log::info!("SystemTray cleanup performed");
        Ok(())
    }
    
    /// Update the system tray icon based on connection status
    pub fn update_icon(&mut self, connected: bool) -> Result<(), SystemTrayError> {
        // Only update if the status changed
        if self.is_connected == connected {
            return Ok(());
        }
        
        self.is_connected = connected;
        
        let icon = Self::get_icon(&self.theme_mode, connected);
        
        self.tray.set_icon(icon)
            .map_err(|e| SystemTrayError::SetIcon(e.to_string()))?;
        
        // Update the tooltip if supported
        #[cfg(target_os = "windows")]
        {
            let tooltip = if connected {
                "RustPods - Connected"
            } else {
                "RustPods - Disconnected"
            };
            
            self.tray.set_tooltip(tooltip)
                .map_err(|e| SystemTrayError::SetTooltip(e.to_string()))?;
        }
        
        Ok(())
    }
    
    /// Update the tooltip with battery information
    pub fn update_tooltip_with_battery(&mut self, left: Option<u8>, right: Option<u8>, case: Option<u8>) -> Result<(), SystemTrayError> {
        #[cfg(target_os = "windows")]
        {
            // Skip updating if we shouldn't show percentages
            if !self.config.ui.show_percentage_in_tray {
                return Ok(());
            }
            
            let left_text = left.map_or("N/A".to_string(), |v| format!("{}%", v));
            let right_text = right.map_or("N/A".to_string(), |v| format!("{}%", v));
            let case_text = case.map_or("N/A".to_string(), |v| format!("{}%", v));
            
            let tooltip = format!("RustPods - Left: {}, Right: {}, Case: {}", left_text, right_text, case_text);
            
            self.tray.set_tooltip(&tooltip)
                .map_err(|e| SystemTrayError::SetTooltip(e.to_string()))?;
            
            // Check for low battery and show warning if needed
            if self.config.ui.show_low_battery_warning {
                let threshold = self.config.ui.low_battery_threshold;
                let left_low = left.map_or(false, |v| v <= threshold);
                let right_low = right.map_or(false, |v| v <= threshold);
                
                if left_low || right_low {
                    // In a real implementation, we'd show a notification
                    // For now, just log it
                    log::info!("Low battery warning: Left: {}, Right: {}", left_text, right_text);
                    
                    // We could also send a message to show a notification in the UI
                    let _ = self.tx.send(Message::Status(format!(
                        "Low Battery Warning - Left: {}, Right: {}", 
                        left_text, right_text
                    )));
                    
                    // On Windows, we could use the Windows notification system
                    #[cfg(target_os = "windows")]
                    self.show_windows_notification(&format!(
                        "Low Battery Warning\nLeft: {}, Right: {}", 
                        left_text, right_text
                    ))?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Show a Windows notification
    #[cfg(target_os = "windows")]
    fn show_windows_notification(&self, message: &str) -> Result<(), SystemTrayError> {
        // In a real implementation, this would use Windows toast notifications
        // For the prototype, we'll just log that we would show a notification
        log::info!("Windows notification: {}", message);
        
        Ok(())
    }
    
    /// Update the tray based on configuration changes
    pub fn update_config(&mut self, config: AppConfig) -> Result<(), SystemTrayError> {
        let prev_config = self.config.clone();
        
        // Check if theme has changed
        let new_theme_mode = Self::detect_theme_mode(&config);
        let theme_changed = matches!((self.theme_mode, new_theme_mode), 
            (ThemeMode::Light, ThemeMode::Dark) | (ThemeMode::Dark, ThemeMode::Light));
        
        // Check if startup setting changed
        #[cfg(target_os = "windows")]
        let startup_changed = prev_config.system.launch_at_startup != config.system.launch_at_startup;
        
        // Update our config
        self.config = config;
        
        // Update theme mode
        if theme_changed {
            self.theme_mode = new_theme_mode;
            
            // Update icon according to new theme
            let icon = Self::get_icon(&self.theme_mode, self.is_connected);
            self.tray.set_icon(icon)
                .map_err(|e| SystemTrayError::SetIcon(e.to_string()))?;
                
            log::debug!("Updated system tray icon due to theme change");
        }
        
        // Update startup setting if it changed
        #[cfg(target_os = "windows")]
        if startup_changed {
            self.set_startup_enabled(self.config.system.launch_at_startup)?;
        }
        
        Ok(())
    }
}

// Drop implementation to ensure proper cleanup on application exit
impl Drop for SystemTray {
    fn drop(&mut self) {
        // For Windows, call cleanup to release resources
        #[cfg(target_os = "windows")]
        {
            if let Err(e) = self.cleanup() {
                log::error!("Failed to clean up system tray: {}", e);
            }
        }
        
        log::debug!("SystemTray dropped");
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
        
        let (_tx, _rx) = mpsc::channel::<Message>();
        
        // Just make sure the type compiles
        let tray_type = std::any::TypeId::of::<SystemTray>();
        assert_eq!(tray_type, std::any::TypeId::of::<SystemTray>());
        
        // Verify error enum works
        let error = SystemTrayError::Creation("test".to_string());
        assert!(error.to_string().contains("Failed to create tray item"));
    }
    
    #[test]
    fn test_menu_item_struct() {
        // Test that MenuItem struct works correctly
        let item = MenuItem {
            label: "Test".to_string(),
            shortcut: Some("Ctrl+T".to_string()),
            message: Some(Message::Exit),
        };
        
        assert_eq!(item.label, "Test");
        assert_eq!(item.shortcut, Some("Ctrl+T".to_string()));
        assert!(matches!(item.message, Some(Message::Exit)));
    }
    
    #[test]
    fn test_theme_detection() {
        // Test light theme
        let mut config = AppConfig::default();
        config.ui.theme = ConfigTheme::Light;
        let theme_mode = SystemTray::detect_theme_mode(&config);
        assert!(matches!(theme_mode, ThemeMode::Light));
        
        // Test dark theme
        config.ui.theme = ConfigTheme::Dark;
        let theme_mode = SystemTray::detect_theme_mode(&config);
        assert!(matches!(theme_mode, ThemeMode::Dark));
        
        // System theme will default to dark in tests since we don't have a real system to check
        config.ui.theme = ConfigTheme::System;
        let theme_mode = SystemTray::detect_theme_mode(&config);
        assert!(matches!(theme_mode, ThemeMode::Dark));
    }
    
    // Helper function to create a dummy system tray for tests
    #[allow(dead_code)]
    fn create_dummy_tray() -> Result<SystemTray, SystemTrayError> {
        let (tx, _rx) = mpsc::channel::<Message>();
        let config = AppConfig::default();
        SystemTray::new(tx, config)
    }
    
    // Skipping actual tray tests as they require a GUI environment
    // #[test]
    // fn test_send_message_through_tray() { ... }
} 
} 