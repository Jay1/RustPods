#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::window_visibility::{WindowPosition, WindowVisibilityManager};
    use iced::{Rectangle, Point};
    use crate::config::AppConfig;

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
