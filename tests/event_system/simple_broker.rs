//! Tests for simplified event broker implementation

use std::time::Duration;
use rustpods::bluetooth::{EventFilter, BleEvent};
use rustpods::bluetooth::events::{EventBroker, EventType};
use tokio::time::timeout;
use tokio::sync::mpsc::error::TryRecvError;
use btleplug::api::BDAddr;
use futures::{StreamExt, pin_mut};

use crate::common_test_helpers::{receiver_to_stream, medium_delay, wait_ms};
use crate::bluetooth::common_utils::create_test_device;

/// Helper to create a simple test broker for testing
async fn create_test_broker() -> EventBroker {
    let broker = EventBroker::new();
    // Allow time for broker creation
    wait_ms(100).await;
    broker
}

#[tokio::test]
async fn test_event_broker_subscription_basic() {
    println!("Starting basic subscription test");
    
    // Create a broker
    let mut broker = create_test_broker().await;
    println!("Test broker created");
    
    // Start the broker
    let _handle = broker.start();
    println!("Broker started");
    
    // Subscribe to all events
    let (id, rx) = broker.subscribe(EventFilter::all());
    println!("Subscribed with ID: {}", id);
    
    // Convert receiver to stream for easier testing
    let stream = receiver_to_stream(rx);
    pin_mut!(stream);
    
    // Create a test event
    let device = create_test_device(
        [1, 2, 3, 4, 5, 6], 
        Some("Test Device"), 
        Some(-70),
        false,  // is_airpods
        None,   // prefix
        false   // has_battery
    );
    let test_event = BleEvent::DeviceDiscovered(device);
    
    // Send the event
    println!("Sending device discovered event");
    let sender = broker.get_sender();
    match sender.send(test_event.clone()).await {
        Ok(_) => println!("Event sent successfully"),
        Err(e) => {
            println!("❌ Failed to send event: {:?}", e);
            panic!("Failed to send event: {:?}", e);
        }
    }
    
    // Wait for event processing
    println!("Waiting for event to be processed...");
    wait_ms(500).await;  // Increased wait time
    
    // Check if the event was received
    println!("Checking if event was received...");
    
    // Try to receive with a long timeout - 5 seconds should be plenty
    let receive_result = match timeout(Duration::from_secs(5), stream.next()).await {
        Ok(Some(event)) => {
            println!("✅ Event received: {:?}", event);
            "Success"
        },
        Ok(None) => {
            println!("❌ No event in stream");
            "Empty"
        },
        Err(e) => {
            println!("❌ Event was not received (timeout): {:?}", e);
            "Timeout"
        }
    };
    
    println!("Receive result: {:?}", receive_result);
    
    if receive_result == "Timeout" || receive_result == "Empty" {
        println!("❌ Event was not received (timeout)");
        panic!("Event was not received (timeout)");
    }
}

#[tokio::test]
async fn test_multiple_subscribers() {
    println!("Starting multiple subscribers test");
    
    // Create a broker
    let mut broker = create_test_broker().await;
    
    // Start the broker
    let _handle = broker.start();
    
    // Subscribe to all events with two different subscribers
    let (id1, rx1) = broker.subscribe(EventFilter::all());
    let (id2, rx2) = broker.subscribe(EventFilter::all());
    
    // Convert receivers to streams
    let stream1 = receiver_to_stream(rx1);
    let stream2 = receiver_to_stream(rx2);
    pin_mut!(stream1);
    pin_mut!(stream2);
    
    // Create two different test events
    let device = create_test_device(
        [1, 2, 3, 4, 5, 6], 
        Some("Test Device"), 
        Some(-70),
        false,  // is_airpods
        None,   // prefix
        false   // has_battery
    );
    let device_event = BleEvent::DeviceDiscovered(device);
    let error_event = BleEvent::Error("Test error".to_string());
    
    // Send the events
    println!("Sending device discovered event");
    let sender = broker.get_sender();
    match sender.send(device_event.clone()).await {
        Ok(_) => println!("✅ Device event sent successfully"),
        Err(e) => {
            println!("❌ Failed to send device event: {:?}", e);
            panic!("Failed to send device event: {:?}", e);
        }
    }
    
    println!("Sending error event");
    match sender.send(error_event.clone()).await {
        Ok(_) => println!("✅ Error event sent successfully"),
        Err(e) => {
            println!("❌ Failed to send error event: {:?}", e);
            panic!("Failed to send error event: {:?}", e);
        }
    }
    
    // Wait for event processing
    println!("Waiting for events to be processed...");
    wait_ms(500).await;  // Increased wait time
    
    // Check if the events were received by the first subscriber
    println!("Checking first subscriber...");
    
    // Use longer timeouts - 5 seconds for each
    let first_sub_event1 = match timeout(Duration::from_secs(5), stream1.next()).await {
        Ok(Some(event)) => {
            println!("✅ First subscriber received first event: {:?}", event);
            event
        },
        Ok(None) => {
            println!("❌ First subscriber stream ended");
            panic!("First subscriber stream ended unexpectedly");
        },
        Err(e) => {
            println!("❌ First subscriber error: Timeout waiting for first event: {:?}", e);
            panic!("Timeout waiting for first event: {:?}", e);
        }
    };
    
    let first_sub_event2 = match timeout(Duration::from_secs(5), stream1.next()).await {
        Ok(Some(event)) => {
            println!("✅ First subscriber received second event: {:?}", event);
            event
        },
        Ok(None) => {
            println!("❌ First subscriber stream ended after first event");
            panic!("First subscriber stream ended after first event");
        },
        Err(e) => {
            println!("❌ First subscriber error: Timeout waiting for second event: {:?}", e);
            panic!("Timeout waiting for second event: {:?}", e);
        }
    };
    
    // Check if events were received by the second subscriber
    println!("Checking second subscriber...");
    
    let second_sub_event1 = match timeout(Duration::from_secs(5), stream2.next()).await {
        Ok(Some(event)) => {
            println!("✅ Second subscriber received first event: {:?}", event);
            event
        },
        Ok(None) => {
            println!("❌ Second subscriber stream ended");
            panic!("Second subscriber stream ended unexpectedly");
        },
        Err(e) => {
            println!("❌ Second subscriber error: Timeout waiting for first event: {:?}", e);
            panic!("Timeout waiting for first event: {:?}", e);
        }
    };
    
    let second_sub_event2 = match timeout(Duration::from_secs(5), stream2.next()).await {
        Ok(Some(event)) => {
            println!("✅ Second subscriber received second event: {:?}", event);
            event
        },
        Ok(None) => {
            println!("❌ Second subscriber stream ended after first event");
            panic!("Second subscriber stream ended after first event");
        },
        Err(e) => {
            println!("❌ Second subscriber error: Timeout waiting for second event: {:?}", e);
            panic!("Timeout waiting for second event: {:?}", e);
        }
    };
} 