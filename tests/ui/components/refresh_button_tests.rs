#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::components::refresh_button::RefreshButton;

    #[test]
    fn test_refresh_button_creation() {
        // Test button in scanning state
        let scanning_button = RefreshButton::new();
        assert!(!scanning_button.is_loading);
        assert_eq!(scanning_button.animation_progress, 0.0);
        // Test with animation progress
        let animated_button = RefreshButton::new()
            .with_animation_progress(0.5);
        assert!(!animated_button.is_loading);
        assert_eq!(animated_button.animation_progress, 0.5);
    }
} 