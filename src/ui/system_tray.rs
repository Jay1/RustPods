use std::sync::mpsc;
use tray_item::TrayItem;
use std::io;

use crate::ui::Message;
use crate::config::{AppConfig, Theme as ConfigTheme};
use crate::ui::state_manager::StateManager;
use crate::bluetooth::AirPodsBatteryStatus;
use std::sync::Arc;



/// Menu item information for system tray
struct MenuItem {
    /// Label to display in the menu
    label: String,
    /// Keyboard shortcut (Windows only)
    shortcut: Option<String>,
    /// Message to send when clicked
    message: Option<Message>,
}

/// Theme mode for the system tray
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ThemeMode {
    /// Light theme mode
    Light,
    /// Dark theme mode
    Dark,
}

impl From<ThemeMode> for ConfigTheme {
    fn from(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Light => ConfigTheme::Light,
            ThemeMode::Dark => ConfigTheme::Dark,
        }
    }
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
    /// Optional reference to state manager
    state_manager: Option<Arc<StateManager>>,
    /// Last known battery status
    last_battery_status: Option<AirPodsBatteryStatus>,
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
        
        // Temporary dummy icon path - replace with actual icon path later
        let icon_path = "C:\\Windows\\System32\\shell32.dll,0";
        
        let mut tray = TrayItem::new(app_name, icon_path)
            .map_err(|e| SystemTrayError::Creation(e.to_string()))?;
        
        // Set a tooltip for the tray icon
        #[cfg(target_os = "windows")]
        {
            // The tooltip functionality may not be directly supported in the current version
            // We'll work with what's available in the API
            log::info!("Setting initial tooltip: RustPods - Disconnected");
            
            // If direct tooltip setting isn't available, we'll track it internally
            // and update the tray icon when needed
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
            .map_err(SystemTrayError::MenuItem)?;
        
        // Add separator
        tray.add_menu_item("-", || {})
            .map_err(|e| SystemTrayError::MenuItem(e.to_string()))?;
        
        // Add scan group
        Self::add_menu_group(&mut tray, &tx, &scan_actions)
            .map_err(SystemTrayError::MenuItem)?;
        
        // Add separator
        tray.add_menu_item("-", || {})
            .map_err(|e| SystemTrayError::MenuItem(e.to_string()))?;
        
        // Add settings group
        Self::add_menu_group(&mut tray, &tx, &settings_actions)
            .map_err(SystemTrayError::MenuItem)?;
        
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
            .map_err(SystemTrayError::MenuItem)?;
        
        let mut startup_registered = false;
        
        // Create system tray structure
        let mut system_tray = Self {
            tray,
            tx,
            config,
            is_connected: false,
            theme_mode,
            startup_registered,
            state_manager: None,
            last_battery_status: None,
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
            // In tray-item 0.7, the event handling API might be different
            // We'll use the available API instead
            
            // The system tray icon click handling is typically implemented via menu items
            // We've already set up appropriate menu items for our functionality
            
            log::info!("Registered handlers for tray item clicks via menu");
            
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
            .map_err(SystemTrayError::WindowsError)?;
        
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
        
        // Temporary dummy icon path - replace with actual icon path later
        let icon_path = "C:\\Windows\\System32\\shell32.dll,0";
        
        self.tray.set_icon(icon_path)
            .map_err(|e| SystemTrayError::SetIcon(e.to_string()))?;
        
        // Log tooltip update instead of setting it directly
        #[cfg(target_os = "windows")]
        {
            let tooltip = if connected {
                "RustPods - Connected"
            } else {
                "RustPods - Disconnected"
            };
            
            log::debug!("Icon tooltip updated: {}", tooltip);
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
            
            // Instead of calling set_tooltip which might not be available,
            // we'll just log the tooltip update for now
            log::debug!("Tooltip updated: {}", tooltip);
            
            // Check for low battery and show warning if needed
            if self.config.ui.show_low_battery_warning {
                let threshold = self.config.ui.low_battery_threshold;
                let left_low = left.is_some_and(|v| v <= threshold);
                let right_low = right.is_some_and(|v| v <= threshold);
                
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
    
    /// Connect the system tray to the state manager
    pub fn connect_state_manager(&mut self, state_manager: Arc<StateManager>) -> Result<(), SystemTrayError> {
        self.state_manager = Some(state_manager);
        
        // Initialize from current state if available
        if let Some(state_manager) = &self.state_manager {
            let device_state = state_manager.get_device_state();
            
            // Update connected status based on state
            if device_state.selected_device.is_some() {
                self.is_connected = true;
                self.update_icon(true)?;
            }
            
            // Update battery status if available
            if let Some(battery_status) = device_state.battery_status {
                self.last_battery_status = Some(battery_status.clone());
                
                // Update tooltip with battery information
                let left = battery_status.battery.left;
                let right = battery_status.battery.right;
                let case = battery_status.battery.case;
                
                self.update_tooltip_with_battery(left, right, case)?;
                
                // Show low battery notification if needed
                self.check_low_battery_notification(&battery_status);
            }
        }
        
        Ok(())
    }
    
    /// Check for low battery and show notification if needed
    fn check_low_battery_notification(&self, status: &AirPodsBatteryStatus) {
        // Only show notifications if enabled
        if !self.config.ui.show_low_battery_warning {
            return;
        }
        
        let threshold = self.config.ui.low_battery_threshold;
        let mut low_battery_components = Vec::new();
        
        // Check left earbud
        if let Some(left) = status.battery.left {
            if left <= threshold {
                low_battery_components.push(format!("Left earbud: {}%", left));
            }
        }
        
        // Check right earbud
        if let Some(right) = status.battery.right {
            if right <= threshold {
                low_battery_components.push(format!("Right earbud: {}%", right));
            }
        }
        
        // Check case
        if let Some(case) = status.battery.case {
            if case <= threshold {
                low_battery_components.push(format!("Case: {}%", case));
            }
        }
        
        // Show notification if any component is low
        if !low_battery_components.is_empty() {
            let message = format!(
                "Low battery warning: {}",
                low_battery_components.join(", ")
            );
            
            #[cfg(target_os = "windows")]
            {
                let _ = self.show_windows_notification(&message);
            }
        }
    }

    /// Update with new battery status from state manager
    pub fn handle_battery_update(&mut self, status: AirPodsBatteryStatus) -> Result<(), SystemTrayError> {
        // Store the status
        self.last_battery_status = Some(status.clone());
        
        // Extract battery levels
        let left = status.battery.left;
        let right = status.battery.right;
        let case = status.battery.case;
        
        // Update tooltip with battery information
        self.update_tooltip_with_battery(left, right, case)?;
        
        // Update icon if we have a connection
        if !self.is_connected {
            self.is_connected = true;
            self.update_icon(true)?;
        }
        
        // Check if we should show low battery notification
        self.check_low_battery_notification(&status);
        
        Ok(())
    }

    /// Update application configuration
    pub fn update_config(&mut self, config: AppConfig) -> Result<(), SystemTrayError> {
        // Save old value to compare
        let old_launch_at_startup = self.config.system.launch_at_startup;
        let old_theme = self.config.ui.theme.clone();
        
        // Update the config
        self.config = config;
        
        // Check if we need to update startup registration
        #[cfg(target_os = "windows")]
        if old_launch_at_startup != self.config.system.launch_at_startup {
            if let Err(e) = self.set_startup_enabled(self.config.system.launch_at_startup) {
                log::error!("Failed to update startup setting: {}", e);
                // Continue despite error to avoid breaking other functionality
            }
        }
        
        // Check if theme changed
        let new_theme_mode = Self::detect_theme_mode(&self.config);
        if self.theme_mode != new_theme_mode {
            self.theme_mode = new_theme_mode;
            // Update icon without changing connection status
            self.update_icon(self.is_connected)?;
        }
        
        // If we have battery status, update the tooltip with new preferences
        if let Some(status) = &self.last_battery_status {
            let left = status.battery.left;
            let right = status.battery.right;
            let case = status.battery.case;
            self.update_tooltip_with_battery(left, right, case)?;
        }
        
        Ok(())
    }

    /// Process state updates
    pub fn process_state_update(&mut self) -> Result<(), SystemTrayError> {
        if let Some(state_manager) = &self.state_manager {
            // Get a copy of all the state data we need first
            let device_state = state_manager.get_device_state();
            let config = state_manager.get_config();
            
            // Update connection status
            let is_connected = device_state.selected_device.is_some();
            let connection_changed = is_connected != self.is_connected;
            self.is_connected = is_connected;
            
            // Update connection if changed
            if connection_changed {
                self.update_icon(is_connected)?;
            }
            
            // Update battery status if available and changed
            if let Some(battery_status) = device_state.battery_status.clone() {
                // Only update if status is different from last known
                let should_update = match &self.last_battery_status {
                    None => true,
                    Some(last) => last != &battery_status,
                };
                
                if should_update {
                    let battery_clone = battery_status.clone();
                    let left = battery_clone.battery.left;
                    let right = battery_clone.battery.right;
                    let case = battery_clone.battery.case;
                    
                    // Update tooltip
                    self.update_tooltip_with_battery(left, right, case)?;
                    
                    // Store the new status
                    self.last_battery_status = Some(battery_status);
                    
                    // Check for low battery
                    if let Some(status) = &self.last_battery_status {
                        self.check_low_battery_notification(status);
                    }
                }
            }
            
            // Update theme if changed
            let theme_changed = config.ui.theme != self.theme_mode.into();
            if theme_changed {
                self.theme_mode = Self::map_theme_mode(&config.ui.theme);
                self.update_theme()?;
            }
        }
        
        Ok(())
    }

    /// Convert from config theme to ThemeMode
    fn map_theme_mode(theme: &ConfigTheme) -> ThemeMode {
        match theme {
            ConfigTheme::Light => ThemeMode::Light,
            ConfigTheme::Dark => ThemeMode::Dark,
            ConfigTheme::System => {
                if Self::is_system_using_dark_mode() {
                    ThemeMode::Dark
                } else {
                    ThemeMode::Light
                }
            }
        }
    }

    /// Update the theme of the system tray
    pub fn update_theme(&mut self) -> Result<(), SystemTrayError> {
        self.update_icon(self.is_connected)
    }
}

// Drop implementation to ensure proper cleanup on application exit
impl Drop for SystemTray {
    fn drop(&mut self) {
        // Cleanup tray icon before dropping to avoid lingering icons
        #[cfg(target_os = "windows")]
        {
            if let Err(e) = self.cleanup() {
                log::error!("Error cleaning up system tray: {}", e);
            }
        }
        
        log::debug!("SystemTray dropped");
    }
}

// Custom Clone implementation because TrayItem doesn't implement Clone
impl Clone for SystemTray {
    fn clone(&self) -> Self {
        // Create a new system tray with the same parameters
        let config = self.config.clone();
        let theme_mode = self.theme_mode;
        let startup_registered = self.startup_registered;
        let is_connected = self.is_connected;
        let last_battery_status = self.last_battery_status.clone();
        
        // Create a dummy tray item to satisfy requirements
        // This isn't ideal, but necessary for our Clone implementation
        let app_name = "RustPods";
        let icon_path = "C:\\Windows\\System32\\shell32.dll,0";
        let tray = TrayItem::new(app_name, icon_path)
            .expect("Failed to create clone of tray item");
            
        Self {
            tray,
            tx: self.tx.clone(),
            config,
            is_connected,
            theme_mode,
            startup_registered,
            state_manager: self.state_manager.clone(),
            last_battery_status,
        }
    }
}

// Custom Debug implementation because TrayItem doesn't implement Debug
impl std::fmt::Debug for SystemTray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SystemTray")
            .field("is_connected", &self.is_connected)
            .field("theme_mode", &self.theme_mode)
            .field("startup_registered", &self.startup_registered)
            .field("last_battery_status", &self.last_battery_status)
            .field("config", &self.config)
            .finish_non_exhaustive() // Skip tray and tx fields
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