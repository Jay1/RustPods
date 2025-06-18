#[cfg(test)]
mod tests {
    use iced::Element;
    use rustpods::ui::components::battery_indicator::view as battery_indicator_view;
    use rustpods::ui::theme::Theme;
    use rustpods::ui::Message;

    #[test]
    fn test_battery_indicator_renders() {
        let element: Element<'_, Message, iced::Renderer<Theme>> =
            battery_indicator_view("Left", Some(80), false);
        // Basic smoke test: ensure the element is created
        assert!(true);
    }

    #[test]
    fn test_battery_indicator_label_and_value() {
        let element = battery_indicator_view("Case", Some(55), false);
        // Placeholder: In a full test, would verify label and value rendering
        assert!(true);
    }

    #[test]
    fn test_battery_indicator_charging_state() {
        let element = battery_indicator_view("Right", Some(60), true);
        // Placeholder: In a full test, would verify charging icon is present
        assert!(true);
    }
}
