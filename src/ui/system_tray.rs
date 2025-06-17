//! System tray implementation for RustPods

use crate::config::{AppConfig, Theme as ConfigTheme};
use crate::ui::message::Message;
use log;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::UnboundedSender;
use tray_icon::{TrayIcon, TrayIconBuilder, TrayIconEvent, MouseButton, menu::{Menu, MenuEvent}};
use tray_icon::menu::MenuItem as TrayMenuItem;
use tray_icon::Icon;
use std::path::Path;

/// Theme mode for system tray icons
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
}

impl From<ConfigTheme> for ThemeMode {
    fn from(theme: ConfigTheme) -> Self {
        match theme {
            ConfigTheme::Light => ThemeMode::Light,
            ConfigTheme::Dark => ThemeMode::Dark,
            ConfigTheme::System => ThemeMode::Dark, // Default to dark for system theme
        }
    }
}

/// System tray error types
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
    IoError(String),

    #[error("Icon loading error: {0}")]
    IconLoad(String),
}

/// Simple window controller for system tray
#[derive(Debug, Clone)]
pub struct DirectWindowController {
    ui_sender: Arc<Mutex<Option<UnboundedSender<Message>>>>,
}

impl Default for DirectWindowController {
    fn default() -> Self {
        Self::new()
    }
}

impl DirectWindowController {
    pub fn new() -> Self {
        Self {
            ui_sender: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_ui_sender(&mut self, sender: UnboundedSender<Message>) {
        if let Ok(mut ui_sender) = self.ui_sender.lock() {
            *ui_sender = Some(sender);
        }
    }

    pub fn toggle_window(&self) -> Result<(), SystemTrayError> {
        if let Ok(ui_sender) = self.ui_sender.lock() {
            if let Some(ref sender) = *ui_sender {
                let _ = sender.send(Message::ToggleWindow);
            }
        }
        Ok(())
    }

    pub fn show_window(&self) -> Result<(), SystemTrayError> {
        if let Ok(ui_sender) = self.ui_sender.lock() {
            if let Some(ref sender) = *ui_sender {
                let _ = sender.send(Message::ShowWindow);
            }
        }
        Ok(())
    }

    pub fn exit_application(&self) -> Result<(), SystemTrayError> {
        if let Ok(ui_sender) = self.ui_sender.lock() {
            if let Some(ref sender) = *ui_sender {
                let _ = sender.send(Message::Exit);
            }
        }
        std::process::exit(0);
    }
}

/// System tray implementation
pub struct SystemTray {
    /// The system tray icon
    tray: Option<TrayIcon>,
    /// Menu items
    menu: Option<Menu>,
    /// Menu item IDs
    show_hide_item: Option<TrayMenuItem>,
    exit_item: Option<TrayMenuItem>,
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
    /// Event receiver
    menu_receiver: Option<crossbeam_channel::Receiver<MenuEvent>>,
    tray_receiver: Option<crossbeam_channel::Receiver<TrayIconEvent>>,
}

impl std::fmt::Debug for SystemTray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SystemTray")
            .field("initialized", &self.initialized)
            .field("is_connected", &self.is_connected)
            .finish()
    }
}

impl Clone for SystemTray {
    fn clone(&self) -> Self {
        Self {
            tray: None, // TrayIcon is not cloneable
            menu: None,
            show_hide_item: None,
            exit_item: None,
            window_controller: self.window_controller.clone(),
            config: self.config.clone(),
            is_connected: self.is_connected,
            theme_mode: self.theme_mode,
            initialized: false,
            menu_receiver: None,
            tray_receiver: None,
        }
    }
}

impl SystemTray {
    /// Create a new system tray instance
    pub fn new(config: AppConfig) -> Result<Self, SystemTrayError> {
        let theme_mode = ThemeMode::from(config.ui.theme.clone());
        
        Ok(Self {
            tray: None,
            menu: None,
            show_hide_item: None,
            exit_item: None,
            window_controller: DirectWindowController::new(),
            config,
            is_connected: false,
            theme_mode,
            initialized: false,
            menu_receiver: None,
            tray_receiver: None,
        })
    }

    /// Set the UI sender for fallback communication
    pub fn set_ui_sender(&mut self, sender: UnboundedSender<Message>) {
        self.window_controller.set_ui_sender(sender);
    }

    /// Get the appropriate icon path based on connection status and theme
    fn get_icon_path(&self) -> String {
        let theme_str = match self.theme_mode {
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
        };
        
        let status_str = if self.is_connected { "connected" } else { "disconnected" };
        
        // Get the executable directory and construct absolute path
        let exe_path = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("./rustpods.exe"));
        let exe_dir = exe_path.parent().unwrap_or_else(|| std::path::Path::new("."));
        
        // Try multiple possible locations for the icon
        let icon_paths = vec![
            // 1. Tray icons in same directory as executable (for release builds)
            exe_dir.join(format!("rustpods-tray-{}-{}.ico", theme_str, status_str)),
            // 2. Assets folder relative to executable
            exe_dir.join("assets").join("icons").join("tray").join(format!("rustpods-tray-{}-{}.ico", theme_str, status_str)),
            // 3. Project root assets (for development)
            exe_dir.parent().and_then(|p| p.parent()).map(|project_root| 
                project_root.join("assets").join("icons").join("tray").join(format!("rustpods-tray-{}-{}.ico", theme_str, status_str))
            ).unwrap_or_default(),
            // 4. Current working directory
            std::env::current_dir().unwrap_or_default().join("assets").join("icons").join("tray").join(format!("rustpods-tray-{}-{}.ico", theme_str, status_str)),
        ];
        
        // Find the first existing icon file
        for icon_path in icon_paths {
            if icon_path.exists() {
                log::debug!("Found tray icon at: {}", icon_path.display());
                return icon_path.to_string_lossy().to_string();
            }
        }
        
        // Fallback to relative path if none found
        let fallback = format!("assets/icons/tray/rustpods-tray-{}-{}.ico", theme_str, status_str);
        log::warn!("No tray icon found, using fallback: {}", fallback);
        fallback
    }

    /// Load icon from file path
    fn load_icon(&self, path: &str) -> Result<Icon, SystemTrayError> {
        let icon_path = Path::new(path);
        
        if !icon_path.exists() {
            return Err(SystemTrayError::IconLoad(format!("Icon file not found: {}", path)));
        }

        // For ICO files, we need to use from_path instead of from_rgba
        if path.ends_with(".ico") {
            Icon::from_path(icon_path, Some((32, 32)))
                .map_err(|e| SystemTrayError::IconLoad(format!("Failed to create icon from ICO file {}: {}", path, e)))
        } else {
            // For other formats, try to load as image and convert to RGBA
            let icon_bytes = std::fs::read(icon_path)
                .map_err(|e| SystemTrayError::IconLoad(format!("Failed to read icon file {}: {}", path, e)))?;

            Icon::from_rgba(icon_bytes, 32, 32)
                .map_err(|e| SystemTrayError::IconLoad(format!("Failed to create icon from {}: {}", path, e)))
        }
    }

    /// Initialize the system tray (creates the actual icon and menu)
    pub fn initialize(&mut self) -> Result<(), SystemTrayError> {
        if self.initialized {
            log::debug!("System tray already initialized");
            return Ok(());
        }

        log::info!("Initializing system tray...");

        // Create menu items
        let show_hide_item = TrayMenuItem::new("Show/Hide", true, None);
        let exit_item = TrayMenuItem::new("Exit", true, None);

        // Create menu
        let menu = Menu::new();
        menu.append(&show_hide_item)
            .map_err(|e| SystemTrayError::MenuItem(format!("Failed to add show/hide item: {}", e)))?;
        menu.append(&exit_item)
            .map_err(|e| SystemTrayError::MenuItem(format!("Failed to add exit item: {}", e)))?;

        // Get icon path and load icon
        let icon_path = self.get_icon_path();
        let icon = self.load_icon(&icon_path)?;

        // Create tray icon
        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu.clone()))
            .with_tooltip("RustPods - AirPods Battery Monitor")
            .with_icon(icon)
            .build()
            .map_err(|e| SystemTrayError::Creation(format!("Failed to create tray icon: {}", e)))?;

        // Set up event channels
        let menu_channel = MenuEvent::receiver().clone();
        let tray_channel = TrayIconEvent::receiver().clone();

        // Store everything
        self.tray = Some(tray);
        self.menu = Some(menu);
        self.show_hide_item = Some(show_hide_item);
        self.exit_item = Some(exit_item);
        self.menu_receiver = Some(menu_channel);
        self.tray_receiver = Some(tray_channel);
        self.initialized = true;

        log::info!("System tray initialized successfully");
        Ok(())
    }

    /// Process tray events (should be called regularly)
    pub fn process_events(&mut self) -> Result<(), SystemTrayError> {
        if !self.initialized {
            return Ok(());
        }

        // Collect events first to avoid borrowing issues
        let mut menu_events = Vec::new();
        let mut tray_events = Vec::new();

        // Process menu events
        if let Some(ref menu_receiver) = self.menu_receiver {
            while let Ok(event) = menu_receiver.try_recv() {
                menu_events.push(event);
            }
        }

        // Process tray events
        if let Some(ref tray_receiver) = self.tray_receiver {
            while let Ok(event) = tray_receiver.try_recv() {
                tray_events.push(event);
            }
        }

        // Handle collected events
        for event in menu_events {
            self.handle_menu_event(event)?;
        }

        for event in tray_events {
            self.handle_tray_event(event)?;
        }

        Ok(())
    }

    /// Handle menu events
    fn handle_menu_event(&mut self, event: MenuEvent) -> Result<(), SystemTrayError> {
        if let Some(ref show_hide_item) = self.show_hide_item {
            if event.id == show_hide_item.id() {
                self.window_controller.toggle_window()?;
                return Ok(());
            }
        }

        if let Some(ref exit_item) = self.exit_item {
            if event.id == exit_item.id() {
                self.window_controller.exit_application()?;
                return Ok(());
            }
        }

        Ok(())
    }

    /// Handle tray icon events
    fn handle_tray_event(&mut self, event: TrayIconEvent) -> Result<(), SystemTrayError> {
        if let TrayIconEvent::Click { button: MouseButton::Left, .. } = event {
            self.window_controller.toggle_window()?;
        }
        Ok(())
    }

    /// Update the tray icon based on connection status
    pub fn update_icon(&mut self, connected: bool) -> Result<(), SystemTrayError> {
        if !self.initialized {
            log::debug!("System tray not initialized, skipping icon update");
            return Ok(());
        }

        if self.is_connected == connected {
            // No change needed
            return Ok(());
        }

        self.is_connected = connected;
        
        // Get the icon path and load new icon
        let icon_path = self.get_icon_path();
        log::debug!("Updating tray icon to: {}", icon_path);
        
        let icon = self.load_icon(&icon_path)?;
        
        if let Some(ref mut tray) = self.tray {
            tray.set_icon(Some(icon))
                .map_err(|e| SystemTrayError::SetIcon(format!("Failed to set icon '{}': {}", icon_path, e)))?;
        }

        Ok(())
    }

    /// Update tooltip with battery information
    pub fn update_tooltip_with_battery(
        &mut self,
        left: Option<u8>,
        right: Option<u8>,
        case: Option<u8>,
    ) -> Result<(), SystemTrayError> {
        if !self.initialized {
            return Ok(());
        }

        let tooltip = match (left, right, case) {
            (Some(l), Some(r), Some(c)) => format!("RustPods - L:{}% R:{}% C:{}%", l, r, c),
            (Some(l), Some(r), None) => format!("RustPods - L:{}% R:{}%", l, r),
            _ => "RustPods - AirPods Battery Monitor".to_string(),
        };

        if let Some(ref mut tray) = self.tray {
            tray.set_tooltip(Some(tooltip.clone()))
                .map_err(|e| SystemTrayError::Tooltip(format!("Failed to set tooltip: {}", e)))?;
        }

        log::debug!("Updated tray tooltip: {}", tooltip);
        Ok(())
    }

    /// Get the window controller
    pub fn window_controller(&self) -> DirectWindowController {
        self.window_controller.clone()
    }

    /// Update configuration
    pub fn update_config(&mut self, config: AppConfig) -> Result<(), SystemTrayError> {
        self.config = config;
        self.theme_mode = ThemeMode::from(self.config.ui.theme.clone());
        Ok(())
    }

    /// Cleanup the system tray
    pub fn cleanup(&mut self) -> Result<(), SystemTrayError> {
        if let Some(_tray) = self.tray.take() {
            log::debug!("System tray cleaned up");
        }
        self.menu = None;
        self.show_hide_item = None;
        self.exit_item = None;
        self.menu_receiver = None;
        self.tray_receiver = None;
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
        let tray = SystemTray::new(config);
        assert!(tray.is_ok());
    }

    #[test]
    fn test_theme_detection() {
        let config = AppConfig::default();
        let tray = SystemTray::new(config).unwrap();
        // Should not panic
        assert!(!tray.initialized);
    }
}
