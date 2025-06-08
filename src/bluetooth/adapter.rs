use log::{debug, error, warn};
use std::collections::HashMap;
use std::default::Default;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use btleplug::api::{BDAddr, Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager};
use tokio::time::sleep;

use crate::bluetooth::scanner::DiscoveredDevice;
use crate::bluetooth::BluetoothError;
use crate::error::{ErrorContext, RecoveryAction};

/// Adapter capabilities
#[derive(Debug, Clone, Default)]
pub struct AdapterCapabilities {
    /// Whether the adapter supports scanning
    pub supports_scanning: bool,

    /// Whether the adapter supports the central role
    pub supports_central_role: bool,

    /// Whether the adapter supports advertising
    pub supports_advertising: bool,

    /// Whether the adapter supports connecting to peripherals
    pub supports_connecting: bool,

    /// Whether the adapter is powered on
    pub is_powered_on: bool,

    /// Maximum number of connections supported
    pub max_connections: u8,

    /// When the capabilities were last checked
    pub last_checked: Option<std::time::Instant>,

    /// Adapter status
    pub status: AdapterStatus,

    /// Adapter information string
    pub adapter_info: String,
}

/// Adapter status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterStatus {
    /// Normal operation
    Normal,
    /// Adapter is ready
    Ready,
    /// Adapter is in error state
    Error,
    /// Adapter is disabled
    Disabled,
    /// Adapter is busy
    Busy,
}

impl Default for AdapterStatus {
    fn default() -> Self {
        Self::Normal
    }
}

/// Information about a Bluetooth adapter
#[derive(Debug, Clone, Default)]
pub struct AdapterInfo {
    /// Adapter index
    pub index: usize,
    /// Adapter address if available
    pub address: Option<BDAddr>,
    /// Adapter name or identifier
    pub name: String,
    /// Whether this is the default adapter
    pub is_default: bool,
    /// Adapter capabilities
    pub capabilities: AdapterCapabilities,
    /// Adapter vendor/manufacturer if available
    pub vendor: Option<String>,
}

impl fmt::Display for AdapterInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}#{}: {}{}{}",
            self.name,
            self.index,
            match self.address {
                Some(addr) => addr.to_string(),
                None => "unknown address".to_string(),
            },
            if self.is_default { " (default)" } else { "" },
            match self.capabilities.status {
                AdapterStatus::Normal => "",
                AdapterStatus::Ready => " (ready)",
                AdapterStatus::Error => " (error)",
                AdapterStatus::Disabled => " (disabled)",
                AdapterStatus::Busy => " (busy)",
            }
        )
    }
}

/// Handle adapter discovery and selection
#[derive(Clone)]
pub struct AdapterManager {
    /// The system's Bluetooth manager
    manager: Arc<Manager>,
    /// List of available adapter info
    available_adapters: Vec<AdapterInfo>,
    /// Currently selected adapter index
    selected_index: Option<usize>,
    /// Adapter status history (address -> status history)
    adapter_history: HashMap<String, Vec<(std::time::Instant, AdapterStatus)>>,
}

impl AdapterManager {
    /// Create a new adapter manager
    pub async fn new() -> Result<Self, BluetoothError> {
        // Create an error context for logging
        let _ctx = ErrorContext::new("AdapterManager", "new");

        // Use enhanced error handling
        let manager = Arc::new(crate::bluetooth::handle_bluetooth_error(
            Manager::new().await,
            "AdapterManager",
            "new",
            Some(RecoveryAction::RestartApplication),
        )?);

        let mut adapter_manager = Self {
            manager,
            available_adapters: Vec::new(),
            selected_index: None,
            adapter_history: HashMap::new(),
        };

        // Discover available adapters
        adapter_manager.refresh_adapters().await?;

        // Auto-select the first adapter if available
        if !adapter_manager.available_adapters.is_empty() {
            adapter_manager.selected_index = Some(0);
        } else {
            warn!("No Bluetooth adapters found during initialization");
        }

        Ok(adapter_manager)
    }

    /// Refresh the list of available adapters with retry logic
    pub async fn refresh_adapters(&mut self) -> Result<(), BluetoothError> {
        let ctx = ErrorContext::new("AdapterManager", "refresh_adapters");

        // Try to refresh the adapters
        let mut result = self.try_refresh_adapters().await;
        let mut attempt = 1;
        let mut _last_error = None;
        let max_attempts = 3;
        let retry_delay = Duration::from_millis(500);

        while result.is_err() && attempt < max_attempts {
            match result {
                Ok(_) => break, // Success, break out of the loop
                Err(e) => {
                    attempt += 1;
                    let error_string = format!("{}", e);
                    _last_error = Some(e);

                    if attempt < max_attempts {
                        warn!(
                            "{}Attempt {}/{} to refresh adapters failed: {}. Retrying...",
                            ctx, attempt, max_attempts, error_string
                        );
                    } else {
                        error!(
                            "{}All {} attempts to refresh adapters failed: {}",
                            ctx, max_attempts, error_string
                        );
                    }
                }
            }

            // Add a delay before retrying
            tokio::time::sleep(retry_delay).await;

            // Try again
            result = self.try_refresh_adapters().await;
        }

        result
    }

    /// Try to refresh adapter list once
    async fn try_refresh_adapters(&mut self) -> Result<(), BluetoothError> {
        let ctx = ErrorContext::new("AdapterManager", "try_refresh_adapters");

        // Get the list of adapters from the system using enhanced error handling
        let adapters = crate::bluetooth::handle_bluetooth_error(
            self.manager.adapters().await,
            "AdapterManager",
            "try_refresh_adapters",
            Some(RecoveryAction::Retry),
        )?;

        let mut adapter_infos = Vec::new();

        for (index, adapter) in adapters.iter().enumerate() {
            // Try to get the adapter's address (may not be available on all platforms)
            let address = Self::get_adapter_address(adapter).await;

            // Generate a unique identifier for history tracking
            let adapter_id = if let Some(addr) = address {
                addr.to_string()
            } else {
                format!("adapter_{}", index)
            };

            // Check adapter capabilities
            let capabilities = self.check_adapter_capabilities(adapter, &adapter_id).await;

            // Try to get vendor information
            let vendor = Self::get_adapter_vendor(adapter).await;

            // Get adapter name
            let name = Self::get_adapter_name(adapter, index).await;

            let info = AdapterInfo {
                index,
                address,
                name,
                is_default: index == 0, // First adapter is considered default
                capabilities,
                vendor,
            };

            log::debug!(
                "{}Found adapter: {} (status: {:?})",
                ctx,
                info.name,
                info.capabilities.status
            );
            adapter_infos.push(info);
        }

        self.available_adapters = adapter_infos;

        // Reset selection if the selected adapter is no longer available
        if let Some(index) = self.selected_index {
            if index >= self.available_adapters.len() {
                log::warn!(
                    "{}Previously selected adapter #{} is no longer available",
                    ctx,
                    index
                );
                self.selected_index = None;
            }
        }

        Ok(())
    }

    /// Check adapter capabilities    
    async fn check_adapter_capabilities(
        &mut self,
        adapter: &Adapter,
        adapter_id: &str,
    ) -> AdapterCapabilities {
        let now = std::time::Instant::now();

        // Check if scanning is supported
        let scanning_supported = self
            .check_scanning_capability(adapter)
            .await
            .unwrap_or(false);

        // Check if connecting is supported by trying to get a list of connected peripherals
        let supports_connecting = adapter.peripherals().await.is_ok();

        // Determine adapter status based on capabilities
        let status = if !scanning_supported {
            AdapterStatus::Error
        } else {
            // Assume powered on since we got this far
            AdapterStatus::Normal
        };

        // Update adapter history
        self.update_adapter_history(adapter_id, status, now);

        // Create and return capabilities with all properties set in the initializer
        AdapterCapabilities {
            supports_scanning: scanning_supported,
            supports_central_role: true, // Assume central role support
            supports_advertising: false, // Not using advertising for now
            supports_connecting,
            is_powered_on: true, // Assume powered on since we got this far
            max_connections: 5,  // Default value
            last_checked: Some(now),
            status,
            adapter_info: format!("Adapter {}", adapter_id),
        }
    }

    /// Update adapter status history
    fn update_adapter_history(
        &mut self,
        adapter_id: &str,
        status: AdapterStatus,
        timestamp: std::time::Instant,
    ) {
        let history = self
            .adapter_history
            .entry(adapter_id.to_string())
            .or_default();

        // Only add entry if status changed or it's been a while since the last update
        if history.is_empty()
            || history
                .last()
                .map(|(_, last_status)| *last_status != status)
                .unwrap_or(true)
            || history
                .last()
                .map(|(last_time, _)| {
                    timestamp.duration_since(*last_time) > Duration::from_secs(300)
                })
                .unwrap_or(true)
        {
            history.push((timestamp, status));

            // Keep history at a reasonable size
            if history.len() > 10 {
                history.remove(0);
            }
        }
    }

    /// Get the address of an adapter (if available)
    async fn get_adapter_address(_adapter: &Adapter) -> Option<BDAddr> {
        // This is not directly supported by btleplug in a cross-platform way
        // For Windows, we can try to get this from the adapter info in the future
        // For now, we return None as a proper implementation would be platform-specific
        None
    }

    /// Try to determine adapter vendor information
    async fn get_adapter_vendor(_adapter: &Adapter) -> Option<String> {
        // This information is not available through btleplug
        // Platform-specific code could be added here
        None
    }

    /// Get a name for the adapter based on platform-specific information
    async fn get_adapter_name(adapter: &Adapter, index: usize) -> String {
        // Try to get a better name for the adapter if possible
        // The adapter.identifier() method might provide useful information on some platforms
        if let Ok(info) = adapter.adapter_info().await {
            // Different BLE libraries might have different field names for the identifier
            // Using info as a string directly instead of trying to access a specific field
            return info.to_string();
        }

        // Fallback: use a generic name with the index
        format!("BluetoothAdapter{}", index)
    }

    /// Get the list of available adapters
    pub fn get_available_adapters(&self) -> &[AdapterInfo] {
        &self.available_adapters
    }

    /// Select an adapter by index
    pub fn select_adapter(&mut self, index: usize) -> Result<(), BluetoothError> {
        let ctx = ErrorContext::new("AdapterManager", "select_adapter")
            .with_metadata("adapter_index", index.to_string());

        if index >= self.available_adapters.len() {
            log::error!(
                "{}Invalid adapter index: {} (available: {})",
                ctx,
                index,
                self.available_adapters.len()
            );
            return Err(BluetoothError::AdapterNotAvailable {
                reason: format!(
                    "Adapter index {} out of bounds (max: {})",
                    index,
                    self.available_adapters.len().saturating_sub(1)
                ),
                recovery: RecoveryAction::SelectDifferentAdapter,
            });
        }

        self.selected_index = Some(index);
        log::debug!(
            "{}Selected adapter index {}: {}",
            ctx,
            index,
            self.available_adapters[index].name
        );
        Ok(())
    }

    /// Select the best available adapter based on capabilities
    pub fn select_best_adapter(&mut self) -> Result<(), BluetoothError> {
        let ctx = ErrorContext::new("AdapterManager", "select_best_adapter");

        if self.available_adapters.is_empty() {
            log::error!("{}No adapters available", ctx);
            return Err(BluetoothError::NoAdapter);
        }

        // Find the first adapter with normal status that supports scanning
        for (index, info) in self.available_adapters.iter().enumerate() {
            if info.capabilities.status == AdapterStatus::Normal
                && info.capabilities.supports_scanning
            {
                self.selected_index = Some(index);
                log::debug!("{}Selected optimal adapter #{}: {}", ctx, index, info.name);
                return Ok(());
            }
        }

        // If no ideal adapter found, use the first one that supports scanning
        for (index, info) in self.available_adapters.iter().enumerate() {
            if info.capabilities.supports_scanning {
                self.selected_index = Some(index);
                log::debug!(
                    "{}Selected adapter with scanning support #{}: {}",
                    ctx,
                    index,
                    info.name
                );
                return Ok(());
            }
        }

        // Last resort: use the first adapter regardless of capabilities
        self.selected_index = Some(0);
        log::warn!(
            "{}Selecting first available adapter regardless of capabilities: {}",
            ctx,
            self.available_adapters[0].name
        );
        Ok(())
    }

    /// Get the currently selected adapter
    pub async fn get_selected_adapter(&self) -> Result<Adapter, BluetoothError> {
        let ctx = ErrorContext::new("AdapterManager", "get_selected_adapter");

        let index = self.selected_index.ok_or_else(|| {
            log::error!("{}No adapter selected", ctx);
            BluetoothError::NoAdapter
        })?;

        let adapters = crate::bluetooth::handle_bluetooth_error(
            self.manager.adapters().await,
            "AdapterManager",
            "get_selected_adapter",
            Some(RecoveryAction::RestartApplication),
        )?;

        adapters.into_iter().nth(index).ok_or_else(|| {
            log::error!("{}Selected adapter no longer available", ctx);
            BluetoothError::AdapterNotAvailable {
                reason: format!("Selected adapter at index {} is no longer available", index),
                recovery: RecoveryAction::SelectDifferentAdapter,
            }
        })
    }

    /// Get info about the currently selected adapter
    pub fn get_selected_adapter_info(&self) -> Option<&AdapterInfo> {
        self.selected_index
            .and_then(|index| self.available_adapters.get(index))
    }

    /// Get adapter history for a specific adapter
    pub fn get_adapter_history(
        &self,
        adapter_id: &str,
    ) -> Option<&[(std::time::Instant, AdapterStatus)]> {
        self.adapter_history.get(adapter_id).map(|h| h.as_slice())
    }

    /// Check if the adapter supports scanning
    async fn check_scanning_capability(
        &mut self,
        adapter: &Adapter,
    ) -> Result<bool, BluetoothError> {
        let ctx = ErrorContext::new("AdapterManager", "check_scanning_capability")
            .with_metadata("adapter_index", format!("{:p}", adapter));

        // Try to start a scan to check if scanning is supported
        log::debug!("{}Testing adapter scanning capability", ctx);
        let result = adapter.start_scan(ScanFilter::default()).await;

        // Cleanup after the test
        let _ = adapter.stop_scan().await;

        match result {
            Ok(_) => {
                // Scan started successfully, so scanning is supported
                log::debug!("{}Adapter supports scanning", ctx);
                Ok(true)
            }
            Err(e) => {
                // Convert error to string for analysis
                let err_str = e.to_string().to_lowercase();

                // Check message to see if scanning is fundamentally unsupported
                let scanning_unsupported =
                    err_str.contains("not supported") || err_str.contains("unsupported");

                if scanning_unsupported {
                    log::warn!("{}Adapter does not support scanning: {}", ctx, err_str);
                    Ok(false)
                } else if err_str.contains("no adapter") || err_str.contains("not found") {
                    log::error!("{}No adapter available for scanning", ctx);
                    Ok(false)
                } else if err_str.contains("permission") {
                    log::error!("{}Permission denied for scanning", ctx);
                    // Permission issues need to be resolved by the user, but the adapter technically supports scanning
                    Ok(true)
                } else {
                    log::warn!("{}Other error during scan test: {}", ctx, err_str);
                    // For other errors, we assume scanning is supported but there's a temporary issue
                    Ok(true)
                }
            }
        }
    }

    /// Attempt to recover an adapter that's in a troubled state
    pub async fn try_recover_adapter(&mut self, index: usize) -> Result<bool, BluetoothError> {
        let ctx = ErrorContext::new("AdapterManager", "try_recover_adapter")
            .with_metadata("adapter_index", index.to_string());

        if index >= self.available_adapters.len() {
            return Err(BluetoothError::NoAdapter);
        }

        // Get the latest adapter list from the system
        let adapters = self
            .manager
            .adapters()
            .await
            .map_err(BluetoothError::from)?;
        let adapter =
            adapters
                .into_iter()
                .nth(index)
                .ok_or_else(|| BluetoothError::AdapterNotAvailable {
                    reason: format!("Adapter at index {} not found", index),
                    recovery: RecoveryAction::RestartApplication,
                })?;

        log::info!("{}Attempting to recover adapter", ctx);

        // First try to check scanning capability again (it might have recovered on its own)
        let supports_scanning = self.check_scanning_capability(&adapter).await?;

        if supports_scanning {
            // Update adapter info with recovered status
            if let Some(info) = self.available_adapters.get_mut(index) {
                info.capabilities.supports_scanning = true;
                info.capabilities.status = AdapterStatus::Normal;
                info.capabilities.last_checked = Some(std::time::Instant::now());

                // Get a copy of the adapter ID before we need to borrow self mutably again
                let adapter_id = if let Some(addr) = info.address {
                    addr.to_string()
                } else {
                    format!("adapter_{}", index)
                };

                // Get a copy of the last_checked timestamp before releasing the borrow
                let timestamp = info.capabilities.last_checked.unwrap();

                // Now update adapter history with the copied values
                self.update_adapter_history(&adapter_id, AdapterStatus::Normal, timestamp);

                log::info!("{}Successfully recovered adapter", ctx);
            }

            return Ok(true);
        }

        log::warn!(
            "{}Could not recover adapter through scanning capability check",
            ctx
        );

        // If checking the capability didn't resolve the issue, we'd need more advanced recovery
        // techniques that are platform-specific and may require system-level permissions.
        // For now, we'll just return false to indicate failure.

        Ok(false)
    }

    /// Get the adapter for the specified address
    pub async fn get_adapter_by_address(&self, address: BDAddr) -> Result<Adapter, BluetoothError> {
        let ctx = ErrorContext::new("AdapterManager", "get_adapter_by_address")
            .with_metadata("adapter_address", address.to_string());

        // Get the list of adapters
        let adapters = crate::bluetooth::handle_bluetooth_error(
            self.manager.adapters().await,
            "AdapterManager",
            "get_adapter_by_address",
            Some(RecoveryAction::RestartApplication),
        )?;

        if adapters.is_empty() {
            return Err(BluetoothError::NoAdapter);
        }

        // Find the adapter with the specified address
        for adapter in adapters {
            if let Some(addr) = Self::get_adapter_address(&adapter).await {
                if addr == address {
                    log::debug!("{}Found adapter matching address {}", ctx, address);
                    return Ok(adapter);
                }
            }
        }

        // If we get here, no adapter was found
        log::warn!("{}No adapter found with address {}", ctx, address);
        Err(BluetoothError::AdapterNotAvailable {
            reason: format!("No adapter with address {} found", address),
            recovery: RecoveryAction::RestartApplication,
        })
    }

    /// Check if Bluetooth is available
    pub async fn is_bluetooth_available(&self) -> Result<bool, BluetoothError> {
        let ctx = ErrorContext::new("AdapterManager", "is_bluetooth_available");

        // Get the list of adapters with error handling
        let adapters_result = self.manager.adapters().await;

        match adapters_result {
            Ok(adapters) => {
                // Check if there are any adapters
                let is_available = !adapters.is_empty();
                log::debug!(
                    "{}Bluetooth is {}",
                    ctx,
                    if is_available {
                        "available"
                    } else {
                        "not available"
                    }
                );
                Ok(is_available)
            }
            Err(e) => {
                // If error relates to Bluetooth being unavailable, return false instead of error
                let err_str = e.to_string().to_lowercase();
                if err_str.contains("bluetooth")
                    && (err_str.contains("unavailable")
                        || err_str.contains("disabled")
                        || err_str.contains("powered off"))
                {
                    log::info!("{}Bluetooth is not available: {}", ctx, e);
                    return Ok(false);
                }

                // For other errors, use our error handling utility
                let error = crate::bluetooth::convert_btleplug_error(
                    e,
                    "AdapterManager",
                    "is_bluetooth_available",
                );
                log::error!("{}Error checking Bluetooth availability: {}", ctx, error);
                Err(error)
            }
        }
    }
}

/// Bluetooth adapter event
#[derive(Debug, Clone)]
pub enum BleAdapterEvent {
    /// Device discovered event
    DeviceDiscovered(DiscoveredDevice),
    /// Device updated event
    DeviceUpdated(DiscoveredDevice),
    /// Scan started event
    ScanStarted,
    /// Scan stopped event
    ScanStopped,
    /// Error event
    Error(String),
}

/// Wrapper around btleplug Adapter for easier use
#[derive(Clone)]
pub struct BluetoothAdapter {
    /// The underlying adapter
    adapter: Arc<Adapter>,
    /// Last known status
    status: AdapterStatus,
    /// Adapter capabilities
    capabilities: AdapterCapabilities,
}

impl BluetoothAdapter {
    /// Create a new BluetoothAdapter
    pub async fn new() -> Result<Self, BluetoothError> {
        let ctx = ErrorContext::new("BluetoothAdapter", "new");

        // Create a manager with proper error conversion
        let manager = Manager::new().await.map_err(|e| {
            log::error!("{}Failed to initialize Bluetooth manager: {}", ctx, e);
            BluetoothError::from(e)
        })?;

        // Get the adapter list
        let adapters = manager.adapters().await.map_err(|e| {
            log::error!("{}Failed to get Bluetooth adapters: {}", ctx, e);
            BluetoothError::from(e)
        })?;

        // Find the first adapter
        let adapter = adapters.into_iter().next().ok_or_else(|| {
            log::error!("{}No Bluetooth adapters found", ctx);
            BluetoothError::NoAdapter
        })?;

        log::debug!("{}Successfully found Bluetooth adapter", ctx);

        // Create capabilities with all properties set in the initializer
        let capabilities = AdapterCapabilities {
            supports_scanning: true, // Assume it supports scanning
            supports_central_role: true,
            supports_advertising: false, // Not using advertising for now
            supports_connecting: true,
            is_powered_on: true,
            max_connections: 5, // Default value
            last_checked: Some(std::time::Instant::now()),
            status: AdapterStatus::Normal,
            adapter_info: format!("{:?}", adapter),
        };

        Ok(Self {
            adapter: Arc::new(adapter),
            capabilities,
            status: AdapterStatus::Normal,
        })
    }

    /// Get the Bluetooth adapter address
    ///
    /// This method returns the address of the Bluetooth adapter if available
    pub async fn get_address(&self) -> Result<Option<BDAddr>, BluetoothError> {
        let ctx = ErrorContext::new("BluetoothAdapter", "get_address");

        // This is not directly supported by btleplug in a cross-platform way
        // For Windows, we can get this from the adapter properties
        // For other platforms, we might need different approaches
        // We just return None for now as a proper implementation would be platform-specific
        // A more robust implementation would use platform-specific code to get the address
        log::debug!(
            "{}Getting adapter address is not supported cross-platform",
            ctx
        );
        Ok(None)
    }

    /// Get the adapter capabilities
    pub fn get_capabilities(&self) -> &AdapterCapabilities {
        &self.capabilities
    }

    /// Get the status
    pub fn get_status(&self) -> AdapterStatus {
        self.status
    }

    /// Get the underlying adapter
    pub fn get_adapter(&self) -> Arc<Adapter> {
        self.adapter.clone()
    }

    /// Start scanning for devices
    pub async fn start_scan(&self) -> Result<Vec<DiscoveredDevice>, BluetoothError> {
        let ctx = ErrorContext::new("BluetoothAdapter", "start_scan");

        // Start scanning with better error handling
        self.adapter
            .start_scan(ScanFilter::default())
            .await
            .map_err(|e| {
                log::error!("{}Failed to start scan: {}", ctx, e);
                BluetoothError::ScanFailed(format!("Failed to start scan: {}", e))
            })?;

        log::debug!("{}Successfully started Bluetooth scan", ctx);

        // Return empty list initially
        Ok(Vec::new())
    }

    /// Stop scanning for devices
    pub async fn stop_scan(&self) -> Result<(), BluetoothError> {
        let ctx = ErrorContext::new("BluetoothAdapter", "stop_scan");

        self.adapter.stop_scan().await.map_err(|e| {
            log::warn!("{}Failed to stop scan: {}", ctx, e);
            BluetoothError::from(e)
        })?;

        log::debug!("{}Successfully stopped Bluetooth scan", ctx);
        Ok(())
    }

    /// Get discovered devices
    pub async fn get_discovered_devices(&self) -> Result<Vec<DiscoveredDevice>, BluetoothError> {
        let ctx = ErrorContext::new("BluetoothAdapter", "get_discovered_devices");

        let peripherals = self.adapter.peripherals().await.map_err(|e| {
            log::error!("{}Failed to get peripherals: {}", ctx, e);
            BluetoothError::from(e)
        })?;

        // Convert peripherals to discovered devices
        let mut devices = Vec::new();

        for peripheral in peripherals {
            // Try to get properties
            match peripheral.properties().await {
                Ok(Some(properties)) => {
                    // Create discovered device
                    let device = DiscoveredDevice {
                        address: properties.address,
                        name: properties.local_name,
                        rssi: properties.rssi,
                        manufacturer_data: properties.manufacturer_data,
                        is_potential_airpods: false, // We'll compute this later
                        last_seen: std::time::Instant::now(),
                        is_connected: false, // Default, will be updated if needed
                        service_data: properties.service_data,
                        services: properties.services,
                        tx_power_level: properties.tx_power_level,
                    };

                    devices.push(device);
                }
                Ok(None) => {
                    log::debug!(
                        "{}Skipping peripheral with no properties: {}",
                        ctx,
                        peripheral.address()
                    );
                }
                Err(e) => {
                    log::warn!(
                        "{}Error getting properties for peripheral {}: {}",
                        ctx,
                        peripheral.address(),
                        e
                    );
                    // Continue with other devices
                }
            }
        }

        log::debug!("{}Found {} devices", ctx, devices.len());
        Ok(devices)
    }

    /// Create a new adapter with retry logic
    pub async fn new_with_retry() -> Result<Self, BluetoothError> {
        let ctx = ErrorContext::new("AdapterManager", "new");
        let mut result = Self::new().await;
        let mut attempt = 0;
        let max_attempts = 3;
        let retry_delay = Duration::from_millis(500);

        while result.is_err() && attempt < max_attempts {
            attempt += 1;
            debug!(
                "{}Adapter initialization attempt {}/{} failed, retrying...",
                ctx, attempt, max_attempts
            );
            sleep(retry_delay).await;
            result = Self::new().await;
        }

        result
    }
}
