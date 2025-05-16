use std::collections::{HashMap};
use std::sync::Arc;
use std::time::{Duration, Instant};
use log::{debug, error, warn, info};
use std::fmt;

use btleplug::api::{
    BDAddr, Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;
use tokio::time::{sleep, interval};
use futures::StreamExt;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::bluetooth::scanner_config::ScanConfig;
use crate::bluetooth::events::{BleEvent, EventBroker, EventFilter};
use crate::airpods::{DetectedAirPods, create_airpods_filter, detect_airpods};
use crate::config::{AppConfig, Configurable};

// Import new error types
use crate::error::{BluetoothError, ErrorContext, RecoveryAction};

// Remove unused imports
// use btleplug::Error as BtlePlugError;
// use std::sync::PoisonError;

/// Configuration for Bluetooth scanner
#[derive(Debug, Clone)]
pub struct BleScannerConfig {
    /// Duration of each scan
    pub scan_duration: std::time::Duration,
    /// Interval between scans
    pub interval_between_scans: std::time::Duration,
    /// Whether to filter out known devices
    pub filter_known_devices: bool,
    /// Whether to only update RSSI for known devices
    pub update_rssi_only: bool,
    /// Interval for updating device data
    pub update_interval: std::time::Duration,
    /// Timeout for scanning
    pub scan_timeout: Option<std::time::Duration>,
    /// Maximum number of retries for operations
    pub max_retries: u8,
    /// Delay between retries
    pub retry_delay: std::time::Duration,
}

impl Default for BleScannerConfig {
    fn default() -> Self {
        Self {
            scan_duration: std::time::Duration::from_secs(10),
            interval_between_scans: std::time::Duration::from_secs(30),
            filter_known_devices: false,
            update_rssi_only: false,
            update_interval: std::time::Duration::from_secs(5),
            scan_timeout: None,
            max_retries: 3,
            retry_delay: std::time::Duration::from_millis(500),
        }
    }
}

/// Legacy error type for backward compatibility
#[deprecated(
    since = "0.1.0",
    note = "Use BluetoothError from crate::error instead"
)]
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

// Implement conversion from BleError to BluetoothError for backward compatibility
impl From<BleError> for BluetoothError {
    fn from(error: BleError) -> Self {
        match error {
            BleError::AdapterNotFound => BluetoothError::NoAdapter,
            BleError::BtlePlugError(msg) => BluetoothError::Other(format!("Bluetooth API error: {}", msg)),
            BleError::ScanInProgress => BluetoothError::ScanFailed("Scan already in progress".to_string()),
            BleError::ScanNotStarted => BluetoothError::ScanFailed("Scan not started".to_string()),
            BleError::AdapterNotInitialized => BluetoothError::Other("Adapter not initialized".to_string()),
            BleError::DeviceNotFound => BluetoothError::DeviceNotFound("Device not found".to_string()),
            BleError::InvalidData => BluetoothError::InvalidData("Invalid data received".to_string()),
            BleError::Timeout => BluetoothError::Timeout(Duration::from_secs(30)), // Default timeout
            BleError::Other(msg) => BluetoothError::Other(msg),
        }
    }
}

// Implement conversion from BluetoothError to BleError for backward compatibility
impl From<BluetoothError> for BleError {
    fn from(error: BluetoothError) -> Self {
        match error {
            BluetoothError::ConnectionFailed(msg) => BleError::BtlePlugError(format!("Connection failed: {}", msg)),
            BluetoothError::DeviceNotFound(_msg) => BleError::DeviceNotFound,
            BluetoothError::ScanFailed(msg) => BleError::BtlePlugError(format!("Scan failed: {}", msg)),
            BluetoothError::DeviceDisconnected(msg) => BleError::BtlePlugError(format!("Device disconnected: {}", msg)),
            BluetoothError::NoAdapter => BleError::AdapterNotFound,
            BluetoothError::PermissionDenied(msg) => BleError::BtlePlugError(format!("Permission denied: {}", msg)),
            BluetoothError::InvalidData(_msg) => BleError::InvalidData,
            BluetoothError::Timeout(_) => BleError::Timeout,
            BluetoothError::ApiError(error) => BleError::BtlePlugError(error.to_string()),
            BluetoothError::AdapterRefreshFailed { error, .. } => BleError::BtlePlugError(format!("Adapter refresh failed: {}", error)),
            BluetoothError::AdapterNotAvailable { reason, .. } => BleError::BtlePlugError(format!("Adapter not available: {}", reason)),
            BluetoothError::AdapterScanFailed { error, .. } => BleError::BtlePlugError(format!("Adapter scan failed: {}", error)),
            BluetoothError::Other(msg) => BleError::Other(msg),
        }
    }
}

/// A discovered Bluetooth device
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    /// Transmit power level if available
    pub tx_power_level: Option<i16>,
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
    pub async fn from_peripheral(peripheral: &Peripheral) -> Result<Self, BluetoothError> {
        // Create error context
        let _ctx = ErrorContext::new("BleScanner", "from_peripheral")
            .with_metadata("peripheral_address", peripheral.address().to_string());
            
        // Use ? operator for error propagation with automatic conversion via From trait
        let properties = peripheral.properties().await
            .map_err(|e| {
                let bt_err: BluetoothError = e.into();
                bt_err
            })?;
            
        let address = peripheral.address();
            
        // Ensure the properties exist
        let properties = match properties {
            Some(props) => props,
            None => return Err(BluetoothError::InvalidData("No device properties available".to_string())),
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
            tx_power_level: properties.tx_power_level,
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
            tx_power_level: properties.tx_power_level,
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
            tx_power_level: None,
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
            .with_scan_duration(config.scan_duration)
            .with_interval(config.interval_between_scans)
            .with_auto_stop(!config.filter_known_devices)
            .with_continuous(false);
        
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
    
    /// Start scanning for devices
    pub async fn start_scanning(&mut self) -> Result<Receiver<BleEvent>, BluetoothError> {
        let _ctx = ErrorContext::new("BleScanner", "start_scanning");
        
        // Check if scanning is already in progress
        if self.is_scanning {
            warn!("{}Scan already in progress, returning existing channel", _ctx);
            
            // If there's an existing event broker, subscribe to it
            if let Some(event_broker) = &mut self.event_broker {
                let (_, rx) = event_broker.subscribe(EventFilter::All);
                return Ok(rx);
            } else {
                return Err(BluetoothError::Other("Scanner is marked as scanning but has no event broker".to_string()));
            }
        }
        
        // Get or initialize the adapter
        let adapter = self.get_or_init_adapter().await?;
        
        // Create channels
        let (event_tx, _event_rx) = channel(100);
        let (cancel_tx, cancel_rx) = channel(1);
        
        // Store the transmitters
        self.event_sender = Some(event_tx.clone());
        self.cancel_sender = Some(cancel_tx);
        
        // Start the scan task
        match self.start_scan_task(event_tx, cancel_rx, adapter).await {
            Ok(task) => {
                // Store the task handle
                self.scan_task = Some(task);
                
                // Mark as scanning
                self.is_scanning = true;
                
                // Initialize event broker if it doesn't exist
                if self.event_broker.is_none() {
                    self.event_broker = Some(EventBroker::new());
                }
                
                // Start the event broker
                let event_broker = self.event_broker.as_mut().unwrap();
                event_broker.start();
                
                // Subscribe to events
                let (_, rx) = event_broker.subscribe(EventFilter::All);
                Ok(rx)
            },
            Err(e) => {
                // Clean up
                self.event_sender = None;
                self.cancel_sender = None;
                
                // Return the error
                Err(e)
            }
        }
    }
    
    /// Get or initialize the Bluetooth adapter with retry logic
    async fn get_or_init_adapter(&mut self) -> Result<Arc<Adapter>, BluetoothError> {
        let _ctx = ErrorContext::new("BleScanner", "get_or_init_adapter");
        
        // If we already have an adapter, return it
        if let Some(adapter) = &self.adapter {
            return Ok(adapter.clone());
        }
        
        // Try to initialize the adapter with retries
        let mut attempts = 0;
        let max_attempts = self.config.max_retries;
        
        while attempts < max_attempts {
            attempts += 1;
            debug!("{}Initializing adapter (attempt {}/{})", _ctx, attempts, max_attempts);
            
            match self.try_initialize().await {
                Ok(()) => {
                    if let Some(adapter) = &self.adapter {
                        info!("{}Successfully initialized adapter", _ctx);
                        return Ok(adapter.clone());
                    }
                },
                Err(e) => {
                    if self.is_error_retryable(&e) && attempts < max_attempts {
                        warn!("{}Adapter initialization failed with retryable error: {}. Retrying ({}/{})", 
                            _ctx, e, attempts, max_attempts);
                        sleep(self.config.retry_delay).await;
                        continue;
                    } else {
                        error!("{}Failed to initialize adapter after {} attempts: {}", _ctx, attempts, e);
                        return Err(BluetoothError::AdapterNotAvailable {
                            reason: format!("Failed to initialize adapter: {}", e),
                            recovery: RecoveryAction::SelectDifferentAdapter
                        });
                    }
                }
            }
        }
        
        // This should not be reached due to the returns in the loop
        Err(BluetoothError::NoAdapter)
    }
    
    /// Start the scan task
    async fn start_scan_task(
        &self, 
        event_tx: Sender<BleEvent>,
        mut cancel_rx: Receiver<()>,
        adapter: Arc<Adapter>
    ) -> Result<JoinHandle<()>, BluetoothError> {
        let _ctx = ErrorContext::new("BleScanner", "start_scan_task");
        
        debug!("{}Starting scan task with interval {:?}", _ctx, self.config.scan_duration);
        
        // Get a clone of the devices map
        let devices = self.devices.clone();
        
        // Create a filter for scanning
        let filter = ScanFilter::default();
        
        // Create a clone of the config
        let config = self.config.clone();
        
        // Use a shared event stream
        let event_stream = match adapter.events().await {
            Ok(stream) => stream,
            Err(e) => {
                error!("{}Failed to get event stream: {}", _ctx, e);
                return Err(BluetoothError::ApiError(format!("Failed to get event stream: {}", e)));
            }
        };
        
        // Spawn the task that will scan for devices
        let task = tokio::spawn(async move {
            let mut interval = interval(config.scan_duration);
            let mut scan_timeout = config.scan_timeout;
            let mut scan_count = 0;
            
            // Create a stream of central events
            let mut event_stream = event_stream;
            
            // Create a context for error logging inside the task
            let inner_ctx = ErrorContext::new("BleScanner", "scan_task");
            
            // Start processing events
            loop {
                // Cancel if requested
                if cancel_rx.try_recv().is_ok() {
                    debug!("{}Scan task cancelled", inner_ctx);
                    break;
                }
                
                // Wait for the next interval
                interval.tick().await;
                
                // Start the scan if it's not a pure event receiver
                scan_count += 1;
                debug!("{}Starting scan cycle {} with timeout {:?}", inner_ctx, scan_count, scan_timeout);
                
                // Start scanning with timeout if set
                let scan_result = match scan_timeout {
                    Some(timeout_duration) => {
                        // Use tokio::time::timeout for the scan
                        match tokio::time::timeout(
                            timeout_duration,
                            adapter.start_scan(filter.clone())
                        ).await {
                            Ok(result) => result,
                            Err(_) => {
                                warn!("{}Scan timed out after {:?}", inner_ctx, timeout_duration);
                                // Continue with the next scan cycle after timeout
                                continue;
                            }
                        }
                    },
                    None => {
                        // No timeout
                        adapter.start_scan(filter.clone()).await
                    }
                };
                
                // Check if the scan started successfully
                if let Err(_e) = scan_result {
                    error!("{}Failed to start scan: {}", inner_ctx, _e);
                    // Try to send an error event
                    let _ = event_tx.send(BleEvent::Error(format!("Failed to start scan: {}", _e))).await;
                    // Continue with the next scan cycle
                    continue;
                }
                
                debug!("{}Scan started successfully, processing events...", inner_ctx);
                
                // Process events
                while let Ok(Some(event)) = tokio::time::timeout(
                    Duration::from_millis(100), // Short timeout to check for cancel
                    event_stream.next()
                ).await {
                    // Check for cancel again
                    if cancel_rx.try_recv().is_ok() {
                        debug!("{}Scan task cancelled during event processing", inner_ctx);
                        break;
                    }
                    
                    match event {
                        CentralEvent::DeviceDiscovered(address) => {
                            debug!("{}Device discovered: {}", inner_ctx, address);
                            // Get the peripheral
                            if let Ok(peripheral) = adapter.peripheral(&address).await {
                                // Convert to our device type
                                match DiscoveredDevice::from_peripheral(&peripheral).await {
                                    Ok(device) => {
                                        // Process the device (updates internal state and sends events)
                                        if let Err(e) = Self::process_discovered_device(&device, &devices, &event_tx, &config).await {
                                            warn!("{}Error processing discovered device: {}", inner_ctx, e);
                                        }
                                    },
                                    Err(e) => {
                                        warn!("{}Error getting peripheral properties: {}", inner_ctx, e);
                                    }
                                }
                            }
                        },
                        CentralEvent::DeviceUpdated(address) => {
                            debug!("{}Device updated: {}", inner_ctx, address);
                            // Get the peripheral
                            if let Ok(peripheral) = adapter.peripheral(&address).await {
                                // Convert to our device type
                                match DiscoveredDevice::from_peripheral(&peripheral).await {
                                    Ok(device) => {
                                        // Process the device as an update
                                        if let Err(e) = Self::process_discovered_device(&device, &devices, &event_tx, &config).await {
                                            warn!("{}Error processing device update: {}", inner_ctx, e);
                                        }
                                    },
                                    Err(e) => {
                                        warn!("{}Error getting peripheral properties: {}", inner_ctx, e);
                                    }
                                }
                            }
                        },
                        CentralEvent::DeviceConnected(address) => {
                            debug!("{}Device connected: {}", inner_ctx, address);
                            // Convert PeripheralId to BDAddr
                            if let Ok(peripheral) = adapter.peripheral(&address).await {
                                let address_bdaddr = peripheral.address();
                                // Update connection status
                                let mut devices_lock = devices.lock().await;
                                if let Some(device) = devices_lock.get_mut(&address_bdaddr) {
                                    device.is_connected = true;
                                    // Send event
                                    let _ = event_tx.send(BleEvent::DeviceUpdated(device.clone())).await;
                                }
                            }
                        },
                        CentralEvent::DeviceDisconnected(address) => {
                            debug!("{}Device disconnected: {}", inner_ctx, address);
                            // Convert PeripheralId to BDAddr
                            if let Ok(peripheral) = adapter.peripheral(&address).await {
                                let address_bdaddr = peripheral.address();
                                // Update connection status
                                let mut devices_lock = devices.lock().await;
                                if let Some(device) = devices_lock.get_mut(&address_bdaddr) {
                                    device.is_connected = false;
                                    // Send event
                                    let _ = event_tx.send(BleEvent::DeviceUpdated(device.clone())).await;
                                }
                            }
                        },
                        _ => {
                            // Other events are not handled
                        }
                    }
                }
                
                // Try to stop the scan if it's not a continuous scanner
                if let Err(e) = adapter.stop_scan().await {
                    warn!("{}Failed to stop scan: {}", inner_ctx, e);
                }
            }
            
            // Ensure scan is stopped when task ends
            debug!("{}Scan task ending, stopping scan...", inner_ctx);
            if let Err(e) = adapter.stop_scan().await {
                warn!("{}Failed to stop scan during cleanup: {}", inner_ctx, e);
            }
            
            // Notify that scan has stopped
            let _ = event_tx.send(BleEvent::ScanStopped).await;
            debug!("{}Scan task completed", inner_ctx);
        });
        
        Ok(task)
    }
    
    /// Stop scanning for devices
    pub async fn stop_scanning(&mut self) -> Result<(), BluetoothError> {
        let _ctx = ErrorContext::new("BleScanner", "stop_scanning");
        
        if !self.is_scanning {
            debug!("{}Scan not started, ignoring stop_scanning request", _ctx);
            return Ok(());
        }

        // Send the cancel signal to the scan task
        if let Some(sender) = &self.cancel_sender {
            if sender.send(()).await.is_err() {
                // The scan task has already terminated
                debug!("{}Failed to send cancel signal, scan task likely already terminated", _ctx);
            }
        }

        // Wait for the task to complete
        if let Some(task) = self.scan_task.take() {
            debug!("{}Waiting for scan task to complete", _ctx);
            match task.await {
                Ok(_) => {
                    debug!("{}Scan task completed successfully", _ctx);
                },
                Err(e) => {
                    warn!("{}Error waiting for scan task to complete: {}", _ctx, e);
                    // Continue anyway, as we want to clean up
                }
            }
        }

        // Reset the state
        self.is_scanning = false;
        self.cancel_sender = None;
        self.event_sender = None;

        // Publish a scan stopped event
        self.event_broker().publish_event(BleEvent::ScanStopped);
        
        info!("{}Scan successfully stopped", _ctx);
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
            .filter_map(|d| detect_airpods(d).ok().flatten())
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
    
    /// Process a discovered device (static version)
    async fn process_discovered_device(
        device: &DiscoveredDevice,
        devices: &Arc<tokio::sync::Mutex<HashMap<BDAddr, DiscoveredDevice>>>,
        event_tx: &Sender<BleEvent>,
        _config: &ScanConfig,
    ) -> Result<(), BluetoothError> {
        let _ctx = ErrorContext::new("BleScanner", "process_discovered_device")
            .with_metadata("address", device.address.to_string());
        
        // Lock the devices map
        let mut devices_map = devices.lock().await;
        
        // Check if this is a new device or an update
        let is_new = !devices_map.contains_key(&device.address);
        
        // Determine if we should send an update event
        let send_event = if is_new {
            // Always send event for new devices
            true
        } else if let Some(existing) = devices_map.get(&device.address) {
            // For existing devices, check if we should update
            // Only update if RSSI or manufacturer data changes
            device.rssi != existing.rssi || 
            device.manufacturer_data != existing.manufacturer_data
        } else {
            false
        };
        
        // Update the device in our map
        devices_map.insert(device.address, device.clone());
        
        // Send the appropriate event
        if send_event {
            debug!("{}Sending device event for {}", _ctx, device.address);
            if is_new {
                event_tx.send(BleEvent::DeviceDiscovered(device.clone())).await
                    .map_err(|_| BluetoothError::Other("Failed to send device discovered event".to_string()))?;
            } else {
                event_tx.send(BleEvent::DeviceUpdated(device.clone())).await
                    .map_err(|_| BluetoothError::Other("Failed to send device updated event".to_string()))?;
            }
        }
        
        Ok(())
    }
    
    /// Get filtered AirPods devices matching a filter
    pub async fn get_filtered_airpods(&self, filter: &crate::airpods::AirPodsFilter) -> Vec<DiscoveredDevice> {
        let devices = self.get_devices().await;
        devices.into_iter()
            .filter(|d| filter(d))
            .collect()
    }
    
    /// Get detected AirPods devices matching a filter
    pub async fn get_filtered_detected_airpods(&self, filter: &crate::airpods::AirPodsFilter) -> Vec<DetectedAirPods> {
        let devices = self.get_devices().await;
        devices.into_iter()
            .filter(|d| filter(d))
            .filter_map(|d| detect_airpods(&d).ok().flatten())
            .collect()
    }
    
    /// Check if there are any AirPods matching the filter
    pub async fn has_airpods_matching(&self, filter: &crate::airpods::AirPodsFilter) -> bool {
        let devices = self.get_devices().await;
        devices.iter().any(filter)
    }
    
    /// Filter devices using an AirPods filter
    pub async fn filter_devices(&self, filter: &crate::airpods::AirPodsFilter) -> Vec<DiscoveredDevice> {
        let devices = self.get_devices().await;
        devices.into_iter()
            .filter(|d| filter(d))
            .collect()
    }
    
    /// Get or create an event broker
    fn event_broker(&mut self) -> &mut EventBroker {
        if self.event_broker.is_none() {
            self.event_broker = Some(EventBroker::new());
        }
        self.event_broker.as_mut().unwrap()
    }
    
    /// Subscribe to all events
    pub fn subscribe_all(&mut self) -> Receiver<BleEvent> {
        let (_, rx) = self.event_broker().subscribe(EventFilter::All);
        rx
    }
    
    /// Subscribe to events with a filter
    pub fn subscribe(&mut self, filter: EventFilter) -> Receiver<BleEvent> {
        let (_, rx) = self.event_broker().subscribe(filter);
        rx
    }
    
    /// Subscribe to AirPods events only
    pub fn subscribe_airpods(&mut self) -> Receiver<BleEvent> {
        let (_, rx) = self.event_broker().subscribe(EventFilter::airpods_only());
        rx
    }
    
    /// Get peripherals by Bluetooth address
    pub async fn get_peripherals_by_address(&self, address: &BDAddr) -> Result<Vec<Peripheral>, BluetoothError> {
        let _ctx = ErrorContext::new("BleScanner", "get_peripherals_by_address")
            .with_metadata("address", address.to_string());
        
        // Make sure we have an adapter
        let adapter = if let Some(adapter) = &self.adapter {
            adapter.clone()
        } else {
            log::error!("{}No adapter initialized", _ctx);
            return Err(BluetoothError::NoAdapter);
        };
        
        // Get all peripherals with proper error handling
        let peripherals = adapter.peripherals().await
            .map_err(|e| {
                log::error!("{}Failed to get peripherals: {}", _ctx, e);
                BluetoothError::from(e)
            })?;
        
        // Filter by address
        let matching_devices = peripherals.into_iter()
            .filter(|peripheral| peripheral.address() == *address)
            .collect::<Vec<_>>();
        
        Ok(matching_devices)
    }
    
    /// Backward compatibility method for code that still expects BleError
    #[deprecated(since = "0.1.0", note = "Use methods returning BluetoothError instead")]
    pub async fn get_peripherals_by_address_with_ble_error(&self, address: &BDAddr) -> Result<Vec<Peripheral>, BleError> {
        self.get_peripherals_by_address(address).await.map_err(|e| e.into())
    }
    
    /// Start scanning with a specific configuration
    pub async fn start_scanning_with_config(
        &mut self,
        config: ScanConfig,
    ) -> Result<Receiver<BleEvent>, BluetoothError> {
        let _ctx = ErrorContext::new("BleScanner", "start_scanning_with_config");
        
        // Similar to start_scanning but with custom config
        if self.scan_task.is_some() {
            log::warn!("{}Scan already in progress", _ctx);
            return Err(BluetoothError::ScanFailed("Scan already in progress".to_string()));
        }
        
        // Make sure we have an adapter
        let adapter = if let Some(adapter) = &self.adapter {
            adapter.clone()
        } else {
            // Try to initialize
            log::info!("{}No adapter available, attempting to initialize", _ctx);
            self.initialize().await?;
            
            // Get the adapter
            self.adapter.as_ref().ok_or_else(|| {
                log::error!("{}Failed to initialize adapter", _ctx);
                BluetoothError::NoAdapter
            })?.clone()
        };
        
        // Create channels for communication
        let (tx, rx) = channel(100);
        self.event_sender = Some(tx.clone());
        
        // Create a channel for cancellation
        let (cancel_tx, cancel_rx) = channel::<()>(1);
        self.cancel_sender = Some(cancel_tx);
        
        // Use the provided config
        self.config = config.clone();
        
        // Start scanning task with proper error handling
        let scan_task = self.start_scan_task(tx, cancel_rx, adapter).await?;
        self.scan_task = Some(scan_task);
        self.is_scanning = true;
        
        log::info!("{}Scanning started successfully with custom configuration", _ctx);
        Ok(rx)
    }
    
    /// Backward compatibility method for code that still expects BleError
    #[deprecated(since = "0.1.0", note = "Use methods returning BluetoothError instead")]
    pub async fn start_scanning_with_config_with_ble_error(
        &mut self,
        config: ScanConfig,
    ) -> Result<Receiver<BleEvent>, BleError> {
        self.start_scanning_with_config(config).await.map_err(|e| e.into())
    }
    
    /// Get a reference to this scanner as a Configurable trait object
    pub fn as_configurable(&mut self) -> &mut dyn Configurable {
        self
    }
    
    /// Initialize the scanner
    pub async fn initialize(&mut self) -> Result<(), BluetoothError> {
        let _ctx = ErrorContext::new("BleScanner", "initialize");
        let max_retries = 3; // Default retry count
        let retry_delay = Duration::from_millis(500); // Default delay between retries
        
        // Try to initialize with retries for transient failures
        for attempt in 0..=max_retries {
            match self.try_initialize().await {
                Ok(_) => {
                    if attempt > 0 {
                        debug!("Successfully initialized Bluetooth adapter after {} retries", attempt);
                    }
                    return Ok(());
                },
                Err(e) if attempt < max_retries && self.is_error_retryable(&e) => {
                    warn!("Bluetooth initialization attempt {} failed: {}. Retrying...",
                           attempt + 1, e);
                    sleep(retry_delay).await;
                    continue;
                },
                Err(e) => return Err(e),
            }
        }
        
        // Should never reach here due to the loop structure, but the compiler needs this
        Err(BluetoothError::NoAdapter)
    }
    
    // Helper method that actually attempts the initialization
    async fn try_initialize(&mut self) -> Result<(), BluetoothError> {
        let _ctx = ErrorContext::new("BleScanner", "try_initialize");
        
        // Create a manager
        let manager = Manager::new().await
            .map_err(BluetoothError::from)?;
            
        // Get the adapter list
        let adapters = manager.adapters().await
            .map_err(BluetoothError::from)?;
            
        // Find the first adapter that can be used
        if let Some(adapter) = adapters.into_iter().next() {
            self.adapter = Some(Arc::new(adapter));
            
            // Initialize the event broker if needed
            if self.event_broker.is_none() {
                self.event_broker = Some(EventBroker::new());
            }
            
            Ok(())
        } else {
            Err(BluetoothError::NoAdapter)
        }
    }
    
    // Helper method to check if an error is retryable
    fn is_error_retryable(&self, error: &BluetoothError) -> bool {
        match error {
            BluetoothError::ConnectionFailed(_) => true,
            BluetoothError::ScanFailed(_) => true,
            BluetoothError::DeviceDisconnected(_) => true,
            BluetoothError::NoAdapter => false, // Adapter missing is not retryable without user action
            BluetoothError::ApiError(_) => true, // API errors might be transient
            BluetoothError::InvalidData(_) => false, // Data validation errors aren't retryable
            BluetoothError::DeviceNotFound(_) => false, // Missing device not retryable
            BluetoothError::PermissionDenied(_) => false, // Permission issues need user intervention
            BluetoothError::Timeout(_) => true,
            BluetoothError::Other(_) => true, // Generic errors, attempt to retry
            BluetoothError::AdapterRefreshFailed { .. } => true,
            BluetoothError::AdapterNotAvailable { .. } => false, // Adapter unavailable needs user action
            BluetoothError::AdapterScanFailed { .. } => true,
        }
    }
} // End of BleScanner implementation

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

    #[test]
    fn test_bluetooth_error_conversion() {
        // Test From<btleplug::Error> for BluetoothError
        let btleplug_error = btleplug::Error::PermissionDenied;
        let bluetooth_error: BluetoothError = btleplug_error.into();
        match bluetooth_error {
            BluetoothError::PermissionDenied(_) => { /* Success */ },
            _ => panic!("Wrong error type, expected PermissionDenied"),
        }
        
        // Test From<BleError> for BluetoothError
        let ble_error = BleError::DeviceNotFound;
        let bluetooth_error: BluetoothError = ble_error.into();
        match bluetooth_error {
            BluetoothError::DeviceNotFound(_) => { /* Success */ },
            _ => panic!("Wrong error type, expected DeviceNotFound"),
        }
        
        // Test From<BluetoothError> for BleError
        let bluetooth_error = BluetoothError::NoAdapter;
        let ble_error: BleError = bluetooth_error.into();
        match ble_error {
            BleError::AdapterNotFound => { /* Success */ },
            _ => panic!("Wrong error type, expected AdapterNotFound"),
        }
    }
    
    #[test]
    fn test_error_context_creation() {
        let ctx = ErrorContext::new("TestComponent", "test_operation");
        assert_eq!(ctx.component, "TestComponent");
        assert_eq!(ctx.operation, "test_operation");
        
        let ctx_with_data = ctx.with_metadata("test_key", "test_value");
        assert!(ctx_with_data.metadata.contains_key("test_key"));
        assert_eq!(ctx_with_data.metadata.get("test_key").unwrap(), "test_value");
    }
    
    #[test]
    fn test_recovery_action_properties() {
        let recovery = RecoveryAction::Retry;
        assert_eq!(recovery.description(), "Retry the operation");
        
        let recovery = RecoveryAction::RestartApplication;
        assert_eq!(recovery.description(), "Restart the application");
    }
    
    #[tokio::test]
    async fn test_retry_logic_for_transient_errors() {
        use std::sync::{Arc, Mutex};
        use std::time::Duration;
        
        // Create a simple mock struct to track retry attempts
        struct RetryTracker {
            attempts: usize,
            should_succeed_after: usize,
            succeeded: bool,
        }
        
        impl RetryTracker {
            fn new(succeed_after: usize) -> Self {
                Self {
                    attempts: 0,
                    should_succeed_after: succeed_after,
                    succeeded: false,
                }
            }
            
            fn attempt_operation(&mut self) -> Result<(), BluetoothError> {
                self.attempts += 1;
                
                if self.attempts >= self.should_succeed_after {
                    self.succeeded = true;
                    Ok(())
                } else {
                    Err(BluetoothError::Timeout(Duration::from_millis(100)))
                }
            }
        }
        
        // Test case 1: Operation succeeds after retries
        let tracker = Arc::new(Mutex::new(RetryTracker::new(3)));
        let tracker_clone = tracker.clone();
        
        // Create a function that will use our retry mechanism
        async fn operation_with_retry(
            tracker: Arc<Mutex<RetryTracker>>, 
            max_retries: usize
        ) -> Result<(), BluetoothError> {
            let mut attempts = 0;
            let retry_delay = Duration::from_millis(10); // Short delay for testing
            
            loop {
                match tracker.lock().unwrap().attempt_operation() {
                    Ok(()) => return Ok(()),
                    Err(e) => {
                        attempts += 1;
                        if attempts >= max_retries {
                            return Err(e);
                        }
                        tokio::time::sleep(retry_delay).await;
                    }
                }
            }
        }
        
        // The operation should succeed with enough retries
        let result = operation_with_retry(tracker_clone, 5).await;
        assert!(result.is_ok());
        assert!(tracker.lock().unwrap().succeeded);
        assert_eq!(tracker.lock().unwrap().attempts, 3);
        
        // Test case 2: Operation fails if max retries is too low
        let tracker = Arc::new(Mutex::new(RetryTracker::new(5)));
        let tracker_clone = tracker.clone();
        
        let result = operation_with_retry(tracker_clone, 3).await;
        assert!(result.is_err());
        
        // Should have made exactly the max number of attempts
        assert_eq!(tracker.lock().unwrap().attempts, 3);
        assert!(!tracker.lock().unwrap().succeeded);
        
        // Verify we got the correct error type
        match result {
            Err(BluetoothError::Timeout(_)) => { /* Success */ },
            _ => panic!("Wrong error type returned"),
        }
    }
} 

// Using EventBroker from events module