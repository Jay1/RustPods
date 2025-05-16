#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::components::battery_display::BatteryDisplay;

    #[test]
    fn test_battery_display_creation() {
        // Create with known values
        let display = BatteryDisplay::new(Some(75), Some(80), Some(90));
        // Verify fields
        assert_eq!(display.left_level, Some(75));
        assert_eq!(display.right_level, Some(80));
        assert_eq!(display.case_level, Some(90));
        // Test value capping at 100
        let display = BatteryDisplay::new(Some(120), Some(80), Some(150));
        assert_eq!(display.left_level, Some(100));
        assert_eq!(display.right_level, Some(80));
        assert_eq!(display.case_level, Some(100));
        // Test empty creation
        let display = BatteryDisplay::empty();
        assert_eq!(display.left_level, None);
        assert_eq!(display.right_level, None);
        assert_eq!(display.case_level, None);
    }
} 