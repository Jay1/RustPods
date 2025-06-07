//! Tests for the EventBroker implementation

use std::time::Duration;
use tokio::time::timeout;

use btleplug::api::BDAddr;
use futures::{StreamExt, pin_mut};

use rustpods::bluetooth::events::{BleEvent, EventBroker, EventFilter, EventType};
use rustpods::bluetooth::{DiscoveredDevice, receiver_to_stream};

// Helper to create a simple test device
fn create_test_device(address: [u8; 6], name: Option<&str>, rssi: Option<i16>) -> DiscoveredDevice {
    use std::collections::HashMap;
    use std::time::Instant;
    
    DiscoveredDevice {
        address: BDAddr::from(address),
        name: name.map(|s| s.to_string()),
        rssi,
        manufacturer_data: HashMap::new(),
        is_potential_airpods: false,
        last_seen: Instant::now(),
        is_connected: false,
        service_data: HashMap::new(),
        services: Vec::new(),
        tx_power_level: None,
    }
}

// Helper to create a test AirPods
fn create_test_airpods(address: [u8; 6], name: Option<&str>) -> rustpods::airpods::DetectedAirPods {
    use rustpods::airpods::{AirPodsType, AirPodsBattery};
    use std::time::Instant;
    
    rustpods::airpods::DetectedAirPods {
        address: BDAddr::from(address),
        name: name.map(|s| s.to_string()),
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

/// Helper to ensure broker is properly shut down
async fn shutdown_broker(broker: &mut EventBroker) {
    println!("Attempting to shut down broker...");
    // Attempt to shut down the broker with a timeout
    match timeout(Duration::from_millis(2000), broker.shutdown()).await {
        Ok(_) => {
            println!("✅ Broker shutdown successful");
            // Successfully shut down, give time for cleanup
            tokio::time::sleep(Duration::from_millis(500)).await;
        },
        Err(_) => {
            println!("⚠️ Warning: Broker shutdown timed out");
            // Still wait a bit to allow potential cleanup
            tokio::time::sleep(Duration::from_millis(300)).await;
        }
    }
}

#[tokio::test]
async fn test_event_broker_subscribe_all() {
    println!("Starting subscribe_all test");
    
    // Create an event broker
    let mut broker = EventBroker::new();
    println!("Broker created");
    
    // Start the broker
    let _handle = broker.start();
    println!("Broker started");
    
    // Wait for broker to fully initialize - increase delay
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Subscribe to all events
    let (subscriber_id, rx) = broker.subscribe(EventFilter::all());
    println!("Subscribed with ID: {}", subscriber_id);
    
    let stream = receiver_to_stream(rx);
    pin_mut!(stream);
    
    // Wait a bit to ensure subscription is registered - increase delay
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Get a sender for events
    let sender = broker.get_sender();
    
    // Send a test event
    let device = create_test_device([1, 2, 3, 4, 5, 6], Some("Test Device"), Some(-60));
    let event = BleEvent::DeviceDiscovered(device);
    println!("Sending test event");
    match sender.send(event.clone()).await {
        Ok(_) => println!("✅ Test event sent successfully"),
        Err(e) => {
            println!("❌ Failed to send test event: {:?}", e);
            panic!("Failed to send test event");
        }
    }
    
    // Wait for event to be processed - increase delay
    println!("Waiting for event to be processed...");
    tokio::time::sleep(Duration::from_millis(800)).await;
    
    // The subscriber should receive the event - use longer timeout
    println!("Checking if event was received...");
    let receive_timeout = Duration::from_millis(5000);
    let received = timeout(receive_timeout, stream.next()).await;
    
    match received {
        Ok(Some(event)) => {
            println!("✅ Event received: {:?}", event);
            match event {
                BleEvent::DeviceDiscovered(device) => {
                    assert_eq!(device.address, BDAddr::from([1, 2, 3, 4, 5, 6]));
                },
                _ => {
                    println!("❌ Received unexpected event type: {:?}", event);
                    panic!("Received unexpected event type");
                }
            }
        },
        Ok(None) => {
            println!("❌ Stream ended unexpectedly");
            panic!("Stream ended unexpectedly");
        },
        Err(e) => {
            println!("❌ Event reception timed out: {:?}", e);
            panic!("Should receive the event");
        }
    }
    
    // Clean up
    println!("Unsubscribing");
    broker.unsubscribe(subscriber_id);
    
    // Give time for unsubscribe to complete
    tokio::time::sleep(Duration::from_millis(300)).await;
    
    // Ensure proper broker shutdown
    let _ = shutdown_broker(&mut broker).await;
    println!("Test completed successfully");
}

#[tokio::test]
async fn test_event_broker_filter_by_type() {
    println!("Starting filter_by_type test");
    
    // Create an event broker
    let mut broker = EventBroker::new();
    println!("Broker created");
    
    // Start the broker
    let _handle = broker.start();
    println!("Broker started");
    
    // Wait a short time for the broker to fully set up
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Subscribe to only AirPods events
    let filter = EventFilter::event_types(vec![EventType::AirPodsDetected]);
    let (subscriber_id, rx) = broker.subscribe(filter);
    println!("Subscribed with ID: {}", subscriber_id);
    
    let stream = receiver_to_stream(rx);
    pin_mut!(stream);
    
    // Wait a bit to ensure subscription is registered
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Get a sender for events
    let sender = broker.get_sender();
    
    // Send a device discovery event (should be filtered out)
    let device = create_test_device([1, 2, 3, 4, 5, 6], Some("Test Device"), Some(-60));
    let device_event = BleEvent::DeviceDiscovered(device);
    
    println!("Sending device discovery event (should be filtered out)");
    match sender.send(device_event.clone()).await {
        Ok(_) => println!("✅ Device event sent successfully"),
        Err(e) => {
            println!("❌ Failed to send device event: {:?}", e);
            panic!("Failed to send device event");
        }
    }
    
    // Wait a bit to ensure event was processed
    tokio::time::sleep(Duration::from_millis(300)).await;
    
    // Send an AirPods event (should be received)
    let airpods = create_test_airpods([2, 3, 4, 5, 6, 7], Some("AirPods"));
    let airpods_event = BleEvent::AirPodsDetected(airpods);
    
    println!("Sending AirPods event (should be received)");
    // Use a timeout for sending
    match timeout(Duration::from_millis(2000), sender.send(airpods_event.clone())).await {
        Ok(result) => match result {
            Ok(_) => println!("✅ AirPods event sent successfully"),
            Err(e) => {
                println!("❌ Failed to send AirPods event: {:?}", e);
                panic!("Failed to send AirPods event");
            }
        },
        Err(e) => {
            println!("❌ Timed out sending AirPods event: {:?}", e);
            panic!("Timed out sending AirPods event");
        }
    }
    
    // Wait a bit to ensure event was processed
    tokio::time::sleep(Duration::from_millis(800)).await;
    
    // The subscriber should receive only the AirPods event - use a longer timeout
    println!("Checking if AirPods event was received...");
    let receive_timeout = Duration::from_millis(5000);
    let received = timeout(receive_timeout, stream.next()).await;
    
    match received {
        Ok(Some(event)) => {
            println!("✅ Event received: {:?}", event);
            match event {
                BleEvent::AirPodsDetected(airpods) => {
                    assert_eq!(airpods.address, BDAddr::from([2, 3, 4, 5, 6, 7]));
                    println!("✅ Correct AirPods event received");
                },
                other => {
                    println!("❌ Received unexpected event type: {:?}", other);
                    panic!("Received unexpected event type: {:?}", other);
                }
            }
        },
        Ok(None) => {
            println!("❌ Stream ended unexpectedly");
            panic!("Stream ended unexpectedly");
        },
        Err(e) => {
            println!("❌ Event reception timed out: {:?}", e);
            panic!("Should receive the AirPods event");
        }
    }
    
    // Should not receive any more events (short timeout is fine here)
    println!("Checking that no more events are received...");
    let no_more_events = timeout(Duration::from_millis(500), stream.next()).await;
    assert!(no_more_events.is_err(), "Should not receive any more events");
    println!("✅ No additional events received (as expected)");
    
    // Clean up
    println!("Unsubscribing");
    broker.unsubscribe(subscriber_id);
    
    // Give time for unsubscribe to complete
    tokio::time::sleep(Duration::from_millis(300)).await;
    
    // Make sure the broker is shut down properly
    let _ = shutdown_broker(&mut broker).await;
    println!("Test completed successfully");
}

#[tokio::test]
async fn test_event_broker_filter_by_device() {
    println!("Starting filter_by_device test");
    
    // Create an event broker
    let mut broker = EventBroker::new();
    println!("Broker created");
    
    // Start the broker
    let _handle = broker.start();
    println!("Broker started");
    
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
    
    // Make sure the broker is shut down properly
    let _ = shutdown_broker(&mut broker).await;
}

#[tokio::test]
async fn test_event_broker_custom_filter() {
    println!("Starting custom_filter test");
    
    // Create an event broker
    let mut broker = EventBroker::new();
    println!("Broker created");
    
    // Start the broker
    let _handle = broker.start();
    println!("Broker started");
    
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
    
    // Ensure proper broker shutdown
    let _ = shutdown_broker(&mut broker).await;
}

#[tokio::test]
async fn test_event_broker_modify_filter() {
    println!("Starting modify_filter test");
    
    // Create an event broker
    let mut broker = EventBroker::new();
    println!("Broker created");
    
    // Start the broker
    let _handle = broker.start();
    println!("Broker started");
    
    // Wait for broker to fully initialize
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    // Initially subscribe to only DeviceDiscovered events
    let filter = EventFilter::event_types(vec![EventType::DeviceDiscovered]);
    let (subscriber_id, rx) = broker.subscribe(filter);
    let stream = receiver_to_stream(rx);
    pin_mut!(stream);
    
    // Get a sender for events
    let sender = broker.get_sender();
    
    // Small delay to ensure subscription is properly registered
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    // First, send a device event (should be received)
    let device = create_test_device([1, 2, 3, 4, 5, 6], Some("Test Device"), Some(-60));
    let device_event = BleEvent::DeviceDiscovered(device);
    sender.send(device_event.clone()).await.unwrap();
    
    // Check that we received it with a timeout
    match timeout(Duration::from_millis(500), stream.next()).await {
        Ok(Some(received)) => {
            assert_eq!(format!("{:?}", received), format!("{:?}", device_event));
        },
        Ok(None) => panic!("Stream ended unexpectedly"),
        Err(_) => panic!("Timeout waiting for device event"),
    };
    
    // Now send an airpods event (should not be received)
    let airpods = create_test_airpods([6, 5, 4, 3, 2, 1], Some("AirPods Pro"));
    let airpods_event = BleEvent::AirPodsDetected(airpods);
    sender.send(airpods_event).await.unwrap();
    
    // Small delay to ensure message is processed
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    // Try to receive with timeout - should time out since we're filtering
    match timeout(Duration::from_millis(100), stream.next()).await {
        Ok(_) => panic!("Shouldn't have received the AirPods event"),
        Err(_) => {} // Timeout is expected
    }
    
    // Now modify the filter to accept AirPods events
    broker.modify_filter(
        subscriber_id, 
        EventFilter::event_types(vec![EventType::AirPodsDetected])
    );
    
    // Small delay to ensure filter update is processed
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    // Send another device event (should not be received now)
    sender.send(device_event).await.unwrap();
    
    // Small delay to ensure message is processed
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    // Try to receive with timeout - should time out since we're filtering
    match timeout(Duration::from_millis(100), stream.next()).await {
        Ok(_) => panic!("Shouldn't have received the Device event after filter change"),
        Err(_) => {} // Timeout is expected
    }
    
    // Send another airpods event (should be received now)
    let airpods2 = create_test_airpods([2, 3, 4, 5, 6, 7], Some("AirPods Pro 2"));
    let airpods_event2 = BleEvent::AirPodsDetected(airpods2);
    sender.send(airpods_event2.clone()).await.unwrap();
    
    // Check that we received it with a timeout
    match timeout(Duration::from_millis(500), stream.next()).await {
        Ok(Some(received)) => {
            assert_eq!(format!("{:?}", received), format!("{:?}", airpods_event2));
        },
        Ok(None) => panic!("Stream ended unexpectedly"),
        Err(_) => panic!("Timeout waiting for airpods event after filter change"),
    };
    
    // Ensure proper broker shutdown
    let _ = shutdown_broker(&mut broker).await;
}

#[tokio::test]
async fn test_event_broker_multiple_subscribers() {
    println!("Starting multiple_subscribers test");
    
    // Create an event broker
    let mut broker = EventBroker::new();
    println!("Broker created");
    
    // Start the broker
    let _handle = broker.start();
    println!("Broker started");
    
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
    
    // Ensure proper broker shutdown
    let _ = shutdown_broker(&mut broker).await;
} 