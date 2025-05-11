//! Integration tests for UI theme implementation

use rustpods::ui::theme::{Theme, TEXT, BASE, ACCENT};
use rustpods::ui::state::AppState;
use iced::application;
use iced::button;
use iced::container;
use iced::Color;

/// Test that theme color constants are correctly defined
#[test]
fn test_theme_color_constants() {
    // Test that core color constants match expected values
    // The exact expected values should match your actual implementation
    
    // TEXT should be a light color for readability
    assert!(TEXT.r > 0.5 || TEXT.g > 0.5 || TEXT.b > 0.5, 
            "TEXT color should be light enough for readability");
    
    // BASE should be a dark color for background
    assert!(BASE.r < 0.5 && BASE.g < 0.5 && BASE.b < 0.5, 
            "BASE color should be dark for background");
    
    // ACCENT should be a vibrant color
    let accent_intensity = ACCENT.r + ACCENT.g + ACCENT.b;
    assert!(accent_intensity > 0.5, 
            "ACCENT color should be vibrant enough");
    
    // Colors should be distinct from each other
    assert!(color_distance(TEXT, BASE) > 0.5, 
            "TEXT and BASE colors should have sufficient contrast");
    assert!(color_distance(ACCENT, BASE) > 0.3, 
            "ACCENT and BASE colors should have sufficient contrast");
}

/// Helper function to calculate color distance (simple Euclidean distance)
fn color_distance(c1: Color, c2: Color) -> f32 {
    let dr = c1.r - c2.r;
    let dg = c1.g - c2.g;
    let db = c1.b - c2.b;
    (dr*dr + dg*dg + db*db).sqrt()
}

/// Test that all themes are properly defined
#[test]
fn test_all_themes_defined() {
    // List of expected theme variants
    let themes = [
        Theme::CatppuccinMocha,
        Theme::CatppuccinLatte,
        Theme::TokyoNight,
        Theme::Nord,
        Theme::Dracula,
        // Add other themes as needed
    ];
    
    // Test that each theme can provide styles for basic UI elements
    for theme in themes {
        // Test application style
        let app_style = <Theme as application::StyleSheet>::appearance(&theme, &());
        assert!(app_style.background_color != Color::TRANSPARENT, 
                "Theme {:?} should provide a valid background color", theme);
        
        // Test button style
        let button_style = <Theme as button::StyleSheet>::appearance(
            &theme, &button::Appearance::default()
        );
        assert!(button_style.background != Color::TRANSPARENT, 
                "Theme {:?} should provide a valid button background", theme);
        
        // Test container style
        let container_style = <Theme as container::StyleSheet>::appearance(&theme, &());
        assert!(container_style.border_width >= 0.0, 
                "Theme {:?} should provide a valid container border width", theme);
    }
}

/// Test theme parsing from string
#[test]
fn test_theme_from_string() {
    // Test valid theme names
    assert_eq!(Theme::from_string("CatppuccinMocha"), Theme::CatppuccinMocha);
    assert_eq!(Theme::from_string("catppuccinmocha"), Theme::CatppuccinMocha);
    assert_eq!(Theme::from_string("CATPPUCCINMOCHA"), Theme::CatppuccinMocha);
    
    // Test invalid theme name falls back to default
    assert_eq!(Theme::from_string("InvalidTheme"), Theme::default());
    assert_eq!(Theme::from_string(""), Theme::default());
}

/// Test theme to string conversion
#[test]
fn test_theme_to_string() {
    // Test all themes convert to proper string representations
    assert_eq!(Theme::CatppuccinMocha.to_string(), "CatppuccinMocha");
    assert_eq!(Theme::CatppuccinLatte.to_string(), "CatppuccinLatte");
    // Add assertions for other themes
    
    // Also verify round-trip conversion
    for theme in [Theme::CatppuccinMocha, Theme::CatppuccinLatte, Theme::TokyoNight] {
        let theme_string = theme.to_string();
        let converted_back = Theme::from_string(&theme_string);
        assert_eq!(theme, converted_back, 
                  "Theme round-trip conversion failed for {:?}", theme);
    }
}

/// Test that AppState initializes with default theme
#[test]
fn test_app_state_default_theme() {
    let state = AppState::default();
    assert_eq!(state.theme(), Theme::default());
}

/// Test theme switching in AppState
#[test]
fn test_app_state_theme_switching() {
    let mut state = AppState::default();
    
    // Get current theme
    let initial_theme = state.theme();
    
    // Find a different theme to switch to
    let new_theme = if initial_theme == Theme::CatppuccinMocha {
        Theme::TokyoNight
    } else {
        Theme::CatppuccinMocha
    };
    
    // Change theme
    state.set_theme(new_theme);
    
    // Verify theme changed
    assert_eq!(state.theme(), new_theme);
    assert_ne!(state.theme(), initial_theme);
}

/// Test theme application on specific UI components
#[test]
fn test_theme_application_to_components() {
    let themes = [Theme::CatppuccinMocha, Theme::CatppuccinLatte];
    
    for theme in themes {
        // Test button styling
        let normal_button = theme.button(&button::Style::Primary);
        let danger_button = theme.button(&button::Style::Destructive);
        
        // Different styles should result in different appearances
        assert_ne!(
            normal_button.background, 
            danger_button.background,
            "Primary and Destructive buttons should have different styling for theme {:?}", 
            theme
        );
        
        // Test container styling
        let normal_container = theme.container(&container::Style::default());
        assert!(normal_container.text_color.is_some(), 
                "Container should have text color for theme {:?}", theme);
    }
}

/// Test theme equality and copying
#[test]
fn test_theme_equality_and_copy() {
    // Themes should implement equality comparison
    assert_eq!(Theme::CatppuccinMocha, Theme::CatppuccinMocha);
    assert_ne!(Theme::CatppuccinMocha, Theme::CatppuccinLatte);
    
    // Themes should be copyable
    let theme1 = Theme::CatppuccinMocha;
    let theme2 = theme1;
    assert_eq!(theme1, theme2);
}

/// Test theme debug representation
#[test]
fn test_theme_debug() {
    let debug_str = format!("{:?}", Theme::CatppuccinMocha);
    assert!(debug_str.contains("CatppuccinMocha"), 
            "Debug representation should contain theme name");
}

/// Test all available themes are distinct
#[test]
fn test_themes_are_distinct() {
    // Test that each theme produces different styles
    let themes = [
        Theme::CatppuccinMocha,
        Theme::CatppuccinLatte,
        Theme::TokyoNight,
        Theme::Nord,
        Theme::Dracula,
    ];
    
    // Compare application background colors between themes
    let backgrounds: Vec<Color> = themes.iter()
        .map(|t| <Theme as application::StyleSheet>::appearance(t, &()).background_color)
        .collect();
    
    // Check that each theme has a unique background color
    for (i, bg1) in backgrounds.iter().enumerate() {
        for (j, bg2) in backgrounds.iter().enumerate() {
            if i != j {
                assert!(*bg1 != *bg2, 
                        "Themes {:?} and {:?} have the same background color", 
                        themes[i], themes[j]);
            }
        }
    }
} 