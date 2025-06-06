//! Mock implementations for Bluetooth components
//! Provides mock implementations for Bluetooth adapters, scanners, and related structures
//! to enable headless testing without requiring real Bluetooth hardware.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::future::Future;
use std::pin::Pin;
use async_trait::async_trait;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;
use mockall::mock;
use mockall::predicate::*;
use futures::future::{Ready, BoxFuture};
use futures::{FutureExt, StreamExt}; // Import these traits for boxed() and next()
use tokio_stream; // For ReceiverStream

use rustpods::bluetooth::{
    AdapterInfo, AdapterStatus, AdapterCapabilities, BleAdapterEvent, BluetoothAdapter,
    AdapterManager, BleScanner, ScanConfig, AirPodsBatteryStatus, EventFilter,
    BleScannerConfig, // Import the correct BleScannerConfig
    DiscoveredDevice,
};
use rustpods::config::Configurable; // Add the missing Configurable trait
use rustpods::airpods::{AirPodsBattery, AirPodsChargingState, AirPodsType};
use rustpods::airpods::detector::DetectedAirPods;
use btleplug::api::{BDAddr, ScanFilter};
use btleplug::platform::{Adapter, Manager, PeripheralId};
use std::str::FromStr;
use rustpods::error::{BluetoothError, RecoveryAction};

/// Generate a mock for the BluetoothAdapter
mock! {
    pub BluetoothAdapter {
        pub fn get_capabilities(&self) -> &AdapterCapabilities;
        pub fn get_status(&self) -> AdapterStatus;
        pub fn get_adapter(&self) -> Arc<Adapter>;
        pub async fn start_scanning(&self, scan_filter: ScanFilter) -> Result<Receiver<BleAdapterEvent>, BluetoothError>;
        pub async fn stop_scanning(&self) -> Result<(), BluetoothError>;
        pub async fn is_powered_on(&self) -> Result<bool, BluetoothError>;
        pub async fn discover_devices(&self) -> Result<Vec<DiscoveredDevice>, BluetoothError>;
        pub fn clone(&self) -> Self;
    }
}

/// Generate a mock for the BleScanner
mock! {
    pub BleScanner {
        pub fn new() -> Self;
        pub fn with_config(config: ScanConfig) -> Self;
        pub fn with_adapter_config(adapter: Arc<Adapter>, config: BleScannerConfig) -> Self;
        pub fn set_config(&mut self, config: ScanConfig);
        pub fn get_config(&self) -> &ScanConfig;
        pub fn as_configurable(&mut self) -> &mut dyn Configurable;
        pub async fn initialize(&mut self) -> Result<(), BluetoothError>;
        pub async fn start_scanning(&mut self) -> Result<Receiver<BleAdapterEvent>, BluetoothError>;
        pub async fn stop_scanning(&mut self) -> Result<(), BluetoothError>;
        pub async fn discover_devices(&mut self) -> Result<Vec<DiscoveredDevice>, BluetoothError>;
        pub fn is_scanning(&self) -> bool;
        pub fn get_discovered_devices(&self) -> Vec<DiscoveredDevice>;
        pub fn adapters(&self) -> Vec<AdapterInfo>;
    }
}

/// Generate a mock for the AdapterManager
mock! {
    pub AdapterManager {
        pub async fn new() -> Result<Self, BluetoothError>;
        pub async fn refresh_adapters(&mut self) -> Result<(), BluetoothError>;
        pub fn get_adapters(&self) -> &Vec<AdapterInfo>;
        pub fn get_selected_adapter_info<'a>(&'a self) -> Option<&'a AdapterInfo>;
        pub async fn get_selected_adapter(&self) -> Result<Adapter, BluetoothError>;
        pub fn select_adapter(&mut self, index: usize) -> Result<(), BluetoothError>;
        pub fn select_best_adapter(&mut self) -> Result<(), BluetoothError>;
        pub fn get_adapter_history<'a>(&'a self, adapter_id: &str) -> Option<&'a [(Instant, AdapterStatus)]>;
    }
}

/// A helper to create preconfigured mock Bluetooth adapters
pub struct MockBluetoothAdapterBuilder {
    capabilities: AdapterCapabilities,
    status: AdapterStatus,
    devices: Vec<DiscoveredDevice>,
    should_fail_scanning: bool,
    should_fail_discovery: bool,
}

impl Default for MockBluetoothAdapterBuilder {
    fn default() -> Self {
        Self {
            capabilities: AdapterCapabilities {
                supports_scanning: true,
                supports_connecting: true,
                is_powered_on: true,
                max_connections: 10,
                last_checked: Some(Instant::now()),
                status: AdapterStatus::Normal,
                supports_central_role: true,
                supports_advertising: true,
                adapter_info: String::new(),
            },
            status: AdapterStatus::Normal,
            devices: vec![],
            should_fail_scanning: false,
            should_fail_discovery: false,
        }
    }
}

impl MockBluetoothAdapterBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set adapter capabilities
    pub fn with_capabilities(mut self, capabilities: AdapterCapabilities) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// Set adapter status
    pub fn with_status(mut self, status: AdapterStatus) -> Self {
        self.status = status;
        self
    }

    /// Add a test device to the mock adapter
    pub fn with_device(mut self, device: DiscoveredDevice) -> Self {
        self.devices.push(device);
        self
    }

    /// Add multiple test devices to the mock adapter
    pub fn with_devices(mut self, devices: Vec<DiscoveredDevice>) -> Self {
        self.devices.extend(devices);
        self
    }

    /// Configure the adapter to fail when starting scan
    pub fn with_scanning_failure(mut self) -> Self {
        self.should_fail_scanning = true;
        self
    }

    /// Configure the adapter to fail when discovering devices
    pub fn with_discovery_failure(mut self) -> Self {
        self.should_fail_discovery = true;
        self
    }

    /// Build a configured mock BluetoothAdapter
    pub fn build(self) -> MockBluetoothAdapter {
        let mut mock = MockBluetoothAdapter::new();
        
        // Setup get_capabilities behavior
        // Use return_const for methods that return references
        mock.expect_get_capabilities()
            .return_const(self.capabilities.clone());
        
        // Setup get_status behavior
        let status = self.status;
        mock.expect_get_status()
            .returning(move || status);
        
        // Setup start_scanning behavior
        let should_fail = self.should_fail_scanning;
        let devices = self.devices.clone();
        mock.expect_start_scanning()
            .returning(move |_| {
                if should_fail {
                    return Err(BluetoothError::ScanFailed("Scan already in progress".to_string()));
                }
                let (tx, rx) = channel(100);
                // Simulate sending some events on the channel
                let devices_clone = devices.clone();
                tokio::spawn(async move {
                    // Send ScanStarted event
                    let _ = tx.send(BleAdapterEvent::ScanStarted).await;
                    
                    // Send device discovered events for each device
                    for device in devices_clone {
                        let _ = tx.send(BleAdapterEvent::DeviceDiscovered(device)).await;
                    }
                    
                    // Small delay to ensure events are received
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                });
                Ok(rx)
            });
        
        // Setup stop_scanning behavior
        mock.expect_stop_scanning()
            .returning(|| Ok(()));
        
        // Setup is_powered_on behavior
        let is_powered = self.capabilities.is_powered_on;
        mock.expect_is_powered_on()
            .returning(move || Ok(is_powered));
        
        // Setup discover_devices behavior
        let should_fail_discovery = self.should_fail_discovery;
        let devices = self.devices.clone();
        mock.expect_discover_devices()
            .returning(move || {
                if should_fail_discovery {
                    Err(BluetoothError::AdapterNotAvailable { reason: "Adapter not initialized".to_string(), recovery: RecoveryAction::ReconnectBluetooth })
                } else {
                    Ok(devices.clone())
                }
            });
        
        // Setup clone behavior
        mock.expect_clone()
            .returning(move || {
                // Create a new mock with the same configuration
                let mut clone = MockBluetoothAdapter::new();
                
                // Copy all the configuration to the clone
                // (This is simplified; in a real implementation you would copy all configurations)
                // let capabilities = self.capabilities.clone();
                // clone.expect_get_capabilities()
                //     .returning(move || &capabilities);
                
                let status = self.status;
                clone.expect_get_status()
                    .returning(move || status);
                
                clone
            });
        
        mock
    }
}

/// A helper to create a mock BleScanner with preconfigured behavior
pub struct MockBleScannerBuilder {
    config: ScanConfig,
    devices: Vec<DiscoveredDevice>,
    adapter_infos: Vec<AdapterInfo>,
    should_fail_initialize: bool,
    should_fail_scanning: bool,
    is_scanning: bool,
}

impl Default for MockBleScannerBuilder {
    fn default() -> Self {
        Self {
            config: ScanConfig::default(),
            devices: vec![],
            adapter_infos: vec![AdapterInfo {
                index: 0,
                is_default: true,
                vendor: Some("Mock".to_string()),
                address: Some(BDAddr::from([0x00, 0x11, 0x22, 0x33, 0x44, 0x55])),
                name: "Mock Adapter 1".to_string(),
                capabilities: AdapterCapabilities {
                    supports_scanning: true,
                    supports_connecting: true,
                    is_powered_on: true,
                    max_connections: 10,
                    last_checked: Some(Instant::now()),
                    status: AdapterStatus::Normal,
                    supports_central_role: true,
                    supports_advertising: true,
                    adapter_info: String::new(),
                },
            }],
            should_fail_initialize: false,
            should_fail_scanning: false,
            is_scanning: false,
        }
    }
}

impl MockBleScannerBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set scanner configuration
    pub fn with_config(mut self, config: ScanConfig) -> Self {
        self.config = config;
        self
    }

    /// Add a discovered device
    pub fn with_device(mut self, device: DiscoveredDevice) -> Self {
        self.devices.push(device);
        self
    }

    /// Add multiple discovered devices
    pub fn with_devices(mut self, devices: Vec<DiscoveredDevice>) -> Self {
        self.devices.extend(devices);
        self
    }

    /// Add an adapter info
    pub fn with_adapter_info(mut self, adapter_info: AdapterInfo) -> Self {
        self.adapter_infos.push(adapter_info);
        self
    }

    /// Configure initialization to fail
    pub fn with_init_failure(mut self) -> Self {
        self.should_fail_initialize = true;
        self
    }

    /// Configure scanning to fail
    pub fn with_scanning_failure(mut self) -> Self {
        self.should_fail_scanning = true;
        self
    }

    /// Set scanning state
    pub fn with_scanning_state(mut self, is_scanning: bool) -> Self {
        self.is_scanning = is_scanning;
        self
    }

    /// Build a configured mock BleScanner
    pub fn build(self) -> MockBleScanner {
        let mut mock = MockBleScanner::new();
        
        // Setup with_config constructor
        // let config = self.config.clone();
        // mock.expect_with_config()
        //     .returning(move |_| {
        //         // Return a new mock with the same configuration
        //         let mut new_mock = MockBleScanner::new();
        //         // Set up basic expectations on the new mock
        //         // (This is simplified, you'd need to set up all expected behaviors)
        //         new_mock
        //     });
        
        // Setup with_adapter_config constructor
        // mock.expect_with_adapter_config()
        //     .returning(|_, _| {
        //         // Return a new mock with default configuration
        //         MockBleScanner::new()
        //     });
        
        // Setup set_config behavior
        mock.expect_set_config()
            .returning(|_| ());
        
        // Setup get_config behavior
        // let config = self.config.clone();
        // mock.expect_get_config()
        //     .returning(move || &config);
        
        // Setup initialize behavior
        let should_fail_initialize = self.should_fail_initialize;
        mock.expect_initialize()
            .returning(move || {
                if should_fail_initialize {
                    Err(BluetoothError::AdapterNotAvailable { reason: "Initialization failed".to_string(), recovery: RecoveryAction::ReconnectBluetooth })
                } else {
                    Ok(())
                }
            });
        
        // Setup start_scanning behavior
        let should_fail_scanning = self.should_fail_scanning;
        let devices = self.devices.clone();
        mock.expect_start_scanning()
            .returning(move || {
                if should_fail_scanning {
                    Err(BluetoothError::ScanFailed("Scan already in progress".to_string()))
                } else {
                    let (tx, rx) = channel(100);
                    // Simulate sending some events on the channel
                    let devices_clone = devices.clone();
                    tokio::spawn(async move {
                        // Send ScanStarted event
                        let _ = tx.send(BleAdapterEvent::ScanStarted).await;
                        
                        // Send device discovered events for each device
                        for device in devices_clone {
                            let _ = tx.send(BleAdapterEvent::DeviceDiscovered(device)).await;
                        }
                        
                        // Small delay to ensure events are received
                        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    });
                    Ok(rx)
                }
            });
        
        // Setup stop_scanning behavior
        mock.expect_stop_scanning()
            .returning(|| Ok(()));
        
        // Setup discover_devices behavior
        let devices = self.devices.clone();
        mock.expect_discover_devices()
            .returning(move || Ok(devices.clone()));
        
        // Setup is_scanning behavior
        let is_scanning = self.is_scanning;
        mock.expect_is_scanning()
            .returning(move || is_scanning);
        
        // Setup get_discovered_devices behavior
        let devices = self.devices.clone();
        mock.expect_get_discovered_devices()
            .returning(move || devices.clone());
        
        // Setup adapters behavior
        let adapter_infos = self.adapter_infos.clone();
        mock.expect_adapters()
            .returning(move || adapter_infos.clone());
        
        mock
    }
}

/// Create a test discovered device for tests
pub fn create_test_discovered_device(
    address: &str,
    name: Option<&str>,
    rssi: Option<i16>,
    manufacturer_data: Option<HashMap<u16, Vec<u8>>>,
) -> DiscoveredDevice {
    DiscoveredDevice {
        address: from_str_or_panic(address),
        name: name.map(String::from),
        rssi,
        manufacturer_data: manufacturer_data.unwrap_or_else(HashMap::new),
        is_potential_airpods: false,
        last_seen: Instant::now(),
        is_connected: false,
        service_data: HashMap::new(),
        services: Vec::new(),
        tx_power_level: None,
    }
}

/// Create an Apple device for tests with manufacturer data
pub fn create_apple_device(
    address: &str,
    name: Option<&str>,
    rssi: Option<i16>,
    data: Vec<u8>,
) -> DiscoveredDevice {
    let mut manufacturer_data = HashMap::new();
    manufacturer_data.insert(0x004C, data); // Apple Company ID
    
    DiscoveredDevice {
        address: from_str_or_panic(address),
        name: name.map(String::from),
        rssi,
        manufacturer_data,
        is_potential_airpods: true,
        last_seen: Instant::now(),
        is_connected: false,
        service_data: HashMap::new(),
        services: Vec::new(),
        tx_power_level: None,
    }
}

/// Create AirPods manufacturer data with specified battery levels
pub fn create_airpods_manufacturer_data(
    model: AirPodsType,
    left_battery: u8,
    right_battery: u8,
    case_battery: u8,
    status_flags: u8,
) -> Vec<u8> {
    // Model identifier byte based on AirPods type
    let model_byte = match model {
        AirPodsType::AirPods1 => 0x01,
        AirPodsType::AirPods2 => 0x02,
        AirPodsType::AirPods3 => 0x05,
        AirPodsType::AirPodsPro => 0x03,
        AirPodsType::AirPodsPro2 => 0x06,
        AirPodsType::AirPodsMax => 0x04,
        AirPodsType::Unknown => 0x00,
    };
    
    // Create a basic manufacturer data payload
    // This is a simplified version of the actual AirPods protocol
    let mut data = vec![
        model_byte, 0x19, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
    ];
    
    // Set battery levels (simplified, not the actual bit positions)
    data[3] = (left_battery & 0x7F) | if status_flags & 0x01 > 0 { 0x80 } else { 0 };
    data[4] = (right_battery & 0x7F) | if status_flags & 0x02 > 0 { 0x80 } else { 0 };
    data[5] = (case_battery & 0x7F) | if status_flags & 0x04 > 0 { 0x80 } else { 0 };
    data[6] = status_flags;
    
    data
}

/// Generate mock for btleplug Peripheral trait
mock! {
    pub Peripheral {
        pub fn id(&self) -> PeripheralId;
        pub fn address(&self) -> BDAddr;
        pub fn properties(&self) -> Result<Option<btleplug::api::PeripheralProperties>, btleplug::Error>;
        pub async fn discover_services(&self) -> Result<Vec<btleplug::api::Characteristic>, btleplug::Error>;
        pub async fn write(&self, characteristic: &btleplug::api::Characteristic, data: &[u8], write_type: btleplug::api::WriteType) -> Result<(), btleplug::Error>;
        pub async fn read(&self, characteristic: &btleplug::api::Characteristic) -> Result<Vec<u8>, btleplug::Error>;
        pub async fn notify(&self, characteristic: &btleplug::api::Characteristic, enable: bool) -> Result<(), btleplug::Error>;
        pub async fn subscribe(&self, characteristic: &btleplug::api::Characteristic) -> Result<(), btleplug::Error>;
        pub async fn unsubscribe(&self, characteristic: &btleplug::api::Characteristic) -> Result<(), btleplug::Error>;
        pub async fn connect(&self) -> Result<(), btleplug::Error>;
        pub async fn disconnect(&self) -> Result<(), btleplug::Error>;
        pub fn is_connected(&self) -> Result<bool, btleplug::Error>;
        pub fn characteristics(&self) -> Vec<btleplug::api::Characteristic>;
        pub fn services(&self) -> Vec<btleplug::api::Service>;
        pub fn clone(&self) -> Self;
    }
}

/// Create mock battery status for testing
pub fn create_mock_battery_status(left: u8, right: u8, case: u8, charging_status: AirPodsChargingState) -> AirPodsBatteryStatus {
    let battery = AirPodsBattery {
        left: Some(left),
        right: Some(right),
        case: Some(case),
        charging: Some(charging_status),
    };
    
    AirPodsBatteryStatus::new(battery)
}

/// Helper function to convert string address to BDAddr
pub fn from_str_or_panic(s: &str) -> BDAddr {
    // Split the string by ':' and parse each part as a hex number
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 6 {
        panic!("Invalid BDAddr format: {}", s);
    }
    
    let mut bytes = [0u8; 6];
    for (i, part) in parts.iter().enumerate() {
        bytes[i] = u8::from_str_radix(part, 16)
            .unwrap_or_else(|_| panic!("Invalid byte in BDAddr: {}", part));
    }
    
    BDAddr::from(bytes)
}

impl MockBleScanner {
    // Add these methods to support the test_mock_scanner_initialization test
    
    /// Get all discovered devices
    pub fn get_devices(&self) -> Vec<DiscoveredDevice> {
        self.get_discovered_devices()
    }
    
    /// Get event history
    pub fn get_event_history(&self) -> Vec<BleAdapterEvent> {
        vec![] // Return empty history as a stub
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mock_bluetooth_adapter() {
        // Create a mock adapter with default configuration
        let mock_adapter = MockBluetoothAdapterBuilder::new().build();
        
        // Verify the adapter reports correct capabilities
        let capabilities = mock_adapter.get_capabilities();
        assert!(capabilities.supports_scanning);
        assert!(capabilities.is_powered_on);
        
        // Verify the adapter status
        let status = mock_adapter.get_status();
        assert_eq!(status, AdapterStatus::Normal);
        
        // Test the scanning functionality
        let events_rx = mock_adapter.start_scanning(ScanFilter::default()).await.unwrap();
        
        // We should be able to receive events
        let mut event_count = 0;
        let mut events_rx = tokio_stream::wrappers::ReceiverStream::new(events_rx);
        while let Some(event) = tokio::time::timeout(Duration::from_millis(500), events_rx.next()).await.unwrap() {
            match event {
                BleAdapterEvent::ScanStarted => {
                    println!("Scan started event received");
                },
                BleAdapterEvent::DeviceDiscovered(device) => {
                    println!("Device discovered: {:?}", device.address);
                },
                _ => {}
            }
            event_count += 1;
        }
        
        // We should have received at least the ScanStarted event
        assert!(event_count >= 1);
    }
    
    #[tokio::test]
    async fn test_mock_scanner_initialization() {
        // Test creating a scanner and initializing it
        let mut mock_scanner = MockBleScannerBuilder::new()
            .with_devices(vec![
                create_test_discovered_device("00:11:22:33:44:55", Some("Test Device 1"), Some(-60), None),
                create_apple_device("66:77:88:99:AA:BB", Some("AirPods"), Some(-60), vec![1, 2, 3, 4])
            ])
            .build();
        
        let init_result = mock_scanner.initialize().await;
        assert!(init_result.is_ok(), "Scanner initialization should succeed");
        
        // Should be able to get devices
        let devices = mock_scanner.get_devices();
        assert_eq!(devices.len(), 2, "Should have 2 devices");
        
        // Start scanning - we should get events
        let _events_rx = mock_scanner.start_scanning().await.unwrap();
        assert!(mock_scanner.is_scanning(), "Scanner should be scanning");
    }
    
    #[tokio::test]
    async fn test_mock_ble_scanner() {
        // Create a test device
        let test_device = create_test_discovered_device(
            "00:11:22:33:44:55",
            Some("Test Device"),
            Some(-60),
            None
        );
        
        // Create a mock scanner with the test device
        let mut mock_scanner = MockBleScannerBuilder::new()
            .with_device(test_device.clone())
            .build();
        
        // Test the scanner initialization
        let init_result = mock_scanner.initialize().await;
        assert!(init_result.is_ok());
        
        // Test getting discovered devices
        let devices = mock_scanner.get_discovered_devices();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].address, test_device.address);
        
        // Test scanning functionality
        let events_rx = mock_scanner.start_scanning().await.unwrap();
        
        // We should be able to receive events
        let mut event_count = 0;
        let mut events_rx = tokio_stream::wrappers::ReceiverStream::new(events_rx);
        while let Some(event) = tokio::time::timeout(Duration::from_millis(500), events_rx.next()).await.unwrap() {
            match event {
                BleAdapterEvent::ScanStarted => {
                    println!("Scan started event received");
                },
                BleAdapterEvent::DeviceDiscovered(device) => {
                    println!("Device discovered: {:?}", device.address);
                    assert_eq!(device.address, test_device.address);
                },
                _ => {}
            }
            event_count += 1;
        }
        
        // We should have received at least the ScanStarted event and one device
        assert!(event_count >= 2);
    }
    
    #[tokio::test]
    async fn test_create_apple_device() {
        // Create an Apple device with manufacturer data
        let device = create_apple_device(
            "00:11:22:33:44:55",
            Some("AirPods Test"),
            Some(-60),
            vec![0x01, 0x02, 0x03, 0x04]
        );
        
        // Verify the device has the expected properties
        assert_eq!(device.address.to_string(), "00:11:22:33:44:55");
        assert_eq!(device.name.unwrap(), "AirPods Test");
        assert_eq!(device.rssi.unwrap(), -60);
        
        // Verify the manufacturer data
        assert!(device.manufacturer_data.contains_key(&76));
        assert_eq!(device.manufacturer_data[&76], vec![0x01, 0x02, 0x03, 0x04]);
    }
    
    #[tokio::test]
    async fn test_create_airpods_manufacturer_data() {
        // Create manufacturer data for AirPods Pro
        let data = create_airpods_manufacturer_data(
            AirPodsType::AirPodsPro,
            80, // left battery
            75, // right battery
            90, // case battery
            0x03 // status flags (left and right charging)
        );
        
        // Verify the data has the expected format
        assert_eq!(data[0], 0x03); // AirPods Pro model byte
        
        // Check battery levels and charging flags (simplified for the test)
        assert_eq!(data[3] & 0x7F, 80); // left battery level
        assert_eq!(data[4] & 0x7F, 75); // right battery level
        assert_eq!(data[5] & 0x7F, 90); // case battery level
        
        // Check charging flags
        assert!(data[3] & 0x80 != 0); // left charging
        assert!(data[4] & 0x80 != 0); // right charging
        assert!(data[5] & 0x80 == 0); // case not charging
        
        // Status flags
        assert_eq!(data[6], 0x03);
    }
    
    #[tokio::test]
    async fn test_initialize_scanner_handles_error() {
        // Create a scanner that will return error
        let mut mock_scanner = MockBleScannerBuilder::new()
            .with_init_failure()
            .build();
        
        // Initialize should fail
        let init_result = mock_scanner.initialize().await;
        
        // Verify we got the expected error
        assert!(init_result.is_err());
        match init_result {
            Err(BluetoothError::AdapterNotAvailable { .. }) => {
                println!("✅ Expected error received");
            },
            _ => panic!("Expected AdapterNotAvailable error"),
        }
        
        // Should be able to try again - it will fail again but that's ok
        let events_rx = mock_scanner.start_scanning().await;
        assert!(events_rx.is_err());
    }
} 