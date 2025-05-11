//! Integration tests for the Event Broker system

use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{channel, Receiver};
use tokio::time::timeout;
use futures::{Stream, StreamExt};
use futures::pin_mut;
use btleplug::api::BDAddr;

use rustpods::bluetooth::{
    EventFilter, BleEvent, events::EventType, events::EventBroker,
    DiscoveredDevice, receiver_to_stream
};
use rustpods::airpods::{DetectedAirPods, AirPodsType, AirPodsBattery, ChargingStatus};

/// Helper to create a test device
fn create_test_device(address: [u8; 6], name: Option<&str>, rssi: Option<i16>) -> DiscoveredDevice {
    DiscoveredDevice {
        address: BDAddr::from(address),
        name: name.map(|s| s.to_string()),
        rssi,
        manufacturer_data: HashMap::new(),
        is_potential_airpods: false,
        last_seen: Instant::now(),
    }
}

/// Helper to create test AirPods
fn create_test_airpods(address: [u8; 6], name: Option<&str>) -> DetectedAirPods {
    DetectedAirPods {
        address: BDAddr::from(address),
        name: name.map(|s| s.to_string()),
        device_type: AirPodsType::AirPods1,
        battery: AirPodsBattery {
            left: Some(80),
            right: Some(75),
            case: Some(90),
            charging: ChargingStatus {
                left: false,
                right: false,
                case: true,
            },
        },
        rssi: Some(-60),
        raw_data: vec![1, 2, 3, 4, 5],
    }
}

#[tokio::test]
async fn test_event_broker_subscribe_all() {
    // Create an event broker
    let mut broker = EventBroker::new();
    
    // Start the broker
    broker.start();
    
    // Subscribe to all events
    let (subscriber_id, rx) = broker.subscribe(EventFilter::all());
    let stream = receiver_to_stream(rx);
    pin_mut!(stream);
    
    // Get a sender for events
    let sender = broker.get_sender();
    
    // Send a test event
    let device = create_test_device([1, 2, 3, 4, 5, 6], Some("Test Device"), Some(-60));
    let event = BleEvent::DeviceDiscovered(device);
    sender.send(event.clone()).await.unwrap();
    
    // The subscriber should receive the event
    let received = timeout(Duration::from_millis(100), stream.next()).await;
    assert!(received.is_ok(), "Should receive the event");
    
    if let Ok(Some(received_event)) = received {
        match received_event {
            BleEvent::DeviceDiscovered(device) => {
                assert_eq!(device.address, BDAddr::from([1, 2, 3, 4, 5, 6]));
            },
            _ => panic!("Received unexpected event type"),
        }
    }
    
    // Clean up
    broker.unsubscribe(subscriber_id);
}

#[tokio::test]
async fn test_event_broker_filter_by_type() {
    // Create an event broker
    let mut broker = EventBroker::new();
    
    // Start the broker
    broker.start();
    
    // Subscribe to only AirPods events
    let (subscriber_id, rx) = broker.subscribe(EventFilter::event_types(vec![EventType::AirPodsDetected]));
    let stream = receiver_to_stream(rx);
    pin_mut!(stream);
    
    // Get a sender for events
    let sender = broker.get_sender();
    
    // Send a device discovery event (should be filtered out)
    let device = create_test_device([1, 2, 3, 4, 5, 6], Some("Test Device"), Some(-60));
    let device_event = BleEvent::DeviceDiscovered(device);
    sender.send(device_event.clone()).await.unwrap();
    
    // Send an AirPods event (should be received)
    let airpods = create_test_airpods([2, 3, 4, 5, 6, 7], Some("AirPods"));
    let airpods_event = BleEvent::AirPodsDetected(airpods);
    sender.send(airpods_event.clone()).await.unwrap();
    
    // The subscriber should receive only the AirPods event
    let received = timeout(Duration::from_millis(100), stream.next()).await;
    assert!(received.is_ok(), "Should receive the AirPods event");
    
    if let Ok(Some(received_event)) = received {
        match received_event {
            BleEvent::AirPodsDetected(airpods) => {
                assert_eq!(airpods.address, BDAddr::from([2, 3, 4, 5, 6, 7]));
            },
            _ => panic!("Received unexpected event type"),
        }
    }
    
    // Should not receive the Device event
    let no_more_events = timeout(Duration::from_millis(100), stream.next()).await;
    assert!(no_more_events.is_err(), "Should not receive any more events");
    
    // Clean up
    broker.unsubscribe(subscriber_id);
}

#[tokio::test]
async fn test_event_broker_filter_by_device() {
    // Create an event broker
    let mut broker = EventBroker::new();
    
    // Start the broker
    broker.start();
    
    // Define a specific device address to filter on
    let target_address = BDAddr::from([1, 2, 3, 4, 5, 6]);
    
    // Subscribe to events only for the target device
    let (subscriber_id, rx) = broker.subscribe(EventFilter::devices(vec![target_address]));
    let stream = receiver_to_stream(rx);
    pin_mut!(stream);
    
    // Get a sender for events
    let sender = broker.get_sender();
    
    // Send a matching device event
    let device1 = create_test_device([1, 2, 3, 4, 5, 6], Some("Target Device"), Some(-60));
    let event1 = BleEvent::DeviceDiscovered(device1);
    sender.send(event1.clone()).await.unwrap();
    
    // Send a non-matching device event
    let device2 = create_test_device([9, 8, 7, 6, 5, 4], Some("Other Device"), Some(-70));
    let event2 = BleEvent::DeviceDiscovered(device2);
    sender.send(event2.clone()).await.unwrap();
    
    // Send a device lost event for the target device
    let event3 = BleEvent::DeviceLost(target_address);
    sender.send(event3.clone()).await.unwrap();
    
    // The subscriber should receive both the matching events
    let received1 = timeout(Duration::from_millis(100), stream.next()).await;
    assert!(received1.is_ok(), "Should receive the first event");
    
    if let Ok(Some(received_event)) = received1 {
        match received_event {
            BleEvent::DeviceDiscovered(device) => {
                assert_eq!(device.address, target_address);
            },
            _ => panic!("Received unexpected event type"),
        }
    }
    
    let received2 = timeout(Duration::from_millis(100), stream.next()).await;
    assert!(received2.is_ok(), "Should receive the third event");
    
    // Should not receive the non-matching device event
    let no_more_events = timeout(Duration::from_millis(100), stream.next()).await;
    assert!(no_more_events.is_err(), "Should not receive any more events");
    
    // Clean up
    broker.unsubscribe(subscriber_id);
}

#[tokio::test]
async fn test_event_broker_custom_filter() {
    // Create an event broker
    let mut broker = EventBroker::new();
    
    // Start the broker
    broker.start();
    
    // Create a custom filter for devices with strong signal (RSSI > -70)
    let (subscriber_id, rx) = broker.subscribe(EventFilter::custom(|event| {
        match event {
            BleEvent::DeviceDiscovered(device) => {
                if let Some(rssi) = device.rssi {
                    rssi > -70
                } else {
                    false
                }
            },
            _ => false,
        }
    }));
    
    let stream = receiver_to_stream(rx);
    pin_mut!(stream);
    
    // Get a sender for events
    let sender = broker.get_sender();
    
    // Send a device with strong signal
    let strong_device = create_test_device([1, 2, 3, 4, 5, 6], Some("Strong Signal"), Some(-60));
    let strong_event = BleEvent::DeviceDiscovered(strong_device);
    sender.send(strong_event.clone()).await.unwrap();
    
    // Send a device with weak signal
    let weak_device = create_test_device([6, 5, 4, 3, 2, 1], Some("Weak Signal"), Some(-80));
    let weak_event = BleEvent::DeviceDiscovered(weak_device);
    sender.send(weak_event.clone()).await.unwrap();
    
    // Send an event of a different type
    let other_event = BleEvent::Error("Test error".to_string());
    sender.send(other_event.clone()).await.unwrap();
    
    // The subscriber should receive only the strong signal device
    let received = timeout(Duration::from_millis(100), stream.next()).await;
    assert!(received.is_ok(), "Should receive the strong signal event");
    
    if let Ok(Some(received_event)) = received {
        match received_event {
            BleEvent::DeviceDiscovered(device) => {
                assert_eq!(device.address, BDAddr::from([1, 2, 3, 4, 5, 6]));
                assert_eq!(device.rssi, Some(-60));
            },
            _ => panic!("Received unexpected event type"),
        }
    }
    
    // Should not receive the weak signal device or other event
    let no_more_events = timeout(Duration::from_millis(100), stream.next()).await;
    assert!(no_more_events.is_err(), "Should not receive any more events");
    
    // Clean up
    broker.unsubscribe(subscriber_id);
}

#[tokio::test]
async fn test_event_broker_modify_filter() {
    // Create an event broker
    let mut broker = EventBroker::new();
    
    // Start the broker
    broker.start();
    
    // Initially subscribe to only DeviceDiscovered events
    let (subscriber_id, rx) = broker.subscribe(EventFilter::event_types(vec![EventType::DeviceDiscovered]));
    let stream = receiver_to_stream(rx);
    pin_mut!(stream);
    
    // Get a sender for events
    let sender = broker.get_sender();
    
    // Send a device event (should be received)
    let device = create_test_device([1, 2, 3, 4, 5, 6], Some("Test Device"), Some(-60));
    let device_event = BleEvent::DeviceDiscovered(device);
    sender.send(device_event.clone()).await.unwrap();
    
    // The subscriber should receive the device event
    let received = timeout(Duration::from_millis(100), stream.next()).await;
    assert!(received.is_ok(), "Should receive the device event");
    
    // Now modify the filter to only receive AirPods events
    let modified = broker.modify_filter(subscriber_id, EventFilter::event_types(vec![EventType::AirPodsDetected]));
    assert!(modified, "Should successfully modify the filter");
    
    // Send another device event (should not be received with the new filter)
    let device2 = create_test_device([9, 8, 7, 6, 5, 4], Some("Test Device 2"), Some(-65));
    let device_event2 = BleEvent::DeviceDiscovered(device2);
    sender.send(device_event2.clone()).await.unwrap();
    
    // Send an AirPods event (should be received with the new filter)
    let airpods = create_test_airpods([2, 3, 4, 5, 6, 7], Some("AirPods"));
    let airpods_event = BleEvent::AirPodsDetected(airpods);
    sender.send(airpods_event.clone()).await.unwrap();
    
    // Should receive only the AirPods event
    let received = timeout(Duration::from_millis(100), stream.next()).await;
    assert!(received.is_ok(), "Should receive the AirPods event");
    
    if let Ok(Some(received_event)) = received {
        match received_event {
            BleEvent::AirPodsDetected(_) => {
                // Expected
            },
            _ => panic!("Received unexpected event type"),
        }
    }
    
    // Clean up
    broker.unsubscribe(subscriber_id);
}

#[tokio::test]
async fn test_event_broker_multiple_subscribers() {
    // Create an event broker
    let mut broker = EventBroker::new();
    
    // Start the broker
    broker.start();
    
    // Subscribe to all events
    let (all_id, all_rx) = broker.subscribe(EventFilter::all());
    let all_stream = receiver_to_stream(all_rx);
    pin_mut!(all_stream);
    
    // Subscribe to only AirPods events
    let (airpods_id, airpods_rx) = broker.subscribe(EventFilter::event_types(vec![EventType::AirPodsDetected]));
    let airpods_stream = receiver_to_stream(airpods_rx);
    pin_mut!(airpods_stream);
    
    // Get a sender for events
    let sender = broker.get_sender();
    
    // Send a device event
    let device = create_test_device([1, 2, 3, 4, 5, 6], Some("Test Device"), Some(-60));
    let device_event = BleEvent::DeviceDiscovered(device);
    sender.send(device_event.clone()).await.unwrap();
    
    // Send an AirPods event
    let airpods = create_test_airpods([2, 3, 4, 5, 6, 7], Some("AirPods"));
    let airpods_event = BleEvent::AirPodsDetected(airpods);
    sender.send(airpods_event.clone()).await.unwrap();
    
    // All subscriber should receive both events
    let all_received1 = timeout(Duration::from_millis(100), all_stream.next()).await;
    assert!(all_received1.is_ok(), "All subscriber should receive the first event");
    
    let all_received2 = timeout(Duration::from_millis(100), all_stream.next()).await;
    assert!(all_received2.is_ok(), "All subscriber should receive the second event");
    
    // AirPods subscriber should receive only the AirPods event
    let airpods_received = timeout(Duration::from_millis(100), airpods_stream.next()).await;
    assert!(airpods_received.is_ok(), "AirPods subscriber should receive the AirPods event");
    
    if let Ok(Some(received_event)) = airpods_received {
        match received_event {
            BleEvent::AirPodsDetected(_) => {
                // Expected
            },
            _ => panic!("AirPods subscriber received unexpected event type"),
        }
    }
    
    // AirPods subscriber should not receive any more events
    let no_more_airpods_events = timeout(Duration::from_millis(100), airpods_stream.next()).await;
    assert!(no_more_airpods_events.is_err(), "AirPods subscriber should not receive any more events");
    
    // Clean up
    broker.unsubscribe(all_id);
    broker.unsubscribe(airpods_id);
} 