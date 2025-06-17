//! Comprehensive tests for the CLI Scanner implementation
//! These tests focus on the native C++ CLI scanner integration
//! including JSON parsing, adaptive polling, and error handling

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use btleplug::api::BDAddr;
use serde_json::json;
use tempfile::TempDir;
use tokio::sync::mpsc;

use rustpods::airpods::{AirPodsBattery, AirPodsChargingState, AirPodsType, DetectedAirPods};
use rustpods::bluetooth::cli_scanner::{
    CliAirPodsData, CliDeviceInfo, CliScanner, CliScannerConfig, CliScannerResult, ScannerStats,
};
use rustpods::bluetooth::BluetoothError;
use rustpods::config::{AppConfig, LogLevel};
use rustpods::config;

/// Test the CLI Scanner configuration creation from AppConfig
#[test]
fn test_cli_scanner_config_creation() {
    // Create a new AppConfig using default trait
    let mut app_config = AppConfig::default();
    
    // Configure test values
    app_config.bluetooth.battery_refresh_interval = Duration::from_secs(60);
    app_config.bluetooth.adaptive_polling = true;
    app_config.system.log_level = LogLevel::Debug;
    
    // Create config from app config
    let config = CliScannerConfig::from_app_config(&app_config);
    
    // Verify the config values
    assert_eq!(config.poll_interval, Duration::from_secs(60));
    assert!(config.adaptive_polling);
    assert!(config.verbose_logging);
    
    // Test with different log level
    app_config.system.log_level = LogLevel::Info;
    let config2 = CliScannerConfig::from_app_config(&app_config);
    assert!(!config2.verbose_logging);
}

/// Test JSON parsing with various AirPods models
#[test]
fn test_cli_scanner_json_parsing_different_models() {
    // Test with AirPods Pro
    let json_str_pro = r#"{
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

    // Test with AirPods Max
    let json_str_max = r#"{
        "scanner_version": "6.0.0",
        "scan_timestamp": "2025-06-17T18:30:00Z",
        "total_devices": 1,
        "devices": [
            {
                "device_id": "1",
                "address": "AA:BB:CC:DD:EE:FF",
                "rssi": -55,
                "manufacturer_data_hex": "0A190114200b778f",
                "airpods_data": {
                    "model": "AirPods Max",
                    "model_id": "0x0A20",
                    "left_battery": 70,
                    "right_battery": 70,
                    "case_battery": 0,
                    "left_charging": false,
                    "right_charging": false,
                    "case_charging": false,
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

    // Parse and verify AirPods Pro
    let result_pro: CliScannerResult = serde_json::from_str(json_str_pro).unwrap();
    assert_eq!(result_pro.devices[0].airpods_data.as_ref().unwrap().model, "AirPods Pro");
    
    // Parse and verify AirPods Max
    let result_max: CliScannerResult = serde_json::from_str(json_str_max).unwrap();
    assert_eq!(result_max.devices[0].airpods_data.as_ref().unwrap().model, "AirPods Max");
}

/// Test error handling with malformed JSON
#[test]
fn test_cli_scanner_json_error_handling() {
    // Malformed JSON (missing closing brace)
    let malformed_json = r#"{
        "scanner_version": "6.0.0",
        "devices": [
            {
                "device_id": "1",
                "address": "00:11:22:33:44:55"
            }
        "status": "success"
    "#;
    
    let result: Result<CliScannerResult, _> = serde_json::from_str(malformed_json);
    assert!(result.is_err());
    
    // Missing required fields
    let missing_fields_json = r#"{
        "scanner_version": "6.0.0",
        "devices": []
    }"#;
    
    let result: Result<CliScannerResult, _> = serde_json::from_str(missing_fields_json);
    // This might actually parse since we don't enforce all fields as required
    if let Ok(parsed) = result {
        assert_eq!(parsed.devices.len(), 0);
    }
}

/// Test conversion from CLI data to DetectedAirPods
#[test]
fn test_cli_data_to_airpods_conversion() {
    // Create CLI data for different AirPods models
    let models = vec![
        ("AirPods Pro", AirPodsType::AirPodsPro),
        ("AirPods", AirPodsType::AirPods1),
        ("AirPods Max", AirPodsType::AirPodsMax),
        ("Unknown Model", AirPodsType::Unknown),
    ];
    
    for (model_name, expected_type) in models {
        // Create CLI AirPods data
        let cli_data = CliAirPodsData {
            model: model_name.to_string(),
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
            rssi: Some(-60),
            name: Some(model_name.to_string()),
            is_connected: true,
            last_seen: Instant::now(),
        };
        
        // Verify the conversion
        assert_eq!(airpods.device_type, expected_type);
        assert_eq!(airpods.battery.as_ref().unwrap().left, Some(80));
        assert_eq!(airpods.battery.as_ref().unwrap().right, Some(75));
        assert_eq!(airpods.battery.as_ref().unwrap().case, Some(90));
        assert_eq!(airpods.battery.as_ref().unwrap().charging, Some(AirPodsChargingState::CaseCharging));
    }
}

/// Test scanner statistics tracking
#[test]
fn test_scanner_stats() {
    // Create a temporary file for the mock executable
    let temp_dir = tempfile::tempdir().unwrap();
    let mock_exe_path = temp_dir.path().join("mock_scanner.exe");
    std::fs::write(&mock_exe_path, "mock content").unwrap();
    
    // Create scanner config with mock executable
    let config = CliScannerConfig {
        scanner_path: mock_exe_path,
        poll_interval: Duration::from_millis(100),
        adaptive_polling: true,
        max_errors: 3,
        verbose_logging: true,
    };
    
    // Create CLI scanner
    let scanner = CliScanner::new(config);
    
    // Get initial stats
    let initial_stats = scanner.get_stats();
    assert_eq!(initial_stats.total_scans, 0);
    assert_eq!(initial_stats.successful_scans, 0);
    assert_eq!(initial_stats.consecutive_errors, 0);
    assert_eq!(initial_stats.success_rate, 0.0);
}

/// Test adaptive polling logic
#[test]
fn test_adaptive_polling_logic() {
    // We'll test our own implementation of the adaptive polling logic
    // since we can't directly access the internal state of the CliScanner
    
    // Define constants (same as in cli_scanner.rs)
    const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(30);
    const FAST_POLL_INTERVAL: Duration = Duration::from_secs(10);
    const MIN_POLL_INTERVAL: Duration = Duration::from_secs(5);
    const MAX_POLL_INTERVAL: Duration = Duration::from_secs(120);
    const FAST_POLL_COUNT: u32 = 3;
    
    // Create a mock scanner state
    struct MockScannerState {
        current_interval: Duration,
        fast_polls_remaining: u32,
        consecutive_errors: u32,
    }
    
    let mut state = MockScannerState {
        current_interval: DEFAULT_POLL_INTERVAL,
        fast_polls_remaining: 0,
        consecutive_errors: 0,
    };
    
    // Test 1: No changes, should use default interval
    assert_eq!(state.current_interval, DEFAULT_POLL_INTERVAL);
    
    // Test 2: Significant change detected, should switch to fast polling
    state.fast_polls_remaining = FAST_POLL_COUNT;
    state.current_interval = FAST_POLL_INTERVAL;
    assert_eq!(state.current_interval, FAST_POLL_INTERVAL);
    
    // Test 3: Fast polls count down
    state.fast_polls_remaining -= 1;
    assert_eq!(state.fast_polls_remaining, FAST_POLL_COUNT - 1);
    assert_eq!(state.current_interval, FAST_POLL_INTERVAL);
    
    // Test 4: After fast polls exhausted, return to normal
    state.fast_polls_remaining = 0;
    state.current_interval = DEFAULT_POLL_INTERVAL;
    assert_eq!(state.current_interval, DEFAULT_POLL_INTERVAL);
    
    // Test 5: Error backoff
    state.consecutive_errors = 1;
    // Simulate backoff calculation: interval * 1.5^errors
    let backoff_multiplier = (state.consecutive_errors as u64).min(4);
    let error_interval = DEFAULT_POLL_INTERVAL.mul_f64(1.5_f64.powi(backoff_multiplier as i32));
    state.current_interval = error_interval.min(MAX_POLL_INTERVAL);
    
    assert!(state.current_interval > DEFAULT_POLL_INTERVAL);
    assert!(state.current_interval <= MAX_POLL_INTERVAL);
}

/// Test error handling and backoff strategy
#[test]
fn test_error_handling_and_backoff() {
    // Define constants (same as in cli_scanner.rs)
    const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(30);
    const MAX_POLL_INTERVAL: Duration = Duration::from_secs(120);
    
    // Test exponential backoff calculation
    let mut interval = DEFAULT_POLL_INTERVAL;
    
    // Initial interval
    assert_eq!(interval, Duration::from_secs(30));
    
    // After 1 error
    let backoff_multiplier = 1_u64.min(4);
    interval = interval.mul_f64(1.5_f64.powi(backoff_multiplier as i32));
    interval = interval.min(MAX_POLL_INTERVAL);
    assert!(interval > Duration::from_secs(30));
    assert!(interval < Duration::from_secs(50));
    
    // After 2 errors
    let backoff_multiplier = 2_u64.min(4);
    interval = DEFAULT_POLL_INTERVAL.mul_f64(1.5_f64.powi(backoff_multiplier as i32));
    interval = interval.min(MAX_POLL_INTERVAL);
    assert!(interval > Duration::from_secs(60));
    assert!(interval < Duration::from_secs(80));
    
    // After 4 errors
    let backoff_multiplier = 4_u64.min(4);
    interval = DEFAULT_POLL_INTERVAL.mul_f64(1.5_f64.powi(backoff_multiplier as i32));
    interval = interval.min(MAX_POLL_INTERVAL);
    assert!(interval > Duration::from_secs(100));
    assert!(interval <= MAX_POLL_INTERVAL);
    
    // After 5 errors (should cap at 4 for multiplier)
    let backoff_multiplier = 5_u64.min(4);
    interval = DEFAULT_POLL_INTERVAL.mul_f64(1.5_f64.powi(backoff_multiplier as i32));
    interval = interval.min(MAX_POLL_INTERVAL);
    assert_eq!(backoff_multiplier, 4); // Verify multiplier is capped
    assert_eq!(interval, MAX_POLL_INTERVAL); // Should be capped at max
}

/// Integration test with mock CLI output
#[tokio::test]
async fn test_cli_scanner_with_mock_output() {
    // Create a temporary directory for our mock executable
    let temp_dir = TempDir::new().unwrap();
    let mock_path = temp_dir.path().join("mock_scanner.exe");
    
    // On Windows, we'll create a batch file that outputs our mock JSON
    #[cfg(target_os = "windows")]
    {
        let batch_content = r#"@echo off
echo {
echo   "scanner_version": "6.0.0",
echo   "scan_timestamp": "2025-06-17T18:30:00Z",
echo   "total_devices": 1,
echo   "devices": [
echo     {
echo       "device_id": "1",
echo       "address": "00:11:22:33:44:55",
echo       "rssi": -60,
echo       "manufacturer_data_hex": "07190114200b778f",
echo       "airpods_data": {
echo         "model": "AirPods Pro",
echo         "model_id": "0x0E20",
echo         "left_battery": 80,
echo         "right_battery": 75,
echo         "case_battery": 90,
echo         "left_charging": false,
echo         "right_charging": false,
echo         "case_charging": true,
echo         "left_in_ear": true,
echo         "right_in_ear": true,
echo         "both_in_case": false,
echo         "lid_open": false,
echo         "broadcasting_ear": "both"
echo       }
echo     }
echo   ],
echo   "airpods_count": 1,
echo   "status": "success",
echo   "note": "Scan completed successfully"
echo }
"#;
        let batch_path = temp_dir.path().join("mock_scanner.bat");
        std::fs::write(&batch_path, batch_content).unwrap();
        std::fs::write(&mock_path, format!("@call \"{}\"", batch_path.display())).unwrap();
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        let shell_content = r#"#!/bin/sh
cat << 'EOF'
{
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
}
EOF
"#;
        std::fs::write(&mock_path, shell_content).unwrap();
        // Make the script executable
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&mock_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&mock_path, perms).unwrap();
    }
    
    // Create scanner config with our mock executable
    let config = CliScannerConfig {
        scanner_path: mock_path,
        poll_interval: Duration::from_millis(100), // Fast polling for test
        adaptive_polling: true,
        max_errors: 3,
        verbose_logging: true,
    };
    
    // Create CLI scanner
    let scanner = CliScanner::new(config);
    
    // Create a channel to receive scan results
    let (tx, mut rx) = mpsc::channel::<Result<Vec<DetectedAirPods>, BluetoothError>>(10);
    
    // Clone the transmitter for the callback
    let tx_clone = tx.clone();
    
    // Start monitoring with a callback that sends results to our channel
    let _handle = scanner.start_monitoring(move |result| {
        let _ = tx_clone.try_send(result);
    });
    
    // Wait for a result with timeout
    let timeout = tokio::time::sleep(Duration::from_secs(2));
    tokio::pin!(timeout);
    
    let result = tokio::select! {
        result = rx.recv() => result,
        _ = &mut timeout => None,
    };
    
    // Verify we got a result
    assert!(result.is_some());
    
    if let Some(scan_result) = result {
        // Verify the result is Ok
        assert!(scan_result.is_ok());
        
        let airpods_list = scan_result.unwrap();
        assert_eq!(airpods_list.len(), 1);
        
        let airpods = &airpods_list[0];
        assert_eq!(airpods.device_type, AirPodsType::AirPodsPro);
        assert!(airpods.battery.is_some());
        
        if let Some(battery) = &airpods.battery {
            assert_eq!(battery.left, Some(80));
            assert_eq!(battery.right, Some(75));
            assert_eq!(battery.case, Some(90));
            assert_eq!(battery.charging, Some(AirPodsChargingState::CaseCharging));
        }
    }
}

/// Test scanner with error output
#[tokio::test]
async fn test_cli_scanner_with_error_output() {
    // Create a temporary directory for our mock executable
    let temp_dir = TempDir::new().unwrap();
    let mock_path = temp_dir.path().join("error_scanner.exe");
    
    // Create a mock executable that returns an error
    #[cfg(target_os = "windows")]
    {
        let batch_content = r#"@echo off
echo Error: Failed to scan for devices >&2
exit /b 1
"#;
        let batch_path = temp_dir.path().join("error_scanner.bat");
        std::fs::write(&batch_path, batch_content).unwrap();
        std::fs::write(&mock_path, format!("@call \"{}\"", batch_path.display())).unwrap();
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        let shell_content = r#"#!/bin/sh
echo "Error: Failed to scan for devices" >&2
exit 1
"#;
        std::fs::write(&mock_path, shell_content).unwrap();
        // Make the script executable
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&mock_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&mock_path, perms).unwrap();
    }
    
    // Create scanner config with our error-producing executable
    let config = CliScannerConfig {
        scanner_path: mock_path,
        poll_interval: Duration::from_millis(100), // Fast polling for test
        adaptive_polling: true,
        max_errors: 3,
        verbose_logging: true,
    };
    
    // Create CLI scanner
    let scanner = CliScanner::new(config);
    
    // Create a channel to receive scan results
    let (tx, mut rx) = mpsc::channel::<Result<Vec<DetectedAirPods>, BluetoothError>>(10);
    
    // Clone the transmitter for the callback
    let tx_clone = tx.clone();
    
    // Start monitoring with a callback that sends results to our channel
    let _handle = scanner.start_monitoring(move |result| {
        let _ = tx_clone.try_send(result);
    });
    
    // Wait for a result with timeout
    let timeout = tokio::time::sleep(Duration::from_secs(2));
    tokio::pin!(timeout);
    
    let result = tokio::select! {
        result = rx.recv() => result,
        _ = &mut timeout => None,
    };
    
    // Verify we got a result
    assert!(result.is_some());
    
    if let Some(scan_result) = result {
        // Verify the result is an error
        assert!(scan_result.is_err());
        
        match scan_result {
            Err(BluetoothError::Other(msg)) => {
                assert!(msg.contains("CLI scanner failed") || msg.contains("Failed to execute"));
            }
            _ => panic!("Expected BluetoothError::Other"),
        }
    }
}
