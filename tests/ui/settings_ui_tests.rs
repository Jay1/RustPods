#[test]
fn test_save_button_always_clickable_and_toast() {
    // TODO: Implement a test that checks the save button is always enabled
    // and that the correct toast is shown when there are no changes to save.
    // For now, this is a stub.
    assert!(true);
}

#[test]
fn test_save_button_logic() {
    use rustpods::ui::settings_window::SettingsWindow;
    use rustpods::config::AppConfig;
    use rustpods::ui::Message;
    use iced::Element;
    use rustpods::ui::theme::Theme;

    let mut window = SettingsWindow::new(AppConfig::default());
    // No changes: should trigger ShowToast
    let save_button = window.action_buttons();
    // (In a real test, simulate button press and check Message)
    // TODO: Simulate button press and assert correct Message variant

    // Mark changes and test again
    window.mark_changed();
    let save_button = window.action_buttons();
    // TODO: Simulate button press and assert correct Message variant
} 