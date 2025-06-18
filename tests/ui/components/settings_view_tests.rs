#[cfg(test)]
mod tests {
    use rustpods::config::AppConfig;
    use rustpods::ui::components::settings_view::{SettingsView, UiSetting};

    #[test]
    fn test_settings_view_construction() {
        let config = AppConfig::default();
        let _view = SettingsView::new(config);
        // Construction should not panic
        assert!(true);
    }

    #[test]
    fn test_system_settings_render() {
        let config = AppConfig::default();
        let view = SettingsView::new(config);
        let _element = view.system_settings();
        // Placeholder: In a full test, would verify element structure
        assert!(true);
    }

    #[test]
    fn test_ui_setting_enum() {
        let theme_setting = UiSetting::Theme(rustpods::ui::theme::Theme::CatppuccinMocha);
        let show_notifications = UiSetting::ShowNotifications(true);
        let start_minimized = UiSetting::StartMinimized(false);
        assert!(matches!(theme_setting, UiSetting::Theme(_)));
        assert!(matches!(
            show_notifications,
            UiSetting::ShowNotifications(_)
        ));
        assert!(matches!(start_minimized, UiSetting::StartMinimized(_)));
    }
}
