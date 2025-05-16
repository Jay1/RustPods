//! Window management functionality for RustPods
//!
//! This module handles window-related operations like dragging, positioning,
//! and maintaining window state.

use iced::{Event, Rectangle, Point, Vector, Length, mouse};
use iced::widget::container;

use crate::ui::Message;
use crate::config::AppConfig;
use crate::config::app_config::WindowPosition;

/// Regions of a window that can be dragged
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragRegion {
    /// The title bar of the window
    TitleBar,
    /// The header of a component
    Header,
    /// The entire window
    EntireWindow,
    /// No draggable region
    None,
}

/// Window interaction state
#[derive(Debug, Clone)]
pub struct WindowInteraction {
    /// Whether the window is currently being dragged
    pub dragging: bool,
    /// The mouse position where dragging started
    pub drag_start: Option<Point>,
    /// The original window position before dragging
    pub window_start_position: Option<Point>,
    /// The region that was clicked to start dragging
    pub drag_region: DragRegion,
    /// The last window position
    pub last_window_position: Option<Point>,
}

impl Default for WindowInteraction {
    fn default() -> Self {
        Self {
            dragging: false,
            drag_start: None,
            window_start_position: None,
            drag_region: DragRegion::None,
            last_window_position: None,
        }
    }
}

impl WindowInteraction {
    /// Create a new window interaction state
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Handle a mouse press event to potentially start dragging
    pub fn handle_mouse_press(
        &mut self,
        position: Point,
        window_position: Point,
        region: DragRegion,
    ) {
        if region != DragRegion::None {
            self.dragging = true;
            self.drag_start = Some(position);
            self.window_start_position = Some(window_position);
            self.drag_region = region;
        }
    }
    
    /// Handle a mouse release event to stop dragging
    pub fn handle_mouse_release(&mut self) {
        self.dragging = false;
        self.drag_start = None;
    }
    
    /// Calculate the new window position based on mouse movement
    pub fn calculate_window_position(&self, current_position: Point) -> Option<Point> {
        if !self.dragging || self.drag_start.is_none() || self.window_start_position.is_none() {
            return None;
        }
        
        let drag_start = self.drag_start.unwrap();
        let window_start = self.window_start_position.unwrap();
        
        let delta = Vector::new(
            current_position.x - drag_start.x,
            current_position.y - drag_start.y,
        );
        
        Some(Point::new(
            window_start.x + delta.x,
            window_start.y + delta.y,
        ))
    }

    /// Update window position from settings
    pub fn update_from_config(&mut self, app_config: &AppConfig) {
        // If window position settings exist in UI config, use them
        if let Some(window_position) = app_config.ui.last_window_position {
            // Convert WindowPosition to Point
            let point = iced::Point::new(window_position.x as f32, window_position.y as f32);
            self.last_window_position = Some(point);
        }
    }
}

/// Process window-related events
pub fn handle_window_events(
    event: &Event,
    window_state: &mut WindowInteraction,
    bounds: &Rectangle,
    _app_config: &AppConfig,
) -> Option<Message> {
    match event {
        Event::Mouse(mouse_event) => match mouse_event {
            mouse::Event::ButtonPressed(mouse::Button::Left) => {
                // Use the position from the window's current state instead
                let window_position = Point::new(bounds.x, bounds.y);
                
                // Check if the click is in a draggable region (top 40px)
                let last_cursor_pos = window_state.last_window_position.unwrap_or(Point::new(0.0, 0.0));
                let is_in_title_bar = last_cursor_pos.y - bounds.y <= 40.0;
                
                if is_in_title_bar {
                    window_state.handle_mouse_press(
                        last_cursor_pos,
                        window_position,
                        DragRegion::TitleBar,
                    );
                }
                None
            }
            mouse::Event::ButtonReleased(mouse::Button::Left) => {
                window_state.handle_mouse_release();
                None
            }
            mouse::Event::CursorMoved { position } => {
                if window_state.dragging {
                    window_state.calculate_window_position(*position).map(Message::WindowMove)
                } else {
                    None
                }
            }
            _ => None,
        },
        _ => None,
    }
}

/// Create a draggable container widget
pub fn create_draggable<'a, Message>(
    content: impl Into<iced::Element<'a, Message>>,
    _drag_region: DragRegion,
) -> container::Container<'a, Message> {
    let content = content.into();
    
    container(content)
        .width(Length::Fill)
        .height(Length::Shrink)
}

/// Default window width
pub const DEFAULT_WINDOW_WIDTH: u32 = 800;
/// Default window height
pub const DEFAULT_WINDOW_HEIGHT: u32 = 600;

/// Create a drag region that allows the user to move the window
pub fn create_drag_region(title_bar_height: u16) -> iced::widget::Container<'static, crate::ui::Message> {
    use iced::widget::{container, Space};
    use iced::Length;
    
    container(Space::new(Length::Fill, Length::Fixed(title_bar_height.into())))
        .width(Length::Fill)
        .height(Length::Fixed(title_bar_height.into()))
}

/// Load saved window position and make sure it's on screen
pub fn load_window_position(
    app_config: &AppConfig,
) -> Option<Point> 
{
    // Use saved position if available
    match &app_config.ui.last_window_position {
        Some(pos) => {
            // Convert WindowPosition to Point
            let position = Point::new(pos.x as f32, pos.y as f32);
            Some(position)
        },
        None => {
            // Fallback to a sensible default position
            Some(Point::new(100.0, 100.0))
        }
    }
}

/// Save the window position to the application config
pub fn save_window_position(
    window_position: Option<iced::Point>,
    app_config: &mut AppConfig,
) -> Result<(), crate::config::ConfigError> {
    // Update config with new position
    if let Some(pos) = window_position {
        app_config.ui.last_window_position = Some(WindowPosition {
            x: pos.x as i32,
            y: pos.y as i32,
        });
    } else {
        app_config.ui.last_window_position = None;
    }
    
    // Save config
    app_config.save()?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_window_interaction_creation() {
        let interaction = WindowInteraction::new();
        
        assert!(!interaction.dragging);
        assert_eq!(interaction.drag_start, None);
        assert_eq!(interaction.window_start_position, None);
        assert_eq!(interaction.drag_region, DragRegion::None);
    }
    
    #[test]
    fn test_mouse_press() {
        let mut interaction = WindowInteraction::new();
        
        interaction.handle_mouse_press(
            Point::new(100.0, 100.0),
            Point::new(0.0, 0.0),
            DragRegion::TitleBar,
        );
        
        assert!(interaction.dragging);
        assert_eq!(interaction.drag_start, Some(Point::new(100.0, 100.0)));
        assert_eq!(interaction.window_start_position, Some(Point::new(0.0, 0.0)));
        assert_eq!(interaction.drag_region, DragRegion::TitleBar);
    }
    
    #[test]
    fn test_mouse_release() {
        let mut interaction = WindowInteraction::new();
        
        interaction.handle_mouse_press(
            Point::new(100.0, 100.0),
            Point::new(0.0, 0.0),
            DragRegion::TitleBar,
        );
        
        interaction.handle_mouse_release();
        
        assert!(!interaction.dragging);
        assert_eq!(interaction.drag_start, None);
    }
    
    #[test]
    fn test_calculate_window_position() {
        let mut interaction = WindowInteraction::new();
        
        interaction.handle_mouse_press(
            Point::new(100.0, 100.0),
            Point::new(0.0, 0.0),
            DragRegion::TitleBar,
        );
        
        let new_position = interaction.calculate_window_position(Point::new(150.0, 120.0));
        
        assert_eq!(new_position, Some(Point::new(50.0, 20.0)));
    }
} 