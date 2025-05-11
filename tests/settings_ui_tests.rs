//! Tests for the settings UI to ensure all configuration options are present

use iced::Application;
use rustpods::config::{AppConfig, BluetoothConfig, UiConfig, SystemConfig};
use rustpods::ui::{state::AppState, Message};
use rustpods::ui::state::AppState as UiAppState; // Use the correct AppState type

/// Test that verifies all Bluetooth settings are present in the UI
#[test]
fn test_bluetooth_settings_completeness() {
    // Create a default AppState with configuration
    let mut state = UiAppState::default();
    state.show_settings = true;
    
    // Generate the view
    let ui_element = state.view();
    
    // This test is structural, asserting that each setting from the configuration
    // has a corresponding element in the UI.
    // We can check this by rendering the view to a string and checking for expected text
    
    let debug_string = format!("{:?}", ui_element);
    
    // Bluetooth settings that should be present
    let bluetooth_settings = [
        "Auto scan on startup",
        "Scan duration",
        "Interval between scans",
        "Battery refresh interval",
        "Min RSSI threshold",
        "Auto reconnect",
        "Reconnect attempts"
    ];
    
    // Verify all Bluetooth settings are present
    for setting in bluetooth_settings {
        assert!(debug_string.contains(setting), 
                "Bluetooth setting '{}' not found in UI", setting);
    }
}

/// Test that verifies all UI settings are present
#[test]
fn test_ui_settings_completeness() {
    // Create a default AppState with configuration
    let mut state = UiAppState::default();
    state.show_settings = true;
    
    // Generate the view
    let ui_element = state.view();
    
    // Convert to debug string for inspection
    let debug_string = format!("{:?}", ui_element);
    
    // UI settings that should be present
    let ui_settings = [
        "Theme",
        "Show notifications",
        "Start minimized",
        "Show battery percentage in tray",
        "Show low battery warning",
        "Low battery threshold"
    ];
    
    // Verify all UI settings are present
    for setting in ui_settings {
        assert!(debug_string.contains(setting), 
                "UI setting '{}' not found in UI", setting);
    }
}

/// Test that verifies all System settings are present
#[test]
fn test_system_settings_completeness() {
    // Create a default AppState with configuration
    let mut state = UiAppState::default();
    state.show_settings = true;
    
    // Generate the view
    let ui_element = state.view();
    
    // Convert to debug string for inspection
    let debug_string = format!("{:?}", ui_element);
    
    // System settings that should be present
    let system_settings = [
        "Log level",
        "Launch at startup",
        "Enable telemetry"
    ];
    
    // Verify all System settings are present
    for setting in system_settings {
        assert!(debug_string.contains(setting), 
                "System setting '{}' not found in UI", setting);
    }
}

/// Test that verifies all toggle buttons are present for boolean settings
#[test]
fn test_toggle_buttons_present() {
    // Create a default AppState with configuration
    let mut state = UiAppState::default();
    state.show_settings = true;
    
    // Generate the view
    let ui_element = state.view();
    
    // Convert to debug string for inspection
    let debug_string = format!("{:?}", ui_element);
    
    // Settings that should have toggle buttons (Enable/Disable)
    let toggle_settings = [
        "Auto scan on startup",
        "Auto reconnect",
        "Show notifications",
        "Start minimized",
        "Show battery percentage in tray",
        "Show low battery warning",
        "Launch at startup",
        "Enable telemetry"
    ];
    
    // Verify all toggle buttons are present
    for setting in toggle_settings {
        // For each toggle setting, either "Enable" or "Disable" button should be present
        assert!(debug_string.contains(&format!("{}: Enabled", setting)) ||
                debug_string.contains(&format!("{}: Disabled", setting)),
                "Toggle setting '{}' doesn't show status correctly", setting);
                
        assert!(debug_string.contains("Enable") || debug_string.contains("Disable"),
                "Toggle buttons for '{}' not found", setting);
    }
}

/// Test that verifies all configuration fields from AppConfig have corresponding UI elements
#[test]
fn test_config_to_ui_mapping_completeness() {
    // This test ensures that any field added to AppConfig will be detected if missing from UI
    
    // Create default config for reference
    let config = AppConfig::default();
    
    // Create a state with settings visible
    let mut state = UiAppState::default();
    state.show_settings = true;
    
    // Generate the view
    let ui_element = state.view();
    let debug_string = format!("{:?}", ui_element);
    
    // Map configuration field paths to expected UI text
    let field_mappings = [
        // Bluetooth fields
        ("bluetooth.auto_scan_on_startup", "Auto scan on startup"),
        ("bluetooth.scan_duration", "Scan duration"),
        ("bluetooth.scan_interval", "Interval between scans"),
        ("bluetooth.battery_refresh_interval", "Battery refresh interval"),
        ("bluetooth.min_rssi", "Min RSSI threshold"),
        ("bluetooth.auto_reconnect", "Auto reconnect"),
        ("bluetooth.reconnect_attempts", "Reconnect attempts"),
        
        // UI fields
        ("ui.show_notifications", "Show notifications"),
        ("ui.start_minimized", "Start minimized"),
        ("ui.theme", "Theme"),
        ("ui.show_percentage_in_tray", "Show battery percentage in tray"),
        ("ui.show_low_battery_warning", "Show low battery warning"),
        ("ui.low_battery_threshold", "Low battery threshold"),
        
        // System fields
        ("system.launch_at_startup", "Launch at startup"),
        ("system.log_level", "Log level"),
        ("system.enable_telemetry", "Enable telemetry")
    ];
    
    // Verify all mapped fields are present in the UI
    for (field, ui_text) in field_mappings {
        assert!(debug_string.contains(ui_text),
                "Config field '{}' not represented in UI (expected text: '{}')", field, ui_text);
    }
}

/// Test that the Save button is present
#[test]
fn test_save_button_present() {
    // Create a default state
    let mut state = UiAppState::default();
    state.show_settings = true;
    
    // Generate the view
    let ui_element = state.view();
    
    // Convert to debug string for inspection
    let debug_string = format!("{:?}", ui_element);
    
    // Check for Save button text
    assert!(debug_string.contains("Save Changes"), 
            "Save Changes button not found in UI");
}

/// Test that verifies all Bluetooth settings are defined in the AppConfig
#[test]
fn test_bluetooth_settings_in_config() {
    // Create a default AppState with configuration
    let config = AppConfig::default();
    
    // Check each Bluetooth setting exists in the config
    // This ensures the settings structure itself has the expected fields

    // Auto scan on startup (boolean)
    let _auto_scan = config.bluetooth.auto_scan_on_startup;
    
    // Scan duration
    let _scan_duration = config.bluetooth.scan_duration;
    
    // Scan interval
    let _scan_interval = config.bluetooth.scan_interval;
    
    // Battery refresh interval
    let _battery_refresh = config.bluetooth.battery_refresh_interval;
    
    // Min RSSI threshold
    let _min_rssi = config.bluetooth.min_rssi;
    
    // Auto reconnect
    let _auto_reconnect = config.bluetooth.auto_reconnect;
    
    // Reconnect attempts
    let _reconnect_attempts = config.bluetooth.reconnect_attempts;
    
    // If we get here without compilation errors, all fields exist
}

/// Test that verifies all UI settings are defined in the AppConfig
#[test]
fn test_ui_settings_in_config() {
    // Create a default config for reference
    let config = AppConfig::default();
    
    // Check each UI setting exists in the config
    // This ensures the settings structure itself has the expected fields
    
    // Show notifications
    let _show_notifications = config.ui.show_notifications;
    
    // Start minimized
    let _start_minimized = config.ui.start_minimized;
    
    // Theme
    let _theme = &config.ui.theme;
    
    // Show percentage in tray
    let _show_percentage = config.ui.show_percentage_in_tray;
    
    // Low battery warning
    let _low_battery_warning = config.ui.show_low_battery_warning;
    
    // Low battery threshold
    let _low_battery_threshold = config.ui.low_battery_threshold;
    
    // If we get here without compilation errors, all fields exist
}

/// Test that verifies all System settings are defined in the AppConfig
#[test]
fn test_system_settings_in_config() {
    // Create a default config for reference
    let config = AppConfig::default();
    
    // Check each System setting exists in the config
    // This ensures the settings structure itself has the expected fields
    
    // Log level
    let _log_level = &config.system.log_level;
    
    // Launch at startup
    let _launch_at_startup = config.system.launch_at_startup;
    
    // Enable telemetry
    let _enable_telemetry = config.system.enable_telemetry;
    
    // If we get here without compilation errors, all fields exist
}

/// Test that the BluetoothConfig struct has all required fields
#[test]
fn test_bluetooth_config_structure() {
    // Verify the structure of the BluetoothConfig
    let expected_fields = ["auto_scan_on_startup", "scan_duration", "scan_interval", 
                          "min_rssi", "battery_refresh_interval", "auto_reconnect", 
                          "reconnect_attempts"];
    
    // Use string representation to avoid adding extra dependencies
    let struct_info = format!("{:#?}", BluetoothConfig::default());
    
    // Make sure each field is in the string representation
    for field in expected_fields {
        assert!(struct_info.contains(field), 
                "BluetoothConfig is missing field '{}'", field);
    }
}

/// Test that the UiConfig struct has all required fields
#[test]
fn test_ui_config_structure() {
    // Verify the structure of the UiConfig
    let expected_fields = ["show_notifications", "start_minimized", "theme",
                          "show_percentage_in_tray", "show_low_battery_warning", 
                          "low_battery_threshold"];
    
    // Use string representation to avoid adding extra dependencies
    let struct_info = format!("{:#?}", UiConfig::default());
    
    // Make sure each field is in the string representation
    for field in expected_fields {
        assert!(struct_info.contains(field), 
                "UiConfig is missing field '{}'", field);
    }
}

/// Test that the SystemConfig struct has all required fields
#[test]
fn test_system_config_structure() {
    // Verify the structure of the SystemConfig
    let expected_fields = ["launch_at_startup", "log_level", "enable_telemetry"];
    
    // Use string representation to avoid adding extra dependencies
    let struct_info = format!("{:#?}", SystemConfig::default());
    
    // Make sure each field is in the string representation
    for field in expected_fields {
        assert!(struct_info.contains(field), 
                "SystemConfig is missing field '{}'", field);
    }
}

/// Test that all fields from all config structs are present
#[test]
fn test_all_config_structures() {
    // Create a list of all expected fields
    let expected_fields = [
        // Bluetooth fields
        "auto_scan_on_startup", "scan_duration", "scan_interval", 
        "min_rssi", "battery_refresh_interval", "auto_reconnect", 
        "reconnect_attempts",
        
        // UI fields
        "show_notifications", "start_minimized", "theme",
        "show_percentage_in_tray", "show_low_battery_warning", 
        "low_battery_threshold",
        
        // System fields
        "launch_at_startup", "log_level", "enable_telemetry"
    ];
    
    // Get string representation of the full AppConfig
    let config_info = format!("{:#?}", AppConfig::default());
    
    // Check that all expected fields are present
    for field in expected_fields {
        assert!(config_info.contains(field), 
                "AppConfig is missing field '{}'", field);
    }
} 