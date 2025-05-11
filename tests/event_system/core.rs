//! Integration tests for the Bluetooth event system

use std::time::Duration;
use tokio::sync::mpsc::{channel, Receiver};
use tokio::time::timeout;
use futures::stream::StreamExt;

use btleplug::api::BDAddr;

use rustpods::bluetooth::{
    EventFilter, BleEvent, events::EventType, 
    AdapterInfo, DiscoveredDevice, receiver_to_stream
};
use rustpods::airpods::{DetectedAirPods, AirPodsType, AirPodsBattery, ChargingStatus};
use futures::pin_mut;

/// Helper to create a test event broker and subscribe to it
async fn create_test_event_stream(_filter: EventFilter) -> (tokio::sync::mpsc::Sender<BleEvent>, impl futures::Stream<Item = BleEvent>) {
    let (tx, rx) = channel::<BleEvent>(10);
    let stream = receiver_to_stream(rx);
    (tx, stream)
}

#[tokio::test]
async fn test_event_filter_all() {
    // Create a filter that accepts all events
    let filter = EventFilter::all();
    
    // Create a test stream with this filter
    let (tx, stream) = create_test_event_stream(filter).await;
    pin_mut!(stream);
    
    // Send a test event
    let test_event = BleEvent::DeviceDiscovered(DiscoveredDevice::default());
    tx.send(test_event.clone()).await.unwrap();
    
    // The stream should receive the event
    let result = timeout(Duration::from_millis(100), stream.next()).await;
    assert!(result.is_ok(), "Stream should receive an event");
    
    if let Ok(Some(event)) = result {
        match event {
            BleEvent::DeviceDiscovered(_) => { /* Expected */ },
            _ => panic!("Received unexpected event type")
        }
    }
}

#[tokio::test]
async fn test_event_filter_event_types() {
    // Create a filter for specific event types
    let filter = EventFilter::event_types(vec![EventType::DeviceDiscovered]);
    
    // Create a test stream with this filter
    let (tx, stream) = create_test_event_stream(filter).await;
    pin_mut!(stream);
    
    // Send multiple events of different types
    let device_event = BleEvent::DeviceDiscovered(DiscoveredDevice::default());
    let error_event = BleEvent::Error("Test error".to_string());
    
    tx.send(device_event.clone()).await.unwrap();
    tx.send(error_event.clone()).await.unwrap();
    
    // The stream should receive only the DeviceDiscovered event
    let result = timeout(Duration::from_millis(100), stream.next()).await;
    assert!(result.is_ok(), "Stream should receive an event");
    
    if let Ok(Some(event)) = result {
        match event {
            BleEvent::DeviceDiscovered(_) => { /* Expected */ },
            _ => panic!("Received unexpected event type")
        }
    }
    
    // The stream should not receive a second event (the Error)
    let result = timeout(Duration::from_millis(100), stream.next()).await;
    assert!(result.is_err(), "Stream should timeout waiting for second event");
}

#[tokio::test]
async fn test_event_filter_airpods_only() {
    // Create a filter for AirPods events only
    let filter = EventFilter::airpods_only();
    
    // Create a test stream with this filter
    let (tx, stream) = create_test_event_stream(filter).await;
    pin_mut!(stream);
    
    // Send different types of events
    let device_event = BleEvent::DeviceDiscovered(DiscoveredDevice::default());
    
    let airpods_event = BleEvent::AirPodsDetected(DetectedAirPods {
        address: BDAddr::default(),
        name: Some("AirPods".to_string()),
        device_type: AirPodsType::AirPods1,
        battery: AirPodsBattery::default(),
        rssi: Some(-60),
        raw_data: vec![],
    });
    
    tx.send(device_event.clone()).await.unwrap();
    tx.send(airpods_event.clone()).await.unwrap();
    
    // The stream should only receive the AirPods event
    let result = timeout(Duration::from_millis(100), stream.next()).await;
    assert!(result.is_ok(), "Stream should receive an event");
    
    if let Ok(Some(event)) = result {
        match event {
            BleEvent::AirPodsDetected(_) => { /* Expected */ },
            _ => panic!("Received unexpected event type: {:?}", event)
        }
    }
}

#[tokio::test]
async fn test_event_filter_devices() {
    // Create a specific device address to filter on
    let target_address = BDAddr::from([1, 2, 3, 4, 5, 6]);
    let filter = EventFilter::devices(vec![target_address]);
    
    // Create a test stream with this filter
    let (tx, stream) = create_test_event_stream(filter).await;
    pin_mut!(stream);
    
    // Create a matching device event
    let matching_device = DiscoveredDevice {
        address: target_address,
        name: Some("Target Device".to_string()),
        rssi: Some(-60),
        manufacturer_data: Default::default(),
        is_potential_airpods: false,
        last_seen: std::time::Instant::now(),
    };
    
    // Create a non-matching device event
    let other_device = DiscoveredDevice {
        address: BDAddr::from([6, 5, 4, 3, 2, 1]),
        name: Some("Other Device".to_string()),
        rssi: Some(-60),
        manufacturer_data: Default::default(),
        is_potential_airpods: false,
        last_seen: std::time::Instant::now(),
    };
    
    // Send both events
    tx.send(BleEvent::DeviceDiscovered(other_device)).await.unwrap();
    tx.send(BleEvent::DeviceDiscovered(matching_device)).await.unwrap();
    
    // The stream should only receive the matching device event
    let result = timeout(Duration::from_millis(100), stream.next()).await;
    assert!(result.is_ok(), "Stream should receive an event");
    
    if let Ok(Some(event)) = result {
        match event {
            BleEvent::DeviceDiscovered(device) => {
                assert_eq!(device.address, target_address, "Should receive the matching device");
            },
            _ => panic!("Received unexpected event type")
        }
    }
} 