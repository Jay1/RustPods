#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::window_management::{WindowInteraction, DragRegion};
    use iced::Point;

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
