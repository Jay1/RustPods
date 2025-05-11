use std::sync::Arc;
use std::fmt;
use std::default::Default;

use btleplug::api::{
    BDAddr, Central, Manager as _, 
    ScanFilter,
};
use btleplug::platform::{Adapter, Manager};

use crate::bluetooth::BleError;

/// Information about a Bluetooth adapter
#[derive(Debug, Clone)]
pub struct AdapterInfo {
    /// Adapter index
    pub index: usize,
    /// Adapter address if available
    pub address: Option<BDAddr>,
    /// Adapter name or identifier
    pub name: String,
    /// Whether this is the default adapter
    pub is_default: bool,
}

impl fmt::Display for AdapterInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}#{}: {}{}",
            self.name,
            self.index,
            match self.address {
                Some(addr) => addr.to_string(),
                None => "unknown address".to_string(),
            },
            if self.is_default { " (default)" } else { "" }
        )
    }
}

impl Default for AdapterInfo {
    fn default() -> Self {
        Self {
            index: 0,
            name: String::new(),
            address: None,
            is_default: false,
        }
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
}

impl AdapterManager {
    /// Create a new adapter manager
    pub async fn new() -> Result<Self, BleError> {
        let manager = Arc::new(Manager::new().await?);
        
        let mut adapter_manager = Self {
            manager,
            available_adapters: Vec::new(),
            selected_index: None,
        };
        
        // Discover available adapters
        adapter_manager.refresh_adapters().await?;
        
        // Auto-select the first adapter if available
        if !adapter_manager.available_adapters.is_empty() {
            adapter_manager.selected_index = Some(0);
        }
        
        Ok(adapter_manager)
    }
    
    /// Refresh the list of available adapters
    pub async fn refresh_adapters(&mut self) -> Result<(), BleError> {
        let adapters = self.manager.adapters().await?;
        let mut adapter_infos = Vec::new();
        
        for (index, adapter) in adapters.iter().enumerate() {
            // Try to get the adapter's address (may not be available on all platforms)
            let address = Self::get_adapter_address(adapter).await;
            
            let info = AdapterInfo {
                index,
                address,
                name: Self::get_adapter_name(adapter, index).await,
                is_default: index == 0, // First adapter is considered default
            };
            
            adapter_infos.push(info);
        }
        
        self.available_adapters = adapter_infos;
        
        // Reset selection if the selected adapter is no longer available
        if let Some(index) = self.selected_index {
            if index >= self.available_adapters.len() {
                self.selected_index = None;
            }
        }
        
        Ok(())
    }
    
    /// Get the address of an adapter (if available)
    async fn get_adapter_address(_adapter: &Adapter) -> Option<BDAddr> {
        // This information may not be available on all platforms
        // On some systems, we can only use the adapter without knowing its address
        None
    }
    
    /// Get a name for the adapter based on platform-specific information
    async fn get_adapter_name(_adapter: &Adapter, index: usize) -> String {
        // Try to get a better name for the adapter if possible
        // For now, we'll just use a generic name with the index
        format!("BluetoothAdapter{}", index)
    }
    
    /// Get the list of available adapters
    pub fn get_available_adapters(&self) -> &[AdapterInfo] {
        &self.available_adapters
    }
    
    /// Select an adapter by index
    pub fn select_adapter(&mut self, index: usize) -> Result<(), BleError> {
        if index >= self.available_adapters.len() {
            return Err(BleError::AdapterNotFound);
        }
        
        self.selected_index = Some(index);
        Ok(())
    }
    
    /// Get the currently selected adapter
    pub async fn get_selected_adapter(&self) -> Result<Adapter, BleError> {
        let index = self.selected_index.ok_or(BleError::AdapterNotFound)?;
        
        let adapters = self.manager.adapters().await?;
        adapters.into_iter().nth(index).ok_or(BleError::AdapterNotFound)
    }
    
    /// Get info about the currently selected adapter
    pub fn get_selected_adapter_info(&self) -> Option<&AdapterInfo> {
        self.selected_index.and_then(|index| self.available_adapters.get(index))
    }
    
    /// Check if the adapter supports scanning
    pub async fn check_scanning_capability(&self) -> Result<bool, BleError> {
        let adapter = self.get_selected_adapter().await?;
        
        // Try to start a very brief scan to see if scanning is supported
        let result = adapter.start_scan(ScanFilter::default()).await;
        
        // If the scan started successfully, stop it immediately
        if result.is_ok() {
            let _ = adapter.stop_scan().await;
            return Ok(true);
        }
        
        // Check if the error indicates scanning is not supported
        match result {
            Err(btleplug::Error::NotSupported(_)) => Ok(false),
            Err(e) => Err(BleError::BtlePlugError(e)),
            _ => Ok(true),
        }
    }
} 