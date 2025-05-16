#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::keyboard_shortcuts::{KeyboardShortcut, KeyCode, Modifiers, KeyboardShortcutManager};
    use crate::ui::Message;

    #[test]
    fn test_keyboard_shortcut_creation() {
        let shortcut = KeyboardShortcut::ctrl(KeyCode::S);
        assert_eq!(shortcut.key, KeyCode::S);
        assert_eq!(shortcut.modifiers, Modifiers::CTRL);
        let shortcut_alt = KeyboardShortcut::alt(KeyCode::F4);
        assert_eq!(shortcut_alt.key, KeyCode::F4);
        assert_eq!(shortcut_alt.modifiers, Modifiers::ALT);
    }

    #[test]
    fn test_matches() {
        let shortcut = KeyboardShortcut::ctrl_shift(KeyCode::S);
        assert!(shortcut.matches(KeyCode::S, Modifiers::CTRL | Modifiers::SHIFT));
        assert!(!shortcut.matches(KeyCode::S, Modifiers::CTRL));
        assert!(!shortcut.matches(KeyCode::A, Modifiers::CTRL | Modifiers::SHIFT));
    }

    #[test]
    fn test_to_string() {
        let shortcut = KeyboardShortcut::ctrl(KeyCode::S);
        assert_eq!(shortcut.to_string(), "Ctrl+S");
        let shortcut2 = KeyboardShortcut::ctrl_shift(KeyCode::F5);
        assert_eq!(shortcut2.to_string(), "Ctrl+Shift+F5");
    }

    #[test]
    fn test_shortcut_manager() {
        let mut manager = KeyboardShortcutManager::new();
        manager.register(
            KeyboardShortcut::ctrl(KeyCode::S),
            Message::SaveSettings,
        );
        let shortcuts = manager.get_shortcuts();
        assert_eq!(shortcuts.len(), 1);
        let descriptions = manager.get_shortcut_descriptions();
        assert_eq!(descriptions.len(), 1);
        assert_eq!(descriptions[0].0, "Ctrl+S");
    }
} 