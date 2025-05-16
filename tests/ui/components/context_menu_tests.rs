#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::components::context_menu::{ContextMenu, ContextMenuItem};

    #[test]
    fn test_context_menu_item_creation() {
        let item = ContextMenuItem::new("refresh", "Refresh")
            .with_shortcut("Ctrl+R");
        assert_eq!(item.id, "refresh");
        assert_eq!(item.text, "Refresh");
        assert_eq!(item.shortcut, Some("Ctrl+R".to_string()));
        assert!(!item.disabled);
        assert!(!item.is_separator);
        let separator = ContextMenuItem::separator("sep1");
        assert!(separator.is_separator);
    }

    #[test]
    fn test_context_menu_creation() {
        let mut menu = ContextMenu::new();
        menu.add_item(ContextMenuItem::new("refresh", "Refresh"));
        menu.add_item(ContextMenuItem::separator("sep1"));
        menu.add_item(ContextMenuItem::new("exit", "Exit"));
        assert_eq!(menu.items.len(), 3);
        assert!(!menu.is_visible());
        menu.show();
        assert!(menu.is_visible());
        menu.hide();
        assert!(!menu.is_visible());
    }

    #[test]
    fn test_standard_menu() {
        let menu = ContextMenu::standard();
        // Should have several items including separators
        assert!(menu.items.len() > 5);
        // Check for expected standard items
        let has_exit = menu.items.iter().any(|item| item.id == "exit");
        let has_settings = menu.items.iter().any(|item| item.id == "settings");
        assert!(has_exit);
        assert!(has_settings);
    }
} 