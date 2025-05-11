//! Simplified tests for EventBroker functionality

use rustpods::bluetooth::{EventBroker, EventFilter, BleEvent};
use rustpods::bluetooth::events::EventType;
use std::time::Duration;
use std::collections::HashMap;
use tokio::sync::mpsc::error::TryRecvError;
use btleplug::api::BDAddr;
use rustpods::airpods::DetectedAirPods;
use std::time::Instant;

// Helper function to create a test device
fn create_test_device(address: BDAddr, rssi: i16) -> rustpods::bluetooth::DiscoveredDevice {
    rustpods::bluetooth::DiscoveredDevice {
        address,
        name: Some("Test Device".to_string()),
        rssi: Some(rssi),
        manufacturer_data: HashMap::new(),
        is_potential_airpods: false,
        last_seen: Instant::now(),
    }
}

#[tokio::test]
async fn test_event_filter_basic() {
    // Test all the different event filter types
    let all_filter = EventFilter::all();
    let event_types_filter = EventFilter::event_types(vec![EventType::DeviceDiscovered]);
    let device_addr = BDAddr::default();
    let devices_filter = EventFilter::devices(vec![device_addr]);
    
    // Test events
    let discovered_event = BleEvent::DeviceDiscovered(create_test_device(device_addr, -60));
    let error_event = BleEvent::Error("Test error".to_string());
    
    // Test all filter
    assert!(all_filter.matches(&discovered_event));
    assert!(all_filter.matches(&error_event));
    
    // Test event types filter
    assert!(event_types_filter.matches(&discovered_event));
    assert!(!event_types_filter.matches(&error_event));
    
    // Test devices filter
    assert!(devices_filter.matches(&discovered_event));
    assert!(!devices_filter.matches(&error_event));
}

#[tokio::test]
async fn test_event_broker_subscription_basic() {
    let mut broker = EventBroker::new();
    broker.start();
    
    // Subscribe with the default "all" filter
    let (id, mut rx) = broker.subscribe(EventFilter::all());
    
    // Send a device discovered event
    let device = create_test_device(BDAddr::default(), -60);
    let event = BleEvent::DeviceDiscovered(device);
    let sender = broker.get_sender();
    
    println!("Sending device discovered event");
    let _ = sender.send(event.clone()).await;
    
    // Increase wait time to 200ms
    println!("Waiting for event to be processed...");
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // Check if we received the event
    println!("Checking if event was received...");
    match rx.try_recv() {
        Ok(received) => {
            println!("Event received successfully");
            assert!(matches!(received, BleEvent::DeviceDiscovered(_)));
        },
        Err(TryRecvError::Empty) => {
            panic!("Event was not received (channel empty)");
        },
        Err(e) => {
            panic!("Error receiving event: {:?}", e);
        }
    }
    
    // Cleanup: unsubscribe
    broker.unsubscribe(id);
}

#[tokio::test]
async fn test_event_filter_airpods_only() {
    // Test the airpods_only filter
    let filter = EventFilter::airpods_only();
    
    // Create test events
    let airpods_event = BleEvent::AirPodsDetected(DetectedAirPods::default());
    let device_event = BleEvent::DeviceDiscovered(create_test_device(BDAddr::default(), -60));
    
    // Test filter
    assert!(filter.matches(&airpods_event));
    assert!(!filter.matches(&device_event));
}

#[tokio::test]
async fn test_multiple_subscribers() {
    let mut broker = EventBroker::new();
    broker.start();
    
    println!("Starting multiple subscribers test");
    
    // Create two subscribers with different filters
    let (id1, mut rx1) = broker.subscribe(EventFilter::event_types(vec![EventType::DeviceDiscovered]));
    let (id2, mut rx2) = broker.subscribe(EventFilter::event_types(vec![EventType::Error]));
    
    // Send events
    let sender = broker.get_sender();
    let device_event = BleEvent::DeviceDiscovered(create_test_device(BDAddr::default(), -60));
    let error_event = BleEvent::Error("Test error".to_string());
    
    println!("Sending device discovered event");
    let _ = sender.send(device_event.clone()).await;
    
    println!("Sending error event");
    let _ = sender.send(error_event.clone()).await;
    
    // Increase wait time to 500ms
    println!("Waiting for events to be processed...");
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Check first subscriber (should receive device event only)
    println!("Checking first subscriber...");
    match rx1.try_recv() {
        Ok(received) => {
            println!("First subscriber received event: {:?}", received);
            assert!(matches!(received, BleEvent::DeviceDiscovered(_)));
        },
        Err(e) => {
            panic!("First subscriber error: {:?}", e);
        }
    }
    
    // Check second subscriber (should receive error event only)
    println!("Checking second subscriber...");
    match rx2.try_recv() {
        Ok(received) => {
            println!("Second subscriber received event: {:?}", received);
            assert!(matches!(received, BleEvent::Error(_)));
        },
        Err(e) => {
            panic!("Second subscriber error: {:?}", e);
        }
    }
    
    // No more events for first subscriber
    println!("Checking no more events for first subscriber...");
    match rx1.try_recv() {
        Err(TryRecvError::Empty) => {
            println!("First subscriber has no more events (as expected)");
        },
        Ok(e) => panic!("Unexpected extra event for first subscriber: {:?}", e),
        Err(e) => panic!("Unexpected error for first subscriber: {:?}", e),
    }
    
    // No more events for second subscriber
    println!("Checking no more events for second subscriber...");
    match rx2.try_recv() {
        Err(TryRecvError::Empty) => {
            println!("Second subscriber has no more events (as expected)");
        },
        Ok(e) => panic!("Unexpected extra event for second subscriber: {:?}", e),
        Err(e) => panic!("Unexpected error for second subscriber: {:?}", e),
    }
    
    // Cleanup
    broker.unsubscribe(id1);
    broker.unsubscribe(id2);
} 