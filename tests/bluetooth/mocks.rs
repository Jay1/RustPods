//! Mock implementations for Bluetooth components
//! Provides mock implementations for Bluetooth adapters, scanners, and related structures
//! to enable headless testing without requiring real Bluetooth hardware.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::future::Future;
use std::pin::Pin;
use async_trait::async_trait;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;
use mockall::mock;
use mockall::predicate::*;

use rustpods::bluetooth::{
    AdapterInfo, AdapterStatus, AdapterCapabilities, BleError, 
    DiscoveredDevice, BleAdapterEvent, BluetoothAdapter,
    AdapterManager, BleScanner, ScanConfig, BleScannerConfig
};
use rustpods::airpods::{DetectedAirPods, AirPodsType, AirPodsBattery};
use btleplug::api::{BDAddr, Central, Peripheral, ScanFilter};
use btleplug::platform::{Adapter, Manager, PeripheralId};

/// Generate a mock for the BluetoothAdapter
mock! {
    pub BluetoothAdapter {
        pub fn get_capabilities(&self) -> &AdapterCapabilities;
        pub fn get_status(&self) -> AdapterStatus;
        pub fn get_adapter(&self) -> Arc<Adapter>;
        pub async fn start_scanning(&self, scan_filter: ScanFilter) -> Result<Receiver<BleAdapterEvent>, BleError>;
        pub async fn stop_scanning(&self) -> Result<(), BleError>;
        pub async fn is_powered_on(&self) -> Result<bool, BleError>;
        pub async fn discover_devices(&self) -> Result<Vec<DiscoveredDevice>, BleError>;
        pub fn clone(&self) -> Self;
    }

    impl Clone for BluetoothAdapter {
        fn clone(&self) -> Self;
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
        pub async fn initialize(&mut self) -> Result<(), BleError>;
        pub async fn start_scanning(&mut self) -> Result<Receiver<BleAdapterEvent>, BleError>;
        pub async fn stop_scanning(&mut self) -> Result<(), BleError>;
        pub async fn discover_devices(&mut self) -> Result<Vec<DiscoveredDevice>, BleError>;
        pub fn is_scanning(&self) -> bool;
        pub fn get_discovered_devices(&self) -> Vec<DiscoveredDevice>;
        pub fn adapters(&self) -> Vec<AdapterInfo>;
    }
}

/// Generate a mock for the AdapterManager
mock! {
    pub AdapterManager {
        pub async fn new() -> Result<Self, BleError>;
        pub async fn refresh_adapters(&mut self) -> Result<(), BleError>;
        pub fn get_adapters(&self) -> &Vec<AdapterInfo>;
        pub fn get_selected_adapter_info(&self) -> Option<&AdapterInfo>;
        pub async fn get_selected_adapter(&self) -> Result<Adapter, BleError>;
        pub fn select_adapter(&mut self, index: usize) -> Result<(), BleError>;
        pub fn select_best_adapter(&mut self) -> Result<(), BleError>;
        pub fn get_adapter_history(&self, adapter_id: &str) -> Option<&[(Instant, AdapterStatus)]>;
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
                max_connections: Some(10),
                last_checked: Instant::now(),
                status: AdapterStatus::Normal,
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
        let capabilities = self.capabilities.clone();
        mock.expect_get_capabilities()
            .returning(move || &capabilities);
        
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
                    return futures::future::ready(Err(BleError::ScanningAlreadyInProgress)).boxed();
                }
                
                let devices = devices.clone();
                let (tx, rx) = channel(100);
                
                // Simulate sending some events on the channel
                let tx_clone = tx.clone();
                tokio::spawn(async move {
                    // Send scanning started event
                    let _ = tx_clone.send(BleAdapterEvent::ScanStarted).await;
                    
                    // Small delay to simulate scanning process
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    
                    // Send device discovered events for each device
                    for device in devices {
                        let _ = tx_clone.send(BleAdapterEvent::DeviceDiscovered(device)).await;
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                });
                
                futures::future::ready(Ok(rx)).boxed()
            });
        
        // Setup stop_scanning behavior
        mock.expect_stop_scanning()
            .returning(|| futures::future::ready(Ok(())).boxed());
        
        // Setup is_powered_on behavior
        let is_powered = self.capabilities.is_powered_on;
        mock.expect_is_powered_on()
            .returning(move || futures::future::ready(Ok(is_powered)).boxed());
        
        // Setup discover_devices behavior
        let should_fail = self.should_fail_discovery;
        let devices = self.devices.clone();
        mock.expect_discover_devices()
            .returning(move || {
                if should_fail {
                    return futures::future::ready(Err(BleError::AdapterNotConnected)).boxed();
                }
                futures::future::ready(Ok(devices.clone())).boxed()
            });
        
        // Setup clone behavior
        mock.expect_clone()
            .returning(move || {
                // Create a new mock with the same configuration
                let mut clone = MockBluetoothAdapter::new();
                
                // Copy all the configuration to the clone
                // (This is simplified; in a real implementation you would copy all configurations)
                let capabilities = self.capabilities.clone();
                clone.expect_get_capabilities()
                    .returning(move || &capabilities);
                
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
                id: "mock-adapter-1".to_string(),
                address: BDAddr::from([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]),
                name: Some("Mock Adapter 1".to_string()),
                capabilities: AdapterCapabilities {
                    supports_scanning: true,
                    supports_connecting: true,
                    is_powered_on: true,
                    max_connections: Some(10),
                    last_checked: Instant::now(),
                    status: AdapterStatus::Normal,
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
        let config = self.config.clone();
        mock.expect_with_config()
            .returning(move |_| {
                // Return a new mock with the same configuration
                let mut new_mock = MockBleScanner::new();
                // Set up basic expectations on the new mock
                // (This is simplified, you'd need to set up all expected behaviors)
                new_mock
            });
        
        // Setup set_config behavior
        mock.expect_set_config()
            .returning(|_| ());
        
        // Setup get_config behavior
        let config = self.config.clone();
        mock.expect_get_config()
            .returning(move || &config);
        
        // Setup initialize behavior
        let should_fail = self.should_fail_initialize;
        mock.expect_initialize()
            .returning(move || {
                if should_fail {
                    futures::future::ready(Err(BleError::AdapterNotFound)).boxed()
                } else {
                    futures::future::ready(Ok(())).boxed()
                }
            });
        
        // Setup start_scanning behavior
        let should_fail = self.should_fail_scanning;
        let devices = self.devices.clone();
        mock.expect_start_scanning()
            .returning(move || {
                if should_fail {
                    return futures::future::ready(Err(BleError::ScanningAlreadyInProgress)).boxed();
                }
                
                let devices = devices.clone();
                let (tx, rx) = channel(100);
                
                // Simulate sending some events on the channel
                let tx_clone = tx.clone();
                tokio::spawn(async move {
                    // Send scanning started event
                    let _ = tx_clone.send(BleAdapterEvent::ScanStarted).await;
                    
                    // Small delay to simulate scanning process
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    
                    // Send device discovered events for each device
                    for device in devices {
                        let _ = tx_clone.send(BleAdapterEvent::DeviceDiscovered(device)).await;
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                });
                
                futures::future::ready(Ok(rx)).boxed()
            });
        
        // Setup stop_scanning behavior
        mock.expect_stop_scanning()
            .returning(|| futures::future::ready(Ok(())).boxed());
        
        // Setup discover_devices behavior
        let devices = self.devices.clone();
        mock.expect_discover_devices()
            .returning(move || futures::future::ready(Ok(devices.clone())).boxed());
        
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

/// Create a test discovered device
pub fn create_test_discovered_device(
    address: &str,
    name: Option<&str>,
    rssi: Option<i16>,
    manufacturer_data: Option<HashMap<u16, Vec<u8>>>,
) -> DiscoveredDevice {
    let addr = address.parse().unwrap_or_else(|_| {
        BDAddr::from([0x11, 0x22, 0x33, 0x44, 0x55, 0x66])
    });
    
    DiscoveredDevice {
        address: addr,
        name: name.map(String::from),
        rssi,
        manufacturer_data: manufacturer_data.unwrap_or_default(),
        services: vec![],
    }
}

/// Create an Apple device with the specified manufacturer data
pub fn create_apple_device(
    address: &str,
    name: Option<&str>,
    rssi: Option<i16>,
    data: Vec<u8>,
) -> DiscoveredDevice {
    let mut manufacturer_data = HashMap::new();
    manufacturer_data.insert(76, data); // 76 is Apple's manufacturer ID
    
    create_test_discovered_device(address, name, rssi, Some(manufacturer_data))
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

    impl Clone for Peripheral {
        fn clone(&self) -> Self;
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
    async fn test_mock_ble_scanner() {
        // Create a test device
        let test_device = create_test_discovered_device(
            "00:11:22:33:44:55",
            Some("Test Device"),
            Some(-60),
            None
        );
        
        // Create a mock scanner with the test device
        let mock_scanner = MockBleScannerBuilder::new()
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
} 