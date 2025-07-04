#[cfg(test)]
mod tests {
    // Tests are organized in this module
    use rustpods::ui::components::{battery_icon_svg_string, refresh_icon_svg_string};

    #[test]
    fn test_refresh_icon_generation() {
        let svg = refresh_icon_svg_string(false, 0.0);
        assert!(svg.contains("viewBox=\"0 0 24 24\""));
        assert!(svg.contains("stroke=\"currentColor\""));
        // Test animated version
        let animated_svg = refresh_icon_svg_string(true, 0.5);
        assert!(animated_svg.contains("transform=\"rotate(180.0 12 12)\""));
    }

    #[test]
    fn test_battery_icon_generation() {
        // Test empty battery
        let empty_svg = battery_icon_svg_string(0.0, false);
        assert!(empty_svg.contains("viewBox=\"0 0 16 24\""));
        assert!(!empty_svg.contains("<rect")); // No fill rect for empty battery
                                               // Test full battery
        let full_svg = battery_icon_svg_string(1.0, false);
        assert!(full_svg.contains("<rect"));
        // Test charging
        let charging_svg = battery_icon_svg_string(0.5, true);
        assert!(charging_svg.contains("<path d=\"M9 10L7 14H9L7 18L11 13H8.5L10 10H9Z\""));
    }

    #[test]
    fn test_settings_icon_contrast_color() {
        // TODO: Implement a test that renders the settings icon with a contrasting color (e.g., LAVENDER)
        // and checks that the SVG string contains the correct color hex.
        // For now, this is a stub.
        assert!(true);
    }
}
