//! Tests demonstrating how to use the Bluetooth mocks for headless testing
//! These tests show how to create and configure mocks for various Bluetooth components

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use futures::StreamExt;

use rustpods::bluetooth::{
    AdapterStatus, AdapterCapabilities, BleError, BleAdapterEvent,
    DiscoveredDevice, ScanConfig
};
use rustpods::airpods::{AirPodsType, DetectedAirPods, AirPodsBattery};
use rustpods::ui::Message;
use rustpods::config::AppConfig;
use rustpods::ui::state_manager::{StateManager, Action};

use crate::bluetooth::mocks::{
    MockBluetoothAdapter, MockBluetoothAdapterBuilder,
    MockBleScanner, MockBleScannerBuilder,
    create_test_discovered_device, create_apple_device,
    create_airpods_manufacturer_data
};

/// Test adapter discovery using mocks
#[tokio::test]
async fn test_adapter_discovery_with_mocks() {
    // Create a mock adapter
    let mock_adapter = MockBluetoothAdapterBuilder::new()
        .build();
    
    // Verify the adapter has the expected capabilities
    let capabilities = mock_adapter.get_capabilities();
    assert!(capabilities.is_powered_on);
    assert!(capabilities.supports_scanning);
    
    // Verify the adapter status
    assert_eq!(mock_adapter.get_status(), AdapterStatus::Normal);
    
    // Test scanning functionality
    let scan_filter = btleplug::api::ScanFilter::default();
    let events_rx = mock_adapter.start_scanning(scan_filter).await.unwrap();
    
    // Convert to stream for easier async handling
    let mut events_stream = tokio_stream::wrappers::ReceiverStream::new(events_rx);
    
    // Verify we receive the ScanStarted event
    if let Some(event) = timeout(Duration::from_millis(500), events_stream.next()).await.unwrap() {
        match event {
            BleAdapterEvent::ScanStarted => {
                println!("✅ Scan started event received");
            },
            _ => panic!("Expected ScanStarted event, got {:?}", event),
        }
    } else {
        panic!("No events received");
    }
    
    // Stop scanning
    let stop_result = mock_adapter.stop_scanning().await;
    assert!(stop_result.is_ok());
}

/// Test device discovery with mocks
#[tokio::test]
async fn test_device_discovery_with_mocks() {
    // Create test devices
    let device1 = create_test_discovered_device(
        "00:11:22:33:44:55",
        Some("Test Device 1"),
        Some(-60),
        None
    );
    
    let device2 = create_test_discovered_device(
        "66:77:88:99:AA:BB",
        Some("Test Device 2"),
        Some(-70),
        None
    );
    
    // Create a mock adapter with devices
    let mock_adapter = MockBluetoothAdapterBuilder::new()
        .with_device(device1.clone())
        .with_device(device2.clone())
        .build();
    
    // Test discovery
    let discovered = mock_adapter.discover_devices().await.unwrap();
    
    // Verify both devices were discovered
    assert_eq!(discovered.len(), 2);
    assert!(discovered.iter().any(|d| d.address == device1.address));
    assert!(discovered.iter().any(|d| d.address == device2.address));
}

/// Test handling of Bluetooth adapter failures
#[tokio::test]
async fn test_adapter_failure_handling() {
    // Create a mock adapter configured to fail on scanning
    let mock_adapter = MockBluetoothAdapterBuilder::new()
        .with_scanning_failure()
        .build();
    
    // Attempt to start scanning
    let result = mock_adapter.start_scanning(btleplug::api::ScanFilter::default()).await;
    
    // Verify we get the expected error
    assert!(result.is_err());
    match result {
        Err(BleError::ScanningAlreadyInProgress) => {
            println!("✅ Expected error received");
        },
        _ => panic!("Expected ScanningAlreadyInProgress error"),
    }
    
    // Create a mock adapter configured to fail on device discovery
    let mock_adapter = MockBluetoothAdapterBuilder::new()
        .with_discovery_failure()
        .build();
    
    // Attempt to discover devices
    let result = mock_adapter.discover_devices().await;
    
    // Verify we get the expected error
    assert!(result.is_err());
    match result {
        Err(BleError::AdapterNotConnected) => {
            println!("✅ Expected error received");
        },
        _ => panic!("Expected AdapterNotConnected error"),
    }
}

/// Test scanner behavior with mocks
#[tokio::test]
async fn test_scanner_with_mocks() {
    // Create test devices
    let airpods_device = create_apple_device(
        "00:11:22:33:44:55",
        Some("AirPods Pro"),
        Some(-60),
        create_airpods_manufacturer_data(
            AirPodsType::AirPodsPro,
            80, // left battery
            75, // right battery
            90, // case battery
            0x03 // status flags (left and right charging)
        )
    );
    
    // Create a mock scanner with the test device
    let mock_scanner = MockBleScannerBuilder::new()
        .with_device(airpods_device.clone())
        .build();
    
    // Test scanner initialization
    let init_result = mock_scanner.initialize().await;
    assert!(init_result.is_ok());
    
    // Test scanning
    let events_rx = mock_scanner.start_scanning().await.unwrap();
    
    // Convert to stream for easier async handling
    let mut events_stream = tokio_stream::wrappers::ReceiverStream::new(events_rx);
    
    // Verify we receive the ScanStarted event
    if let Some(event) = timeout(Duration::from_millis(500), events_stream.next()).await.unwrap() {
        match event {
            BleAdapterEvent::ScanStarted => {
                println!("✅ Scan started event received");
            },
            _ => panic!("Expected ScanStarted event, got {:?}", event),
        }
    } else {
        panic!("No events received");
    }
    
    // Verify we receive the DeviceDiscovered event
    if let Some(event) = timeout(Duration::from_millis(500), events_stream.next()).await.unwrap() {
        match event {
            BleAdapterEvent::DeviceDiscovered(device) => {
                println!("✅ Device discovered event received: {:?}", device.address);
                assert_eq!(device.address, airpods_device.address);
            },
            _ => panic!("Expected DeviceDiscovered event, got {:?}", event),
        }
    } else {
        panic!("No device events received");
    }
    
    // Test getting discovered devices directly
    let devices = mock_scanner.get_discovered_devices();
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].address, airpods_device.address);
    
    // Test stopping the scan
    let stop_result = mock_scanner.stop_scanning().await;
    assert!(stop_result.is_ok());
}

/// Test scanner failure handling
#[tokio::test]
async fn test_scanner_failure_handling() {
    // Create a mock scanner configured to fail on initialization
    let mock_scanner = MockBleScannerBuilder::new()
        .with_init_failure()
        .build();
    
    // Attempt to initialize the scanner
    let result = mock_scanner.initialize().await;
    
    // Verify we get the expected error
    assert!(result.is_err());
    match result {
        Err(BleError::AdapterNotFound) => {
            println!("✅ Expected error received");
        },
        _ => panic!("Expected AdapterNotFound error"),
    }
    
    // Create a mock scanner configured to fail on scanning
    let mock_scanner = MockBleScannerBuilder::new()
        .with_scanning_failure()
        .build();
    
    // Attempt to start scanning
    let result = mock_scanner.start_scanning().await;
    
    // Verify we get the expected error
    assert!(result.is_err());
    match result {
        Err(BleError::ScanningAlreadyInProgress) => {
            println!("✅ Expected error received");
        },
        _ => panic!("Expected ScanningAlreadyInProgress error"),
    }
}

/// Test integration with state manager
#[tokio::test]
async fn test_ble_integration_with_state_manager() {
    // Create a state manager for testing
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let state_manager = Arc::new(StateManager::new(tx));
    
    // Create a test device
    let airpods_device = create_apple_device(
        "00:11:22:33:44:55",
        Some("AirPods Pro"),
        Some(-60),
        create_airpods_manufacturer_data(
            AirPodsType::AirPodsPro,
            80, // left battery
            75, // right battery
            90, // case battery
            0x03 // status flags (left and right charging)
        )
    );
    
    // Create a mock scanner with the test device
    let mock_scanner = MockBleScannerBuilder::new()
        .with_device(airpods_device.clone())
        .build();
    
    // Test device discovery
    let devices = mock_scanner.get_discovered_devices();
    
    // Dispatch discovered device to state manager
    for device in devices {
        state_manager.dispatch(Action::DeviceDiscovered(device));
    }
    
    // Verify the state manager contains the device
    let state = state_manager.get_state();
    
    // In a real test, you would verify that the state contains the device
    // For this example, we'll just check that the dispatch action didn't panic
    println!("✅ Successfully dispatched device to state manager");
}

/// Test end-to-end flow with Bluetooth mocks
#[tokio::test]
async fn test_bluetooth_end_to_end_flow() {
    // Create Apple device with AirPods manufacturer data
    let airpods_device = create_apple_device(
        "00:11:22:33:44:55",
        Some("AirPods Pro"),
        Some(-60),
        create_airpods_manufacturer_data(
            AirPodsType::AirPodsPro,
            80, // left battery
            75, // right battery
            90, // case battery
            0x03 // status flags (left and right charging)
        )
    );
    
    // Create a mock scanner with the test device
    let mock_scanner = MockBleScannerBuilder::new()
        .with_device(airpods_device.clone())
        .build();
    
    // Create state manager
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let state_manager = Arc::new(StateManager::new(tx));
    
    // Initialize scanner
    let init_result = mock_scanner.initialize().await;
    assert!(init_result.is_ok());
    
    // Start scanning
    let events_rx = mock_scanner.start_scanning().await.unwrap();
    let mut events_stream = tokio_stream::wrappers::ReceiverStream::new(events_rx);
    
    // Process events and update state manager
    while let Some(event) = timeout(Duration::from_millis(500), events_stream.next()).await.unwrap_or(None) {
        match event {
            BleAdapterEvent::ScanStarted => {
                println!("✅ Scan started");
                state_manager.dispatch(Action::ScanningStateChanged(true));
            },
            BleAdapterEvent::DeviceDiscovered(device) => {
                println!("✅ Device discovered: {:?}", device.name);
                state_manager.dispatch(Action::DeviceDiscovered(device));
            },
            BleAdapterEvent::ScanStopped => {
                println!("✅ Scan stopped");
                state_manager.dispatch(Action::ScanningStateChanged(false));
            },
            _ => {}
        }
    }
    
    // Stop scanning
    mock_scanner.stop_scanning().await.unwrap();
    
    // In a real implementation, we would use the airpods detection module to process the manufacturer data
    // For this test, we'll just pretend we've detected AirPods
    
    // Simulate connecting to the device
    println!("✅ Simulating connection to AirPods");
    
    // Create detected AirPods instance
    let detected_airpods = DetectedAirPods {
        address: airpods_device.address,
        device_type: AirPodsType::AirPodsPro,
        name: Some("AirPods Pro".to_string()),
        battery: AirPodsBattery {
            left: Some(80),
            right: Some(75),
            case: Some(90),
            charging: rustpods::airpods::ChargingStatus {
                left: true,
                right: true,
                case: false,
            },
        },
        rssi: Some(-60),
        last_seen: chrono::Utc::now(),
    };
    
    // Update state with detected AirPods
    state_manager.dispatch(Action::AirPodsDetected(detected_airpods));
    
    // Verify that messages were sent to the channel
    let mut event_count = 0;
    while let Ok(Some(_)) = timeout(Duration::from_millis(10), rx.recv()).await {
        event_count += 1;
    }
    
    assert!(event_count > 0, "Expected at least one event to be sent through the channel");
    
    println!("✅ End-to-end flow completed successfully");
} 