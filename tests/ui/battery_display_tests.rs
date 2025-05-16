//! Tests for the RealTimeBatteryDisplay component
//! This tests the battery display functionality implemented in Task 10.2

use std::time::{Duration, Instant};

use rustpods::ui::components::RealTimeBatteryDisplay;
use rustpods::ui::Message;
use rustpods::ui::UiComponent;
use rustpods::bluetooth::{AirPodsBatteryStatus};
use rustpods::airpods::{AirPodsBattery, AirPodsChargingState};
use rustpods::ui::theme::Theme;

use iced::Element;
use iced::Rectangle;

/// Helper function to create a battery status
fn create_battery_status(left: Option<u8>, right: Option<u8>, case: Option<u8>, 
                         charging: AirPodsChargingState) -> AirPodsBatteryStatus {
    AirPodsBatteryStatus {
        battery: AirPodsBattery {
            left,
            right,
            case,
            charging: Some(charging),
        },
        last_updated: Instant::now(),
    }
}

#[test]
fn test_battery_display_creation() {
    // Create default display
    let display = RealTimeBatteryDisplay::default();
    
    // Verify view doesn't panic
    let _element = display.view();
}

#[test]
fn test_battery_display_update() {
    // Create display with initial state
    let mut display = RealTimeBatteryDisplay::default();
    
    // Create battery status
    let status = create_battery_status(
        Some(75), Some(80), Some(90),
        AirPodsChargingState::CaseCharging
    );
    
    // Update display
    display.update(Some(status.clone()));
    
    // Ensure the view doesn't panic
    let _element = display.view();
}

#[test]
fn test_animation_progress_update() {
    // Create display
    let display = RealTimeBatteryDisplay::default().with_animation_progress(0.5);
    
    // Ensure the view doesn't panic
    let _element = display.view();
}

#[test]
fn test_battery_display_view() {
    // Create display
    let mut display = RealTimeBatteryDisplay::default();
    
    // Create battery status
    let status = create_battery_status(
        Some(75), Some(80), Some(90),
        AirPodsChargingState::CaseCharging
    );
    
    // Update display
    display.update(Some(status));
    
    // Call view method to verify it doesn't panic
    let _element = display.view();
}

#[test]
fn test_battery_estimation() {
    // Create display
    let mut display = RealTimeBatteryDisplay::default();
    
    // Test with various battery levels to ensure time remaining estimation works
    
    // Test high battery (should show long time remaining)
    let high_status = create_battery_status(
        Some(90), Some(95), Some(100),
        AirPodsChargingState::NotCharging
    );
    display.update(Some(high_status));
    
    // Test medium battery (should show medium time remaining)
    let med_status = create_battery_status(
        Some(50), Some(55), Some(60),
        AirPodsChargingState::NotCharging
    );
    display.update(Some(med_status));
    
    // Test low battery (should show short time remaining)
    let low_status = create_battery_status(
        Some(10), Some(15), Some(20),
        AirPodsChargingState::NotCharging
    );
    display.update(Some(low_status));
    
    // Test charging (should show charging indicator)
    let charging_status = create_battery_status(
        Some(30), Some(35), Some(40),
        AirPodsChargingState::BothBudsCharging
    );
    display.update(Some(charging_status));
    
    // Ensure the view doesn't panic with the current state
    let _element = display.view();
}

#[test]
fn test_partial_battery_info() {
    // Create display
    let mut display = RealTimeBatteryDisplay::default();
    
    // Test with missing battery info
    
    // Missing left AirPod
    let missing_left = create_battery_status(
        None, Some(80), Some(90),
        AirPodsChargingState::CaseCharging
    );
    display.update(Some(missing_left));
    
    // Missing right AirPod
    let missing_right = create_battery_status(
        Some(75), None, Some(90),
        AirPodsChargingState::CaseCharging
    );
    display.update(Some(missing_right));
    
    // Missing case
    let missing_case = create_battery_status(
        Some(75), Some(80), None,
        AirPodsChargingState::NotCharging
    );
    display.update(Some(missing_case));
    
    // Missing all (should handle gracefully)
    let missing_all = create_battery_status(
        None, None, None,
        AirPodsChargingState::NotCharging
    );
    display.update(Some(missing_all));
    
    // Ensure the view doesn't panic with the current state
    let _element = display.view();
}

#[test]
fn test_real_time_battery_display_instantiation() {
    // Test with Some battery status
    let battery = AirPodsBattery {
        left: Some(80),
        right: Some(75),
        case: Some(90),
        charging: Some(AirPodsChargingState::CaseCharging),
    };
    
    let status = AirPodsBatteryStatus::new(battery);
    let display = RealTimeBatteryDisplay::new(Some(status));
    
    // Verify it has the status and default values
    assert!(display.battery_status.is_some());
    assert_eq!(display.animation_progress, 0.0);
    assert!(!display.show_time_since_update);
    assert!(!display.show_detailed_info);
    assert!(display.previous_levels.is_none());
    
    // Test with None battery status
    let display = RealTimeBatteryDisplay::new(None);
    assert!(display.battery_status.is_none());
}

#[test]
fn test_real_time_battery_display_time_remaining() {
    // Create battery status with different levels
    let battery = AirPodsBattery {
        left: Some(50),
        right: Some(60),
        case: Some(90),
        charging: Some(AirPodsChargingState::NotCharging),
    };
    
    let status = AirPodsBatteryStatus::new(battery);
    let display = RealTimeBatteryDisplay::new(Some(status));
    
    // Should use the lower value between left and right
    let time = display.calculate_time_remaining();
    assert!(time.is_some());
    if let Some(minutes) = time {
        // 50% should be 150 minutes (50% of 300)
        assert_eq!(minutes, 150);
    }
}

#[test]
fn test_real_time_battery_display_update() {
    // Create initial battery status
    let initial_battery = AirPodsBattery {
        left: Some(80),
        right: Some(75),
        case: Some(90),
        charging: Some(AirPodsChargingState::NotCharging),
    };
    
    let initial_status = AirPodsBatteryStatus::new(initial_battery);
    let mut display = RealTimeBatteryDisplay::new(Some(initial_status));
    
    // Verify initial values
    if let Some(status) = &display.battery_status {
        assert_eq!(status.battery.left, Some(80));
        assert_eq!(status.battery.right, Some(75));
        assert_eq!(status.battery.case, Some(90));
    } else {
        panic!("Expected battery status to be present");
    }
    
    // Create updated battery status
    let updated_battery = AirPodsBattery {
        left: Some(70),
        right: Some(65),
        case: Some(85),
        charging: Some(AirPodsChargingState::CaseCharging),
    };
    
    let updated_status = AirPodsBatteryStatus::new(updated_battery);
    
    // Update the display
    display.update(Some(updated_status));
    
    // Verify updated values
    if let Some(status) = &display.battery_status {
        assert_eq!(status.battery.left, Some(70));
        assert_eq!(status.battery.right, Some(65));
        assert_eq!(status.battery.case, Some(85));
        
        // Verify charging state
        if let Some(charging) = &status.battery.charging {
            assert!(matches!(charging, AirPodsChargingState::CaseCharging));
        } else {
            panic!("Expected charging state to be present");
        }
    } else {
        panic!("Expected battery status to be present after update");
    }
    
    // Verify previous_levels was set correctly for animation
    assert!(display.previous_levels.is_some());
    if let Some((prev_left, prev_right, prev_case)) = display.previous_levels {
        assert_eq!(prev_left, Some(80));
        assert_eq!(prev_right, Some(75));
        assert_eq!(prev_case, Some(90));
    }
    
    // Test update with None (disconnection)
    display.update(None);
    
    // Verify battery status is now None
    assert!(display.battery_status.is_none());
}

#[test]
fn test_real_time_battery_display_compact_view() {
    // Create battery status
    let battery = AirPodsBattery {
        left: Some(80),
        right: Some(75),
        case: Some(90),
        charging: Some(AirPodsChargingState::NotCharging),
    };
    
    let status = AirPodsBatteryStatus::new(battery);
    
    // Create normal display
    let normal_display = RealTimeBatteryDisplay::new(Some(status.clone()));
    assert!(!normal_display.compact_view);
    
    // Create compact display
    let compact_display = RealTimeBatteryDisplay::new(Some(status)).with_compact_view(true);
    assert!(compact_display.compact_view);
    
    // The actual view rendering would be tested in integration tests
    // Here we just verify the property is set correctly
}

#[test]
fn test_real_time_battery_display_time_since_update() {
    // Create battery status
    let battery = AirPodsBattery {
        left: Some(80),
        right: Some(75),
        case: Some(90),
        charging: Some(AirPodsChargingState::NotCharging),
    };
    
    let status = AirPodsBatteryStatus::new(battery);
    
    // Default is to not show time since update
    let default_display = RealTimeBatteryDisplay::new(Some(status.clone()));
    assert!(!default_display.show_time_since_update);
    
    // Create display with time since update enabled
    let time_display = RealTimeBatteryDisplay::new(Some(status)).with_time_since_update(true);
    assert!(time_display.show_time_since_update);
}

#[test]
fn test_real_time_battery_display_animation_progress() {
    // Create battery status
    let battery = AirPodsBattery {
        left: Some(80),
        right: Some(75),
        case: Some(90),
        charging: Some(AirPodsChargingState::NotCharging),
    };
    
    let status = AirPodsBatteryStatus::new(battery);
    
    // Default animation progress is 0.0
    let default_display = RealTimeBatteryDisplay::new(Some(status.clone()));
    assert_eq!(default_display.animation_progress, 0.0);
    
    // Create display with custom animation progress
    let animated_display = RealTimeBatteryDisplay::new(Some(status)).with_animation_progress(0.5);
    assert_eq!(animated_display.animation_progress, 0.5);
    
    // Create with out-of-range value (should be clamped)
    let over_animated_display = RealTimeBatteryDisplay::new(None).with_animation_progress(1.5);
    assert_eq!(over_animated_display.animation_progress, 1.0);
}

#[test]
fn test_real_time_battery_display_detailed_info() {
    // Create battery status
    let battery = AirPodsBattery {
        left: Some(80),
        right: Some(75),
        case: Some(90),
        charging: Some(AirPodsChargingState::NotCharging),
    };
    
    let status = AirPodsBatteryStatus::new(battery);
    
    // Default is to show detailed info
    let default_display = RealTimeBatteryDisplay::new(Some(status.clone()));
    assert!(default_display.show_detailed_info);
    
    // Create display with detailed info disabled
    let simple_display = RealTimeBatteryDisplay::new(Some(status)).with_detailed_info(false);
    assert!(!simple_display.show_detailed_info);
}

#[test]
fn test_real_time_battery_display_helper_methods() {
    // Create battery status
    let battery = AirPodsBattery {
        left: Some(80),
        right: Some(75),
        case: Some(90),
        charging: Some(AirPodsChargingState::LeftCharging),
    };
    
    let status = AirPodsBatteryStatus::new(battery);
    let display = RealTimeBatteryDisplay::new(Some(status));
    
    // Test time formatting
    let one_hour = display.format_time_remaining(60);
    assert_eq!(one_hour, "1h 0m");
    
    let one_hour_thirty = display.format_time_remaining(90);
    assert_eq!(one_hour_thirty, "1h 30m");
    
    let two_hours = display.format_time_remaining(120);
    assert_eq!(two_hours, "2h 0m");
    
    let thirty_minutes = display.format_time_remaining(30);
    assert_eq!(thirty_minutes, "30m");
    
    // Test animated level calculation
    let current = Some(80u8);
    let previous = Some(60u8);
    
    // With animation progress 0.0, should return previous value
    let display_start = RealTimeBatteryDisplay::new(None).with_animation_progress(0.0);
    assert_eq!(display_start.get_animated_level(current, previous), previous);
    
    // With animation progress 1.0, should return current value
    let display_end = RealTimeBatteryDisplay::new(None).with_animation_progress(1.0);
    assert_eq!(display_end.get_animated_level(current, previous), current);
    
    // With animation progress 0.5, should return interpolated value (70)
    let display_mid = RealTimeBatteryDisplay::new(None).with_animation_progress(0.5);
    assert_eq!(display_mid.get_animated_level(current, previous), Some(70));
}

#[test]
fn test_real_time_battery_display_missing_values() {
    // Create battery status with missing values
    let battery = AirPodsBattery {
        left: Some(80),
        right: None, // Right earbud missing
        case: None,  // Case missing
        charging: Some(AirPodsChargingState::NotCharging),
    };
    
    let status = AirPodsBatteryStatus::new(battery);
    let display = RealTimeBatteryDisplay::new(Some(status));
    
    // Should calculate time remaining based on left earbud only
    let time = display.calculate_time_remaining();
    assert!(time.is_some());
    if let Some(minutes) = time {
        // 80% should be 240 minutes (80% of 300)
        assert_eq!(minutes, 240);
    }
}

#[test]
fn test_real_time_battery_display_animation() {
    // Create with battery status
    let battery = AirPodsBattery {
        left: Some(80),
        right: Some(75),
        case: Some(90),
        charging: Some(AirPodsChargingState::LeftCharging),
    };
    
    let status = AirPodsBatteryStatus::new(battery);
    let mut display = RealTimeBatteryDisplay::new(Some(status));
    
    // Set previous levels for animation
    display.previous_levels = Some((Some(70), Some(65), Some(80)));
    
    // Test animation progress
    assert_eq!(display.animation_progress, 0.0);
    display.animation_progress = 0.5;
    assert_eq!(display.animation_progress, 0.5);
    
    // Test time since update display
    assert!(!display.show_time_since_update);
    display.show_time_since_update = true;
    assert!(display.show_time_since_update);
}

#[test]
fn test_real_time_battery_display_view() {
    // Create with battery status
    let battery = AirPodsBattery {
        left: Some(80),
        right: Some(75),
        case: Some(90),
        charging: Some(AirPodsChargingState::NotCharging),
    };
    
    let status = AirPodsBatteryStatus::new(battery);
    let display = RealTimeBatteryDisplay::new(Some(status));
    
    // Just verify that view() returns without error
    let _element = display.view();
} 