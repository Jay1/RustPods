//! Integration tests for Bluetooth scanner functionality

use rustpods::bluetooth::{BleScanner, BleEvent, ScanConfig, DiscoveredDevice};
use rustpods::error::BluetoothError;
use tokio::sync::mpsc::Receiver;
use tokio::time::{timeout, Duration};
use std::collections::HashMap;
use std::time::Duration as StdDuration;

/// Create a test helper to receive the next BLE event with a timeout
async fn receive_next_event(rx: &mut Receiver<BleEvent>, duration: Duration) -> Option<BleEvent> {
    match timeout(duration, rx.recv()).await {
        Ok(Some(event)) => Some(event),
        _ => None,
    }
}

/// Test scanner creation and basic properties
#[tokio::test]
async fn test_scanner_creation() {
    // Create a new BLE scanner
    let mut scanner = BleScanner::new();
    
    // Initialize the scanner
    let result = scanner.initialize().await;
    
    // Verify scanner initialization succeeds on systems with Bluetooth
    // This test may need to be skipped on systems without BT hardware
    if result.is_ok() {
        // Verify initial state
        assert!(!scanner.is_scanning());
    } else {
        // On systems without Bluetooth, this test can be skipped
        println!("Skipping scanner test as no BT adapters were found");
    }
}

/// Test scanner configuration
#[tokio::test]
async fn test_scanner_config() {
    // Create a custom scan configuration
    let config = ScanConfig {
        scan_duration: StdDuration::from_secs(5),
        min_rssi: Some(-80),
        ..Default::default()
    };
    
    // Create a scanner with the custom config
    let mut scanner = BleScanner::with_config(config.clone());
    
    // Initialize the scanner
    if scanner.initialize().await.is_ok() {
        // Verify config was applied
        assert_eq!(scanner.get_config().scan_duration, StdDuration::from_secs(5));
        assert_eq!(scanner.get_config().min_rssi, Some(-80));
    }
}

/// Test starting and stopping the scanner
#[tokio::test]
async fn test_scanner_start_stop() {
    // Create a new scanner with short scan duration
    let config = ScanConfig {
        scan_duration: StdDuration::from_secs(2),
        ..Default::default()
    };
    
    // Create scanner and initialize it
    let mut scanner = BleScanner::with_config(config);
    
    if scanner.initialize().await.is_ok() {
        // Start scanning
        let mut rx = scanner.start_scanning().await.unwrap();
        
        // Verify scanner is running
        assert!(scanner.is_scanning());
        
        // Verify we receive scan start event
        if let Some(BleEvent::ScanStarted) = receive_next_event(&mut rx, Duration::from_secs(1)).await {
            // Expected event
        } else {
            panic!("Did not receive ScanStarted event");
        }
        
        // Stop scanning
        scanner.stop_scanning().await.unwrap();
        
        // Verify scanner stopped
        assert!(!scanner.is_scanning());
        
        // Verify we receive scan stopped event
        if let Some(BleEvent::ScanStopped) = receive_next_event(&mut rx, Duration::from_secs(1)).await {
            // Expected event
        } else {
            panic!("Did not receive ScanStopped event");
        }
    }
}

/// Test device discovery during scanning
#[tokio::test]
async fn test_device_discovery() {
    // Create a config that's likely to discover some devices
    let config = ScanConfig {
        scan_duration: StdDuration::from_secs(5),
        min_rssi: Some(-90), // Lower threshold to detect more devices
        ..Default::default()
    };
    
    // Create and initialize scanner
    let mut scanner = BleScanner::with_config(config);
    
    if scanner.initialize().await.is_ok() {
        // Start scanning
        let mut rx = scanner.start_scanning().await.unwrap();
        
        // Wait for scan started event
        if let Some(BleEvent::ScanStarted) = receive_next_event(&mut rx, Duration::from_secs(1)).await {
            println!("Scan started");
        } else {
            panic!("Did not receive ScanStarted event");
        }
        
        // Wait for device discovery events (with a timeout)
        let mut device_found = false;
        let mut timeout_duration = Duration::from_secs(6);
        
        println!("Waiting for device discovery events...");
        
        while let Some(event) = receive_next_event(&mut rx, timeout_duration).await {
            match event {
                BleEvent::DeviceDiscovered(device) => {
                    println!("Device discovered: {:?} (RSSI: {:?})", device.name.unwrap_or_else(|| device.address.to_string()), device.rssi);
                    device_found = true;
                    // Only wait a short time for additional devices
                    timeout_duration = Duration::from_millis(500);
                },
                BleEvent::DeviceUpdated(device) => {
                    println!("Device updated: {:?} (RSSI: {:?})", device.name.unwrap_or_else(|| device.address.to_string()), device.rssi);
                    device_found = true;
                    timeout_duration = Duration::from_millis(500);
                },
                BleEvent::ScanStopped => {
                    println!("Scan stopped");
                    break;
                },
                _ => {}
            }
        }
        
        // Stop scanning if still running
        if scanner.is_scanning() {
            scanner.stop_scanning().await.unwrap();
        }
        
        // This test is informational rather than assertive
        // Not finding devices shouldn't fail the test as it depends on environment
        if device_found {
            println!("✅ Successfully discovered Bluetooth devices");
        } else {
            println!("⚠️ No devices found during scan - this is not an error if no BLE devices are nearby");
        }
    } else {
        println!("Skipping device discovery test as scanner couldn't be initialized");
    }
}

/// Test RSSI filtering during scanning
#[tokio::test]
async fn test_rssi_filtering() {
    // Create two scanners with different RSSI thresholds
    let permissive_config = ScanConfig {
        scan_duration: StdDuration::from_secs(3),
        min_rssi: Some(-90), // Lower threshold to detect more devices
        ..Default::default()
    };
    
    let strict_config = ScanConfig {
        scan_duration: StdDuration::from_secs(3),
        min_rssi: Some(-30), // Very high threshold (most devices won't meet this)
        ..Default::default()
    };
    
    // Test with permissive scanner first
    let mut permissive_scanner = BleScanner::with_config(permissive_config);
    
    if permissive_scanner.initialize().await.is_ok() {
        let mut permissive_rx = permissive_scanner.start_scanning().await.unwrap();
        
        // Count devices found with permissive settings
        let mut permissive_device_count = 0;
        let mut timeout_duration = Duration::from_secs(4);
        
        while let Some(event) = receive_next_event(&mut permissive_rx, timeout_duration).await {
            match event {
                BleEvent::DeviceDiscovered(_) => {
                    permissive_device_count += 1;
                    timeout_duration = Duration::from_millis(500);
                },
                BleEvent::ScanStopped => break,
                _ => {}
            }
        }
        
        // Stop first scanner
        if permissive_scanner.is_scanning() {
            permissive_scanner.stop_scanning().await.unwrap();
        }
        
        println!("Permissive scanner found {} devices", permissive_device_count);
        
        // Now test with strict scanner
        let mut strict_scanner = BleScanner::with_config(strict_config);
        
        if strict_scanner.initialize().await.is_ok() {
            let mut strict_rx = strict_scanner.start_scanning().await.unwrap();
            
            // Count devices found with strict settings
            let mut strict_device_count = 0;
            let mut timeout_duration = Duration::from_secs(4);
            
            while let Some(event) = receive_next_event(&mut strict_rx, timeout_duration).await {
                match event {
                    BleEvent::DeviceDiscovered(_) => {
                        strict_device_count += 1;
                        timeout_duration = Duration::from_millis(500);
                    },
                    BleEvent::ScanStopped => break,
                    _ => {}
                }
            }
            
            // Stop second scanner
            if strict_scanner.is_scanning() {
                strict_scanner.stop_scanning().await.unwrap();
            }
            
            println!("Strict scanner found {} devices", strict_device_count);
            
            // We expect the strict scanner to find fewer devices
            // But this is not a hard assertion as it depends on the environment
            if strict_device_count <= permissive_device_count {
                println!("✅ RSSI filtering behaving as expected");
            } else {
                println!("⚠️ Unexpected RSSI filtering behavior (strict found more than permissive)");
            }
        }
    }
}

/// Test error handling when Bluetooth is unavailable
#[tokio::test]
async fn test_scanner_error_handling() {
    // Create a scanner
    let mut scanner = BleScanner::new();
    
    // First initialize the scanner
    let init_result = scanner.initialize().await;
    
    // If initialization fails, the test can't meaningfully continue on this system
    if init_result.is_err() {
        println!("Scanner initialization failed, skipping test_scanner_error_handling");
        return;
    }
    
    // Start scanning
    let scan_result = scanner.start_scanning().await;
    
    // This should succeed on systems with Bluetooth
    assert!(scan_result.is_ok(), "Scan should start successfully after initialization");
    
    // Start again (should fail because already scanning)
    let second_scan = scanner.start_scanning().await;
    assert!(second_scan.is_err(), "Should not be able to start scanning when already scanning");
    
    // Check error type - should be ScanFailed with "already scanning" message
    if let Err(err) = second_scan {
        match err {
            BluetoothError::ScanFailed(msg) => {
                assert!(msg.contains("already") || msg.contains("in progress"), 
                       "Error should indicate scan is already in progress, got: {}", msg);
            },
            _ => panic!("Expected ScanFailed error, got: {:?}", err),
        }
    }
    
    // Stop scanning
    let stop_result = scanner.stop_scanning().await;
    assert!(stop_result.is_ok(), "Should be able to stop scanning");
    
    // Stop again (may or may not fail depending on implementation)
    let second_stop = scanner.stop_scanning().await;
    println!("Second stop result: {:?}", second_stop);
}

/// Test adapter information retrieval
#[tokio::test]
async fn test_adapter_info() {
    // Create a scanner
    let mut scanner = BleScanner::new();
    
    // Initialize the scanner
    if scanner.initialize().await.is_ok() {
        // Get peripherals by address should work, even with invalid address
        // It should return an empty list, not an error
        let invalid_addr = btleplug::api::BDAddr::from([0, 0, 0, 0, 0, 0]);
        let peripherals = scanner.get_peripherals_by_address(&invalid_addr).await;
        
        // Check that we get a valid result (empty list)
        assert!(peripherals.is_ok(), "Should not error on invalid address");
        assert!(peripherals.unwrap().is_empty(), "Should return empty list for invalid address");
    } else {
        println!("Skipping test_adapter_info as scanner initialization failed");
    }
}

/// Test device connection with mock
#[tokio::test]
async fn test_device_connection_mock() {
    // This is a placeholder for tests that would use mocks
    // Actual implementation would require a mock framework
    println!("Mock connection tests should be implemented in a separate test file");
} 