//! Tests for the user interaction components
//! This tests the user interaction functionality implemented in Task 10.3

use std::collections::HashMap;

use iced::{Event, keyboard, mouse, Rectangle, Point};
use iced::keyboard::{KeyCode, Modifiers};

use rustpods::ui::keyboard_shortcuts::{KeyboardShortcut, KeyboardShortcutManager, handle_events};
use rustpods::ui::window_management::{WindowInteraction, DragRegion};
use rustpods::ui::form_validation::{ValidationRule, FieldValidator, ValidationResult};
use rustpods::ui::Message;

#[test]
fn test_keyboard_shortcut_creation() {
    // Create a keyboard shortcut
    let shortcut = KeyboardShortcut::new(
        KeyCode::S, 
        Modifiers::CTRL
    );
    
    // Verify shortcut properties
    assert_eq!(shortcut.key, KeyCode::S);
    assert_eq!(shortcut.modifiers, Modifiers::CTRL);
    
    // Test matching with key code and modifiers directly
    let key_code = KeyCode::S;
    let modifiers = Modifiers::CTRL;
    
    assert!(shortcut.matches(key_code, modifiers));
    
    // Test non-matching
    let non_matching_modifiers = Modifiers::SHIFT;
    assert!(!shortcut.matches(key_code, non_matching_modifiers));
}

#[test]
fn test_keyboard_shortcut_manager() {
    // Create shortcuts
    let shortcut1 = KeyboardShortcut::new(KeyCode::S, Modifiers::CTRL);
    let shortcut2 = KeyboardShortcut::new(KeyCode::Escape, Modifiers::default());
    
    // Create manager
    let mut manager = KeyboardShortcutManager::new();
    
    // Register shortcuts with messages
    manager.register(shortcut1, Message::StartScan);
    manager.register(shortcut2, Message::Exit);
    
    // Should have 2 shortcuts
    assert_eq!(manager.get_shortcuts().len(), 2);
    
    // Test matching events using handle_event method
    let matching_event = Event::Keyboard(keyboard::Event::KeyPressed {
        key_code: KeyCode::S,
        modifiers: Modifiers::CTRL,
    });
    
    let message = manager.handle_event(&matching_event);
    assert!(message.is_some());
    assert!(matches!(message.unwrap(), Message::StartScan));
    
    // Test non-matching event
    let non_matching_event = Event::Keyboard(keyboard::Event::KeyPressed {
        key_code: KeyCode::A,
        modifiers: Modifiers::default(),
    });
    
    let message = manager.handle_event(&non_matching_event);
    assert!(message.is_none());
}

#[test]
fn test_handle_events_function() {
    // Create a shortcut manager
    let mut manager = KeyboardShortcutManager::new();
    
    // Register a test shortcut
    manager.register(
        KeyboardShortcut::new(KeyCode::S, Modifiers::CTRL),
        Message::StartScan
    );
    
    // Create event that doesn't match any shortcut
    let event = Event::Keyboard(keyboard::Event::KeyPressed {
        key_code: KeyCode::A,
        modifiers: Modifiers::default(),
    });
    
    // Should return None
    let result = handle_events(event, &manager);
    assert!(result.is_none());
    
    // Create event that matches our shortcut
    let matching_event = Event::Keyboard(keyboard::Event::KeyPressed {
        key_code: KeyCode::S,
        modifiers: Modifiers::CTRL,
    });
    
    // Should return Some(Message::StartScan)
    let result = handle_events(matching_event, &manager);
    assert!(result.is_some());
    assert!(matches!(result.unwrap(), Message::StartScan));
}

#[test]
fn test_window_drag_manager() {
    // Create drag manager
    let mut window_interaction = WindowInteraction::default();
    
    // Initially not dragging
    assert!(!window_interaction.dragging);
    
    // Start drag
    let drag_area = Rectangle {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 30.0,
    };
    
    let point = Point { x: 50.0, y: 15.0 };
    
    // Point is inside drag area, should start dragging
    window_interaction.handle_mouse_press(point, Point::new(0.0, 0.0), DragRegion::TitleBar);
    assert!(window_interaction.dragging);
    
    // Test drag release
    window_interaction.handle_mouse_release();
    assert!(!window_interaction.dragging);
}

#[test]
fn test_form_validation() {
    // Create validators
    let required_validator = FieldValidator::required("Field is required");
    
    // Test required field validator
    let result = required_validator.validate("");
    assert!(!result.is_valid());
    assert_eq!(result.error(), Some("Field is required"));
    
    let result = required_validator.validate("test");
    assert!(result.is_valid());
    assert_eq!(result.error(), None);
    
    // Create number range validator
    let range_validator = FieldValidator::number_range(1, 100, "Number must be between 1 and 100");
    
    // Test number range validator
    let result = range_validator.validate("0");
    assert!(!result.is_valid());
    assert_eq!(result.error(), Some("Number must be between 1 and 100"));
    
    let result = range_validator.validate("50");
    assert!(result.is_valid());
    assert_eq!(result.error(), None);
    
    let result = range_validator.validate("101");
    assert!(!result.is_valid());
    assert_eq!(result.error(), Some("Number must be between 1 and 100"));
    
    // Test non-numeric input
    let result = range_validator.validate("abc");
    assert!(!result.is_valid());
}

#[test]
fn test_combined_validators() {
    // Create validators
    let required_validator = FieldValidator::required("Field is required");
    let range_validator = FieldValidator::number_range(1, 100, "Number must be between 1 and 100");
    
    // Combine validators
    let combined_validator = FieldValidator::chain(vec![
        required_validator,
        range_validator,
    ]);
    
    // Test combined validator
    
    // Empty fails required check
    let result = combined_validator.validate("");
    assert!(!result.is_valid());
    assert_eq!(result.error(), Some("Field is required"));
    
    // Out of range fails range check
    let result = combined_validator.validate("0");
    assert!(!result.is_valid());
    assert_eq!(result.error(), Some("Number must be between 1 and 100"));
    
    // Valid passes both checks
    let result = combined_validator.validate("50");
    assert!(result.is_valid());
    assert_eq!(result.error(), None);
} 