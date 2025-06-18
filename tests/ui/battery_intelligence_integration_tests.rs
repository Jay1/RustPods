//! Integration tests for BatteryIntelligence system with UI
//!
//! Tests the integration between the new BatteryIntelligence system and the UI state management

use rustpods::airpods::battery::AirPodsBatteryInfo;
use rustpods::ui::state::AppState;
use std::time::SystemTime;

#[test]
fn test_battery_intelligence_initialization() {
    let (sender, _receiver) = tokio::sync::mpsc::unbounded_channel();

    let state = AppState::new(sender);

    // Verify that BatteryIntelligence is initialized
    assert!(
        state.battery_intelligence.storage_dir.exists()
            || state
                .battery_intelligence
                .storage_dir
                .to_string_lossy()
                .contains("battery_intelligence")
    );

    // Note: device_profiles may not be empty if there are existing profile files from previous runs
    // This is expected behavior - the system loads existing profiles on startup
}

#[test]
fn test_battery_intelligence_device_update() {
    let (sender, _receiver) = tokio::sync::mpsc::unbounded_channel();

    let mut state = AppState::new(sender);
    state.config.battery.enable_estimation = true;

    // Create test AirPods data
    let airpods = AirPodsBatteryInfo {
        address: 123456789,
        canonical_address: "75bcd15".to_string(), // hex representation of 123456789
        name: "Test AirPods Pro".to_string(),
        model_id: 0x2014, // AirPods Pro 2
        left_battery: 85,
        left_charging: false,
        right_battery: 80,
        right_charging: false,
        case_battery: 95,
        case_charging: true,
        left_in_ear: Some(true),
        right_in_ear: Some(true),
        case_lid_open: Some(false),
        side: None,
        both_in_case: Some(false),
        color: None,
        switch_count: None,
        rssi: Some(-45),
        timestamp: None,
        raw_manufacturer_data: None,
    };

    // Update the state with AirPods data
    state.airpods_devices = vec![airpods];
    state.update_merged_devices();

    // Verify that BatteryIntelligence received the update
    assert!(state.battery_intelligence.device_profile.is_some());

    let profile = state.battery_intelligence.device_profile.as_ref().unwrap();
    assert_eq!(profile.device_name, "Test AirPods Pro");
    assert_eq!(profile.current_left, Some(85));
    assert_eq!(profile.current_right, Some(80));
    assert_eq!(profile.current_case, Some(95));
    assert!(profile.case_charging);
    assert!(profile.left_in_ear);
    assert!(profile.right_in_ear);
}

#[test]
fn test_battery_intelligence_fractional_estimates() {
    let (sender, _receiver) = tokio::sync::mpsc::unbounded_channel();

    let mut state = AppState::new(sender);
    state.config.battery.enable_estimation = true;

    // Create test AirPods data
    let airpods = AirPodsBatteryInfo {
        address: 987654321,
        canonical_address: "3ade68b1".to_string(), // hex representation of 987654321
        name: "Test AirPods".to_string(),
        model_id: 0x200F, // AirPods 2
        left_battery: 75,
        left_charging: false,
        right_battery: 70,
        right_charging: false,
        case_battery: 50,
        case_charging: false,
        left_in_ear: Some(false),
        right_in_ear: Some(false),
        case_lid_open: Some(true),
        side: None,
        both_in_case: Some(true),
        color: None,
        switch_count: None,
        rssi: Some(-55),
        timestamp: None,
        raw_manufacturer_data: None,
    };

    // Update the state with AirPods data
    state.airpods_devices = vec![airpods];
    state.update_merged_devices();

    // Get estimates from BatteryIntelligence
    if let Some((left_est, right_est, case_est)) =
        state.battery_intelligence.get_battery_estimates()
    {
        // For fresh data, estimates should match real data
        assert_eq!(left_est.level, 75.0);
        assert_eq!(right_est.level, 70.0);
        assert_eq!(case_est.level, 50.0);
        assert!(left_est.is_real_data);
        assert!(right_est.is_real_data);
        assert!(case_est.is_real_data);
        assert_eq!(left_est.confidence, 1.0);
        assert_eq!(right_est.confidence, 1.0);
        assert_eq!(case_est.confidence, 1.0);
    } else {
        panic!("Expected battery estimates to be available");
    }
}

#[test]
fn test_battery_intelligence_significance_filtering() {
    let (sender, _receiver) = tokio::sync::mpsc::unbounded_channel();

    let mut state = AppState::new(sender);
    state.config.battery.enable_estimation = true;

    // Create AirPods data
    let mut airpods = AirPodsBatteryInfo {
        address: 333444555,
        canonical_address: "13dde4b3".to_string(), // hex representation of 333444555
        name: "Test AirPods Pro 2".to_string(),
        model_id: 0x200E,
        left_battery: 90,
        left_charging: false,
        right_battery: 90,
        right_charging: false,
        case_battery: 100,
        case_charging: false,
        left_in_ear: Some(true),
        right_in_ear: Some(true),
        case_lid_open: Some(false),
        side: None,
        both_in_case: Some(false),
        color: None,
        switch_count: None,
        rssi: Some(-45),
        timestamp: None,
        raw_manufacturer_data: None,
    };

    // First update
    state.airpods_devices = vec![airpods.clone()];
    state.update_merged_devices();

    let profile = state.battery_intelligence.device_profile.as_ref().unwrap();
    let initial_event_count = profile.events.len();

    // Update with same data (should not create new event due to significance filtering)
    state.airpods_devices = vec![airpods.clone()];
    state.update_merged_devices();

    let profile = state.battery_intelligence.device_profile.as_ref().unwrap();
    assert_eq!(
        profile.events.len(),
        initial_event_count,
        "No new event should be created for identical data"
    );

    // Simulate time passing to ensure depletion rate can be calculated
    let profile = state.battery_intelligence.device_profile.as_mut().unwrap();
    profile.last_update = Some(SystemTime::now() - std::time::Duration::from_secs(3600)); // 1 hour ago

    // Update with very significant battery change (should create new event and depletion rate samples)
    airpods.left_battery = 60; // 30% drop from original 90%, significant enough for depletion rate calculation
    airpods.right_battery = 60; // Also drop right battery by same amount to test both channels
    state.airpods_devices = vec![airpods.clone()];
    state.update_merged_devices();

    let profile = state.battery_intelligence.device_profile.as_ref().unwrap();
    assert!(
        profile.events.len() > initial_event_count,
        "New event should be created for significant battery change"
    );

    // Verify depletion rate samples were created
    let left_depletion_samples = profile
        .depletion_rates
        .get_sample_count(rustpods::airpods::battery_intelligence::DepletionTarget::LeftEarbud);
    let right_depletion_samples = profile
        .depletion_rates
        .get_sample_count(rustpods::airpods::battery_intelligence::DepletionTarget::RightEarbud);

    println!("Left depletion samples: {}", left_depletion_samples);
    println!("Right depletion samples: {}", right_depletion_samples);

    assert!(
        left_depletion_samples > 0,
        "Depletion rate samples should be created for 30% battery drop"
    );
    assert!(
        right_depletion_samples > 0,
        "Depletion rate samples should be created for 30% battery drop"
    );
}

#[test]
fn test_battery_intelligence_charging_state_detection() {
    let (sender, _receiver) = tokio::sync::mpsc::unbounded_channel();

    let mut state = AppState::new(sender);
    state.config.battery.enable_estimation = true;

    // Create AirPods data - not charging
    let mut airpods = AirPodsBatteryInfo {
        address: 444555666,
        canonical_address: "1a83f92a".to_string(), // hex representation of 444555666
        name: "Test AirPods Max".to_string(),
        model_id: 0x200A,
        left_battery: 60,
        left_charging: false,
        right_battery: 60,
        right_charging: false,
        case_battery: 0, // AirPods Max don't have a case
        case_charging: false,
        left_in_ear: Some(true),
        right_in_ear: Some(true),
        case_lid_open: None,
        side: None,
        both_in_case: Some(false),
        color: None,
        switch_count: None,
        rssi: Some(-35),
        timestamp: None,
        raw_manufacturer_data: None,
    };

    // First update - not charging
    state.airpods_devices = vec![airpods.clone()];
    state.update_merged_devices();

    let profile = state.battery_intelligence.device_profile.as_ref().unwrap();
    assert!(!profile.left_charging);
    assert!(!profile.right_charging);

    // Update to charging state
    airpods.left_charging = true;
    airpods.right_charging = true;
    state.airpods_devices = vec![airpods.clone()];
    state.update_merged_devices();

    let profile = state.battery_intelligence.device_profile.as_ref().unwrap();
    assert!(profile.left_charging);
    assert!(profile.right_charging);

    // Verify that a charging event was created
    let has_charging_event = profile.events.iter().any(|event| {
        matches!(
            event.event_type,
            rustpods::airpods::battery_intelligence::BatteryEventType::ChargingStarted
        )
    });
    assert!(
        has_charging_event,
        "Should have created a ChargingStarted event"
    );
}

#[test]
fn test_battery_intelligence_fallback_to_old_estimator() {
    let (sender, _receiver) = tokio::sync::mpsc::unbounded_channel();

    let mut state = AppState::new(sender);
    state.config.battery.enable_estimation = true;

    // Don't add any AirPods devices, so BatteryIntelligence won't have data
    state.airpods_devices = vec![];
    state.update_merged_devices();

    // The system should fall back to the old estimator when BatteryIntelligence has no data
    // This is tested by ensuring the update_merged_devices function doesn't panic
    // and that the old estimator is still being updated
    assert!(state
        .battery_estimator
        .left_history
        .discharge_rates
        .is_empty());
    assert!(state
        .battery_estimator
        .right_history
        .discharge_rates
        .is_empty());
    assert!(state
        .battery_estimator
        .case_history
        .discharge_rates
        .is_empty());
}

#[test]
fn test_battery_intelligence_storage_directory() {
    let (sender, _receiver) = tokio::sync::mpsc::unbounded_channel();

    let state = AppState::new(sender);

    // Verify that the storage directory is set correctly
    let expected_path = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("RustPods")
        .join("battery_intelligence");
    assert_eq!(state.battery_intelligence.storage_dir, expected_path);
}

#[test]
fn test_battery_intelligence_multiple_devices() {
    let (sender, _receiver) = tokio::sync::mpsc::unbounded_channel();

    let mut state = AppState::new(sender);
    state.config.battery.enable_estimation = true;

    // Create multiple AirPods devices
    let airpods1 = AirPodsBatteryInfo {
        address: 111111111,
        canonical_address: "69f6bcf".to_string(), // hex representation of 111111111
        name: "AirPods Pro 1".to_string(),
        model_id: 0x2014,
        left_battery: 80,
        left_charging: false,
        right_battery: 75,
        right_charging: false,
        case_battery: 90,
        case_charging: false,
        left_in_ear: Some(true),
        right_in_ear: Some(true),
        case_lid_open: Some(false),
        side: None,
        both_in_case: Some(false),
        color: None,
        switch_count: None,
        rssi: Some(-45),
        timestamp: None,
        raw_manufacturer_data: None,
    };

    let airpods2 = AirPodsBatteryInfo {
        address: 222222222,
        canonical_address: "d3ec5ce".to_string(), // hex representation of 222222222
        name: "AirPods Pro 2".to_string(),
        model_id: 0x2014,
        left_battery: 65,
        left_charging: true,
        right_battery: 70,
        right_charging: true,
        case_battery: 85,
        case_charging: true,
        left_in_ear: Some(false),
        right_in_ear: Some(false),
        case_lid_open: Some(true),
        side: None,
        both_in_case: Some(true),
        color: None,
        switch_count: None,
        rssi: Some(-50),
        timestamp: None,
        raw_manufacturer_data: None,
    };

    // Update with multiple devices
    state.airpods_devices = vec![airpods1, airpods2];
    state.update_merged_devices();

    // Verify that only one device profile exists (singleton pattern)
    // The battery intelligence system now only tracks one device at a time
    assert!(state.battery_intelligence.device_profile.is_some());

    let profile = state.battery_intelligence.device_profile.as_ref().unwrap();

    // The profile should be for one of the devices (likely the selected one)
    assert!(profile.device_name == "AirPods Pro 1" || profile.device_name == "AirPods Pro 2");

    // Note: The singleton implementation only gets estimates from the current device
    // This is the expected behavior for the simplified single-device approach
    if let Some((left_est, right_est, case_est)) =
        state.battery_intelligence.get_battery_estimates()
    {
        // The estimates should match one of the devices' battery levels
        assert!(left_est.level == 80.0 || left_est.level == 65.0);
        assert!(right_est.level == 75.0 || right_est.level == 70.0);
        assert!(case_est.level == 90.0 || case_est.level == 85.0);
    }
}
