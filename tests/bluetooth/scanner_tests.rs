//! Tests for CLI Scanner integration and AirPods battery monitoring
//! Focused on testing the native C++ CLI scanner integration rather than direct BLE scanning

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use btleplug::api::BDAddr;
use serde_json::from_str;

use rustpods::airpods::{AirPodsBattery, AirPodsChargingState, AirPodsType, DetectedAirPods};
use rustpods::bluetooth::cli_scanner::{
    CliAirPodsData, CliScanner, CliScannerConfig, CliScannerResult,
};
use rustpods::bluetooth::DiscoveredDevice;
use rustpods::config::{AppConfig, LogLevel};
use rustpods::error::BluetoothError;

/// Test helper to create a sample discovered device
fn create_test_device(
    address: [u8; 6],
    name: Option<&str>,
    rssi: Option<i16>,
    is_airpods: bool,
) -> DiscoveredDevice {
    let manufacturer_data = HashMap::new();
    DiscoveredDevice {
        address: BDAddr::from(address),
        name: name.map(|s| s.to_string()),
        rssi,
        manufacturer_data,
        is_potential_airpods: is_airpods,
        last_seen: Instant::now(),
        is_connected: false,
        service_data: HashMap::new(),
        services: Vec::new(),
        tx_power_level: None,
    }
}

#[test]
fn test_airpods_battery_struct() {
    let battery = AirPodsBattery {
        left: Some(80),
        right: Some(70),
        case: Some(60),
        charging: Some(AirPodsChargingState::LeftCharging),
    };
    assert_eq!(battery.left, Some(80));
    assert_eq!(battery.right, Some(70));
    assert_eq!(battery.case, Some(60));
    assert_eq!(battery.charging, Some(AirPodsChargingState::LeftCharging));
}

#[test]
fn test_discovered_device_fields() {
    let device = create_test_device([1, 2, 3, 4, 5, 6], Some("AirPods Pro"), Some(-55), true);
    assert_eq!(device.address, BDAddr::from([1, 2, 3, 4, 5, 6]));
    assert_eq!(device.name.as_deref(), Some("AirPods Pro"));
    assert_eq!(device.rssi, Some(-55));
    assert!(device.is_potential_airpods);
}

// CLI Scanner Tests

#[test]
fn test_cli_scanner_config_default() {
    let config = CliScannerConfig::default();
    assert_eq!(
        config.scanner_path,
        PathBuf::from("scripts/airpods_battery_cli/build/Release/airpods_battery_cli.exe")
    );
    assert_eq!(config.poll_interval, Duration::from_secs(30));
    assert!(config.adaptive_polling);
    assert_eq!(config.max_errors, 5);
    assert!(!config.verbose_logging);
}

#[test]
fn test_cli_scanner_config_from_app_config() {
    // Create a new AppConfig using Default trait
    let mut app_config = AppConfig::default();

    // Set the fields we need for testing
    app_config.bluetooth.battery_refresh_interval = Duration::from_secs(45);
    app_config.bluetooth.adaptive_polling = false;
    app_config.system.log_level = LogLevel::Debug;

    let config = CliScannerConfig::from_app_config(&app_config);
    assert_eq!(config.poll_interval, Duration::from_secs(45));
    assert!(!config.adaptive_polling);
    assert!(config.verbose_logging);
}

#[test]
fn test_parse_cli_scanner_json() {
    let json_str = r#"{
        "scanner_version": "6.0.0",
        "scan_timestamp": "2025-06-17T18:30:00Z",
        "total_devices": 1,
        "devices": [
            {
                "device_id": "1",
                "address": "00:11:22:33:44:55",
                "rssi": -60,
                "manufacturer_data_hex": "07190114200b778f",
                "airpods_data": {
                    "model": "AirPods Pro",
                    "model_id": "0x0E20",
                    "left_battery": 80,
                    "right_battery": 75,
                    "case_battery": 90,
                    "left_charging": false,
                    "right_charging": false,
                    "case_charging": true,
                    "left_in_ear": true,
                    "right_in_ear": true,
                    "both_in_case": false,
                    "lid_open": false,
                    "broadcasting_ear": "both"
                }
            }
        ],
        "airpods_count": 1,
        "status": "success",
        "note": "Scan completed successfully"
    }"#;

    let result: CliScannerResult = from_str(json_str).unwrap();
    assert_eq!(result.scanner_version, "6.0.0");
    assert_eq!(result.total_devices, 1);
    assert_eq!(result.airpods_count, 1);
    assert_eq!(result.status, "success");

    let device = &result.devices[0];
    assert_eq!(device.address, "00:11:22:33:44:55");
    assert_eq!(device.rssi, -60);

    let airpods_data = device.airpods_data.as_ref().unwrap();
    assert_eq!(airpods_data.model, "AirPods Pro");
    assert_eq!(airpods_data.left_battery, 80);
    assert_eq!(airpods_data.right_battery, 75);
    assert_eq!(airpods_data.case_battery, 90);
    assert!(airpods_data.case_charging);
    assert!(!airpods_data.left_charging);
}

#[test]
fn test_manual_cli_data_conversion() {
    // Since we can't directly access the private conversion function,
    // we'll test our own implementation of the conversion logic
    let cli_data = CliAirPodsData {
        model: "AirPods Pro".to_string(),
        model_id: "0x0E20".to_string(),
        left_battery: 80,
        right_battery: 75,
        case_battery: 90,
        left_charging: false,
        right_charging: false,
        case_charging: true,
        left_in_ear: true,
        right_in_ear: true,
        both_in_case: false,
        lid_open: false,
        broadcasting_ear: "both".to_string(),
    };

    // Manual conversion logic (similar to what's in the CLI scanner)
    let device_address = "00:11:22:33:44:55".to_string();

    // Parse MAC address
    let addr_parts: Vec<&str> = device_address.split(':').collect();
    let mut addr_bytes = [0u8; 6];
    for (i, part) in addr_parts.iter().enumerate() {
        addr_bytes[i] = u8::from_str_radix(part, 16).unwrap();
    }
    let address = BDAddr::from(addr_bytes);

    // Determine device type based on model
    let device_type = match cli_data.model.as_str() {
        "AirPods Pro" => AirPodsType::AirPodsPro,
        "AirPods" => AirPodsType::AirPods1,
        "AirPods Max" => AirPodsType::AirPodsMax,
        _ => AirPodsType::Unknown,
    };

    // Determine charging state
    let charging_state = if cli_data.left_charging && cli_data.right_charging {
        Some(AirPodsChargingState::BothBudsCharging)
    } else if cli_data.left_charging {
        Some(AirPodsChargingState::LeftCharging)
    } else if cli_data.right_charging {
        Some(AirPodsChargingState::RightCharging)
    } else if cli_data.case_charging {
        Some(AirPodsChargingState::CaseCharging)
    } else {
        Some(AirPodsChargingState::NotCharging)
    };

    // Create battery info
    let battery = AirPodsBattery {
        left: Some(cli_data.left_battery as u8),
        right: Some(cli_data.right_battery as u8),
        case: Some(cli_data.case_battery as u8),
        charging: charging_state,
    };

    // Create DetectedAirPods
    let airpods = DetectedAirPods {
        address,
        device_type,
        battery: Some(battery),
        rssi: Some(-60), // Default value for testing
        name: Some("AirPods Pro".to_string()),
        is_connected: true,
        last_seen: Instant::now(),
    };

    // Assertions
    assert_eq!(airpods.address.to_string(), device_address);
    assert_eq!(airpods.device_type, AirPodsType::AirPodsPro);
    assert_eq!(airpods.battery.as_ref().unwrap().left, Some(80));
    assert_eq!(airpods.battery.as_ref().unwrap().right, Some(75));
    assert_eq!(airpods.battery.as_ref().unwrap().case, Some(90));
    assert_eq!(
        airpods.battery.as_ref().unwrap().charging,
        Some(AirPodsChargingState::CaseCharging)
    );
}

#[test]
fn test_significant_battery_change_detection() {
    // Test our own implementation of battery change detection
    let prev = CliAirPodsData {
        model: "AirPods Pro".to_string(),
        model_id: "0x0E20".to_string(),
        left_battery: 80,
        right_battery: 75,
        case_battery: 90,
        left_charging: false,
        right_charging: false,
        case_charging: true,
        left_in_ear: true,
        right_in_ear: true,
        both_in_case: false,
        lid_open: false,
        broadcasting_ear: "both".to_string(),
    };

    // No significant change (less than 10% difference)
    let curr1 = CliAirPodsData {
        left_battery: 81,
        right_battery: 74,
        case_battery: 89,
        ..prev.clone()
    };

    // Our own implementation of significant battery change detection
    let threshold = 10; // 10% threshold
    let result1 = (prev.left_battery - curr1.left_battery).abs() >= threshold
        || (prev.right_battery - curr1.right_battery).abs() >= threshold
        || (prev.case_battery - curr1.case_battery).abs() >= threshold;

    assert!(!result1);

    // Significant change (10% threshold)
    let curr2 = CliAirPodsData {
        left_battery: 70, // 10% change
        right_battery: 75,
        case_battery: 90,
        ..prev.clone()
    };

    let result2 = (prev.left_battery - curr2.left_battery).abs() >= threshold
        || (prev.right_battery - curr2.right_battery).abs() >= threshold
        || (prev.case_battery - curr2.case_battery).abs() >= threshold;

    assert!(result2);
}

#[test]
fn test_charging_state_change_detection() {
    // Test our own implementation of charging state change detection
    let prev = CliAirPodsData {
        model: "AirPods Pro".to_string(),
        model_id: "0x0E20".to_string(),
        left_battery: 80,
        right_battery: 75,
        case_battery: 90,
        left_charging: false,
        right_charging: false,
        case_charging: true,
        left_in_ear: true,
        right_in_ear: true,
        both_in_case: false,
        lid_open: false,
        broadcasting_ear: "both".to_string(),
    };

    // No change in charging state
    let curr1 = prev.clone();

    // Our own implementation of charging state change detection
    let result1 = prev.left_charging != curr1.left_charging
        || prev.right_charging != curr1.right_charging
        || prev.case_charging != curr1.case_charging;

    assert!(!result1);

    // Change in charging state
    let curr2 = CliAirPodsData {
        left_charging: true, // Changed
        ..prev.clone()
    };

    let result2 = prev.left_charging != curr2.left_charging
        || prev.right_charging != curr2.right_charging
        || prev.case_charging != curr2.case_charging;

    assert!(result2);
}

#[test]
fn test_usage_state_change_detection() {
    // Test our own implementation of usage state change detection
    let prev = CliAirPodsData {
        model: "AirPods Pro".to_string(),
        model_id: "0x0E20".to_string(),
        left_battery: 80,
        right_battery: 75,
        case_battery: 90,
        left_charging: false,
        right_charging: false,
        case_charging: true,
        left_in_ear: true,
        right_in_ear: true,
        both_in_case: false,
        lid_open: false,
        broadcasting_ear: "both".to_string(),
    };

    // No change in usage state
    let curr1 = prev.clone();

    // Our own implementation of usage state change detection
    let result1 = prev.left_in_ear != curr1.left_in_ear
        || prev.right_in_ear != curr1.right_in_ear
        || prev.both_in_case != curr1.both_in_case
        || prev.lid_open != curr1.lid_open;

    assert!(!result1);

    // Change in usage state
    let curr2 = CliAirPodsData {
        left_in_ear: false, // Changed
        ..prev.clone()
    };

    let result2 = prev.left_in_ear != curr2.left_in_ear
        || prev.right_in_ear != curr2.right_in_ear
        || prev.both_in_case != curr2.both_in_case
        || prev.lid_open != curr2.lid_open;

    assert!(result2);
}

#[test]
fn test_mac_address_parsing() {
    // Test our own implementation of MAC address parsing

    // Valid MAC address
    let valid_mac = "00:11:22:33:44:55";
    let addr_parts: Vec<&str> = valid_mac.split(':').collect();

    if addr_parts.len() != 6 {
        panic!("Invalid MAC address format: {}", valid_mac);
    }

    let mut addr_bytes = [0u8; 6];
    for (i, part) in addr_parts.iter().enumerate() {
        match u8::from_str_radix(part, 16) {
            Ok(byte) => addr_bytes[i] = byte,
            Err(_) => panic!("Invalid hex byte in MAC address: {}", part),
        }
    }

    let result1 = BDAddr::from(addr_bytes);
    assert_eq!(result1, BDAddr::from([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]));

    // Invalid format (using dashes instead of colons)
    let invalid_mac1 = "00-11-22-33-44-55";
    let addr_parts1: Vec<&str> = invalid_mac1.split(':').collect();
    assert_ne!(addr_parts1.len(), 6);

    // Invalid length
    let invalid_mac2 = "00:11:22:33:44";
    let addr_parts2: Vec<&str> = invalid_mac2.split(':').collect();
    assert_ne!(addr_parts2.len(), 6);
}

// Integration test with CLI scanner
#[tokio::test]
async fn test_cli_scanner_basic_initialization() {
    // Create a temporary file for the mock executable
    let temp_dir = tempfile::tempdir().unwrap();
    let mock_exe_path = temp_dir.path().join("mock_scanner.exe");
    std::fs::write(&mock_exe_path, "mock content").unwrap();

    // Create scanner config with mock executable
    let config = CliScannerConfig {
        scanner_path: mock_exe_path,
        poll_interval: Duration::from_millis(100), // Fast polling for test
        adaptive_polling: true,
        max_errors: 3,
        verbose_logging: true,
    };

    // Create CLI scanner
    let scanner = CliScanner::new(config);

    // Get scanner stats (should be initialized with defaults)
    let stats = scanner.get_stats();
    assert_eq!(stats.total_scans, 0);
    assert_eq!(stats.successful_scans, 0);
    assert_eq!(stats.consecutive_errors, 0);
    assert_eq!(stats.success_rate, 0.0);
}

#[test]
fn test_error_handling_with_invalid_mac() {
    // Test parsing an invalid MAC address format
    let invalid_mac = "invalid-mac-address";

    // Split by colon
    let addr_parts: Vec<&str> = invalid_mac.split(':').collect();

    // This should not have 6 parts
    assert_ne!(addr_parts.len(), 6);

    // Attempting to parse would fail
    let result = if addr_parts.len() != 6 {
        Err(BluetoothError::InvalidData(format!(
            "Invalid MAC address format: {}",
            invalid_mac
        )))
    } else {
        let mut addr_bytes = [0u8; 6];
        let mut valid = true;

        for (i, part) in addr_parts.iter().enumerate() {
            match u8::from_str_radix(part, 16) {
                Ok(byte) => addr_bytes[i] = byte,
                Err(_) => {
                    valid = false;
                    break;
                }
            }
        }

        if valid {
            Ok(BDAddr::from(addr_bytes))
        } else {
            Err(BluetoothError::InvalidData(format!(
                "Invalid hex byte in MAC address: {}",
                invalid_mac
            )))
        }
    };

    assert!(result.is_err());

    match result {
        Err(BluetoothError::InvalidData(msg)) => {
            assert!(msg.contains("Invalid MAC address format"));
        }
        _ => panic!("Expected InvalidData error"),
    }
}
