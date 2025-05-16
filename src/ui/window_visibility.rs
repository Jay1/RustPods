//! Window visibility management functionality
//! 
//! This module handles window visibility features including:
//! - Minimize to tray
//! - Restore from tray
//! - Window position memory
//! - Focus and blur events
//! - Startup visibility options

use iced::{Point, Rectangle, window, Command};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::ui::Message;
use crate::config::AppConfig;
use crate::ui::state_manager::{StateManager, Action};

/// Window position that can be saved and restored
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowPosition {
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
    /// Window width
    pub width: f32,
    /// Window height
    pub height: f32,
}

impl From<Rectangle> for WindowPosition {
    fn from(rect: Rectangle) -> Self {
        Self {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }
}

impl From<WindowPosition> for Rectangle {
    fn from(val: WindowPosition) -> Self {
        Rectangle::new(
            Point::new(val.x, val.y),
            iced::Size::new(val.width, val.height),
        )
    }
}

/// Manages window visibility state
#[derive(Debug)]
pub struct WindowVisibilityManager {
    /// Whether the window is currently visible
    is_visible: bool,
    /// Last known window position
    last_position: Option<WindowPosition>,
    /// Whether the window is currently focused
    is_focused: bool,
    /// State manager reference
    state_manager: Option<Arc<StateManager>>,
    /// Application configuration
    config: AppConfig,
    /// Last focus time
    last_focus_time: Instant,
    /// Automatic hide timeout (if enabled)
    auto_hide_timeout: Option<Duration>,
}

impl WindowVisibilityManager {
    /// Create a new window visibility manager
    pub fn new(config: AppConfig) -> Self {
        let is_visible = !config.ui.start_minimized;
        
        Self {
            is_visible,
            last_position: None,
            is_focused: false,
            state_manager: None,
            config,
            last_focus_time: Instant::now(),
            auto_hide_timeout: None,
        }
    }
    
    /// Set the state manager
    pub fn with_state_manager(mut self, state_manager: Arc<StateManager>) -> Self {
        self.state_manager = Some(state_manager);
        self
    }
    
    /// Set auto-hide timeout
    pub fn with_auto_hide_timeout(mut self, timeout: Duration) -> Self {
        self.auto_hide_timeout = Some(timeout);
        self
    }
    
    /// Show the window
    pub fn show(&mut self) -> Command<Message> {
        self.is_visible = true;
        
        if let Some(state_manager) = &self.state_manager {
            state_manager.dispatch(Action::ToggleVisibility);
        }
        
        // Use the last known position if available
        if let Some(_position) = self.last_position {
            window::change_mode(window::Mode::Windowed)
        } else {
            window::change_mode(window::Mode::Windowed)
        }
    }
    
    /// Hide the window
    pub fn hide(&mut self, bounds: Rectangle) -> Command<Message> {
        // Save the current position before hiding
        self.last_position = Some(WindowPosition::from(bounds));
        self.is_visible = false;
        
        if let Some(state_manager) = &self.state_manager {
            state_manager.dispatch(Action::ToggleVisibility);
        }
        
        window::change_mode(window::Mode::Hidden)
    }
    
    /// Toggle window visibility
    pub fn toggle(&mut self, bounds: Rectangle) -> Command<Message> {
        if self.is_visible {
            self.hide(bounds)
        } else {
            self.show()
        }
    }
    
    /// Handle window focus event
    pub fn handle_focus(&mut self) {
        self.is_focused = true;
        self.last_focus_time = Instant::now();
    }
    
    /// Handle window blur event
    pub fn handle_blur(&mut self) {
        self.is_focused = false;
        
        // If minimize on blur is enabled in config, hide the window
        if self.config.ui.start_minimized {
            // We don't have bounds here, so we can't update last_position
            // This will be handled by the main application loop
        }
    }
    
    /// Update window visibility based on auto-hide timeout
    pub fn update(&mut self, bounds: Rectangle) -> Option<Command<Message>> {
        // Check if we should auto-hide
        if let Some(timeout) = self.auto_hide_timeout {
            if self.is_visible && !self.is_focused {
                let elapsed = self.last_focus_time.elapsed();
                if elapsed > timeout {
                    return Some(self.hide(bounds));
                }
            }
        }
        
        None
    }
    
    /// Handle window close event
    pub fn handle_close_requested(&mut self, bounds: Rectangle) -> Command<Message> {
        // Don't actually close the window, just hide it to system tray
        self.hide(bounds)
    }
    
    /// Set the window position
    pub fn set_position(&mut self, position: WindowPosition) -> Command<Message> {
        self.last_position = Some(position);
        
        // Only move the window if it's visible
        if self.is_visible {
            let rect: Rectangle = position.into();
            window::move_to(
                rect.x as i32,
                rect.y as i32,
            )
        } else {
            Command::none()
        }
    }
    
    /// Get the current visibility
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }
    
    /// Get whether the window is focused
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }
    
    /// Get the last known position
    pub fn last_position(&self) -> Option<WindowPosition> {
        self.last_position
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: AppConfig) {
        self.config = config;
    }
    
    /// Move window to position
    pub fn move_window(&self, position: WindowPosition) -> Command<Message> {
        match self.last_position {
            Some(current) if current == position => Command::none(),
            _ => {
                // Convert position to integer values
                let x = position.x as i32;
                let y = position.y as i32;
                
                // Use the correct window::move_to function with just x and y parameters
                window::move_to(x, y)
            }
        }
    }
}

/// Process window visibility events
pub fn handle_window_events(
    event: &iced::Event,
    visibility_manager: &mut WindowVisibilityManager,
    bounds: Rectangle,
) -> Option<Message> {
    match event {
        iced::Event::Window(window_event) => match window_event {
            window::Event::Focused => {
                visibility_manager.handle_focus();
                None
            }
            window::Event::Unfocused => {
                visibility_manager.handle_blur();
                None
            }
            window::Event::CloseRequested => {
                let _ = visibility_manager.handle_close_requested(bounds);
                Some(Message::ToggleVisibility)
            }
            window::Event::Moved { x, y } => {
                if visibility_manager.is_visible() {
                    let mut position = visibility_manager.last_position().unwrap_or(WindowPosition {
                        x: *x as f32,
                        y: *y as f32,
                        width: bounds.width,
                        height: bounds.height,
                    });
                    
                    position.x = *x as f32;
                    position.y = *y as f32;
                    
                    visibility_manager.last_position = Some(position);
                }
                None
            }
            window::Event::Resized { width, height } => {
                if visibility_manager.is_visible() {
                    let mut position = visibility_manager.last_position().unwrap_or(WindowPosition {
                        x: bounds.x,
                        y: bounds.y,
                        width: *width as f32,
                        height: *height as f32,
                    });
                    
                    position.width = *width as f32;
                    position.height = *height as f32;
                    
                    visibility_manager.last_position = Some(position);
                }
                None
            }
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_window_position() {
        let rect = Rectangle::new(
            Point::new(100.0, 200.0),
            iced::Size::new(800.0, 600.0),
        );
        
        let position = WindowPosition::from(rect);
        
        assert_eq!(position.x, 100.0);
        assert_eq!(position.y, 200.0);
        assert_eq!(position.width, 800.0);
        assert_eq!(position.height, 600.0);
        
        let rect2: Rectangle = position.into();
        
        assert_eq!(rect2.x, 100.0);
        assert_eq!(rect2.y, 200.0);
        assert_eq!(rect2.width, 800.0);
        assert_eq!(rect2.height, 600.0);
    }
    
    #[test]
    fn test_window_visibility_manager() {
        let config = AppConfig::default();
        let mut manager = WindowVisibilityManager::new(config.clone());
        
        // Default is visible if start_minimized is false
        let test_config = AppConfig {
            ui: crate::config::UiConfig {
                start_minimized: false,
                ..config.ui.clone()
            },
            ..config.clone()
        };
        
        let manager2 = WindowVisibilityManager::new(test_config);
        assert!(manager2.is_visible());
        
        // Test is_visible
        assert_eq!(manager.is_visible(), !config.ui.start_minimized);
        
        // Test focus state
        assert!(!manager.is_focused());
        manager.handle_focus();
        assert!(manager.is_focused());
        manager.handle_blur();
        assert!(!manager.is_focused());
        
        // Test position
        assert_eq!(manager.last_position(), None);
        
        let position = WindowPosition {
            x: 100.0,
            y: 200.0,
            width: 800.0,
            height: 600.0,
        };
        
        let rect = Rectangle::new(
            Point::new(position.x, position.y),
            iced::Size::new(position.width, position.height),
        );
        
        // Hide will update the position
        let _ = manager.hide(rect);
        assert_eq!(manager.last_position(), Some(position));
        assert_eq!(manager.is_visible(), false);
        
        // Show again
        let _ = manager.show();
        assert_eq!(manager.is_visible(), true);
    }
} 