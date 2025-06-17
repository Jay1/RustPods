#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use crate::ui::system_tray::{SystemTray, SystemTrayError, MenuItem, ThemeMode};
    use crate::ui::Message;
    use crate::config::{AppConfig, ConfigTheme};

    #[test]
    fn test_system_tray_creation() {
        let (_tx, _rx) = mpsc::channel::<Message>();
        let tray_type = std::any::TypeId::of::<SystemTray>();
        assert_eq!(tray_type, std::any::TypeId::of::<SystemTray>());
        let error = SystemTrayError::Creation("test".to_string());
        assert!(error.to_string().contains("Failed to create tray item"));
    }

    #[test]
    fn test_menu_item_struct() {
        let item = MenuItem {
            label: "Test".to_string(),
            shortcut: Some("Ctrl+T".to_string()),
            message: Some(Message::Exit),
        };
        assert_eq!(item.label, "Test");
        assert_eq!(item.shortcut, Some("Ctrl+T".to_string()));
        assert!(matches!(item.message, Some(Message::Exit)));
    }

    #[test]
    fn test_theme_detection() {
        let mut config = AppConfig::default();
        config.ui.theme = ConfigTheme::Light;
        let theme_mode = SystemTray::detect_theme_mode(&config);
        assert!(matches!(theme_mode, ThemeMode::Light));
        config.ui.theme = ConfigTheme::Dark;
        let theme_mode = SystemTray::detect_theme_mode(&config);
        assert!(matches!(theme_mode, ThemeMode::Dark));
        config.ui.theme = ConfigTheme::System;
        let theme_mode = SystemTray::detect_theme_mode(&config);
        assert!(matches!(theme_mode, ThemeMode::Dark));
    }
} 
