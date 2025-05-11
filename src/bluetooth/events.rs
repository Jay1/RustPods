//! Bluetooth event system for managing device discovery events

use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::collections::HashMap;

use tokio::sync::mpsc::{channel, Sender, Receiver};
use tokio::task::JoinHandle;
use tokio::time::Duration;
use futures::Stream;

use btleplug::api::BDAddr;
use crate::airpods::{DetectedAirPods, AirPodsType};
use crate::bluetooth::DiscoveredDevice;

/// Type of BLE event
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventType {
    /// Device discovered event
    DeviceDiscovered,
    /// Device lost event
    DeviceLost,
    /// Error event
    Error,
    /// Adapter changed event
    AdapterChanged,
    /// Scan cycle completed event
    ScanCycleCompleted,
    /// Scanning completed event
    ScanningCompleted,
    /// AirPods detected event
    AirPodsDetected,
}

/// Event type for discovered devices
#[derive(Debug, Clone)]
pub enum BleEvent {
    /// A new device was discovered or updated
    DeviceDiscovered(DiscoveredDevice),
    /// A device was lost (went out of range)
    DeviceLost(BDAddr),
    /// An error occurred during scanning
    Error(String),
    /// The adapter was changed
    AdapterChanged(crate::bluetooth::AdapterInfo),
    /// A scan cycle has completed
    ScanCycleCompleted { devices_found: usize },
    /// All scanning has completed (due to cycle limit or cancellation)
    ScanningCompleted,
    /// AirPods detected with battery information
    AirPodsDetected(DetectedAirPods),
}

impl BleEvent {
    /// Get the type of this event
    pub fn get_type(&self) -> EventType {
        match self {
            Self::DeviceDiscovered(_) => EventType::DeviceDiscovered,
            Self::DeviceLost(_) => EventType::DeviceLost,
            Self::Error(_) => EventType::Error,
            Self::AdapterChanged(_) => EventType::AdapterChanged,
            Self::ScanCycleCompleted { .. } => EventType::ScanCycleCompleted,
            Self::ScanningCompleted => EventType::ScanningCompleted,
            Self::AirPodsDetected(_) => EventType::AirPodsDetected,
        }
    }
    
    /// Get the device address from this event, if available
    pub fn get_device_address(&self) -> Option<BDAddr> {
        match self {
            Self::DeviceDiscovered(device) => Some(device.address),
            Self::DeviceLost(addr) => Some(*addr),
            Self::AirPodsDetected(airpods) => Some(airpods.address),
            _ => None,
        }
    }
}

/// Defines which types of events a subscriber is interested in
pub enum EventFilter {
    /// Accept all events
    All,
    /// Only specific event types
    EventTypes(Vec<EventType>),
    /// Only events for specific devices
    Devices(Vec<BDAddr>),
    /// Custom filter function
    Custom(Box<dyn Fn(&BleEvent) -> bool + Send + Sync + 'static>),
}

impl Clone for EventFilter {
    fn clone(&self) -> Self {
        match self {
            Self::All => Self::All,
            Self::EventTypes(types) => Self::EventTypes(types.clone()),
            Self::Devices(addresses) => Self::Devices(addresses.clone()),
            Self::Custom(_) => Self::All, // Replace with the All filter as a fallback
        }
    }
}

impl std::fmt::Debug for EventFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::All => write!(f, "EventFilter::All"),
            Self::EventTypes(types) => write!(f, "EventFilter::EventTypes({:?})", types),
            Self::Devices(addresses) => write!(f, "EventFilter::Devices({:?})", addresses),
            Self::Custom(_) => write!(f, "EventFilter::Custom(<function>)"),
        }
    }
}

impl EventFilter {
    /// Create a new default filter that includes all events
    pub fn all() -> Self {
        Self::All
    }
    
    /// Create a filter for specific event types
    pub fn event_types(types: Vec<EventType>) -> Self {
        Self::EventTypes(types)
    }
    
    /// Create a filter for specific devices
    pub fn devices(addresses: Vec<BDAddr>) -> Self {
        Self::Devices(addresses)
    }
    
    /// Create a custom filter with a closure
    pub fn custom<F>(filter_fn: F) -> Self 
    where
        F: Fn(&BleEvent) -> bool + Send + Sync + 'static
    {
        Self::Custom(Box::new(filter_fn))
    }
    
    /// Create a filter that only accepts AirPods-related events
    pub fn airpods_only() -> Self {
        Self::event_types(vec![EventType::AirPodsDetected])
    }
    
    /// Check if an event matches this filter
    pub fn matches(&self, event: &BleEvent) -> bool {
        match self {
            Self::All => true,
            Self::EventTypes(types) => {
                let event_type = event.get_type();
                types.contains(&event_type)
            },
            Self::Devices(addresses) => {
                if let Some(address) = event.get_device_address() {
                    addresses.contains(&address)
                } else {
                    false
                }
            },
            Self::Custom(filter_fn) => filter_fn(event),
        }
    }
}

/// Subscriber ID type
pub type SubscriberId = u32;

/// A subscriber to BLE events
#[derive(Clone)]
struct Subscriber {
    /// Unique ID for this subscriber
    id: SubscriberId,
    /// Sender channel
    sender: Sender<BleEvent>,
    /// Filter for events
    filter: EventFilter,
    /// Last activity timestamp
    last_active: Instant,
}

/// The Bluetooth event broker manages subscribers and distributes events
pub struct EventBroker {
    /// Next subscriber ID to use
    next_subscriber_id: SubscriberId,
    /// Active subscribers
    subscribers: Vec<Subscriber>,
    /// Timeout for inactive subscribers (set to None to disable)
    inactive_timeout: Option<Duration>,
    /// Handle for the cleanup task
    cleanup_task: Option<JoinHandle<()>>,
    /// Sender for internal events
    event_sender: Sender<BleEvent>,
    /// Receiver for internal events
    event_receiver: Arc<Mutex<Option<Receiver<BleEvent>>>>,
}

impl EventBroker {
    /// Create a new event broker
    pub fn new() -> Self {
        let (tx, rx) = channel(100);
        Self {
            next_subscriber_id: 1,
            subscribers: Vec::new(),
            inactive_timeout: Some(Duration::from_secs(60)), // 1 minute default timeout
            cleanup_task: None,
            event_sender: tx,
            event_receiver: Arc::new(Mutex::new(Some(rx))),
        }
    }
    
    /// Get the sender for this broker
    pub fn get_sender(&self) -> Sender<BleEvent> {
        self.event_sender.clone()
    }
    
    /// Start the event broker
    pub fn start(&mut self) {
        // Start the event distribution task
        let rx = self.take_receiver();
        let subscribers = Arc::new(Mutex::new(self.subscribers.clone()));
        
        // Use tokio::spawn instead of just creating a task
        tokio::spawn(async move {
            let mut rx = rx;
            while let Some(event) = rx.recv().await {
                // Distribute the event to all subscribers
                let mut subscribers = subscribers.lock().unwrap();
                let now = Instant::now();
                
                for subscriber in subscribers.iter_mut() {
                    // Update last active timestamp
                    subscriber.last_active = now;
                    
                    // Check if the subscriber's filter accepts this event
                    if subscriber.filter.matches(&event) {
                        // Try to send the event, ignoring errors if the channel is closed
                        let _ = subscriber.sender.try_send(event.clone());
                    }
                }
            }
        });
        
        // Start the cleanup task if a timeout is set
        if let Some(timeout) = self.inactive_timeout {
            let subscribers = Arc::new(Mutex::new(self.subscribers.clone()));
            
            self.cleanup_task = Some(tokio::spawn(async move {
                loop {
                    // Sleep for a while
                    tokio::time::sleep(timeout / 2).await;
                    
                    // Check for inactive subscribers
                    let mut subscribers = subscribers.lock().unwrap();
                    let now = Instant::now();
                    
                    subscribers.retain(|subscriber| {
                        now.duration_since(subscriber.last_active) < timeout
                    });
                }
            }));
        }
    }
    
    /// Subscribe to events with a custom filter
    pub fn subscribe(&mut self, filter: EventFilter) -> (SubscriberId, Receiver<BleEvent>) {
        let (tx, rx) = channel(100);
        let id = self.next_subscriber_id;
        self.next_subscriber_id += 1;
        
        self.subscribers.push(Subscriber {
            id,
            sender: tx,
            filter,
            last_active: Instant::now(),
        });
        
        (id, rx)
    }
    
    /// Unsubscribe from events
    pub fn unsubscribe(&mut self, id: SubscriberId) {
        self.subscribers.retain(|s| s.id != id);
    }
    
    /// Modify a subscriber's filter
    pub fn modify_filter(&mut self, id: SubscriberId, filter: EventFilter) -> bool {
        if let Some(subscriber) = self.subscribers.iter_mut().find(|s| s.id == id) {
            subscriber.filter = filter;
            true
        } else {
            false
        }
    }
    
    /// Set timeout for inactive subscribers (None to disable)
    pub fn set_inactive_timeout(&mut self, timeout: Option<Duration>) {
        self.inactive_timeout = timeout;
    }
    
    /// Shutdown the event broker, closing all channels and stopping tasks
    pub async fn shutdown(&mut self) {
        // Stop the cleanup task if it's running
        if let Some(task) = self.cleanup_task.take() {
            task.abort();
        }
        
        // Close all subscriber channels to signal shutdown
        // Instead of waiting for each channel to close, simply drop all senders
        // This fixes the potential hang in the original implementation
        for subscriber in &self.subscribers {
            // We don't need to actively wait for closure, just drop it
            // Removing the await here prevents potential hanging
        }
        
        // Clear the subscribers list
        self.subscribers.clear();
        
        // Create a new channel so the old one gets dropped
        let (tx, _) = channel(1);
        self.event_sender = tx;
        
        // Create a new receiver for potential restart
        let (_, rx) = channel(1);
        *self.event_receiver.lock().unwrap() = Some(rx);
    }
    
    /// Take ownership of the receiver
    fn take_receiver(&self) -> Receiver<BleEvent> {
        self.event_receiver.lock().unwrap().take().expect("Receiver already taken")
    }
}

impl Clone for EventBroker {
    fn clone(&self) -> Self {
        Self {
            next_subscriber_id: self.next_subscriber_id,
            subscribers: self.subscribers.clone(),
            inactive_timeout: self.inactive_timeout,
            cleanup_task: None,
            event_sender: self.event_sender.clone(),
            event_receiver: self.event_receiver.clone(),
        }
    }
}

impl Drop for EventBroker {
    fn drop(&mut self) {
        // Stop the cleanup task if it's running
        if let Some(task) = self.cleanup_task.take() {
            task.abort();
        }
    }
}

/// A helper to create a Stream from an event receiver
pub fn receiver_to_stream(mut rx: Receiver<BleEvent>) -> impl Stream<Item = BleEvent> {
    use futures::stream::StreamExt;
    
    async_stream::stream! {
        while let Some(event) = rx.recv().await {
            yield event;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_filter_all() {
        let filter = EventFilter::all();
        
        assert!(filter.matches(&BleEvent::DeviceDiscovered(DiscoveredDevice::default())));
        assert!(filter.matches(&BleEvent::DeviceLost(BDAddr::default())));
        assert!(filter.matches(&BleEvent::Error("test".to_string())));
        assert!(filter.matches(&BleEvent::AdapterChanged(crate::bluetooth::AdapterInfo::default())));
        assert!(filter.matches(&BleEvent::ScanCycleCompleted { devices_found: 0 }));
        assert!(filter.matches(&BleEvent::ScanningCompleted));
        assert!(filter.matches(&BleEvent::AirPodsDetected(DetectedAirPods::default())));
    }
    
    #[test]
    fn test_event_filter_airpods_only() {
        // Create a filter for AirPods events only
        let filter = EventFilter::airpods_only();
        
        // This should test that the filter only matches AirPodsDetected events
        // and not any other event types
        assert!(!filter.matches(&BleEvent::DeviceDiscovered(DiscoveredDevice::default())));
        assert!(!filter.matches(&BleEvent::DeviceLost(BDAddr::default())));
        assert!(!filter.matches(&BleEvent::Error("test".to_string())));
        assert!(!filter.matches(&BleEvent::AdapterChanged(crate::bluetooth::AdapterInfo::default())));
        assert!(!filter.matches(&BleEvent::ScanCycleCompleted { devices_found: 0 }));
        assert!(!filter.matches(&BleEvent::ScanningCompleted));
        assert!(filter.matches(&BleEvent::AirPodsDetected(DetectedAirPods::default())));
    }
    
    #[test]
    fn test_filter_with_min_rssi() {
        // Create a custom filter that only accepts events with RSSI >= -75
        let filter = EventFilter::custom(|event| {
            if let BleEvent::DeviceDiscovered(device) = event {
                return device.rssi.unwrap_or(-100) >= -75;
            }
            false
        });
        
        // Should match device with good signal
        assert!(filter.matches(&BleEvent::DeviceDiscovered(DiscoveredDevice {
            address: BDAddr::default(),
            rssi: Some(-70),
            ..DiscoveredDevice::default()
        })));
        
        // Should not match device with weak signal
        assert!(!filter.matches(&BleEvent::DeviceDiscovered(DiscoveredDevice {
            address: BDAddr::default(),
            rssi: Some(-80),
            ..DiscoveredDevice::default()
        })));
        
        // Should not match other event types
        assert!(!filter.matches(&BleEvent::DeviceLost(BDAddr::default())));
    }
    
    #[tokio::test]
    async fn test_event_broker_shutdown() {
        let mut broker = EventBroker::new();
        broker.start();
        
        // Add some subscribers
        let (_, _rx1) = broker.subscribe(EventFilter::all());
        let (_, _rx2) = broker.subscribe(EventFilter::all());
        
        // Test shutdown 
        broker.shutdown().await;
        
        // Only check that subscribers are cleared
        assert!(broker.subscribers.is_empty(), "Subscribers should be cleared after shutdown");
    }
} 