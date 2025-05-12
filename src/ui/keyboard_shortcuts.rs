//! Keyboard shortcuts handling for RustPods
//!
//! This module provides keyboard shortcut handling for the application,
//! including global shortcuts, keymaps, and shortcut configuration.

use iced::keyboard::{self, KeyCode, Modifiers};
use iced::Event;
use iced::event::{self};
use std::collections::HashMap;

use crate::ui::Message;

/// Represents a keyboard shortcut
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyboardShortcut {
    /// Key code for the shortcut
    pub key: KeyCode,
    /// Modifier keys for the shortcut (Ctrl, Shift, Alt, etc.)
    pub modifiers: Modifiers,
}

impl KeyboardShortcut {
    /// Create a new keyboard shortcut with a key and modifiers
    pub fn new(key: KeyCode, modifiers: Modifiers) -> Self {
        Self { key, modifiers }
    }
    
    /// Create a shortcut with Ctrl modifier
    pub fn ctrl(key: KeyCode) -> Self {
        Self::new(key, Modifiers::CTRL)
    }
    
    /// Create a shortcut with Shift modifier
    pub fn shift(key: KeyCode) -> Self {
        Self::new(key, Modifiers::SHIFT)
    }
    
    /// Create a shortcut with Alt modifier
    pub fn alt(key: KeyCode) -> Self {
        Self::new(key, Modifiers::ALT)
    }
    
    /// Create a shortcut with Ctrl+Shift modifiers
    pub fn ctrl_shift(key: KeyCode) -> Self {
        Self::new(key, Modifiers::CTRL | Modifiers::SHIFT)
    }
    
    /// Create a shortcut with Ctrl+Alt modifiers
    pub fn ctrl_alt(key: KeyCode) -> Self {
        Self::new(key, Modifiers::CTRL | Modifiers::ALT)
    }
    
    /// Check if the shortcut matches a keyboard event
    pub fn matches(&self, key_code: KeyCode, modifiers: Modifiers) -> bool {
        self.key == key_code && self.modifiers == modifiers
    }
    
    /// Get a human-readable representation of the shortcut
    pub fn to_string(&self) -> String {
        let mut parts = Vec::new();
        
        if self.modifiers.contains(Modifiers::CTRL) {
            parts.push("Ctrl".to_string());
        }
        
        if self.modifiers.contains(Modifiers::SHIFT) {
            parts.push("Shift".to_string());
        }
        
        if self.modifiers.contains(Modifiers::ALT) {
            parts.push("Alt".to_string());
        }
        
        if self.modifiers.contains(Modifiers::LOGO) {
            parts.push("Win".to_string());
        }
        
        let key_name = match self.key {
            KeyCode::F1 => "F1".to_string(),
            KeyCode::F2 => "F2".to_string(),
            KeyCode::F3 => "F3".to_string(),
            KeyCode::F4 => "F4".to_string(),
            KeyCode::F5 => "F5".to_string(),
            KeyCode::F6 => "F6".to_string(),
            KeyCode::F7 => "F7".to_string(),
            KeyCode::F8 => "F8".to_string(),
            KeyCode::F9 => "F9".to_string(),
            KeyCode::F10 => "F10".to_string(),
            KeyCode::F11 => "F11".to_string(),
            KeyCode::F12 => "F12".to_string(),
            KeyCode::Escape => "Esc".to_string(),
            KeyCode::Tab => "Tab".to_string(),
            KeyCode::Space => "Space".to_string(),
            KeyCode::Enter => "Enter".to_string(),
            KeyCode::Backspace => "Backspace".to_string(),
            KeyCode::Delete => "Delete".to_string(),
            KeyCode::Home => "Home".to_string(),
            KeyCode::End => "End".to_string(),
            KeyCode::PageUp => "Page Up".to_string(),
            KeyCode::PageDown => "Page Down".to_string(),
            KeyCode::Left => "←".to_string(),
            KeyCode::Right => "→".to_string(),
            KeyCode::Up => "↑".to_string(),
            KeyCode::Down => "↓".to_string(),
            _ => format!("{:?}", self.key),
        };
        
        parts.push(key_name);
        
        parts.join("+")
    }
}

/// Keyboard shortcut manager for the application
#[derive(Debug, Clone)]
pub struct KeyboardShortcutManager {
    /// Mapping of keyboard shortcuts to application messages
    shortcuts: HashMap<KeyboardShortcut, Message>,
}

impl Default for KeyboardShortcutManager {
    fn default() -> Self {
        let mut manager = Self::new();
        manager.register_default_shortcuts();
        manager
    }
}

impl KeyboardShortcutManager {
    /// Create a new empty keyboard shortcut manager
    pub fn new() -> Self {
        Self {
            shortcuts: HashMap::new(),
        }
    }
    
    /// Register a new keyboard shortcut for a message
    pub fn register(&mut self, shortcut: KeyboardShortcut, message: Message) {
        self.shortcuts.insert(shortcut, message);
    }
    
    /// Register the default keyboard shortcuts for the application
    pub fn register_default_shortcuts(&mut self) {
        // Main application shortcuts
        self.register(KeyboardShortcut::ctrl(KeyCode::R), Message::StartScan);
        self.register(KeyboardShortcut::ctrl(KeyCode::S), Message::StopScan);
        self.register(KeyboardShortcut::ctrl(KeyCode::Q), Message::Exit);
        self.register(KeyboardShortcut::ctrl(KeyCode::H), Message::ToggleVisibility);
        
        // Settings shortcuts
        self.register(KeyboardShortcut::ctrl(KeyCode::Comma), Message::OpenSettings);
        self.register(KeyboardShortcut::ctrl(KeyCode::Period), Message::CloseSettings);
        self.register(KeyboardShortcut::ctrl_shift(KeyCode::S), Message::SaveSettings);
    }
    
    /// Process keyboard events and generate corresponding messages
    pub fn handle_event(&self, event: &Event) -> Option<Message> {
        if let Event::Keyboard(keyboard::Event::KeyPressed { key_code, modifiers, .. }) = event {
            // Check if this key combination matches any registered shortcut
            for (shortcut, message) in &self.shortcuts {
                if shortcut.matches(*key_code, *modifiers) {
                    return Some(message.clone());
                }
            }
        }
        
        None
    }
    
    /// Get all registered shortcuts
    pub fn get_shortcuts(&self) -> &HashMap<KeyboardShortcut, Message> {
        &self.shortcuts
    }
    
    /// Get human-readable descriptions of all keyboard shortcuts
    pub fn get_shortcut_descriptions(&self) -> Vec<(String, String)> {
        let mut descriptions = Vec::new();
        
        for (shortcut, message) in &self.shortcuts {
            let description = match message {
                Message::StartScan => "Start scanning for devices",
                Message::StopScan => "Stop scanning for devices",
                Message::Exit => "Exit application",
                Message::ToggleVisibility => "Show/hide application window",
                Message::OpenSettings => "Open settings",
                Message::CloseSettings => "Close settings",
                Message::SaveSettings => "Save settings",
                _ => continue, // Skip messages without descriptions
            };
            
            descriptions.push((shortcut.to_string(), description.to_string()));
        }
        
        // Sort by description
        descriptions.sort_by(|a, b| a.1.cmp(&b.1));
        
        descriptions
    }
}

/// Process events from Iced and handle keyboard shortcuts
pub fn handle_events(
    event: Event,
    shortcut_manager: &KeyboardShortcutManager,
) -> Option<Message> {
    match event {
        Event::Keyboard(keyboard_event) => {
            match keyboard_event {
                keyboard::Event::KeyPressed { key_code, modifiers, .. } => {
                    // Check if this key combination matches any registered shortcut
                    for (shortcut, message) in shortcut_manager.get_shortcuts() {
                        if shortcut.matches(key_code, modifiers) {
                            return Some(message.clone());
                        }
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    
    None
}

/// Format key for display
pub fn format_key_for_display(key: KeyCode) -> String {
    match key {
        KeyCode::Space => "Space".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::Escape => "Esc".to_string(),
        KeyCode::Left => "←".to_string(),
        KeyCode::Right => "→".to_string(),
        KeyCode::Up => "↑".to_string(),
        KeyCode::Down => "↓".to_string(),
        // ... rest of the function ...
        _ => format!("{:?}", key),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
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