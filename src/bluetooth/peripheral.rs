//! Bluetooth peripheral management
//! 
//! Provides utilities for managing Bluetooth peripherals and connections


use std::time::Duration;

use btleplug::api::{
    BDAddr, Central, Characteristic, Peripheral as _, WriteType, 
    ValueNotification
};
use uuid::Uuid;
use btleplug::platform::{Adapter, Peripheral};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use log::{debug, error, info, warn};

use crate::bluetooth::BleError;

/// Maximum connection attempts
const MAX_CONNECTION_ATTEMPTS: u8 = 3;

/// Connection timeout in seconds
const CONNECTION_TIMEOUT_SECS: u64 = 10;

/// A wrapper around btleplug's Peripheral to provide additional functionality
pub struct PeripheralManager {
    /// The underlying Bluetooth peripheral
    peripheral: Peripheral,
    
    /// Adapter that owns this peripheral
    adapter: Adapter,
    
    /// Whether the peripheral is connected
    connected: bool,
    
    /// Notification tasks
    notification_tasks: Vec<JoinHandle<()>>,
}

/// Notification handler type
pub type NotificationHandler = Box<dyn Fn(ValueNotification) + Send + Sync + 'static>;

impl PeripheralManager {
    /// Create a new peripheral manager
    pub fn new(peripheral: Peripheral, adapter: Adapter) -> Self {
        Self {
            peripheral,
            adapter,
            connected: false,
            notification_tasks: Vec::new(),
        }
    }
    
    /// Get the peripheral's address
    pub fn address(&self) -> BDAddr {
        self.peripheral.address()
    }
    
    /// Check if the peripheral is connected
    pub async fn is_connected(&self) -> Result<bool, BleError> {
        match self.peripheral.is_connected().await {
            Ok(connected) => {
                if connected != self.connected {
                    debug!("Connection state mismatch for {}: peripheral reports {}, we have {}", 
                          self.address(), connected, self.connected);
                }
                Ok(connected)
            },
            Err(e) => Err(BleError::BtlePlugError(e.to_string())),
        }
    }
    
    /// Connect to the peripheral
    pub async fn connect(&mut self) -> Result<(), BleError> {
        // Check if already connected
        if self.is_connected().await? {
            debug!("Already connected to {}", self.address());
            self.connected = true;
            return Ok(());
        }
        
        // Try to connect with multiple attempts
        let mut attempts = 0;
        let mut last_error = None;
        
        while attempts < MAX_CONNECTION_ATTEMPTS {
            attempts += 1;
            debug!("Connecting to {} (attempt {}/{})", self.address(), attempts, MAX_CONNECTION_ATTEMPTS);
            
            match timeout(
                Duration::from_secs(CONNECTION_TIMEOUT_SECS),
                self.peripheral.connect()
            ).await {
                Ok(Ok(_)) => {
                    info!("Connected to {}", self.address());
                    self.connected = true;
                    return Ok(());
                },
                Ok(Err(e)) => {
                    warn!("Failed to connect to {}: {}", self.address(), e);
                    last_error = Some(e.to_string());
                    
                    // Wait before retrying
                    tokio::time::sleep(Duration::from_secs(1)).await;
                },
                Err(_) => {
                    warn!("Connection timeout for {}", self.address());
                    last_error = Some("Connection timeout".to_string());
                    
                    // Wait before retrying
                    tokio::time::sleep(Duration::from_secs(1)).await;
                },
            }
        }
        
        // If we get here, all connection attempts failed
        error!("Failed to connect to {} after {} attempts", self.address(), attempts);
        Err(BleError::Other(format!("Failed after {} attempts: {}", 
            attempts, last_error.unwrap_or_else(|| "Unknown error".to_string()))))
    }
    
    /// Disconnect from the peripheral
    pub async fn disconnect(&mut self) -> Result<(), BleError> {
        // Cancel all notification tasks
        for task in self.notification_tasks.drain(..) {
            task.abort();
        }
        
        // Disconnect from the peripheral
        if self.is_connected().await? {
            debug!("Disconnecting from {}", self.address());
            self.peripheral.disconnect().await?;
            self.connected = false;
            info!("Disconnected from {}", self.address());
        }
        
        Ok(())
    }
    
    /// Read value from a characteristic
    pub async fn read_characteristic(&self, uuid: Uuid) -> Result<Vec<u8>, BleError> {
        // Ensure connected
        if !self.is_connected().await? {
            return Err(BleError::Other("Not connected".to_string()));
        }
        
        // Find the characteristic
        let characteristics = self.peripheral.characteristics();
        let characteristic = characteristics.iter()
            .find(|c| c.uuid == uuid)
            .ok_or(BleError::Other(format!("Characteristic not found: {}", uuid)))?;
        
        // Read the value
        debug!("Reading characteristic {}", uuid);
        let value = self.peripheral.read(characteristic).await?;
        debug!("Read {} bytes from characteristic {}", value.len(), uuid);
        
        Ok(value)
    }
    
    /// Write value to a characteristic
    pub async fn write_characteristic(&self, uuid: Uuid, data: &[u8], write_type: WriteType) -> Result<(), BleError> {
        // Ensure connected
        if !self.is_connected().await? {
            return Err(BleError::Other("Not connected".to_string()));
        }
        
        // Find the characteristic
        let characteristics = self.peripheral.characteristics();
        let characteristic = characteristics.iter()
            .find(|c| c.uuid == uuid)
            .ok_or(BleError::Other(format!("Characteristic not found: {}", uuid)))?;
        
        // Write the value
        debug!("Writing {} bytes to characteristic {}", data.len(), uuid);
        self.peripheral.write(characteristic, data, write_type).await?;
        
        Ok(())
    }
    
    /// Subscribe to notifications for a characteristic
    pub async fn subscribe(&mut self, uuid: Uuid, handler: NotificationHandler) -> Result<(), BleError> {
        // Ensure connected
        if !self.is_connected().await? {
            return Err(BleError::Other("Not connected".to_string()));
        }
        
        // Find the characteristic
        let characteristics = self.peripheral.characteristics();
        let characteristic = characteristics.iter()
            .find(|c| c.uuid == uuid)
            .ok_or(BleError::Other(format!("Characteristic not found: {}", uuid)))?;
        
        // Check if the characteristic supports notifications
        if !characteristic.properties.contains(btleplug::api::CharPropFlags::NOTIFY) {
            return Err(BleError::Other(format!("Notifications not supported for characteristic: {}", uuid)));
        }
        
        // Subscribe to notifications
        debug!("Subscribing to notifications for characteristic {}", uuid);
        self.peripheral.subscribe(characteristic).await?;
        
        // Create a channel for notifications
        let (tx, mut rx) = mpsc::channel(100);
        
        // Clone the peripheral for the notification task
        let peripheral_clone = self.peripheral.clone();
        
        // Create a task to handle notifications
        let task = tokio::spawn(async move {
            let mut notification_stream = peripheral_clone.notifications().await.unwrap();
            
            // Forward notifications to the channel
            while let Some(notification) = notification_stream.next().await {
                if let Err(_) = tx.send(notification).await {
                    break;
                }
            }
        });
        
        // Create a task to process notifications
        let handler_task = tokio::spawn(async move {
            while let Some(notification) = rx.recv().await {
                // Only process notifications for the requested characteristic
                if notification.uuid == uuid {
                    handler(notification);
                }
            }
        });
        
        // Store the notification tasks
        self.notification_tasks.push(task);
        self.notification_tasks.push(handler_task);
        
        Ok(())
    }
    
    /// Unsubscribe from notifications for a characteristic
    pub async fn unsubscribe(&self, uuid: Uuid) -> Result<(), BleError> {
        // Ensure connected
        if !self.is_connected().await? {
            return Err(BleError::Other("Not connected".to_string()));
        }
        
        // Find the characteristic
        let characteristics = self.peripheral.characteristics();
        let characteristic = characteristics.iter()
            .find(|c| c.uuid == uuid)
            .ok_or(BleError::Other(format!("Characteristic not found: {}", uuid)))?;
        
        // Unsubscribe from notifications
        debug!("Unsubscribing from notifications for characteristic {}", uuid);
        self.peripheral.unsubscribe(characteristic).await?;
        
        Ok(())
    }
    
    /// Get all characteristics
    pub async fn discover_characteristics(&self) -> Result<Vec<Characteristic>, BleError> {
        // Ensure connected
        if !self.is_connected().await? {
            return Err(BleError::Other("Not connected".to_string()));
        }
        
        // Discover services and characteristics
        debug!("Discovering characteristics for {}", self.address());
        Ok(self.peripheral.characteristics().into_iter().collect())
    }
    
    /// Connect to a service by UUID
    pub async fn connect_service(&mut self, service_uuid: Uuid) -> Result<(), BleError> {
        // Ensure connected
        if !self.is_connected().await? {
            return Err(BleError::Other("Not connected".to_string()));
        }
        
        // Find the service
        debug!("Looking for service {}", service_uuid);
        self.peripheral.discover_services().await?;
        
        let services = self.peripheral.services();
        services.iter()
            .find(|s| s.uuid == service_uuid)
            .ok_or_else(|| BleError::Other(format!("Service not found: {}", service_uuid)))?;
            
        Ok(())
    }
}

impl Drop for PeripheralManager {
    fn drop(&mut self) {
        // Cancel all notification tasks
        for task in self.notification_tasks.drain(..) {
            task.abort();
        }
    }
}

/// Extension trait for futures::stream::Stream
trait StreamExt<T> {
    /// Get the next item from the stream
    async fn next(&mut self) -> Option<T>;
}

impl<T> StreamExt<T> for futures::stream::BoxStream<'_, T> {
    async fn next(&mut self) -> Option<T> {
        futures::StreamExt::next(self).await
    }
} 