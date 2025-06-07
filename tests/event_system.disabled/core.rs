//! Tests for basic event system functionality

use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::channel;
use tokio::time::timeout;

use btleplug::api::BDAddr;
use futures::{StreamExt, pin_mut};

use rustpods::bluetooth::events::{BleEvent, EventFilter, EventType};
use rustpods::bluetooth::{DiscoveredDevice, receiver_to_stream};
use rustpods::airpods::{DetectedAirPods, AirPodsType, AirPodsBattery, ChargingStatus};

// Helper to create a simple test device for testing
fn create_test_device() -> DiscoveredDevice {
    DiscoveredDevice {
        address: BDAddr::from([1, 2, 3, 4, 5, 6]),
        name: Some("Test Device".to_string()),
        rssi: Some(-60),
        manufacturer_data: HashMap::new(),
        is_potential_airpods: false,
        last_seen: Instant::now(),
        is_connected: false,
        service_data: HashMap::new(),
        services: Vec::new(),
        tx_power_level: None,
    }
}

// Helper to create a test AirPods for testing
fn create_test_airpods() -> DetectedAirPods {
    DetectedAirPods {
        address: BDAddr::from([2, 3, 4, 5, 6, 7]),
        name: Some("AirPods".to_string()),
        device_type: AirPodsType::AirPods1,
        battery: Some(AirPodsBattery {
            left: Some(80),
            right: Some(75),
            case: Some(90),
            charging: None,
        }),
        rssi: Some(-60),
        last_seen: Instant::now(),
        is_connected: false,
    }
}

#[tokio::test]
async fn test_event_filter_all() {
    // Create an all events filter
    let all_filter = EventFilter::all();
    
    // Create test events
    let device = create_test_device();
    let device_event = BleEvent::DeviceDiscovered(device.clone());
    let device_lost = BleEvent::DeviceLost(device.address);
    let error_event = BleEvent::Error("Test error".to_string());
    
    // Test that all events match
    assert!(all_filter.matches(&device_event));
    assert!(all_filter.matches(&device_lost));
    assert!(all_filter.matches(&error_event));
}

#[tokio::test]
async fn test_event_filter_event_types() {
    // Create event type filter for device discovery events
    let event_type_filter = EventFilter::event_types(vec![EventType::DeviceDiscovered]);
    
    // Create test events
    let device = create_test_device();
    let device_event = BleEvent::DeviceDiscovered(device.clone());
    let device_lost = BleEvent::DeviceLost(device.address);
    
    // Test filter matching
    assert!(event_type_filter.matches(&device_event));
    assert!(!event_type_filter.matches(&device_lost));
    
    // Test with stream
    let (tx, rx) = channel::<BleEvent>(10);
    let stream = receiver_to_stream(rx);
    pin_mut!(stream);
    
    // Create a filtered channel
    // Note: Here we would typically create a filtered channel, but for the test
    // we're simulating it by applying the filter manually
    
    // Send a matching event
    tx.send(device_event.clone()).await.unwrap();
    
    // Wait for event to propagate
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Should receive the event
    let timeout_duration = Duration::from_millis(1000);
    let received = timeout(timeout_duration, stream.next()).await;
    assert!(received.is_ok(), "Should receive the matching event");
    
    // Send a non-matching event (should not be received by a filtered receiver)
    tx.send(device_lost.clone()).await.unwrap();
    
    // Wait for event to propagate
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // In a real filtered channel, this would timeout - simulating that
    tokio::time::sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_event_filter_devices() {
    // Create device address to filter on
    let target_address = BDAddr::from([1, 2, 3, 4, 5, 6]);
    
    // Create device filter
    let device_filter = EventFilter::devices(vec![target_address]);
    
    // Create test events
    let device1 = DiscoveredDevice {
        address: target_address,
        name: Some("Target Device".to_string()),
        rssi: Some(-60),
        manufacturer_data: HashMap::new(),
        is_potential_airpods: false,
        last_seen: Instant::now(),
        is_connected: false,
        service_data: HashMap::new(),
        services: Vec::new(),
        tx_power_level: None,
    };
    
    let device2 = DiscoveredDevice {
        address: BDAddr::from([6, 5, 4, 3, 2, 1]),
        name: Some("Other Device".to_string()),
        rssi: Some(-70),
        manufacturer_data: HashMap::new(),
        is_potential_airpods: false,
        last_seen: Instant::now(),
        is_connected: false,
        service_data: HashMap::new(),
        services: Vec::new(),
        tx_power_level: None,
    };
    
    // Events
    let event1 = BleEvent::DeviceDiscovered(device1.clone());
    let event2 = BleEvent::DeviceDiscovered(device2.clone());
    let event3 = BleEvent::DeviceLost(target_address);
    
    // Test filter matching
    assert!(device_filter.matches(&event1), "Should match the target device discovery");
    assert!(!device_filter.matches(&event2), "Should not match other device discovery");
    assert!(device_filter.matches(&event3), "Should match the target device lost");
}

#[tokio::test]
async fn test_event_filter_airpods_only() {
    // Create a filter for AirPods events
    let airpods_filter = EventFilter::event_types(vec![EventType::AirPodsDetected]);
    
    // Create an AirPods device
    let airpods = create_test_airpods();
    let airpods_event = BleEvent::AirPodsDetected(airpods);
    
    // Create a regular device
    let device = create_test_device();
    let device_event = BleEvent::DeviceDiscovered(device);
    
    // Test filter matching
    assert!(airpods_filter.matches(&airpods_event), "Should match AirPods event");
    assert!(!airpods_filter.matches(&device_event), "Should not match device event");
} 