//! Mock implementations for Bluetooth components
//!
//! This module provides mock implementations of Bluetooth adapter, scanner, and related
//! components for use in headless testing environments without requiring actual hardware.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::sync::mpsc::{self, Receiver, Sender, UnboundedReceiver, UnboundedSender};
use btleplug::api::BDAddr;
use tokio::task::JoinHandle;

use rustpods::bluetooth::{
    DiscoveredDevice, BleEvent, EventFilter, 
    AirPodsBatteryStatus
};
use rustpods::error::BluetoothError;
use rustpods::airpods::{DetectedAirPods, AirPodsType, AirPodsBattery, AirPodsChargingState};
use rustpods::config::AppConfig;

// SECTION: Mock Bluetooth Adapter

/// Mock implementation of the BluetoothAdapter for testing
#[derive(Debug, Clone)]
pub struct MockBluetoothAdapter {
    /// Whether the adapter is enabled
    pub is_enabled: bool,
    /// Whether the adapter supports scanning
    pub supports_scanning: bool,
    /// Whether the adapter is currently scanning
    pub is_scanning: bool,
    /// Last error that occurred
    pub last_error: Option<String>,
    /// Known devices
    pub devices: Arc<Mutex<HashMap<String, MockDevice>>>,
    /// Event history
    pub events: Arc<Mutex<Vec<MockAdapterEvent>>>,
    /// Custom behavior flags for testing
    pub behavior_flags: Arc<Mutex<MockBehaviorFlags>>,
}

/// Mock device that stores additional testing details
#[derive(Debug, Clone)]
pub struct MockDevice {
    /// Device address
    pub address: String,
    /// Device name
    pub name: Option<String>,
    /// Signal strength
    pub rssi: Option<i16>,
    /// Manufacturer data
    pub manufacturer_data: HashMap<u16, Vec<u8>>,
    /// Is the device connected
    pub is_connected: bool,
    /// Is the device an AirPods
    pub is_airpods: bool,
    /// AirPods battery status if applicable
    pub battery_status: Option<AirPodsBatteryStatus>,
    /// AirPods type if applicable
    pub airpods_type: Option<AirPodsType>,
    /// Last seen time
    pub last_seen: Instant,
    /// Connection attempts
    pub connection_attempts: usize,
}

/// Events that can be triggered by the mock adapter
#[derive(Debug, Clone)]
pub enum MockAdapterEvent {
    /// Device discovered
    DeviceDiscovered(String),
    /// Device connected
    DeviceConnected(String),
    /// Device disconnected
    DeviceDisconnected(String),
    /// Scan started
    ScanStarted,
    /// Scan stopped
    ScanStopped,
    /// Adapter enabled
    AdapterEnabled,
    /// Adapter disabled
    AdapterDisabled,
    /// Error occurred
    Error(String),
}

/// Behavior configuration for the mock adapter
#[derive(Debug, Clone, Default)]
pub struct MockBehaviorFlags {
    /// Whether to fail on next scan
    pub fail_next_scan: bool,
    /// Whether to fail on next connect
    pub fail_next_connect: bool,
    /// Delay to simulate operations
    pub operation_delay: Option<Duration>,
    /// Make the next device discovery fail
    pub fail_next_discovery: bool,
    /// Simulate device battery update failure
    pub fail_battery_update: bool,
    /// Number of fake devices to generate on scan
    pub fake_device_count: usize,
    /// Percentage of fake devices that should be AirPods
    pub airpods_percentage: usize,
}

impl MockBluetoothAdapter {
    /// Create a new mock adapter
    pub fn new() -> Self {
        Self {
            is_enabled: true,
            supports_scanning: true,
            is_scanning: false,
            last_error: None,
            devices: Arc::new(Mutex::new(HashMap::new())),
            events: Arc::new(Mutex::new(Vec::new())),
            behavior_flags: Arc::new(Mutex::new(MockBehaviorFlags::default())),
        }
    }
    
    /// Configure the adapter's behavior for testing
    pub fn with_behavior_flags(mut self, flags: MockBehaviorFlags) -> Self {
        *self.behavior_flags.lock().unwrap() = flags;
        self
    }
    
    /// Add a test device to the adapter
    pub fn add_device(&self, device: MockDevice) {
        let mut devices = self.devices.lock().unwrap();
        devices.insert(device.address.clone(), device);
    }
    
    /// Record an event
    pub fn record_event(&self, event: MockAdapterEvent) {
        let mut events = self.events.lock().unwrap();
        events.push(event);
    }
    
    /// Start scanning for devices
    pub async fn start_scanning(&mut self) -> Result<(), BluetoothError> {
        // Check if we should fail this scan
        {
            let mut flags = self.behavior_flags.lock().unwrap();
            if flags.fail_next_scan {
                flags.fail_next_scan = false;
                self.last_error = Some("Scan failed".to_string());
                self.record_event(MockAdapterEvent::Error("Scan failed".to_string()));
                return Err(BluetoothError::Other("Scanning not supported".to_string()));
            }
            
            // Apply operation delay if configured
            if let Some(delay) = flags.operation_delay {
                tokio::time::sleep(delay).await;
            }
        }
        
        if !self.is_enabled {
            self.last_error = Some("Adapter disabled".to_string());
            self.record_event(MockAdapterEvent::Error("Adapter disabled".to_string()));
            return Err(BluetoothError::Other("Bluetooth is disabled".to_string()));
        }
        
        if !self.supports_scanning {
            self.last_error = Some("Scanning not supported".to_string());
            self.record_event(MockAdapterEvent::Error("Scanning not supported".to_string()));
            return Err(BluetoothError::Other("Scanning not supported".to_string()));
        }
        
        self.is_scanning = true;
        self.record_event(MockAdapterEvent::ScanStarted);
        
        // Generate fake devices if configured
        {
            let flags = self.behavior_flags.lock().unwrap();
            if flags.fake_device_count > 0 {
                for i in 0..flags.fake_device_count {
                    let is_airpods = i % 100 < flags.airpods_percentage;
                    let addr = format!("11:22:33:44:55:{:02X}", i);
                    let name = if is_airpods {
                        Some(format!("AirPods #{}", i))
                    } else {
                        Some(format!("Device #{}", i))
                    };
                    
                    let mut manufacturer_data = HashMap::new();
                    if is_airpods {
                        // Mimics AirPods manufacturer data
                        manufacturer_data.insert(0x004C, vec![0x07, 0x19, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06]);
                    }
                    
                    let device = MockDevice {
                        address: addr.clone(),
                        name,
                        rssi: Some(-60 - (i as i16 % 40)),
                        manufacturer_data,
                        is_connected: false,
                        is_airpods,
                        battery_status: if is_airpods {
                            Some(AirPodsBatteryStatus {
                                battery: AirPodsBattery {
                                    left: Some(80 - (i as u8 % 30)),
                                    right: Some(85 - (i as u8 % 20)),
                                    case: Some(90 - (i as u8 % 15)),
                                    charging: Some(if i % 2 == 0 {
                                        AirPodsChargingState::BothBudsCharging
                                    } else if i % 3 == 0 {
                                        AirPodsChargingState::LeftCharging
                                    } else if i % 5 == 0 {
                                        AirPodsChargingState::CaseCharging
                                    } else {
                                        AirPodsChargingState::NotCharging
                                    }),
                                },
                                last_updated: Instant::now(),
                            })
                        } else {
                            None
                        },
                        airpods_type: if is_airpods {
                            Some(match i % 4 {
                                0 => AirPodsType::AirPods2,
                                1 => AirPodsType::AirPodsPro,
                                2 => AirPodsType::AirPods3,
                                _ => AirPodsType::AirPodsMax,
                            })
                        } else {
                            None
                        },
                        last_seen: Instant::now(),
                        connection_attempts: 0,
                    };
                    
                    self.add_device(device);
                    self.record_event(MockAdapterEvent::DeviceDiscovered(addr));
                }
            }
        }
        
        Ok(())
    }
    
    /// Stop scanning for devices
    pub async fn stop_scanning(&mut self) -> Result<(), BluetoothError> {
        if !self.is_scanning {
            return Ok(());
        }
        
        self.is_scanning = false;
        self.record_event(MockAdapterEvent::ScanStopped);
        
        // Apply operation delay if configured
        let flags = self.behavior_flags.lock().unwrap();
        if let Some(delay) = flags.operation_delay {
            tokio::time::sleep(delay).await;
        }
        
        Ok(())
    }
    
    /// Connect to a device
    pub async fn connect_to_device(&mut self, address: &str) -> Result<(), BluetoothError> {
        // Check if we should fail this connection
        {
            let mut flags = self.behavior_flags.lock().unwrap();
            if flags.fail_next_connect {
                flags.fail_next_connect = false;
                self.last_error = Some(format!("Failed to connect to {}", address));
                self.record_event(MockAdapterEvent::Error(format!("Failed to connect to {}", address)));
                return Err(BluetoothError::ConnectionFailed(format!("Failed to connect to {}", address)));
            }
            
            // Apply operation delay if configured
            if let Some(delay) = flags.operation_delay {
                tokio::time::sleep(delay).await;
            }
        }
        
        let mut devices = self.devices.lock().unwrap();
        match devices.get_mut(address) {
            Some(device) => {
                device.connection_attempts += 1;
                device.is_connected = true;
                device.last_seen = Instant::now();
                self.record_event(MockAdapterEvent::DeviceConnected(address.to_string()));
                Ok(())
            }
            None => {
                self.last_error = Some(format!("Device not found: {}", address));
                self.record_event(MockAdapterEvent::Error(format!("Device not found: {}", address)));
                Err(BluetoothError::DeviceNotFound(format!("Device not found: {}", address)))
            }
        }
    }
    
    /// Disconnect from a device
    pub async fn disconnect_from_device(&mut self, address: &str) -> Result<(), BluetoothError> {
        // Check if we should fail this disconnect
        {
            let flags = self.behavior_flags.lock().unwrap();
            
            // Apply operation delay if configured
            if let Some(delay) = flags.operation_delay {
                tokio::time::sleep(delay).await;
            }
        }
        
        // Update the device connection state
        let mut devices = self.devices.lock().unwrap();
        
        if let Some(device) = devices.get_mut(address) {
            device.is_connected = false;
            
            // Record the event
            self.record_event(MockAdapterEvent::DeviceDisconnected(address.to_string()));
            Ok(())
        } else {
            Err(BluetoothError::DeviceNotFound(format!("Device not found: {}", address)))
        }
    }
    
    /// Get device battery status (for AirPods)
    pub async fn get_device_battery(&self, address: &str) -> Result<AirPodsBatteryStatus, BluetoothError> {
        // Check if we should fail this battery update
        {
            let flags = self.behavior_flags.lock().unwrap();
            if flags.fail_battery_update {
                // Don't try to modify self.last_error since self is immutable
                return Err(BluetoothError::Other(format!("Failed to get battery status for {}", address)));
            }
            
            // Apply operation delay if configured
            if let Some(delay) = flags.operation_delay {
                tokio::time::sleep(delay).await;
            }
        }
        
        let devices = self.devices.lock().unwrap();
        match devices.get(address) {
            Some(device) => {
                if let Some(battery) = &device.battery_status {
                    Ok(battery.clone())
                } else {
                    Err(BluetoothError::Other(format!("Device {} does not support battery status", address)))
                }
            }
            None => {
                Err(BluetoothError::DeviceNotFound(format!("Device not found: {}", address)))
            }
        }
    }
    
    /// Get a discovered device by address
    pub fn get_device(&self, address: &str) -> Option<DiscoveredDevice> {
        let devices = self.devices.lock().unwrap();
        devices.get(address).map(|mock_device| {
            DiscoveredDevice {
                address: BDAddr::from_str_hex(address).unwrap_or_default(),
                name: mock_device.name.clone(),
                rssi: mock_device.rssi,
                manufacturer_data: mock_device.manufacturer_data.clone(),
                is_potential_airpods: mock_device.is_airpods,
                last_seen: mock_device.last_seen,
                is_connected: mock_device.is_connected,
                service_data: HashMap::new(),
                services: Vec::new(),
                tx_power_level: None,
            }
        })
    }
    
    /// Get all discovered devices
    pub fn get_all_devices(&self) -> Vec<DiscoveredDevice> {
        let devices = self.devices.lock().unwrap();
        devices.values().map(|mock_device| {
            DiscoveredDevice {
                address: BDAddr::from_str_hex(&mock_device.address).unwrap_or_default(),
                name: mock_device.name.clone(),
                rssi: mock_device.rssi,
                manufacturer_data: mock_device.manufacturer_data.clone(),
                is_potential_airpods: mock_device.is_airpods,
                last_seen: mock_device.last_seen,
                is_connected: mock_device.is_connected,
                service_data: HashMap::new(),
                services: Vec::new(),
                tx_power_level: None,
            }
        }).collect()
    }
    
    /// Get all AirPods devices
    pub fn get_airpods_devices(&self) -> Vec<DetectedAirPods> {
        let devices = self.devices.lock().unwrap();
        devices.values()
            .filter(|device| device.is_airpods)
            .filter_map(|mock_device| {
                // Only include devices that are AirPods
                if !mock_device.is_airpods || mock_device.airpods_type.is_none() || mock_device.battery_status.is_none() {
                    return None;
                }
                
                Some(DetectedAirPods {
                    address: BDAddr::from_str_hex(&mock_device.address).unwrap_or_default(),
                    name: mock_device.name.clone(),
                    device_type: mock_device.airpods_type.as_ref().unwrap_or(&AirPodsType::AirPods2).clone(),
                    battery: mock_device.battery_status.as_ref().map(|status| status.battery.clone()),
                    rssi: mock_device.rssi,
                    last_seen: mock_device.last_seen,
                    is_connected: mock_device.is_connected,
                })
            })
            .collect()
    }
}

impl Default for MockBluetoothAdapter {
    fn default() -> Self {
        Self::new()
    }
}

// Helper extension for BDAddr to create from hex string
trait BDAddrExt {
    fn from_str_hex(s: &str) -> Option<Self> where Self: Sized;
}

impl BDAddrExt for BDAddr {
    fn from_str_hex(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 6 {
            return None;
        }
        
        let mut bytes = [0u8; 6];
        for (i, part) in parts.iter().enumerate() {
            if let Ok(byte) = u8::from_str_radix(part, 16) {
                bytes[i] = byte;
            } else {
                return None;
            }
        }
        
        Some(BDAddr::from(bytes))
    }
}

// SECTION: Mock Scanner

/// A mock implementation of the BleScanner
pub struct MockBleScanner {
    /// Mock adapter
    adapter: Arc<Mutex<MockBluetoothAdapter>>,
    /// Is scanning in progress
    is_scanning: bool,
    /// Discovered devices
    devices: Arc<Mutex<HashMap<BDAddr, DiscoveredDevice>>>,
    /// Event sender
    event_sender: Option<mpsc::Sender<BleEvent>>,
    /// Scan task handle
    scan_task: Option<JoinHandle<()>>,
    /// Cancel channel
    cancel_sender: Option<mpsc::Sender<()>>,
    /// Event history
    event_history: Arc<Mutex<Vec<BleEvent>>>,
}

impl MockBleScanner {
    /// Create a new mock scanner
    pub fn new(adapter: MockBluetoothAdapter) -> Self {
        Self {
            adapter: Arc::new(Mutex::new(adapter)),
            is_scanning: false,
            devices: Arc::new(Mutex::new(HashMap::new())),
            event_sender: None,
            scan_task: None,
            cancel_sender: None,
            event_history: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Create a new method to handle scanning properly
    async fn run_scan_task(
        adapter_clone: Arc<Mutex<MockBluetoothAdapter>>,
        devices_clone: Arc<Mutex<HashMap<BDAddr, DiscoveredDevice>>>,
        event_tx: mpsc::Sender<BleEvent>,
        mut cancel_rx: mpsc::Receiver<()>,
        event_history_clone: Arc<Mutex<Vec<BleEvent>>>,
    ) {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    // Get a copy of the devices before locking
                    let mock_devices = {
                        let adapter_guard = adapter_clone.lock().unwrap();
                        adapter_guard.get_all_devices()
                    };
                    
                    for device in mock_devices {
                        // Update devices map
                        {
                            let mut devices = devices_clone.lock().unwrap();
                            devices.insert(device.address, device.clone());
                        }
                        
                        // Send discovery event
                        let event = BleEvent::DeviceDiscovered(device);
                        if let Err(e) = event_tx.send(event.clone()).await {
                            log::error!("Failed to send event: {:?}", e);
                            break;
                        }
                        
                        // Record event
                        {
                            let mut history = event_history_clone.lock().unwrap();
                            history.push(event);
                        }
                    }
                    
                    // Simulate AirPods device updates
                    let airpods = {
                        let adapter_guard = adapter_clone.lock().unwrap();
                        adapter_guard.get_airpods_devices()
                    };
                    
                    for device in airpods {
                        // Send AirPods event
                        let event = BleEvent::AirPodsDetected(device);
                        if let Err(e) = event_tx.send(event.clone()).await {
                            log::error!("Failed to send event: {:?}", e);
                            break;
                        }
                        
                        // Record event
                        {
                            let mut history = event_history_clone.lock().unwrap();
                            history.push(event);
                        }
                    }
                }
                _ = cancel_rx.recv() => {
                    // Cancellation received, exit loop
                    break;
                }
            }
        }
    }
    
    /// Start scanning for devices
    pub async fn start_scanning(&mut self) -> Result<mpsc::Receiver<BleEvent>, BluetoothError> {
        if self.is_scanning {
            return Err(BluetoothError::Other("Scan already in progress".to_string()));
        }
        
        // Create channels for events and cancellation
        let (event_tx, event_rx) = mpsc::channel(100);
        let (cancel_tx, cancel_rx) = mpsc::channel(1);
        
        self.event_sender = Some(event_tx.clone());
        self.cancel_sender = Some(cancel_tx);
        
        // Start mock scanning in the adapter
        {
            let mut adapter = self.adapter.lock().unwrap();
            adapter.start_scanning().await?;
        }
        
        self.is_scanning = true;
        
        // Start a background task to simulate scanning
        let adapter_clone = self.adapter.clone();
        let devices_clone = self.devices.clone();
        let event_history_clone = self.event_history.clone();
        
        // Use the new method to avoid thread safety issues
        let task = tokio::spawn(Self::run_scan_task(
            adapter_clone,
            devices_clone,
            event_tx,
            cancel_rx,
            event_history_clone
        ));
        
        self.scan_task = Some(task);
        
        Ok(event_rx)
    }
    
    /// Stop scanning for devices
    pub async fn stop_scanning(&mut self) -> Result<(), BluetoothError> {
        if !self.is_scanning {
            return Ok(());
        }
        
        // Send cancellation signal
        if let Some(sender) = &self.cancel_sender {
            let _ = sender.send(()).await;
        }
        
        // Stop mock scanning in the adapter
        {
            let mut adapter = self.adapter.lock().unwrap();
            adapter.stop_scanning().await?;
        }
        
        self.is_scanning = false;
        self.scan_task = None;
        self.cancel_sender = None;
        
        Ok(())
    }
    
    /// Get all discovered devices
    pub fn get_devices(&self) -> Vec<DiscoveredDevice> {
        let devices = self.devices.lock().unwrap();
        devices.values().cloned().collect()
    }
    
    /// Get device by address
    pub fn get_device(&self, address: &BDAddr) -> Option<DiscoveredDevice> {
        let devices = self.devices.lock().unwrap();
        devices.get(address).cloned()
    }
    
    /// Is scanning in progress
    pub fn is_scanning(&self) -> bool {
        self.is_scanning
    }
    
    /// Get event history
    pub fn get_event_history(&self) -> Vec<BleEvent> {
        let history = self.event_history.lock().unwrap();
        history.clone()
    }
    
    /// Get the mock adapter
    pub fn get_adapter(&self) -> Arc<Mutex<MockBluetoothAdapter>> {
        self.adapter.clone()
    }
}

// SECTION: Mock Event Broker

/// A mock implementation of the EventBroker
pub struct MockEventBroker {
    /// Internal sender for events
    sender: mpsc::UnboundedSender<BleEvent>,
    /// Subscriptions by ID
    subscriptions: Arc<Mutex<HashMap<String, (EventFilter, mpsc::UnboundedSender<BleEvent>)>>>,
    /// Next subscription ID
    next_id: Arc<Mutex<usize>>,
    /// Event history
    event_history: Arc<Mutex<Vec<BleEvent>>>,
    /// Is running
    is_running: Arc<Mutex<bool>>,
    /// Task handle
    task_handle: Option<JoinHandle<()>>,
}

impl MockEventBroker {
    /// Create a new mock event broker
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let subscriptions = Arc::new(Mutex::new(HashMap::new()));
        let next_id = Arc::new(Mutex::new(0));
        let event_history = Arc::new(Mutex::new(Vec::new()));
        let is_running = Arc::new(Mutex::new(false));
        
        Self {
            sender,
            subscriptions,
            next_id,
            event_history,
            is_running,
            task_handle: None,
        }
    }
    
    /// Start the event broker
    pub fn start(&mut self) {
        let subscriptions = self.subscriptions.clone();
        let mut receiver: tokio::sync::mpsc::UnboundedReceiver<BleEvent> = mpsc::unbounded_channel().1; // Dummy receiver for initial setup
        std::mem::swap(&mut receiver, &mut mpsc::unbounded_channel().1);
        let event_history = self.event_history.clone();
        let is_running = self.is_running.clone();
        
        // Set running state
        {
            let mut running = is_running.lock().unwrap();
            *running = true;
        }
        
        // Create task to process events
        let task = tokio::spawn(async move {
            while *is_running.lock().unwrap() {
                // Just do a little delay since we're using a dummy receiver
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });
        
        // Store task handle
        self.task_handle = Some(task);
        // No return value
    }
    
    /// Get the event sender
    pub fn get_sender(&self) -> mpsc::UnboundedSender<BleEvent> {
        self.sender.clone()
    }
    
    /// Subscribe to events with a filter
    pub fn subscribe(&self, filter: EventFilter) -> (String, mpsc::UnboundedReceiver<BleEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        
        // Generate a new subscription ID
        let id = {
            let mut next_id = self.next_id.lock().unwrap();
            let id = format!("sub_{}", *next_id);
            *next_id += 1;
            id
        };
        
        // Add subscription
        {
            let mut subscriptions = self.subscriptions.lock().unwrap();
            subscriptions.insert(id.clone(), (filter, tx));
        }
        
        (id, rx)
    }
    
    /// Unsubscribe from events
    pub fn unsubscribe(&self, id: String) {
        let mut subscriptions = self.subscriptions.lock().unwrap();
        subscriptions.remove(&id);
    }
    
    /// Send an event
    pub async fn send_event(&self, event: BleEvent) -> Result<(), String> {
        // Record event
        {
            let mut history = self.event_history.lock().unwrap();
            history.push(event.clone());
        }
        
        // Distribute to subscribers
        let subscriptions = self.subscriptions.lock().unwrap();
        for (_, (filter, sender)) in subscriptions.iter() {
            if filter.matches(&event) {
                if let Err(e) = sender.send(event.clone()) {
                    return Err(format!("Failed to send event: {:?}", e));
                }
            }
        }
        
        Ok(())
    }
    
    /// Shutdown the broker
    pub async fn shutdown(&mut self) {
        // Set not running
        {
            let mut running = self.is_running.lock().unwrap();
            *running = false;
        }
        
        // Wait for task to complete
        if let Some(handle) = self.task_handle.take() {
            let _ = handle.await;
        }
    }
    
    /// Get event history
    pub fn get_event_history(&self) -> Vec<BleEvent> {
        let history = self.event_history.lock().unwrap();
        history.clone()
    }
    
    /// Get subscription count
    pub fn get_subscription_count(&self) -> usize {
        let subscriptions = self.subscriptions.lock().unwrap();
        subscriptions.len()
    }
}

impl Default for MockEventBroker {
    fn default() -> Self {
        Self::new()
    }
}

// SECTION: Tests

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mock_bluetooth_adapter() {
        let adapter = MockBluetoothAdapter::new();
        assert!(adapter.is_enabled);
        assert!(adapter.supports_scanning);
        assert!(!adapter.is_scanning);
        assert!(adapter.last_error.is_none());
    }
    
    #[tokio::test]
    async fn test_mock_scanning() {
        let mut flags = MockBehaviorFlags::default();
        flags.fake_device_count = 5;
        flags.airpods_percentage = 40;
        
        let mut adapter = MockBluetoothAdapter::new().with_behavior_flags(flags);
        
        // Start scanning
        let result = adapter.start_scanning().await;
        assert!(result.is_ok());
        assert!(adapter.is_scanning);
        
        // Check events
        let events = adapter.events.lock().unwrap();
        assert!(events.iter().any(|e| matches!(e, MockAdapterEvent::ScanStarted)));
        
        // Check devices
        let devices = adapter.get_all_devices();
        assert_eq!(devices.len(), 5);
        
        // Check AirPods devices (should be around 40%)
        let airpods = adapter.get_airpods_devices();
        assert!(airpods.len() > 0);
    }
    
    #[tokio::test]
    async fn test_mock_scanner() {
        let mut flags = MockBehaviorFlags::default();
        flags.fake_device_count = 3;
        flags.airpods_percentage = 100; // All devices should be AirPods
        
        let adapter = MockBluetoothAdapter::new().with_behavior_flags(flags);
        let mut scanner = MockBleScanner::new(adapter);
        
        // Start scanning
        let event_rx = scanner.start_scanning().await.expect("Failed to start scanning");
        
        // Wait a bit for events
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Stop scanning
        scanner.stop_scanning().await.expect("Failed to stop scanning");
        
        // Check discovered devices
        let devices = scanner.get_devices();
        assert_eq!(devices.len(), 3);
        
        // Check event history
        let events = scanner.get_event_history();
        assert!(events.len() > 0);
        assert!(events.iter().any(|e| matches!(e, BleEvent::DeviceDiscovered(_))));
        assert!(events.iter().any(|e| matches!(e, BleEvent::AirPodsDetected(_))));
    }
    
    #[tokio::test]
    async fn test_mock_event_broker() {
        let mut broker = MockEventBroker::new();
        broker.start();
        
        // Create a subscription
        let (id, mut rx) = broker.subscribe(EventFilter::all());
        
        // Send an event
        let device = DiscoveredDevice {
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
        };
        
        let event = BleEvent::DeviceDiscovered(device);
        broker.send_event(event.clone()).await.expect("Failed to send event");
        
        // Check event history
        let events = broker.get_event_history();
        assert_eq!(events.len(), 1);
        
        // Shutdown
        broker.shutdown().await;
    }
} 