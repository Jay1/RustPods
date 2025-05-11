//! Enhanced tests for event broker edge cases

use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use btleplug::api::BDAddr;
use futures::{StreamExt, pin_mut};
use std::sync::Arc;
use tokio::sync::mpsc::error::TryRecvError;

use rustpods::bluetooth::{
    DiscoveredDevice, BleEvent, EventBroker, EventFilter, 
    receiver_to_stream, events::EventType
};
use rustpods::airpods::{DetectedAirPods, AirPodsType, AirPodsBattery, ChargingStatus};

/// Create a test device for event testing
fn create_test_device(
    address: [u8; 6], 
    name: Option<&str>, 
    rssi: Option<i16>, 
    is_airpods: bool
) -> DiscoveredDevice {
    let mut mfg_data = HashMap::new();
    if is_airpods {
        mfg_data.insert(0x004C, vec![0x07, 0x19, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06]);
    }
    
    DiscoveredDevice {
        address: BDAddr::from(address),
        name: name.map(|s| s.to_string()),
        rssi,
        manufacturer_data: mfg_data,
        is_potential_airpods: is_airpods,
        last_seen: Instant::now(),
    }
}

/// Create a test AirPods detection for event testing
fn create_test_airpods(
    address: [u8; 6],
    airpods_type: AirPodsType,
    left: Option<u8>,
    right: Option<u8>,
    case: Option<u8>
) -> DetectedAirPods {
    DetectedAirPods {
        address: BDAddr::from(address),
        name: Some("Test AirPods".to_string()),
        device_type: airpods_type,
        battery: AirPodsBattery {
            left,
            right,
            case,
            charging: ChargingStatus {
                left: false,
                right: false,
                case: false,
            }
        },
        rssi: Some(-60),
        raw_data: vec![0x07, 0x19, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
    }
}

#[tokio::test]
async fn test_event_broker_unsubscribe() {
    // Create event broker
    let mut broker = EventBroker::new();
    broker.start();
    
    // Subscribe to all events
    let (subscriber_id, rx) = broker.subscribe(EventFilter::all());
    let stream = receiver_to_stream(rx);
    pin_mut!(stream);
    
    // Send an event
    let device = create_test_device([1, 2, 3, 4, 5, 6], Some("Test Device"), Some(-60), false);
    let event = BleEvent::DeviceDiscovered(device);
    broker.get_sender().send(event).await.unwrap();
    
    // Receive the event
    let received = timeout(Duration::from_millis(100), stream.next()).await;
    assert!(received.is_ok(), "Should receive the event");
    
    // Unsubscribe
    broker.unsubscribe(subscriber_id);
    
    // Send another event
    let device2 = create_test_device([2, 3, 4, 5, 6, 7], Some("Test Device 2"), Some(-70), false);
    let event2 = BleEvent::DeviceDiscovered(device2);
    broker.get_sender().send(event2).await.unwrap();
    
    // Should not receive the event
    let not_received = timeout(Duration::from_millis(100), stream.next()).await;
    assert!(not_received.is_err(), "Should not receive events after unsubscribing");
}

#[tokio::test]
async fn test_event_broker_multiple_filters() {
    // Create event broker
    let mut broker = EventBroker::new();
    broker.start();
    
    // Create a filter for device address
    let target_address = BDAddr::from([1, 2, 3, 4, 5, 6]);
    
    // Create a custom filter that combines conditions
    let combined_filter = EventFilter::custom(move |event| {
        match event {
            BleEvent::DeviceDiscovered(device) => device.address == target_address,
            _ => false
        }
    });
    
    // Subscribe with the combined filter
    let (_, rx) = broker.subscribe(combined_filter);
    let stream = receiver_to_stream(rx);
    pin_mut!(stream);
    
    // Send events that match only partially or completely
    
    // 1. Matching device but wrong event type (DeviceLost)
    let event1 = BleEvent::DeviceLost(target_address);
    broker.get_sender().send(event1).await.unwrap();
    
    // 2. Matching event type but wrong device
    let other_device = create_test_device([9, 8, 7, 6, 5, 4], Some("Other Device"), Some(-70), false);
    let event2 = BleEvent::DeviceDiscovered(other_device);
    broker.get_sender().send(event2).await.unwrap();
    
    // 3. Matching both criteria
    let target_device = create_test_device([1, 2, 3, 4, 5, 6], Some("Target Device"), Some(-60), false);
    let event3 = BleEvent::DeviceDiscovered(target_device);
    broker.get_sender().send(event3).await.unwrap();
    
    // Should receive only the third event
    let received = timeout(Duration::from_millis(100), stream.next()).await;
    assert!(received.is_ok(), "Should receive the matching event");
    
    // Verify it's the correct event
    match received.unwrap().unwrap() {
        BleEvent::DeviceDiscovered(device) => {
            assert_eq!(device.address, target_address);
        },
        _ => panic!("Received wrong event type"),
    }
    
    // Should not receive any more events
    let no_more = timeout(Duration::from_millis(100), stream.next()).await;
    assert!(no_more.is_err(), "Should not receive any more events");
}

#[tokio::test]
async fn test_event_broker_custom_filter() {
    // Create event broker
    let mut broker = EventBroker::new();
    broker.start();
    
    // Create two filters to test multiple conditions
    let address1 = BDAddr::from([1, 2, 3, 4, 5, 6]);
    let address2 = BDAddr::from([6, 5, 4, 3, 2, 1]);
    
    // Create a custom filter that implements OR logic
    let or_filter = EventFilter::custom(move |event| {
        match event {
            BleEvent::DeviceDiscovered(device) => {
                device.address == address1 || device.address == address2
            },
            BleEvent::DeviceLost(addr) => {
                *addr == address1 || *addr == address2
            },
            _ => false
        }
    });
    
    // Subscribe with the OR filter
    let (_, rx) = broker.subscribe(or_filter);
    let stream = receiver_to_stream(rx);
    pin_mut!(stream);
    
    // Create and send events for both addresses and a third non-matching address
    let device1 = create_test_device([1, 2, 3, 4, 5, 6], Some("Device 1"), Some(-60), false);
    let device2 = create_test_device([6, 5, 4, 3, 2, 1], Some("Device 2"), Some(-70), false);
    let device3 = create_test_device([9, 8, 7, 6, 5, 4], Some("Device 3"), Some(-80), false);
    
    let event1 = BleEvent::DeviceDiscovered(device1.clone());
    let event2 = BleEvent::DeviceDiscovered(device2.clone());
    let event3 = BleEvent::DeviceDiscovered(device3.clone());
    
    broker.get_sender().send(event1).await.unwrap();
    broker.get_sender().send(event2).await.unwrap();
    broker.get_sender().send(event3).await.unwrap();
    
    // Should receive two events (for device1 and device2)
    let received1 = timeout(Duration::from_millis(100), stream.next()).await;
    assert!(received1.is_ok(), "Should receive first matching event");
    
    let received2 = timeout(Duration::from_millis(100), stream.next()).await;
    assert!(received2.is_ok(), "Should receive second matching event");
    
    // Should not receive the third event
    let no_more = timeout(Duration::from_millis(100), stream.next()).await;
    assert!(no_more.is_err(), "Should not receive non-matching event");
}

#[tokio::test]
async fn test_event_broker_multiple_subscribers() {
    // Create event broker
    let mut broker = EventBroker::new();
    broker.start();
    
    // Create filters for different subscribers
    let airpods_filter = EventFilter::event_types(vec![EventType::AirPodsDetected]);
    let device_filter = EventFilter::event_types(vec![EventType::DeviceDiscovered]);
    let all_filter = EventFilter::all();
    
    // Subscribe with different filters
    let (_, airpods_rx) = broker.subscribe(airpods_filter);
    let airpods_stream = receiver_to_stream(airpods_rx);
    pin_mut!(airpods_stream);
    
    let (_, device_rx) = broker.subscribe(device_filter);
    let device_stream = receiver_to_stream(device_rx);
    pin_mut!(device_stream);
    
    let (_, all_rx) = broker.subscribe(all_filter);
    let all_stream = receiver_to_stream(all_rx);
    pin_mut!(all_stream);
    
    // Create and send events
    let device = create_test_device([1, 2, 3, 4, 5, 6], Some("Test Device"), Some(-60), false);
    let device_event = BleEvent::DeviceDiscovered(device.clone());
    
    let airpods = create_test_airpods([2, 3, 4, 5, 6, 7], AirPodsType::AirPodsPro, Some(80), Some(90), Some(100));
    let airpods_event = BleEvent::AirPodsDetected(airpods);
    
    // Send events
    broker.get_sender().send(device_event.clone()).await.unwrap();
    broker.get_sender().send(airpods_event.clone()).await.unwrap();
    
    // Check airpods subscriber - should receive only AirPodsDetected
    let airpods_received = timeout(Duration::from_millis(100), airpods_stream.next()).await;
    assert!(airpods_received.is_ok(), "AirPods subscriber should receive AirPodsDetected event");
    match airpods_received.unwrap().unwrap() {
        BleEvent::AirPodsDetected(_) => {}, // Expected
        _ => panic!("AirPods subscriber received wrong event type"),
    }
    
    // Check device subscriber - should receive only DeviceDiscovered
    let device_received = timeout(Duration::from_millis(100), device_stream.next()).await;
    assert!(device_received.is_ok(), "Device subscriber should receive DeviceDiscovered event");
    match device_received.unwrap().unwrap() {
        BleEvent::DeviceDiscovered(_) => {}, // Expected
        _ => panic!("Device subscriber received wrong event type"),
    }
    
    // Check all subscriber - should receive both events
    let all_received1 = timeout(Duration::from_millis(100), all_stream.next()).await;
    assert!(all_received1.is_ok(), "All subscriber should receive first event");
    
    let all_received2 = timeout(Duration::from_millis(100), all_stream.next()).await;
    assert!(all_received2.is_ok(), "All subscriber should receive second event");
    
    // Should not receive more events
    let all_no_more = timeout(Duration::from_millis(100), all_stream.next()).await;
    assert!(all_no_more.is_err(), "All subscriber should not receive more events");
}

#[tokio::test]
async fn test_event_broker_strong_signal_filter() {
    // Create event broker
    let mut broker = EventBroker::new();
    broker.start();
    
    // Create a custom filter that only passes devices with strong signal (rssi > -70)
    let strong_signal_filter = EventFilter::custom(|event| {
        match event {
            BleEvent::DeviceDiscovered(device) => {
                device.rssi.map_or(false, |rssi| rssi > -70)
            },
            _ => false,
        }
    });
    
    // Subscribe with the custom filter
    let (_, rx) = broker.subscribe(strong_signal_filter);
    let stream = receiver_to_stream(rx);
    pin_mut!(stream);
    
    // Create and send events with different signal strengths
    let strong_device = create_test_device([1, 2, 3, 4, 5, 6], Some("Strong Signal"), Some(-60), false);
    let weak_device = create_test_device([2, 3, 4, 5, 6, 7], Some("Weak Signal"), Some(-80), false);
    let no_rssi_device = create_test_device([3, 4, 5, 6, 7, 8], Some("No RSSI"), None, false);
    
    let strong_event = BleEvent::DeviceDiscovered(strong_device);
    let weak_event = BleEvent::DeviceDiscovered(weak_device);
    let no_rssi_event = BleEvent::DeviceDiscovered(no_rssi_device);
    
    // Send all events
    broker.get_sender().send(strong_event).await.unwrap();
    broker.get_sender().send(weak_event).await.unwrap();
    broker.get_sender().send(no_rssi_event).await.unwrap();
    
    // Should receive only the strong signal event
    let received = timeout(Duration::from_millis(100), stream.next()).await;
    assert!(received.is_ok(), "Should receive the strong signal event");
    
    match received.unwrap().unwrap() {
        BleEvent::DeviceDiscovered(device) => {
            assert_eq!(device.rssi, Some(-60));
            assert_eq!(device.name, Some("Strong Signal".to_string()));
        },
        _ => panic!("Received wrong event type"),
    }
    
    // Should not receive weak signal or no RSSI events
    let no_more = timeout(Duration::from_millis(100), stream.next()).await;
    assert!(no_more.is_err(), "Should not receive weak signal or no RSSI events");
}

#[tokio::test]
async fn test_event_broker_shutdown() {
    // Create event broker
    let mut broker = EventBroker::new();
    broker.start();
    
    // Create subscriptions
    let (_, rx1) = broker.subscribe(EventFilter::all());
    let (_, rx2) = broker.subscribe(EventFilter::all());
    
    // Send an event
    let device = create_test_device([1, 2, 3, 4, 5, 6], Some("Test Device"), Some(-60), false);
    let event = BleEvent::DeviceDiscovered(device);
    
    // Use a timeout to prevent the test from hanging
    match timeout(Duration::from_millis(1000), broker.get_sender().send(event)).await {
        Ok(result) => result.expect("Failed to send event"),
        Err(_) => panic!("Timed out while sending event"),
    };
    
    // Shutdown the broker with a timeout
    match timeout(Duration::from_millis(1000), broker.shutdown()).await {
        Ok(_) => {}, // Shutdown completed
        Err(_) => panic!("Timed out during broker shutdown"),
    }
    
    // We can't directly check if subscribers are empty since it's a private field
    // Instead, verify the shutdown worked by trying to subscribe again
    let (id, _) = broker.subscribe(EventFilter::all());
    // If broker is properly shutdown and reset, the ID should be 1 again
    assert_eq!(id, 1, "Broker should be reset after shutdown");
    
    // Verify streams are closed by attempting to receive from them
    let mut rx1 = rx1;
    match rx1.try_recv() {
        Err(TryRecvError::Disconnected) => {}, // Expected - channel is closed
        _ => panic!("Channel should be closed after shutdown"),
    }
}

#[tokio::test]
async fn test_event_filter_with_min_rssi() {
    // Create a filter that only accepts events with RSSI >= -70
    let filter = EventFilter::custom(|event| {
        if let BleEvent::DeviceDiscovered(device) = event {
            return device.rssi.unwrap_or(-100) >= -70;
        }
        false
    });
    
    // Create events to test against the filter
    let device_with_good_signal = rustpods::bluetooth::DiscoveredDevice {
        address: BDAddr::default(),
        name: Some("Test Device".to_string()),
        rssi: Some(-60),
        manufacturer_data: std::collections::HashMap::new(),
        is_potential_airpods: false,
        last_seen: std::time::Instant::now(),
    };
    
    let device_with_weak_signal = rustpods::bluetooth::DiscoveredDevice {
        address: BDAddr::default(),
        name: Some("Test Device".to_string()),
        rssi: Some(-80),
        manufacturer_data: std::collections::HashMap::new(),
        is_potential_airpods: false,
        last_seen: std::time::Instant::now(),
    };
    
    // Test the filter
    let event_good = BleEvent::DeviceDiscovered(device_with_good_signal);
    let event_weak = BleEvent::DeviceDiscovered(device_with_weak_signal);
    
    assert!(filter.matches(&event_good));
    assert!(!filter.matches(&event_weak));
}

#[tokio::test]
async fn test_event_broker_subscription() {
    // Create event broker
    let mut broker = EventBroker::new();
    broker.start();
    
    // Subscribe to a specific event type
    let filter = EventFilter::event_types(vec![EventType::DeviceDiscovered]);
    let (_, rx) = broker.subscribe(filter);
    let stream = receiver_to_stream(rx);
    pin_mut!(stream);
    
    // Send an event of the right type
    let device = create_test_device([1, 2, 3, 4, 5, 6], Some("Test Device"), Some(-60), false);
    let event1 = BleEvent::DeviceDiscovered(device);
    broker.get_sender().send(event1).await.unwrap();
    
    // Send an event of the wrong type
    let event2 = BleEvent::DeviceLost(BDAddr::from([1, 2, 3, 4, 5, 6]));
    broker.get_sender().send(event2).await.unwrap();
    
    // Should receive only the first event
    let received = timeout(Duration::from_millis(100), stream.next()).await;
    assert!(received.is_ok(), "Should receive the matching event");
    
    match received.unwrap().unwrap() {
        BleEvent::DeviceDiscovered(_) => {}, // This is expected
        _ => panic!("Received wrong event type"),
    }
    
    // Should not receive the second event
    let not_received = timeout(Duration::from_millis(100), stream.next()).await;
    assert!(not_received.is_err(), "Should not receive non-matching event type");
}

#[tokio::test]
async fn test_event_filter_combinations() {
    // Test different combinations of filter operations
    
    // Create separate filters
    let address = BDAddr::from([1, 2, 3, 4, 5, 6]);
    let address_filter = EventFilter::devices(vec![address]);
    let type_filter = EventFilter::event_types(vec![EventType::DeviceDiscovered]);
    
    // Create a custom filter that combines the conditions (AND logic)
    let combined_filter = EventFilter::custom(move |event| {
        if let BleEvent::DeviceDiscovered(device) = event {
            device.address == address
        } else {
            false
        }
    });
    
    // Create event broker
    let mut broker = EventBroker::new();
    broker.start();
    
    // Subscribe with the combined filter
    let (_, rx) = broker.subscribe(combined_filter);
    let stream = receiver_to_stream(rx);
    pin_mut!(stream);
    
    // Test various combinations:
    // 1. Matching address, matching type (should pass)
    let device1 = create_test_device([1, 2, 3, 4, 5, 6], Some("Test Device 1"), Some(-60), false);
    let event1 = BleEvent::DeviceDiscovered(device1);
    broker.get_sender().send(event1).await.unwrap();
    
    // 2. Matching address, non-matching type (should fail)
    let event2 = BleEvent::DeviceLost(address);
    broker.get_sender().send(event2).await.unwrap();
    
    // 3. Non-matching address, matching type (should fail)
    let device3 = create_test_device([6, 5, 4, 3, 2, 1], Some("Test Device 3"), Some(-70), false);
    let event3 = BleEvent::DeviceDiscovered(device3);
    broker.get_sender().send(event3).await.unwrap();
    
    // Should receive only the first event
    let received = timeout(Duration::from_millis(100), stream.next()).await;
    assert!(received.is_ok(), "Should receive the matching event");
    
    // Should not receive any more events
    let not_received = timeout(Duration::from_millis(100), stream.next()).await;
    assert!(not_received.is_err(), "Should not receive any more events");
}

#[tokio::test]
async fn test_modifying_filter() {
    let mut broker = EventBroker::new();
    
    // Initial subscription with all events
    let (id, mut rx) = broker.subscribe(EventFilter::all());
    
    // Modify to only accept AirPods events
    let result = broker.modify_filter(id, EventFilter::airpods_only());
    assert!(result, "Filter modification should succeed");
    
    // Try modifying with an invalid ID
    let invalid_result = broker.modify_filter(9999, EventFilter::all());
    assert!(!invalid_result, "Invalid ID should fail");
} 