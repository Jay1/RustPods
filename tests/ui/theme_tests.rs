//! Integration tests for UI theme implementation

use iced::application;
use iced::widget::{button, container, text_input, progress_bar, rule, text};
use iced::{Background, Color};
use rustpods::config;
use rustpods::ui::theme::{
    Theme, BASE, TEXT, BLUE, GREEN, RED, YELLOW,
    badge_style, button_style, device_row_style, lavender_button_style, close_button_style,
    settings_button_style, secondary_button_style, settings_icon_color
};

/// Test that theme color constants are correctly defined
#[test]
fn test_theme_color_constants() {
    // Test that core color constants match expected values
    // Validate TEXT color properties
    assert!(
        TEXT.r > 0.5 || TEXT.g > 0.5 || TEXT.b > 0.5,
        "TEXT color should be light enough for readability"
    );

    // BASE should be a dark color for background
    assert!(
        BASE.r < 0.5 && BASE.g < 0.5 && BASE.b < 0.5,
        "BASE color should be dark for background"
    );

    // Verify BLUE for highlights
    assert!(BLUE.b > BLUE.r && BLUE.b > BLUE.g, "BLUE should have higher blue component");

    // Verify RED for errors/warnings
    assert!(RED.r > RED.g && RED.r > RED.b, "RED should have higher red component");

    // Verify GREEN for success indicators
    assert!(GREEN.g > GREEN.r && GREEN.g > GREEN.b, "GREEN should have higher green component");

    // Verify YELLOW for warnings/caution
    assert!(YELLOW.r > 0.5 && YELLOW.g > 0.5, "YELLOW should have high red and green components");

    // Colors should have sufficient contrast
    assert!(
        color_distance(TEXT, BASE) > 0.5,
        "TEXT and BASE colors should have sufficient contrast"
    );
}

/// Helper function to calculate color distance (simple Euclidean distance)
fn color_distance(c1: Color, c2: Color) -> f32 {
    let dr = c1.r - c2.r;
    let dg = c1.g - c2.g;
    let db = c1.b - c2.b;
    (dr * dr + dg * dg + db * db).sqrt()
}

/// Test that all themes are properly defined
#[test]
fn test_all_themes_defined() {
    // List of all theme variants
    let themes = [
        Theme::Light,
        Theme::Dark,
        Theme::System,
        Theme::CatppuccinMocha,
    ];

    // Test that each theme can provide styles for basic UI elements
    for theme in themes {
        // Test application style
        let app_style = <Theme as application::StyleSheet>::appearance(&theme, &());
        assert!(
            app_style.background_color != Color::TRANSPARENT,
            "Theme {:?} should provide a valid background color",
            theme
        );
        assert!(
            app_style.text_color != Color::TRANSPARENT,
            "Theme {:?} should provide a valid text color",
            theme
        );

        // Test button style for different variants
        let primary_button = <Theme as button::StyleSheet>::active(&theme, &iced::theme::Button::Primary);
        assert!(primary_button.background.is_some(), "Primary button should have a background");

        let secondary_button = <Theme as button::StyleSheet>::active(&theme, &iced::theme::Button::Secondary);
        assert!(secondary_button.text_color != Color::TRANSPARENT, "Secondary button should have a text color");

        // Test text input style
        let text_input_style = <Theme as text_input::StyleSheet>::active(&theme, &iced::theme::TextInput::Default);
        assert!(
            text_input_style.background != Background::Color(Color::TRANSPARENT),
            "Text input should have a background"
        );

        // Test container style - using default container
        let container_style = <Theme as container::StyleSheet>::appearance(&theme, &iced::theme::Container::default());
        assert!(
            container_style.text_color.is_some(),
            "Container should have a text color"
        );

        // Test text style
        let text_style = <Theme as text::StyleSheet>::appearance(&theme, Color::WHITE);
        assert_eq!(text_style.color, Some(Color::WHITE), "Text style should use the provided color");
    }
}

/// Test theme conversion from config Theme to UI Theme
#[test]
fn test_theme_conversion() {
    // Test conversion from config::Theme to ui::theme::Theme
    assert_eq!(Theme::from(config::Theme::Light), Theme::Light);
    assert_eq!(Theme::from(config::Theme::Dark), Theme::Dark);
    assert_eq!(Theme::from(config::Theme::System), Theme::System);
    
    // Test conversion from ui::theme::Theme to config::Theme
    assert_eq!(config::Theme::from(Theme::Light), config::Theme::Light);
    assert_eq!(config::Theme::from(Theme::Dark), config::Theme::Dark);
    assert_eq!(config::Theme::from(Theme::System), config::Theme::System);
    assert_eq!(config::Theme::from(Theme::CatppuccinMocha), config::Theme::System); // CatppuccinMocha maps to System in this implementation
}

/// Test theme hovered and pressed states
#[test]
fn test_theme_button_states() {
    // Test for each theme
    let themes = [Theme::Light, Theme::Dark, Theme::System, Theme::CatppuccinMocha];
    
    for theme in themes {
        // Test primary button states
        let primary = iced::theme::Button::Primary;
        let active = <Theme as button::StyleSheet>::active(&theme, &primary);
        let hovered = <Theme as button::StyleSheet>::hovered(&theme, &primary);
        let pressed = <Theme as button::StyleSheet>::pressed(&theme, &primary);
        
        // Ensure states are defined (simple checks)
        assert!(active.text_color != Color::TRANSPARENT, "Active button should have a text color");
        assert!(hovered.text_color != Color::TRANSPARENT, "Hovered button should have a text color");
        assert!(pressed.text_color != Color::TRANSPARENT, "Pressed button should have a text color");
    }
}

/// Test text input focus states
#[test]
fn test_text_input_states() {
    let themes = [Theme::Light, Theme::Dark, Theme::System, Theme::CatppuccinMocha];
    
    for theme in themes {
        let style = iced::theme::TextInput::Default;
        
        let active = <Theme as text_input::StyleSheet>::active(&theme, &style);
        let focused = <Theme as text_input::StyleSheet>::focused(&theme, &style);
        let disabled = <Theme as text_input::StyleSheet>::disabled(&theme, &style);
        
        // Focused should have different border color or width
        assert!(
            active.border_color != focused.border_color || active.border_width != focused.border_width,
            "Focused text input should have different border than active"
        );
        
        // Disabled should have different opacity or color
        assert!(
            active.background != disabled.background || active.border_color != disabled.border_color,
            "Disabled text input should look different from active"
        );
        
        // Test placeholder and value colors
        let placeholder_color = <Theme as text_input::StyleSheet>::placeholder_color(&theme, &style);
        let value_color = <Theme as text_input::StyleSheet>::value_color(&theme, &style);
        let selection_color = <Theme as text_input::StyleSheet>::selection_color(&theme, &style);
        
        assert!(
            color_distance(placeholder_color, value_color) > 0.1,
            "Placeholder color should differ from value color"
        );
        assert!(
            selection_color != Color::TRANSPARENT,
            "Selection color should be defined"
        );
    }
}

/// Test progress bar appearance
#[test]
fn test_progress_bar_appearance() {
    let themes = [Theme::Light, Theme::Dark, Theme::System, Theme::CatppuccinMocha];
    
    for theme in themes {
        // Use the default progress bar style
        let progress_style = &iced::theme::ProgressBar::default();
        let appearance = <Theme as progress_bar::StyleSheet>::appearance(&theme, progress_style);
        
        // Progress bar should have defined background and bar colors
        assert!(appearance.background != Background::Color(Color::TRANSPARENT), "Progress bar should have a background");
        assert!(appearance.bar != Background::Color(Color::TRANSPARENT), "Progress bar should have a bar color");
    }
}

/// Test horizontal rule appearance
#[test]
fn test_rule_appearance() {
    let themes = [Theme::Light, Theme::Dark, Theme::System, Theme::CatppuccinMocha];
    
    for theme in themes {
        // Use rule with explicit parameters
        let horizontal_rule = iced::theme::Rule::default();
        let appearance = <Theme as rule::StyleSheet>::appearance(&theme, &horizontal_rule);
        
        // Rule should have defined color and width
        assert!(appearance.color != Color::TRANSPARENT, "Rule should have a color");
        assert!(appearance.width > 0, "Rule should have positive width");
        
        // We can't easily test vertical vs horizontal here since they're created with functions
        // and not accessible via enum variants, but we can ensure the rule has valid properties
    }
}

/// Test helper style functions
#[test]
fn test_helper_style_functions() {
    // Test badge style with a sample color
    let badge = badge_style(RED);
    let badge_appearance = <Theme as container::StyleSheet>::appearance(&Theme::CatppuccinMocha, &badge);
    assert!(badge_appearance.border_radius.ne(&0.0.into()), "Badge should have rounded corners");
    
    // Test button style
    let btn = button_style();
    let btn_appearance = <Theme as button::StyleSheet>::active(&Theme::CatppuccinMocha, &btn);
    assert!(
        btn_appearance.background.is_some(), 
        "Button style should provide background"
    );
    
    // Test device row style
    let row = device_row_style();
    let row_appearance = <Theme as container::StyleSheet>::appearance(&Theme::CatppuccinMocha, &row);
    assert!(
        row_appearance.background.is_some(),
        "Device row should have background"
    );
    
    // Test lavender button style
    let lavender = lavender_button_style();
    let lavender_appearance = <Theme as button::StyleSheet>::active(&Theme::CatppuccinMocha, &lavender);
    assert!(
        lavender_appearance.text_color != Color::TRANSPARENT,
        "Lavender button should have text color"
    );
    
    // Test close button style
    let close = close_button_style();
    let close_appearance = <Theme as button::StyleSheet>::active(&Theme::CatppuccinMocha, &close);
    assert!(
        close_appearance.text_color != Color::TRANSPARENT,
        "Close button should have text color"
    );
    
    // Test settings button style
    let settings = settings_button_style();
    let settings_appearance = <Theme as button::StyleSheet>::active(&Theme::CatppuccinMocha, &settings);
    assert!(
        settings_appearance.text_color != Color::TRANSPARENT,
        "Settings button should have text color"
    );
    
    // Test secondary button style
    let secondary = secondary_button_style();
    let secondary_appearance = <Theme as button::StyleSheet>::active(&Theme::CatppuccinMocha, &secondary);
    assert!(
        secondary_appearance.border_radius.ne(&0.0.into()),
        "Secondary button should have border radius"
    );
    
    // Test settings icon color
    let icon_color = settings_icon_color(&Theme::CatppuccinMocha);
    assert!(
        icon_color != Color::TRANSPARENT,
        "Settings icon color should be defined"
    );
}

/// Test theme equality and copying
#[test]
fn test_theme_equality_and_copy() {
    // Themes should implement equality comparison
    assert_eq!(Theme::CatppuccinMocha, Theme::CatppuccinMocha);
    assert_ne!(Theme::Light, Theme::Dark);
    assert_ne!(Theme::System, Theme::CatppuccinMocha);
    
    // Themes should be copyable
    let theme1 = Theme::CatppuccinMocha;
    let theme2 = theme1;
    assert_eq!(theme1, theme2);
}

/// Test theme debug and display representation
#[test]
fn test_theme_representations() {
    // Test Debug representation
    let debug_str = format!("{:?}", Theme::CatppuccinMocha);
    assert!(
        debug_str.contains("CatppuccinMocha"),
        "Debug representation should contain theme name"
    );
    
    // Test Display representation
    let display_str = format!("{}", Theme::CatppuccinMocha);
    assert!(
        display_str.contains("Catppuccin Mocha"),
        "Display representation should contain formatted theme name"
    );
    
    let light_str = format!("{}", Theme::Light);
    assert!(
        light_str.contains("Light"),
        "Display representation should contain formatted theme name"
    );
}

/// Test custom font characteristics
#[test]
fn test_custom_font() {
    use rustpods::ui::theme::FONT_FAMILY;
    assert_eq!(FONT_FAMILY, "SpaceMono Nerd Font");
}
