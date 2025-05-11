//! Integration tests for Bluetooth scanner functionality

use rustpods::bluetooth::{BleScanner, BleEvent, ScanConfig, DiscoveredDevice};
use tokio::sync::mpsc::Receiver;
use tokio::time::{timeout, Duration};
use std::collections::HashMap;

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
    let scanner_result = BleScanner::new().await;
    
    // Verify scanner creation succeeds on systems with Bluetooth
    // This test may need to be skipped on systems without BT hardware
    if let Ok(scanner) = scanner_result {
        // Verify initial state
        assert!(!scanner.is_scanning());
        
        // Check if any adapters were found
        assert!(scanner.has_adapters());
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
        scan_duration: 5,
        min_rssi: -80,
        ..Default::default()
    };
    
    // Create a scanner with the custom config
    if let Ok(scanner) = BleScanner::with_config(config.clone()).await {
        // Verify config was applied
        assert_eq!(scanner.config().scan_duration, 5);
        assert_eq!(scanner.config().min_rssi, -80);
    }
}

/// Test starting and stopping the scanner
#[tokio::test]
async fn test_scanner_start_stop() {
    // Create a new scanner with short scan duration
    let config = ScanConfig {
        scan_duration: 2,
        ..Default::default()
    };
    
    if let Ok(mut scanner) = BleScanner::with_config(config).await {
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
    // Skip this test if run in a CI environment without real Bluetooth hardware
    if std::env::var("CI").is_ok() {
        println!("Skipping test_device_discovery in CI environment");
        return;
    }

    // Create a scanner with 5-second scan duration
    let config = ScanConfig {
        scan_duration: 5,
        min_rssi: -90, // Lower threshold to detect more devices
        ..Default::default()
    };
    
    if let Ok(mut scanner) = BleScanner::with_config(config).await {
        // Start scanning
        let mut rx = scanner.start_scanning().await.unwrap();
        
        // Create a map to track discovered devices
        let mut discovered_devices = HashMap::new();
        
        // Listen for events for 5 seconds
        let start_time = std::time::Instant::now();
        let timeout_duration = Duration::from_secs(6);
        
        while let Some(event) = receive_next_event(&mut rx, Duration::from_secs(1)).await {
            match event {
                BleEvent::DeviceDiscovered(device) => {
                    // Add device to our map
                    discovered_devices.insert(device.address, device);
                },
                BleEvent::ScanStopped => break,
                _ => {}
            }
            
            // Check if we've been running for too long
            if start_time.elapsed() > timeout_duration {
                break;
            }
        }
        
        // Stop scanning (if not already stopped)
        if scanner.is_scanning() {
            scanner.stop_scanning().await.unwrap();
        }
        
        // In a real environment with Bluetooth devices nearby, we should have found some
        // But since this is unpredictable, just print the count rather than asserting
        println!("Discovered {} devices during test", discovered_devices.len());
        
        // Print device info for debugging
        for (_, device) in discovered_devices {
            println!("Device: {} RSSI: {:?}", 
                     device.name.unwrap_or_else(|| "Unknown".to_string()), 
                     device.rssi);
        }
    }
}

/// Test filtering devices by RSSI
#[tokio::test]
async fn test_rssi_filtering() {
    // Create a scanner with high RSSI threshold (-30 is very strong, most devices won't meet this)
    let config = ScanConfig {
        scan_duration: 3,
        min_rssi: -30, // Very high threshold (most devices won't meet this)
        ..Default::default()
    };
    
    if let Ok(mut scanner) = BleScanner::with_config(config).await {
        // Start scanning
        let mut rx = scanner.start_scanning().await.unwrap();
        
        // Track discovered devices
        let mut strong_devices = 0;
        
        // Listen for events
        while let Some(event) = receive_next_event(&mut rx, Duration::from_secs(1)).await {
            if let BleEvent::DeviceDiscovered(device) = event {
                // Count devices that pass the filter
                strong_devices += 1;
                
                // Verify the RSSI is above our threshold
                assert!(device.rssi.unwrap_or(-100) >= -30);
            } else if let BleEvent::ScanStopped = event {
                break;
            }
        }
        
        // Stop scanning if not already stopped
        if scanner.is_scanning() {
            scanner.stop_scanning().await.unwrap();
        }
        
        // Just log the count - we expect few or none due to high threshold
        println!("Found {} devices with RSSI >= -30", strong_devices);
    }
}

/// Test scanner error handling (start scanning twice)
#[tokio::test]
async fn test_scanner_error_handling() {
    // Create scanner with short duration
    if let Ok(mut scanner) = BleScanner::new().await {
        // Start scanning first time
        let _rx1 = scanner.start_scanning().await.unwrap();
        
        // Try to start scanning again while already scanning
        let result = scanner.start_scanning().await;
        
        // Expect an error
        assert!(result.is_err(), "Starting scanner twice should return an error");
        
        // Stop scanning
        scanner.stop_scanning().await.unwrap();
    }
}

/// Test adapter information
#[tokio::test]
async fn test_adapter_info() {
    if let Ok(scanner) = BleScanner::new().await {
        // Get adapter info
        let adapters = scanner.adapters();
        
        // If we have adapters, verify they have basic information
        if !adapters.is_empty() {
            for adapter in adapters {
                // Check adapter has an address
                assert!(!adapter.address.to_string().is_empty(), "Adapter should have an address");
                
                // Most adapters should have a name
                if let Some(name) = &adapter.name {
                    println!("Found adapter: {}", name);
                    assert!(!name.is_empty(), "Adapter name should not be empty");
                }
            }
        } else {
            println!("No Bluetooth adapters found");
        }
    }
}

/// Mock test for device connection (using test helpers to create a mock device)
#[tokio::test]
async fn test_device_connection_mock() {
    // This test demonstrates how to use a mock device for testing
    // In a real implementation, you would use a mock framework like mockall
    
    // Create a fake device for testing
    let device = DiscoveredDevice {
        address: "00:11:22:33:44:55".parse().unwrap(),
        name: Some("Test Device".to_string()),
        rssi: Some(-60),
        manufacturer_data: HashMap::new(),
        services: vec![],
    };
    
    // Verify device properties
    assert_eq!(device.name, Some("Test Device".to_string()));
    assert_eq!(device.rssi, Some(-60));
    
    // In a real test, we would connect to this device
    // For now, just demonstrate the concept
    println!("Would connect to device: {}", device.address);
} 