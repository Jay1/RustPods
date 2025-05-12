//! Tests for the user interaction components
//! This tests the user interaction functionality implemented in Task 10.3

use std::collections::HashMap;

use iced::{Event, keyboard, mouse, Rectangle, Point};
use iced::keyboard::{KeyCode, Modifiers};

use rustpods::ui::keyboard_shortcuts::{KeyboardShortcut, KeyboardShortcutManager, handle_events};
use rustpods::ui::window_management::{WindowInteraction, DragManager};
use rustpods::ui::form_validation::{Validator, ValidationResult, FieldValidator};
use rustpods::ui::Message;

#[test]
fn test_keyboard_shortcut_creation() {
    // Create a keyboard shortcut
    let shortcut = KeyboardShortcut::new(
        KeyCode::S, 
        Modifiers {
            control: true,
            shift: false,
            alt: false,
            ..Default::default()
        },
        Message::StartScan
    );
    
    // Verify shortcut properties
    assert_eq!(shortcut.key, KeyCode::S);
    assert!(shortcut.modifiers.control);
    assert!(!shortcut.modifiers.shift);
    assert!(!shortcut.modifiers.alt);
    
    // Shortcut should match with Ctrl+S
    let matching_event = Event::Keyboard(keyboard::Event::KeyPressed {
        key: KeyCode::S,
        modifiers: Modifiers {
            control: true,
            shift: false,
            alt: false,
            ..Default::default()
        },
    });
    
    let non_matching_event = Event::Keyboard(keyboard::Event::KeyPressed {
        key: KeyCode::S,
        modifiers: Modifiers {
            control: false,
            shift: true,
            alt: false,
            ..Default::default()
        },
    });
    
    assert!(shortcut.matches(&matching_event));
    assert!(!shortcut.matches(&non_matching_event));
}

#[test]
fn test_keyboard_shortcut_manager() {
    // Create shortcuts
    let shortcut1 = KeyboardShortcut::new(
        KeyCode::S, 
        Modifiers {
            control: true,
            ..Default::default()
        },
        Message::StartScan
    );
    
    let shortcut2 = KeyboardShortcut::new(
        KeyCode::Escape, 
        Modifiers::default(),
        Message::Exit
    );
    
    // Create manager
    let mut manager = KeyboardShortcutManager::new();
    
    // Register shortcuts
    manager.register(shortcut1);
    manager.register(shortcut2);
    
    // Should have 2 shortcuts
    assert_eq!(manager.shortcuts().len(), 2);
    
    // Test matching events
    let matching_event = Event::Keyboard(keyboard::Event::KeyPressed {
        key: KeyCode::S,
        modifiers: Modifiers {
            control: true,
            ..Default::default()
        },
    });
    
    let message = manager.process_event(&matching_event);
    assert!(message.is_some());
    assert!(matches!(message.unwrap(), Message::StartScan));
    
    // Test non-matching event
    let non_matching_event = Event::Keyboard(keyboard::Event::KeyPressed {
        key: KeyCode::A,
        modifiers: Modifiers::default(),
    });
    
    let message = manager.process_event(&non_matching_event);
    assert!(message.is_none());
}

#[test]
fn test_handle_events_function() {
    // Create event that doesn't match any shortcut
    let event = Event::Keyboard(keyboard::Event::KeyPressed {
        key: KeyCode::A,
        modifiers: Modifiers::default(),
    });
    
    // Should return None
    let result = handle_events(&event);
    assert!(result.is_none());
}

#[test]
fn test_window_drag_manager() {
    // Create drag manager
    let mut drag_manager = DragManager::default();
    
    // Initially not dragging
    assert!(!drag_manager.is_dragging());
    
    // Start drag
    let drag_area = Rectangle {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 30.0,
    };
    
    let point = Point { x: 50.0, y: 15.0 };
    
    // Point is inside drag area, should start dragging
    let interaction = drag_manager.process_press(drag_area, point);
    assert!(matches!(interaction, WindowInteraction::Drag { .. }));
    assert!(drag_manager.is_dragging());
    
    // Test point outside drag area
    let outside_point = Point { x: 150.0, y: 15.0 };
    drag_manager = DragManager::default(); // Reset
    
    let interaction = drag_manager.process_press(drag_area, outside_point);
    assert!(matches!(interaction, WindowInteraction::None));
    assert!(!drag_manager.is_dragging());
    
    // Test drag release
    drag_manager = DragManager::default();
    let _ = drag_manager.process_press(drag_area, point);
    assert!(drag_manager.is_dragging());
    
    drag_manager.process_release();
    assert!(!drag_manager.is_dragging());
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