
use tokio::sync::mpsc::UnboundedSender;
use tray_icon::{TrayIcon, TrayIconBuilder, TrayIconEvent, MouseButton, menu::{Menu, MenuEvent}};
use tray_icon::menu::MenuItem as TrayMenuItem;
use tray_icon::Icon;

use std::path::Path;

use crate::ui::Message;
use crate::config::{AppConfig, Theme as ConfigTheme};

use std::sync::Arc;
use std::sync::{Mutex};

/// Menu item configuration
#[allow(dead_code)]
struct MenuItem {
    /// Label to display in the menu
    label: String,
    /// Keyboard shortcut (Windows only)
    shortcut: Option<String>,
    /// Message to send when clicked
    message: Option<Message>,
}

/// Light or dark theme mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
}

impl From<ConfigTheme> for ThemeMode {
    fn from(theme: ConfigTheme) -> Self {
        match theme {
            ConfigTheme::Dark => ThemeMode::Dark,
            ConfigTheme::Light => ThemeMode::Light,
            _ => ThemeMode::Dark, // Default to dark
        }
    }
}

/// System tray related errors
#[derive(Debug, thiserror::Error)]
pub enum SystemTrayError {
    #[error("Failed to create system tray: {0}")]
    Creation(String),
    
    #[error("Failed to set tray icon: {0}")]
    SetIcon(String),
    
    #[error("Failed to add menu item: {0}")]
    MenuItem(String),
    
    #[error("Failed to update tooltip: {0}")]
    Tooltip(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Image error: {0}")]
    ImageError(String),

    #[error("Windows API error: {0}")]
    WindowsApi(String),
}

/// Window control commands
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum WindowCommand {
    Show,
    Hide,
    Toggle,
    Exit,
}

/// Direct Windows window controller
pub struct DirectWindowController {
    window_handle: Arc<Mutex<Option<isize>>>,
    ui_sender: Option<UnboundedSender<Message>>,
}

impl DirectWindowController {
    pub fn new() -> Self {
        Self {
            window_handle: Arc::new(Mutex::new(None)),
            ui_sender: None,
        }
    }

    pub fn set_ui_sender(&mut self, sender: UnboundedSender<Message>) {
        self.ui_sender = Some(sender);
    }

    pub fn set_window_handle(&self, handle: isize) {
        if let Ok(mut guard) = self.window_handle.lock() {
            *guard = Some(handle);
            log::info!("Window handle set: {}", handle);
        }
    }

    #[cfg(target_os = "windows")]
    pub fn show_window(&self) -> Result<(), SystemTrayError> {
        log::info!("System tray: Requesting window show via UI message (avoiding Windows API)");
        
        // Only use UI messages - let Iced handle the actual window operations
        if let Some(ref sender) = self.ui_sender {
            match sender.send(Message::ShowWindow) {
                Ok(_) => {
                    log::debug!("ShowWindow message sent successfully to UI");
                    Ok(())
                },
                Err(e) => {
                    log::error!("Failed to send ShowWindow message: {}", e);
                    Err(SystemTrayError::WindowsApi(format!("Failed to send show message: {}", e)))
                }
            }
        } else {
            log::error!("No UI sender available for ShowWindow message");
            Err(SystemTrayError::WindowsApi("No UI sender available".to_string()))
        }
    }

    #[cfg(target_os = "windows")]
    pub fn hide_window(&self) -> Result<(), SystemTrayError> {
        log::info!("System tray: Requesting window hide via UI message (avoiding Windows API)");
        
        // Only use UI messages - let Iced handle the actual window operations
        if let Some(ref sender) = self.ui_sender {
            match sender.send(Message::HideWindow) {
                Ok(_) => {
                    log::debug!("HideWindow message sent successfully to UI");
                    Ok(())
                },
                Err(e) => {
                    log::error!("Failed to send HideWindow message: {}", e);
                    Err(SystemTrayError::WindowsApi(format!("Failed to send hide message: {}", e)))
                }
            }
        } else {
            log::error!("No UI sender available for HideWindow message");
            Err(SystemTrayError::WindowsApi("No UI sender available".to_string()))
        }
    }

    #[cfg(target_os = "windows")]
    pub fn is_window_visible(&self) -> bool {
        // Since we're avoiding Windows API calls that cause channel issues,
        // we'll assume the window needs to be shown when clicked
        // The toggle logic will be simplified to always show
        log::debug!("Visibility check: assuming window needs to be shown");
        false
    }

    #[cfg(not(target_os = "windows"))]
    pub fn show_window(&self) -> Result<(), SystemTrayError> {
        // Non-Windows fallback
        if let Some(ref sender) = self.ui_sender {
            sender.send(Message::ToggleVisibility)
                .map_err(|e| SystemTrayError::WindowsApi(format!("Failed to send show message: {}", e)))?;
        }
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    pub fn hide_window(&self) -> Result<(), SystemTrayError> {
        // Non-Windows fallback
        if let Some(ref sender) = self.ui_sender {
            sender.send(Message::ToggleVisibility)
                .map_err(|e| SystemTrayError::WindowsApi(format!("Failed to send hide message: {}", e)))?;
        }
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    pub fn is_window_visible(&self) -> bool {
        true // Assume visible on non-Windows
    }

    pub fn toggle_window(&self) -> Result<(), SystemTrayError> {
        let is_visible = self.is_window_visible();
        log::debug!("Toggle window called: window visible = {}", is_visible);
        
        if is_visible {
            log::debug!("Window is visible, hiding it");
            self.hide_window()
        } else {
            log::debug!("Window is not visible, showing it");
            self.show_window()
        }
    }

    pub fn exit_application(&self) -> Result<(), SystemTrayError> {
        log::info!("DirectWindowController: Sending ForceQuit message to UI");
        if let Some(ref sender) = self.ui_sender {
            sender.send(Message::ForceQuit)
                .map_err(|e| SystemTrayError::WindowsApi(format!("Failed to send force quit message: {}", e)))?;
        } else {
            log::warn!("No UI sender available, forcing process exit");
            // Fallback: force exit using process::exit if no UI channel available
            std::process::exit(0);
        }
        Ok(())
    }
}

impl Clone for DirectWindowController {
    fn clone(&self) -> Self {
        Self {
            window_handle: Arc::clone(&self.window_handle),
            ui_sender: self.ui_sender.clone(),
        }
    }
}

/// Manages the system tray icon and menu
pub struct SystemTray {
    /// The system tray icon
    tray: Option<TrayIcon>,
    /// Direct window controller
    window_controller: DirectWindowController,
    /// Application configuration
    config: AppConfig,
    /// Last known connection status
    is_connected: bool,
    /// Current theme mode
    theme_mode: ThemeMode,
    /// Whether the tray is initialized
    initialized: bool,
}

impl std::fmt::Debug for SystemTray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SystemTray")
            .field("is_connected", &self.is_connected)
            .field("theme_mode", &self.theme_mode)
            .field("initialized", &self.initialized)
            .finish()
    }
}

impl Clone for SystemTray {
    fn clone(&self) -> Self {
        // Create a new SystemTray with the same configuration
        // Note: The tray itself cannot be cloned, so we create a new uninitialized one
        Self {
            tray: None,
            window_controller: self.window_controller.clone(),
            config: self.config.clone(),
            is_connected: self.is_connected,
            theme_mode: self.theme_mode,
            initialized: false,
        }
    }
}

impl SystemTray {
    /// Create a new system tray with direct window controller
    pub fn new(config: AppConfig) -> Result<Self, SystemTrayError> {
        let theme_mode = ThemeMode::from(config.ui.theme.clone());
        let window_controller = DirectWindowController::new();
        
        log::info!("Creating system tray with direct window controller and theme mode: {:?}", theme_mode);
        
        Ok(Self {
            tray: None,
            window_controller,
            config,
            is_connected: false,
            theme_mode,
            initialized: false,
        })
    }

    /// Set the UI sender for fallback communication
    pub fn set_ui_sender(&mut self, sender: UnboundedSender<Message>) {
        self.window_controller.set_ui_sender(sender);
    }

    /// Set the window handle for direct Windows API control
    pub fn set_window_handle(&self, handle: isize) {
        self.window_controller.set_window_handle(handle);
    }

    /// Initialize the system tray (creates the actual icon and menu)
    pub fn initialize(&mut self) -> Result<(), SystemTrayError> {
        log::info!("Initializing system tray...");
        
        // Create the menu
        let show_hide_item = TrayMenuItem::new("Show/Hide Window", true, None);
        let quit_item = TrayMenuItem::new("Quit", true, None);
        
        let menu = Menu::new();
        menu.append(&show_hide_item)
            .map_err(|e| SystemTrayError::MenuItem(format!("Failed to add show/hide item: {}", e)))?;
        menu.append(&quit_item)
            .map_err(|e| SystemTrayError::MenuItem(format!("Failed to add quit item: {}", e)))?;
        
        // Load initial icon
        let icon = self.load_icon(false)?;
        
        // Create the tray icon
        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("RustPods - AirPods Battery Monitor")
            .with_icon(icon)
            .build()
            .map_err(|e| SystemTrayError::Creation(format!("Failed to build tray icon: {}", e)))?;
        
        self.tray = Some(tray);
        self.initialized = true;
        
        // Set up event handling with direct window control
        self.setup_direct_window_events()?;
        
        log::info!("System tray initialized successfully");
        Ok(())
    }
    
    /// Set up menu and icon event handling with direct window control
    fn setup_direct_window_events(&self) -> Result<(), SystemTrayError> {
        let window_controller = self.window_controller.clone();
        
        // Handle menu events
        std::thread::spawn(move || {
            let event_receiver = MenuEvent::receiver();
            loop {
                if let Ok(event) = event_receiver.recv() {
                    log::debug!("Menu event received: {:?}", event.id.0);
                    match event.id.0.as_str() {
                        "Show/Hide Window" | "1000" => {
                            log::debug!("Tray menu: Show/Hide Window clicked");
                            if let Err(e) = window_controller.toggle_window() {
                                log::error!("Failed to toggle window from menu: {}", e);
                            }
                        }
                        "Quit" | "1001" => {
                            log::debug!("Tray menu: Quit clicked");
                            if let Err(e) = window_controller.exit_application() {
                                log::error!("Failed to exit application from menu: {}", e);
                            }
                        }
                        _ => {
                            log::debug!("Unknown menu item clicked: {}", event.id.0);
                        }
                    }
                } else {
                    // Channel closed, exit thread
                    log::debug!("Menu event channel closed, exiting thread");
                    break;
                }
            }
        });
        
        // Handle tray icon events (clicks on the icon itself)
        let window_controller2 = self.window_controller.clone();
        std::thread::spawn(move || {
            let event_receiver = TrayIconEvent::receiver();
            loop {
                if let Ok(event) = event_receiver.recv() {
                    log::debug!("Tray icon event received: {:?}", event);
                    match event {
                        TrayIconEvent::Click { 
                            button: MouseButton::Left,
                            ..
                        } => {
                            log::debug!("Tray icon: Left click detected - always showing window");
                            if let Err(e) = window_controller2.show_window() {
                                log::error!("Failed to show window from icon click: {}", e);
                            }
                        }
                        _ => {
                            log::debug!("Other tray icon event: {:?}", event);
                        }
                    }
                } else {
                    // Channel closed, exit thread
                    log::debug!("Tray icon event channel closed, exiting thread");
                    break;
                }
            }
        });
        
        Ok(())
    }
    
    /// Load the appropriate icon based on theme and connection status
    fn load_icon(&self, connected: bool) -> Result<Icon, SystemTrayError> {
        let icon_filename = match (self.theme_mode, connected) {
            (ThemeMode::Light, false) => "rustpods-tray-light-disconnected.ico",
            (ThemeMode::Light, true) => "rustpods-tray-light-connected.ico",
            (ThemeMode::Dark, false) => "rustpods-tray-dark-disconnected.ico",
            (ThemeMode::Dark, true) => "rustpods-tray-dark-connected.ico",
        };
        
        let icon_path = format!("assets/icons/tray/{}", icon_filename);
        
        log::debug!("Loading tray icon from: {}", icon_path);
        
        if !Path::new(&icon_path).exists() {
            log::warn!("Icon file not found: {}, using fallback", icon_path);
            return self.create_fallback_icon();
        }
        
        // Use the correct method to load icon from path
        let icon = Icon::from_path(&icon_path, Some((32, 32)))
            .map_err(|e| SystemTrayError::ImageError(format!("Failed to load icon from path: {}", e)))?;
        
        Ok(icon)
    }
    
    /// Create a simple fallback icon if files are missing
    fn create_fallback_icon(&self) -> Result<Icon, SystemTrayError> {
        log::info!("Creating fallback icon");
        
        // Create a simple 32x32 RGBA icon
        let size = 32u32;
        let mut rgba_data = Vec::with_capacity((size * size * 4) as usize);
        
        for y in 0..size {
            for x in 0..size {
                // Create a simple circle pattern
                let center_x = size as f32 / 2.0;
                let center_y = size as f32 / 2.0;
                let distance = ((x as f32 - center_x).powi(2) + (y as f32 - center_y).powi(2)).sqrt();
                
                if distance < size as f32 / 3.0 {
                    // Inner circle - blue for connected, gray for disconnected
                    if self.is_connected {
                        rgba_data.extend_from_slice(&[70, 130, 255, 255]); // Blue
                    } else {
                        rgba_data.extend_from_slice(&[128, 128, 128, 255]); // Gray
                    }
                } else if distance < size as f32 / 2.5 {
                    // Border
                    rgba_data.extend_from_slice(&[64, 64, 64, 255]); // Dark gray
                } else {
                    // Transparent background
                    rgba_data.extend_from_slice(&[0, 0, 0, 0]);
                }
            }
        }
        
        Icon::from_rgba(rgba_data, size, size)
            .map_err(|e| SystemTrayError::ImageError(format!("Failed to create fallback icon: {}", e)))
    }
    
    /// Update the tray icon based on connection status
    pub fn update_icon(&mut self, connected: bool) -> Result<(), SystemTrayError> {
        if !self.initialized {
            log::debug!("System tray not initialized, skipping icon update");
            return Ok(());
        }
        
        if self.is_connected == connected {
            log::debug!("Connection status unchanged, skipping icon update");
            return Ok(());
        }
        
        self.is_connected = connected;
        
        log::debug!("Updating tray icon for connection status: {}", connected);
        
        let icon = self.load_icon(connected)?;
        
        if let Some(ref tray) = self.tray {
            tray.set_icon(Some(icon))
                .map_err(|e| SystemTrayError::SetIcon(format!("Failed to update icon: {}", e)))?;
        }
        
        Ok(())
    }
    
    /// Update tooltip with battery information
    pub fn update_tooltip_with_battery(&mut self, left: Option<u8>, right: Option<u8>, case: Option<u8>) -> Result<(), SystemTrayError> {
        if !self.initialized {
            return Ok(());
        }
        
        let tooltip = match (left, right, case) {
            (Some(l), Some(r), Some(c)) => {
                format!("RustPods - AirPods Battery\nLeft: {}% | Right: {}% | Case: {}%", l, r, c)
            }
            (Some(l), Some(r), None) => {
                format!("RustPods - AirPods Battery\nLeft: {}% | Right: {}%", l, r)
            }
            _ => {
                "RustPods - AirPods Battery Monitor".to_string()
            }
        };
        
        if let Some(ref tray) = self.tray {
            tray.set_tooltip(Some(tooltip))
                .map_err(|e| SystemTrayError::Tooltip(format!("Failed to update tooltip: {}", e)))?;
        }
        
        Ok(())
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: AppConfig) -> Result<(), SystemTrayError> {
        self.config = config.clone();
        let new_theme_mode = ThemeMode::from(config.ui.theme);
        
        if !matches!(self.theme_mode, _new_theme_mode) {
            self.theme_mode = new_theme_mode;
            // Update icon for new theme
            self.update_icon(self.is_connected)?;
        }
        
        Ok(())
    }
    
    /// Get window controller for external use
    pub fn window_controller(&self) -> DirectWindowController {
        self.window_controller.clone()
    }
    
    /// Check if startup registration is enabled (Windows-specific)
    #[cfg(target_os = "windows")]
    pub fn is_startup_registered(&self) -> bool {
        // For now, return false - this would need to check Windows registry
        // TODO: Implement actual Windows startup registration check
        false
    }
    
    /// Set startup registration (Windows-specific)
    #[cfg(target_os = "windows")]
    pub fn set_startup_enabled(&mut self, enabled: bool) -> Result<(), SystemTrayError> {
        // For now, just log - this would need to modify Windows registry
        // TODO: Implement actual Windows startup registration
        log::info!("Startup registration set to: {} (not implemented)", enabled);
        Ok(())
    }
    
    /// Cleanup the system tray
    pub fn cleanup(&mut self) -> Result<(), SystemTrayError> {
        if let Some(tray) = self.tray.take() {
            log::info!("Cleaning up system tray");
            drop(tray);
        }
        
        self.initialized = false;
        Ok(())
    }
}

impl Drop for SystemTray {
    fn drop(&mut self) {
        if let Err(e) = self.cleanup() {
            log::error!("Failed to cleanup system tray: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_tray_creation() {
        let config = AppConfig::default();
        let result = SystemTray::new(config);
        
        assert!(result.is_ok());
        let tray = result.unwrap();
        assert!(!tray.initialized);
        assert_eq!(tray.theme_mode, ThemeMode::Dark); // Default theme
    }

    #[test]
    fn test_menu_item_struct() {
        let menu_item = MenuItem {
            label: "Test".to_string(),
            shortcut: Some("Ctrl+T".to_string()),
            message: Some(Message::Exit),
        };
        
        assert_eq!(menu_item.label, "Test");
        assert_eq!(menu_item.shortcut, Some("Ctrl+T".to_string()));
    }

    #[test]
    fn test_theme_detection() {
        use crate::config::Theme as ConfigTheme;
        
        assert_eq!(ThemeMode::from(ConfigTheme::Dark), ThemeMode::Dark);
        assert_eq!(ThemeMode::from(ConfigTheme::Light), ThemeMode::Light);
        
        // Test default case
        assert_eq!(ThemeMode::from(ConfigTheme::System), ThemeMode::Dark);
    }

    /// Create a dummy system tray for testing
    #[allow(dead_code)]
    fn create_dummy_tray() -> Result<SystemTray, SystemTrayError> {
        let config = AppConfig::default();
        SystemTray::new(config)
    }
} 