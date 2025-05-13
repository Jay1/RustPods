use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use std::time::Duration;
use log::{debug, error};

use btleplug::api::{
    BDAddr, Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;
use tokio::time::sleep;
use futures::StreamExt;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::bluetooth::scanner_config::ScanConfig;
use crate::bluetooth::events::{BleEvent, EventBroker, EventFilter};
use crate::airpods::{    DetectedAirPods, create_airpods_filter, detect_airpods};
use crate::config::{AppConfig, Configurable};

/// Configuration for Bluetooth scanner
#[derive(Debug, Clone)]
pub struct BleScannerConfig {
    /// Interval between scans
    pub scan_interval: std::time::Duration,
    /// Whether to filter out known devices
    pub filter_known_devices: bool,
    /// Whether to only update RSSI for known devices
    pub update_rssi_only: bool,
    /// Interval for updating device data
    pub update_interval: std::time::Duration,
    /// Timeout for scanning
    pub scan_timeout: Option<std::time::Duration>,
}

impl Default for BleScannerConfig {
    fn default() -> Self {
        Self {
            scan_interval: std::time::Duration::from_secs(30),
            filter_known_devices: false,
            update_rssi_only: false,
            update_interval: std::time::Duration::from_secs(5),
            scan_timeout: None,
        }
    }
}

/// Custom error type for Bluetooth operations
#[derive(Debug, thiserror::Error, Clone)]
pub enum BleError {
    #[error("Failed to find a suitable Bluetooth adapter")]
    AdapterNotFound,
    
    #[error("Bluetooth operation failed: {0}")]
    BtlePlugError(String),
    
    #[error("Scanning is already in progress")]
    ScanInProgress,
    
    #[error("Scan has not been started")]
    ScanNotStarted,
    
    #[error("Adapter is not initialized")]
    AdapterNotInitialized,
    
    #[error("Device not found")]
    DeviceNotFound,
    
    #[error("Invalid data received")]
    InvalidData,
    
    #[error("Operation timed out")]
    Timeout,
    
    #[error("{0}")]
    Other(String),
}

/// A discovered Bluetooth device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredDevice {
    /// Device address
    #[serde(with = "bdaddr_serde")]
    pub address: BDAddr,
    /// Device name if available
    pub name: Option<String>,
    /// RSSI (signal strength) value
    pub rssi: Option<i16>,
    /// Manufacturer data (used for AirPods detection)
    pub manufacturer_data: HashMap<u16, Vec<u8>>,
    /// Whether this might be an AirPods device
    pub is_potential_airpods: bool,
    /// When the device was last seen
    #[serde(skip, default = "std::time::Instant::now")]
    pub last_seen: std::time::Instant,
    /// Whether the device is connected
    pub is_connected: bool,
    /// Service data from the device
    pub service_data: HashMap<Uuid, Vec<u8>>,
    /// Services advertised by the device
    pub services: Vec<Uuid>,
}

// Custom serialization for BDAddr
mod bdaddr_serde {
    use btleplug::api::BDAddr;
    use serde::{Deserialize, Deserializer, Serializer, Serialize};

    pub fn serialize<S>(bdaddr: &BDAddr, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Convert BDAddr to string for serialization
        let addr_str = bdaddr.to_string();
        addr_str.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<BDAddr, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        
        // Deserialize from string
        let addr_str = String::deserialize(deserializer)?;
        
        // Parse back to BDAddr
        let bytes: Vec<&str> = addr_str.split(':').collect();
        
        if bytes.len() != 6 {
            return Err(D::Error::custom(format!("Invalid BDAddr format: {}", addr_str)));
        }
        
        let mut addr = [0u8; 6];
        for (i, byte) in bytes.iter().enumerate() {
            addr[i] = u8::from_str_radix(byte, 16)
                .map_err(|e| D::Error::custom(format!("Invalid hex byte '{}': {}", byte, e)))?;
        }
        
        Ok(BDAddr::from(addr))
    }
}

impl DiscoveredDevice {
    /// Create a new discovered device from a peripheral
    pub async fn from_peripheral(peripheral: &Peripheral) -> Result<Self, BleError> {
        let properties = peripheral.properties().await?;
        let address = peripheral.address();
        
        // Ensure the properties exist
        let properties = match properties {
            Some(props) => props,
            None => return Err(BleError::BtlePlugError("No device properties available".to_string())),
        };
        
        // The manufacturer data is expected to be in properties if available
        let manufacturer_data = properties.manufacturer_data.clone();
        
        // Use the AirPods filter to check if this is a potential AirPods device
        let is_potential_airpods = create_airpods_filter()(&DiscoveredDevice {
            address,
            name: properties.local_name.clone(),
            rssi: properties.rssi,
            manufacturer_data: manufacturer_data.clone(),
            is_potential_airpods: false, // Not used in filter
            last_seen: std::time::Instant::now(),
            is_connected: false,
            service_data: HashMap::new(),
            services: Vec::new(),
        });
        
        Ok(Self {
            address,
            name: properties.local_name,
            rssi: properties.rssi,
            manufacturer_data,
            is_potential_airpods,
            last_seen: std::time::Instant::now(),
            is_connected: false,
            service_data: HashMap::new(),
            services: Vec::new(),
        })
    }
}

impl Default for DiscoveredDevice {
    fn default() -> Self {
        Self {
            address: BDAddr::default(),
            name: None,
            rssi: None,
            manufacturer_data: HashMap::new(),
            is_potential_airpods: false,
            last_seen: Instant::now(),
            is_connected: false,
            service_data: HashMap::new(),
            services: Vec::new(),
        }
    }
}

/// BLE scanner
pub struct BleScanner {
    /// The config for scanning
    config: ScanConfig,
    /// The Bluetooth adapter
    adapter: Option<Arc<Adapter>>,
    /// Whether scanning is in progress
    is_scanning: bool,
    /// Map of discovered devices by address
    devices: Arc<tokio::sync::Mutex<HashMap<BDAddr, DiscoveredDevice>>>,
    /// Task handle for the scan
    scan_task: Option<JoinHandle<()>>,
    /// Channel for canceling the scan
    cancel_sender: Option<Sender<()>>,
    /// Channel for sending events
    event_sender: Option<Sender<BleEvent>>,
    /// Event broker for distribution
    event_broker: Option<EventBroker>,
    /// Current scan cycle count
    scan_cycles_completed: usize,
}

impl Clone for BleScanner {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            adapter: self.adapter.clone(),
            is_scanning: self.is_scanning,
            devices: self.devices.clone(),
            scan_task: None, // Don't clone the JoinHandle
            cancel_sender: None,
            event_sender: None,
            event_broker: self.event_broker.clone(),
            scan_cycles_completed: self.scan_cycles_completed,
        }
    }
}

impl Default for BleScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl BleScanner {
    /// Create a new BLE scanner with default configuration
    pub fn new() -> Self {
        Self {
            config: ScanConfig::default(),
            adapter: None,
            is_scanning: false,
            devices: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            scan_task: None,
            cancel_sender: None,
            event_sender: None,
            event_broker: None,
            scan_cycles_completed: 0,
        }
    }
    
    /// Create a new BLE scanner with the specified configuration
    pub fn with_config(config: ScanConfig) -> Self {
        Self {
            config,
            adapter: None,
            is_scanning: false,
            devices: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            scan_task: None,
            cancel_sender: None,
            event_sender: None,
            event_broker: None,
            scan_cycles_completed: 0,
        }
    }
    
    /// Create a new BLE scanner with a Bluetooth adapter and BleScannerConfig
    pub fn with_adapter_config(adapter: Arc<Adapter>, config: BleScannerConfig) -> Self {
        // Convert BleScannerConfig to ScanConfig
        let scan_config = ScanConfig::default()
            .with_scan_duration(config.scan_interval)
            .with_interval(config.scan_interval)
            .with_auto_stop(!config.filter_known_devices)
            .with_continuous(config.scan_timeout.is_none());
        
        Self {
            config: scan_config,
            adapter: Some(adapter),
            is_scanning: false,
            devices: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            scan_task: None,
            cancel_sender: None,
            event_sender: None,
            event_broker: None,
            scan_cycles_completed: 0,
        }
    }
    
    /// Update the scanner configuration
    pub fn set_config(&mut self, config: ScanConfig) {
        self.config = config;
    }
    
    /// Get the current scanner configuration
    pub fn get_config(&self) -> &ScanConfig {
        &self.config
    }
    
    /// Initialize the scanner
    pub async fn initialize(&mut self) -> Result<(), BleError> {
        // Create a manager
        let manager = Manager::new().await.map_err(|e| BleError::BtlePlugError(e.to_string()))?;
        
        // Get the adapter list
        let adapters = manager.adapters().await.map_err(|e| BleError::BtlePlugError(e.to_string()))?;
        
        // Find the first adapter that can be used
        if let Some(adapter) = adapters.into_iter().next() {
            self.adapter = Some(Arc::new(adapter));
            Ok(())
        } else {
            Err(BleError::AdapterNotFound)
        }
    }
    
    /// Start scanning for devices
    pub async fn start_scanning(&mut self) -> Result<Receiver<BleEvent>, BleError> {
        // Stop scanning if already in progress
        if self.is_scanning {
            return Err(BleError::ScanInProgress);
        }
        
        // Try to get the adapter
        let adapter = match &self.adapter {
            Some(adapter) => adapter.clone(),
            None => {
                // Attempt to initialize again if adapter is not available
                self.initialize().await?;
                
                // If still not available, return error
                match &self.adapter {
                    Some(adapter) => adapter.clone(),
                    None => return Err(BleError::AdapterNotFound),
                }
            }
        };
        
        // Create channel for events
        let (tx, rx) = channel(100);
        
        // Create channel for cancellation
        let (cancel_tx, cancel_rx) = channel(1);
        
        // Save the sender channels
        self.event_sender = Some(tx.clone());
        self.cancel_sender = Some(cancel_tx);
        
        // Start the scan task
        let task = self.start_scan_task(tx, cancel_rx, adapter.clone()).await?;
        
        // Save the task handle
        self.scan_task = Some(task);
        
        // Set scanning state
        self.is_scanning = true;
        
        // Reset the cycle count
        self.scan_cycles_completed = 0;
        
        Ok(rx)
    }
    
    /// Start the scan task with retry mechanism
    async fn start_scan_task(
        &self, 
        event_tx: Sender<BleEvent>, 
        mut cancel_rx: Receiver<()>,
        adapter: Arc<Adapter>
    ) -> Result<JoinHandle<()>, BleError> {
        let devices = self.devices.clone();
        let config = self.config.clone();
        
        // Max number of retry attempts for scan start failures
        const MAX_RETRIES: usize = 3;
        
        // Task to handle scanning        
        let task = tokio::spawn(async move {            
            let mut scan_cycles_performed = 0;            
            let mut last_scan_time = Instant::now();
            
            // Main scan loop
            'scan_loop: loop {
                // Wait for interval if needed
                if last_scan_time.elapsed() < config.interval_between_scans {
                    let wait_time = config.interval_between_scans - last_scan_time.elapsed();
                    
                    // Wait for either the interval or cancellation
                    tokio::select! {
                        _ = sleep(wait_time) => {
                            // Interval completed, proceed with next scan
                        }
                        _ = cancel_rx.recv() => {
                            // Cancellation received
                            break 'scan_loop;
                        }
                    }
                }
                
                // Update last scan time
                last_scan_time = Instant::now();
                
                // Check if we've hit the scan cycle limit
                if let Some(limit) = config.max_scan_cycles {
                    if scan_cycles_performed >= limit {
                        // Send completion event
                        let _ = event_tx.send(BleEvent::ScanningCompleted).await;
                        break 'scan_loop;
                    }
                }
                
                // Start the scan with retry logic
                let scan_result = async {
                    let mut current_attempt = 0;
                    
                    loop {
                        // Attempt to start scanning
                        let result = adapter.start_scan(ScanFilter::default()).await;
                        
                        match result {
                            Ok(_) => return Ok(()),
                            Err(e) => {
                                current_attempt += 1;
                                
                                // If we've reached max retries, return the error
                                if current_attempt >= MAX_RETRIES {
                                    return Err(e);
                                }
                                
                                // Log the retry attempt
                                log::warn!("Scan start failed (attempt {}/{}): {}", 
                                           current_attempt, MAX_RETRIES, e);
                                
                                // Wait briefly before retrying
                                sleep(Duration::from_millis(500)).await;
                            }
                        }
                    }
                }.await;
                
                if let Err(e) = scan_result {
                    // Send error event
                    let error_msg = format!("Failed to start scan after {} retries: {}", 
                                           MAX_RETRIES, e);
                    let _ = event_tx.send(BleEvent::Error(error_msg)).await;
                    break 'scan_loop;
                }
                
                // Set up event stream from adapter
                let mut events = adapter.events().await.unwrap();
                
                // Timeout for this scan cycle
                let scan_timeout = tokio::time::sleep(config.scan_duration);
                let mut devices_found = 0;
                
                // Inner loop for this scan cycle
                tokio::pin!(scan_timeout);
                'cycle_loop: loop {
                    tokio::select! {
                        // Handle Bluetooth events
                        Some(event) = events.next() => {
                            match event {
                                CentralEvent::DeviceDiscovered(addr) => {
                                    // Get peripheral for the discovered device
                                    let peripheral = adapter.peripheral(&addr).await;
                                    
                                    if let Ok(peripheral) = peripheral {
                                        // Retry up to 3 times if getting device properties fails
                                        let mut device_info = None;
                                        let mut prop_retry = 0;
                                        
                                        while prop_retry < 3 && device_info.is_none() {
                                            match DiscoveredDevice::from_peripheral(&peripheral).await {
                                                Ok(device) => {
                                                    device_info = Some(device);
                                                    break;
                                                },
                                                Err(e) => {
                                                    prop_retry += 1;
                                                    if prop_retry >= 3 {
                                                        // Give up after max retries
                                                        let _ = event_tx.send(BleEvent::Error(
                                                            format!("Failed to get device properties: {}", e)
                                                        )).await;
                                                    }
                                                    // Small delay before retry
                                                    sleep(Duration::from_millis(100)).await;
                                                }
                                            }
                                        }
                                        
                                        if let Some(device) = device_info {
                                            // Apply RSSI filtering
                                            if let Some(min_rssi) = config.min_rssi {
                                                if let Some(rssi) = device.rssi {
                                                    if rssi < min_rssi {
                                                        // Skip this device due to weak signal
                                                        continue;
                                                    }
                                                }
                                            }
                                            
                                            // Process the device further
                                            let mut devices_map = devices.lock().await;
                                            devices_map.insert(device.address, device.clone());
                                            devices_found += 1;
                                            
                                            // Send device discovered event
                                            let _ = event_tx.send(BleEvent::DeviceDiscovered(device.clone())).await;
                                            
                                            // Check if this is an AirPods device and attempt detection
                                            if device.is_potential_airpods {
                                                if let Some(airpods) = detect_airpods(&device) {
                                                    // Send AirPods detected event with reliable retry mechanism
                                                    for _ in 0..3 {
                                                        if event_tx.send(BleEvent::AirPodsDetected(airpods.clone())).await.is_ok() {
                                                            break;
                                                        }
                                                        // Brief delay before retry
                                                        sleep(Duration::from_millis(50)).await;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                },
                                CentralEvent::DeviceUpdated(addr) => {
                                    // Handle device updates (similar to discovered)
                                    let peripheral_result = adapter.peripheral(&addr).await;
                                    
                                    if let Ok(peripheral) = peripheral_result {
                                        match DiscoveredDevice::from_peripheral(&peripheral).await {
                                            Ok(device) => {
                                                // Apply RSSI filtering
                                                if let Some(min_rssi) = config.min_rssi {
                                                    if let Some(rssi) = device.rssi {
                                                        if rssi < min_rssi {
                                                            debug!("Filtered device by RSSI: {} ({})", 
                                                                device.name.clone().unwrap_or_else(|| device.address.to_string()), 
                                                                rssi);
                                                            continue;
                                                        }
                                                    }
                                                }
                                                
                                                // Check if the device already exists
                                                let mut devices_map = devices.lock().await;
                                                
                                                let is_airpods = device.is_potential_airpods;
                                                
                                // Create or update device with airpods flag
                                let mut device = device.clone();
                                device.is_potential_airpods = is_airpods;
                                
                                // Update if exists or add as new
                                devices_map.insert(device.address, device.clone());
                                
                                // Send event
                                let _ = event_tx.send(BleEvent::DeviceDiscovered(device)).await;
                                            },
                                            Err(e) => {
                                                error!("Failed to get device details: {}", e);
                                                continue;
                                            }
                                        }
                                    } else {
                                        error!("Failed to get peripheral: {:?}", peripheral_result);
                                    }
                                },
                                CentralEvent::DeviceDisconnected(addr) => {
                                    debug!("Device lost: {}", addr);
                                    let mut devices_map = devices.lock().await;
                                    
                                    // First check if we know about this device using peripheral address
                                    // (we need to convert from PeripheralId to BDAddr)
                                    let peripheral_result = adapter.peripheral(&addr).await;
                                    
                                    if let Ok(peripheral) = peripheral_result {
                                        // Try to get the address of the peripheral
                                        if let Ok(properties) = peripheral.properties().await {
                                            if let Some(props) = properties {
                                                // The correct way to handle properties.address since it's not an Option
                                                let address = props.address;
                                                // Now we have a BDAddr
                                                devices_map.remove(&address);
                                                // Send event with proper BDAddr
                                                let _ = event_tx.send(BleEvent::DeviceLost(address)).await;
                                            } else {
                                                error!("Device lost but no properties available");
                                            }
                                        } else {
                                            error!("Failed to get device properties for lost device");
                                        }
                                    } else {
                                        // This is unfortunate, but we can't do much without the address
                                        error!("Failed to get peripheral for lost device: {:?}", peripheral_result);
                                    }
                                },
                                // Other events can be handled here as needed
                                _ => {}
                            }
                        },
                        // Handle scan duration timeout
                        _ = &mut scan_timeout => {
                            break 'cycle_loop;
                        },
                        // Handle cancellation
                        _ = cancel_rx.recv() => {
                            break 'scan_loop;
                        }
                    }
                }
                
                // Stop scanning for this cycle
                if let Err(e) = adapter.stop_scan().await {
                    // Log error but continue
                    let _ = event_tx.send(BleEvent::Error(
                        format!("Failed to stop scan: {}", e)
                    )).await;
                }
                
                // Increment scan cycle counter
                scan_cycles_performed += 1;
                
                // Send scan cycle completed event
                let _ = event_tx.send(BleEvent::ScanCycleCompleted { 
                    devices_found
                }).await;
                
                // Check if auto stop scan is enabled
                if config.auto_stop_scan {
                    // Send completion event
                    let _ = event_tx.send(BleEvent::ScanningCompleted).await;
                    break 'scan_loop;
                }
            }
            
            // Ensure scanning is stopped
            let _ = adapter.stop_scan().await;
            
            // Send final completion event if not already sent
            let _ = event_tx.send(BleEvent::ScanningCompleted).await;
        });
        
        Ok(task)
    }
    
    /// Stop the current scan if one is running
    pub async fn stop_scanning(&mut self) -> Result<(), BleError> {
        if !self.is_scanning {
            return Ok(());
        }
        
        // Send cancellation signal if available
        if let Some(tx) = &self.cancel_sender {
            let _ = tx.send(()).await;
        }
        
        // Stop the scan directly as well (belt and suspenders)
        if let Some(adapter) = &self.adapter {
            let _ = adapter.stop_scan().await;
        }
        
        // Abort the background task if it's still running
        if let Some(task) = self.scan_task.take() {
            task.abort();
        }
        
        self.is_scanning = false;
        Ok(())
    }
    
    /// Get a list of currently known devices
    pub async fn get_devices(&self) -> Vec<DiscoveredDevice> {
        let devices = self.devices.lock().await;
        devices.values().cloned().collect()
    }
    
    /// Get potential AirPods devices from discovered devices
    pub async fn get_potential_airpods(&self) -> Vec<DiscoveredDevice> {
        let devices = self.devices.lock().await;
        devices
            .values()
            .filter(|d| d.is_potential_airpods)
            .cloned()
            .collect()
    }
    
    /// Get fully detected AirPods devices with battery information
    pub async fn get_detected_airpods(&self) -> Vec<DetectedAirPods> {
        let devices = self.devices.lock().await;
        devices
            .values()
            .filter_map(detect_airpods)
            .collect()
    }
    
    /// Check if scanning is currently in progress
    pub fn is_scanning(&self) -> bool {
        self.is_scanning
    }
    
    /// Get the number of completed scan cycles
    pub fn get_scan_cycles(&self) -> usize {
        self.scan_cycles_completed
    }
    
    /// Clear the device list
    pub async fn clear_devices(&mut self) {
        let mut devices = self.devices.lock().await;
        devices.clear();
    }
    
    /// Get a device by address if it exists
    pub async fn get_device(&self, address: &BDAddr) -> Option<DiscoveredDevice> {
        let devices = self.devices.lock().await;
        devices.get(address).cloned()
    }
    
    /// Process a discovered device, applying filters and sending events    
    #[allow(dead_code)]    
    async fn process_discovered_device(
        &self,
        device: DiscoveredDevice,
        event_tx: &Sender<BleEvent>,
    ) -> Result<(), BleError> {
        // Apply RSSI filtering if configured
        if let Some(min_rssi) = self.config.min_rssi {
            if let Some(rssi) = device.rssi {
                if rssi < min_rssi {
                    // Skip this device as it's too far away
                    return Ok(());
                }
            }
        }
        
        // Store the device
        {
            let mut devices = self.devices.lock().await;
            devices.insert(device.address, device.clone());
        }
        
        // Notify listeners
        let _ = event_tx.send(BleEvent::DeviceDiscovered(device.clone())).await;
        
        // Check for AirPods and notify if found
        if device.is_potential_airpods {
            if let Some(airpods) = detect_airpods(&device) {
                let _ = event_tx.send(BleEvent::AirPodsDetected(airpods)).await;
            }
        }
        
        Ok(())
    }
    
    /// Get filtered AirPods devices matching a filter
    pub async fn get_filtered_airpods(&self, filter: &crate::airpods::detector::AirPodsFilter) -> Vec<DiscoveredDevice> {
        let devices = self.get_devices().await;
        devices.into_iter()
            .filter(|d| filter(d))
            .collect()
    }
    
    /// Get detected AirPods devices matching a filter
    pub async fn get_filtered_detected_airpods(&self, filter: &crate::airpods::detector::AirPodsFilter) -> Vec<DetectedAirPods> {
        let devices = self.get_devices().await;
        devices.into_iter()
            .filter(|d| filter(d))
            .filter_map(|d| detect_airpods(&d))
            .collect()
    }
    
    /// Check if there are any AirPods matching the filter
    pub async fn has_airpods_matching(&self, filter: &crate::airpods::detector::AirPodsFilter) -> bool {
        let devices = self.get_devices().await;
        devices.iter().any(filter)
    }
    
    /// Filter devices using an AirPods filter
    pub async fn filter_devices(&self, filter: &crate::airpods::detector::AirPodsFilter) -> Vec<DiscoveredDevice> {
        let devices = self.get_devices().await;
        devices.into_iter()
            .filter(|d| filter(d))
            .collect()
    }
    
    /// Get or create the event broker
    fn event_broker(&mut self) -> &mut EventBroker {
        if self.event_broker.is_none() {
            self.event_broker = Some(EventBroker::new());
        }
        self.event_broker.as_mut().unwrap()
    }
    
    /// Subscribe to all events
    pub fn subscribe_all(&mut self) -> Receiver<BleEvent> {
        self.subscribe(EventFilter::All)
    }
    
    /// Subscribe to scanner events with custom filter
    pub fn subscribe(&mut self, filter: EventFilter) -> Receiver<BleEvent> {
        let (_, rx) = self.event_broker().subscribe(filter);
        rx
    }
    
    /// Subscribe to AirPods events only
    pub fn subscribe_airpods(&mut self) -> Receiver<BleEvent> {
        self.subscribe(EventFilter::airpods_only())
    }
    
    /// Get peripherals by Bluetooth address
    pub async fn get_peripherals_by_address(&self, address: &BDAddr) -> Result<Vec<Peripheral>, BleError> {
        // Make sure we have an adapter
        let adapter = if let Some(adapter) = &self.adapter {
            adapter.clone()
        } else {
            return Err(BleError::AdapterNotFound);
        };
        
        // Get all peripherals
        let peripherals = adapter.peripherals().await.map_err(|e| BleError::BtlePlugError(e.to_string()))?;
        
        // Filter by address
        let matching_devices = peripherals.into_iter()
            .filter(|peripheral| peripheral.address() == *address)
            .collect::<Vec<_>>();
        
        Ok(matching_devices)
    }
    
    /// Start scanning with a specific configuration
    pub async fn start_scanning_with_config(
        &mut self,
        config: ScanConfig,
    ) -> Result<Receiver<BleEvent>, BleError> {
        // Similar to start_scanning but with custom config
        if self.scan_task.is_some() {
            return Err(BleError::ScanInProgress);
        }
        
        // Make sure we have an adapter
        let adapter = if let Some(adapter) = &self.adapter {
            adapter.clone()
        } else {
            // Try to initialize
            self.initialize().await?;
            
            // Get the adapter
            self.adapter.as_ref().ok_or(BleError::AdapterNotFound)?.clone()
        };
        
        // Create channels for communication
        let (tx, rx) = channel(100);
        self.event_sender = Some(tx.clone());
        
        // Create a channel for cancellation
        let (cancel_tx, cancel_rx) = channel::<()>(1);
        self.cancel_sender = Some(cancel_tx);
        
        // Use the provided config
        self.config = config.clone();
        
        // Start scanning task
        let scan_task = self.start_scan_task(tx, cancel_rx, adapter).await?;
        self.scan_task = Some(scan_task);
        self.is_scanning = true;
        
        Ok(rx)
    }
    
    /// Get a configurable reference to the scanner if available
    pub fn as_configurable(&mut self) -> Option<&mut dyn Configurable> {
        Some(self)
    }
}

// Implement the Configurable trait for BleScanner
impl Configurable for BleScanner {
    fn apply_config(&mut self, config: &AppConfig) {
        // Apply bluetooth-related settings to our ScanConfig
        let mut scan_config = ScanConfig::default();
        scan_config = scan_config.with_scan_duration(config.bluetooth.scan_duration);
        scan_config = scan_config.with_interval(config.bluetooth.scan_interval);
        scan_config = scan_config.with_min_rssi(config.bluetooth.min_rssi);
        
        // Set a reasonable default for max cycles
        scan_config = scan_config.with_max_cycles(Some(5));
        
        self.set_config(scan_config);
    }
}

impl Drop for BleScanner {
    fn drop(&mut self) {
        // If we have a running task, abort it to avoid leaking resources
        if let Some(task) = self.scan_task.take() {
            task.abort();
        }
    }
}

/// Parse a BDAddr from a string
#[allow(dead_code)]
pub fn parse_bdaddr(s: &str) -> Result<BDAddr, String> {
    let bytes: Vec<&str> = s.split(':').collect();
    
    if bytes.len() != 6 {
        return Err(format!("Invalid BDAddr format: {}", s));
    }
    
    let mut addr = [0u8; 6];
    for (i, byte) in bytes.iter().enumerate() {
        addr[i] = u8::from_str_radix(byte, 16)
            .map_err(|e| format!("Invalid hex byte '{}': {}", byte, e))?;
    }
    
    Ok(BDAddr::from(addr))
}

impl BleError {
    /// Get the error category for this error
    pub fn error_category(&self) -> BleErrorCategory {
        match self {
            Self::AdapterNotFound => BleErrorCategory::AdapterIssue,
            Self::BtlePlugError(e) => {
                // Check the error string to determine category
                if e.contains("permission") || e.contains("Permission") {
                    BleErrorCategory::PermissionIssue
                } else if e.contains("not connected") || e.contains("Not connected") {
                    BleErrorCategory::ConnectionIssue
                } else if e.contains("not supported") || e.contains("Not supported") {
                    BleErrorCategory::CapabilityIssue
                } else if e.contains("not found") || e.contains("Not found") {
                    BleErrorCategory::DeviceIssue
                } else {
                    BleErrorCategory::Unknown
                }
            },
            Self::ScanInProgress => BleErrorCategory::OperationIssue,
            Self::ScanNotStarted => BleErrorCategory::OperationIssue,
            Self::AdapterNotInitialized => BleErrorCategory::AdapterIssue,
            Self::DeviceNotFound => BleErrorCategory::DeviceIssue,
            Self::InvalidData => BleErrorCategory::DataIssue,
            Self::Timeout => BleErrorCategory::TimeoutIssue,
            Self::Other(_) => BleErrorCategory::Unknown,
        }
    }
    
    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::BtlePlugError(e) => {
                // Check if the error is recoverable based on the error string
                e.contains("not connected") || e.contains("Not connected") || 
                e.contains("not found") || e.contains("Not found")
            },
            Self::Timeout => true,
            Self::ScanInProgress => true,
            Self::ScanNotStarted => true,
            Self::AdapterNotInitialized => true,
            Self::DeviceNotFound => true,
            _ => false,
        }
    }
    
    /// Get a user-friendly message for this error
    pub fn user_message(&self) -> String {
        match self {
            Self::AdapterNotFound => 
                "No Bluetooth adapter was found on your system. Please check your Bluetooth hardware.".into(),
            
            Self::BtlePlugError(e) => {
                if e.contains("permission") {
                    "Permission denied when accessing Bluetooth. Try running with administrator/root privileges.".into()
                } else if e.contains("not connected") {
                    "Device is not connected. Please make sure your AirPods are nearby and in pairing mode.".into()
                } else if e.contains("not supported") {
                    "This Bluetooth operation is not supported by your adapter.".into()
                } else {
                    format!("Bluetooth error: {}", e)
                }
            },
            
            Self::ScanInProgress => 
                "A Bluetooth scan is already in progress.".into(),
            
            Self::ScanNotStarted => 
                "Bluetooth scanning has not been started.".into(),
            
            Self::AdapterNotInitialized => 
                "Bluetooth adapter has not been initialized.".into(),
            
            Self::DeviceNotFound => 
                "The requested Bluetooth device was not found. Make sure your AirPods are nearby and in pairing mode.".into(),
            
            Self::InvalidData => 
                "Invalid data was received from the Bluetooth device.".into(),
            
            Self::Timeout => 
                "The Bluetooth operation timed out. Please try again.".into(),
            
            Self::Other(msg) => 
                format!("Bluetooth error: {}", msg),
        }
    }
}

/// High-level categorization of Bluetooth errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BleErrorCategory {
    /// Issues with the Bluetooth adapter
    AdapterIssue,
    /// Issues with scanning
    ScanningIssue,
    /// Issues with a specific device
    DeviceIssue,
    /// Issues with connections
    ConnectionIssue,
    /// Timeout issues
    TimeoutIssue,
    /// Permission issues
    PermissionIssue,
    /// System resource issues
    SystemIssue,
    /// Capability issues (feature not supported)
    CapabilityIssue,
    /// Unknown issues
    Unknown,
    /// Operation issues
    OperationIssue,
    /// Data issues
    DataIssue,
}

impl From<btleplug::Error> for BleError {
    fn from(err: btleplug::Error) -> Self {
        BleError::BtlePlugError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bluetooth::scanner_config::ScanConfig;
    
    #[test]
    fn test_parse_bdaddr_valid() {
        let addr_str = "12:34:56:78:9A:BC";
        let result = parse_bdaddr(addr_str);
        assert!(result.is_ok());
        
        let addr = result.unwrap();
        assert_eq!(addr, BDAddr::from([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]));
    }
    
    #[test]
    fn test_parse_bdaddr_invalid_format() {
        // Too few segments
        let result = parse_bdaddr("12:34:56:78:9A");
        assert!(result.is_err());
        
        // Invalid hex
        let result = parse_bdaddr("12:34:56:78:9A:ZZ");
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_scanner_new() {
        let scanner = BleScanner::new();
        assert!(!scanner.is_scanning());
        assert_eq!(scanner.get_scan_cycles(), 0);
        assert!(scanner.get_devices().await.is_empty());
    }
    
    #[test]
    fn test_scanner_with_config() {
        let config = ScanConfig::default();
        let scanner = BleScanner::with_config(config);
        
        assert_eq!(scanner.get_scan_cycles(), 0);
        assert!(!scanner.is_scanning());
        // Note: Not testing get_devices().is_empty() here as it would require async
    }
    
    #[tokio::test]
    async fn test_device_list_operations() {
        let mut scanner = BleScanner::new();
        
        // Initially empty
        assert!(scanner.get_devices().await.is_empty());
        
        // Clear should work even when empty
        scanner.clear_devices().await;
        assert!(scanner.get_devices().await.is_empty());
    }
} 