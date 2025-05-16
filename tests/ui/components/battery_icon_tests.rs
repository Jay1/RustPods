#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::components::battery_icon::{battery_color, BLUE, RED, PEACH, GREEN, OVERLAY1};
    use iced::Color;

    #[test]
    fn test_battery_color() {
        // Test charging color - use the exact same logic as the function to verify
        let animation_progress = 0.0;
        let pulse = (1.0 + (animation_progress * 2.0 * std::f32::consts::PI).sin()) * 0.5;
        let base_color = BLUE;
        let highlight_color = Color::from_rgb(
            base_color.r * 1.2,
            base_color.g * 1.2,
            base_color.b * 1.2
        );
        let expected_charging_color = Color {
            r: base_color.r + (highlight_color.r - base_color.r) * pulse,
            g: base_color.g + (highlight_color.g - base_color.g) * pulse,
            b: base_color.b + (highlight_color.b - base_color.b) * pulse,
            a: 1.0,
        };
        let actual_charging_color = battery_color(Some(50), true, 0.0);
        assert_eq!(actual_charging_color, expected_charging_color);
        // Test low battery color
        let low_color = battery_color(Some(10), false, 0.0);
        assert_eq!(low_color, RED);
        // Test medium battery color
        let medium_color = battery_color(Some(40), false, 0.0);
        assert_eq!(medium_color, PEACH);
        // Test high battery color
        let high_color = battery_color(Some(80), false, 0.0);
        assert_eq!(high_color, GREEN);
        // Test unknown battery color
        let unknown_color = battery_color(None, false, 0.0);
        assert_eq!(unknown_color, OVERLAY1);
    }
} 