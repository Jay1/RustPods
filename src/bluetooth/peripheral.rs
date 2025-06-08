//! Bluetooth peripheral management
//!
//! Provides utilities for managing Bluetooth peripherals and connections

use std::time::Duration;

use btleplug::api::{BDAddr, CharPropFlags, Characteristic, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;
use log::{debug, error, info, warn};
use std::fmt;
use std::sync::Arc;
use std::sync::Mutex;
use uuid::Uuid;

use crate::error::{BluetoothError, ErrorContext};

/// Maximum connection attempts
const MAX_CONNECTION_ATTEMPTS: u8 = 3;

/// Connection timeout in seconds
#[allow(dead_code)]
const CONNECTION_TIMEOUT_SECS: u64 = 10;

/// Notification handler function type
pub type NotificationHandler = Box<dyn Fn(Vec<u8>) + Send + Sync + 'static>;

/// Bluetooth peripheral wrapper
pub struct BlePeripheral {
    /// Btleplug peripheral
    peripheral: Peripheral,
    /// Connection status
    is_connected: bool,
    /// Notification handlers
    notification_handlers: Arc<Mutex<Vec<(Uuid, NotificationHandler)>>>,
}

impl fmt::Debug for BlePeripheral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BlePeripheral")
            .field("peripheral", &self.peripheral.address())
            .field("is_connected", &self.is_connected)
            .finish()
    }
}

impl BlePeripheral {
    /// Create a new peripheral wrapper
    pub fn new(peripheral: Peripheral) -> Self {
        BlePeripheral {
            peripheral,
            is_connected: false,
            notification_handlers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get the peripheral address
    pub fn address(&self) -> BDAddr {
        self.peripheral.address()
    }

    /// Get the peripheral name
    pub async fn name(&self) -> Result<Option<String>, BluetoothError> {
        let ctx = ErrorContext::new("BlePeripheral", "name")
            .with_metadata("address", self.peripheral.address().to_string());

        let properties = self.peripheral.properties().await.map_err(|e| {
            error!("{}Failed to get peripheral properties: {}", ctx, e);
            BluetoothError::ApiError(format!("Failed to get peripheral properties: {}", e))
        })?;

        Ok(properties.and_then(|p| p.local_name))
    }

    /// Check if the device is connected
    pub async fn is_connected(&self) -> Result<bool, BluetoothError> {
        let ctx = ErrorContext::new("BlePeripheral", "is_connected")
            .with_metadata("address", self.peripheral.address().to_string());

        match self.peripheral.is_connected().await {
            Ok(connected) => Ok(connected),
            Err(e) => {
                error!("{}Failed to check connection status: {}", ctx, e);
                Err(BluetoothError::ApiError(format!(
                    "Failed to check connection status: {}",
                    e
                )))
            }
        }
    }

    /// Connect to the device
    pub async fn connect(&mut self) -> Result<(), BluetoothError> {
        let ctx = ErrorContext::new("BlePeripheral", "connect")
            .with_metadata("address", self.peripheral.address().to_string());

        // Already connected?
        if self.is_connected {
            return Ok(());
        }

        // Try to connect with retries
        let max_attempts = MAX_CONNECTION_ATTEMPTS;
        let mut attempt = 0;

        while attempt < max_attempts {
            attempt += 1;
            debug!(
                "{}Attempting to connect (attempt {}/{})",
                ctx, attempt, max_attempts
            );

            match self.peripheral.connect().await {
                Ok(()) => {
                    self.is_connected = true;
                    info!("{}Successfully connected", ctx);
                    return Ok(());
                }
                Err(e) => {
                    if attempt < max_attempts {
                        warn!(
                            "{}Connection attempt {} failed: {}. Retrying...",
                            ctx, attempt, e
                        );
                        // Wait before retry with increasing delay
                        tokio::time::sleep(Duration::from_millis(500 * attempt as u64)).await;
                        continue;
                    } else {
                        error!(
                            "{}Failed to connect after {} attempts: {}",
                            ctx, max_attempts, e
                        );
                        return Err(BluetoothError::ConnectionFailed(format!(
                            "Failed after {} attempts: {}",
                            max_attempts, e
                        )));
                    }
                }
            }
        }

        // This should never be reached due to the error in the loop
        Err(BluetoothError::ConnectionFailed(
            "Failed to connect".to_string(),
        ))
    }

    /// Disconnect from the device
    pub async fn disconnect(&mut self) -> Result<(), BluetoothError> {
        let ctx = ErrorContext::new("BlePeripheral", "disconnect")
            .with_metadata("address", self.peripheral.address().to_string());

        // Already disconnected?
        if !self.is_connected {
            debug!("{}Already disconnected, ignoring disconnect request", ctx);
            return Ok(());
        }

        // Try to disconnect
        match self.peripheral.disconnect().await {
            Ok(()) => {
                self.is_connected = false;
                info!("{}Successfully disconnected", ctx);
                Ok(())
            }
            Err(e) => {
                error!("{}Failed to disconnect: {}", ctx, e);
                Err(BluetoothError::DeviceDisconnected(format!(
                    "Failed to disconnect: {}",
                    e
                )))
            }
        }
    }

    /// Read a characteristic value
    pub async fn read_characteristic(&self, uuid: Uuid) -> Result<Vec<u8>, BluetoothError> {
        let ctx = ErrorContext::new("BlePeripheral", "read_characteristic")
            .with_metadata("address", self.peripheral.address().to_string())
            .with_metadata("characteristic", uuid.to_string());

        // Check if we're connected
        if !self.is_connected {
            return Err(BluetoothError::ConnectionFailed(
                "Not connected".to_string(),
            ));
        }

        // Get characteristics
        let characteristics = self.peripheral.characteristics();

        // Find the characteristic
        let characteristic = characteristics
            .iter()
            .find(|c| c.uuid == uuid)
            .ok_or_else(|| {
                let err =
                    BluetoothError::InvalidData(format!("Characteristic not found: {}", uuid));
                error!("{}Characteristic not found", ctx);
                err
            })?;

        // Read the value
        match self.peripheral.read(characteristic).await {
            Ok(value) => {
                debug!(
                    "{}Successfully read characteristic, got {} bytes",
                    ctx,
                    value.len()
                );
                Ok(value)
            }
            Err(e) => {
                error!("{}Failed to read characteristic: {}", ctx, e);
                Err(BluetoothError::ApiError(format!(
                    "Failed to read characteristic: {}",
                    e
                )))
            }
        }
    }

    /// Write a value to a characteristic
    pub async fn write_characteristic(
        &self,
        uuid: Uuid,
        data: &[u8],
        write_type: WriteType,
    ) -> Result<(), BluetoothError> {
        let ctx = ErrorContext::new("BlePeripheral", "write_characteristic")
            .with_metadata("address", self.peripheral.address().to_string())
            .with_metadata("characteristic", uuid.to_string())
            .with_metadata("data_length", data.len().to_string());

        // Check if we're connected
        if !self.is_connected {
            error!("{}Cannot write characteristic: not connected", ctx);
            return Err(BluetoothError::ConnectionFailed(
                "Not connected".to_string(),
            ));
        }

        // Get characteristics
        let characteristics = self.peripheral.characteristics();

        // Find the characteristic
        let characteristic = characteristics
            .iter()
            .find(|c| c.uuid == uuid)
            .ok_or_else(|| {
                let err =
                    BluetoothError::InvalidData(format!("Characteristic not found: {}", uuid));
                error!("{}Characteristic not found", ctx);
                err
            })?;

        // Write the value
        match self
            .peripheral
            .write(characteristic, data, write_type)
            .await
        {
            Ok(()) => {
                debug!(
                    "{}Successfully wrote {} bytes to characteristic",
                    ctx,
                    data.len()
                );
                Ok(())
            }
            Err(e) => {
                error!("{}Failed to write characteristic: {}", ctx, e);
                Err(BluetoothError::ApiError(format!(
                    "Failed to write characteristic: {}",
                    e
                )))
            }
        }
    }

    /// Subscribe to characteristic notifications
    pub async fn subscribe(
        &mut self,
        uuid: Uuid,
        handler: NotificationHandler,
    ) -> Result<(), BluetoothError> {
        let ctx = ErrorContext::new("BlePeripheral", "subscribe")
            .with_metadata("address", self.peripheral.address().to_string())
            .with_metadata("characteristic", uuid.to_string());

        // Check if we're connected
        if !self.is_connected {
            error!("{}Cannot subscribe: not connected", ctx);
            return Err(BluetoothError::ConnectionFailed(
                "Not connected".to_string(),
            ));
        }

        // Get characteristics
        let characteristics = self.peripheral.characteristics();

        // Find the characteristic
        let characteristic = characteristics
            .iter()
            .find(|c| c.uuid == uuid)
            .ok_or_else(|| {
                let err =
                    BluetoothError::InvalidData(format!("Characteristic not found: {}", uuid));
                error!("{}Characteristic not found", ctx);
                err
            })?;

        // Check if the characteristic supports notifications
        if !characteristic.properties.contains(CharPropFlags::NOTIFY) {
            error!("{}Characteristic doesn't support notifications", ctx);
            return Err(BluetoothError::InvalidData(format!(
                "Characteristic {} doesn't support notifications",
                uuid
            )));
        }

        // Subscribe to notifications
        match self.peripheral.subscribe(characteristic).await {
            Ok(()) => {
                debug!("{}Successfully subscribed to notifications", ctx);

                // Store the handler
                {
                    let mut handlers = self.notification_handlers.lock().map_err(|e| {
                        error!("{}Failed to lock notification handlers: {}", ctx, e);
                        BluetoothError::Other(format!(
                            "Failed to lock notification handlers: {}",
                            e
                        ))
                    })?;

                    handlers.push((uuid, handler));
                }

                // Start the notification handler task
                let peripheral_clone = self.peripheral.clone();
                let notification_handlers_clone = self.notification_handlers.clone();

                tokio::spawn(async move {
                    let mut notification_stream =
                        peripheral_clone.notifications().await.unwrap_or_else(|e| {
                            error!("Failed to get notification stream: {}", e);
                            panic!("Failed to get notification stream: {}", e);
                        });

                    use futures::StreamExt;
                    while let Some(notification) = notification_stream.next().await {
                        let handlers = notification_handlers_clone.lock();
                        if let Ok(handlers) = handlers {
                            for (uuid, handler) in handlers.iter() {
                                if notification.uuid == *uuid {
                                    // Call the handler with the value
                                    handler(notification.value.clone());
                                }
                            }
                        }
                    }
                });

                Ok(())
            }
            Err(e) => {
                error!("{}Failed to subscribe to notifications: {}", ctx, e);
                Err(BluetoothError::ApiError(format!(
                    "Failed to subscribe to notifications: {}",
                    e
                )))
            }
        }
    }

    /// Unsubscribe from characteristic notifications
    pub async fn unsubscribe(&self, uuid: Uuid) -> Result<(), BluetoothError> {
        let ctx = ErrorContext::new("BlePeripheral", "unsubscribe")
            .with_metadata("address", self.peripheral.address().to_string())
            .with_metadata("characteristic", uuid.to_string());

        // Check if we're connected
        if !self.is_connected {
            error!("{}Cannot unsubscribe: not connected", ctx);
            return Err(BluetoothError::ConnectionFailed(
                "Not connected".to_string(),
            ));
        }

        // Get characteristics
        let characteristics = self.peripheral.characteristics();

        // Find the characteristic
        let characteristic = characteristics
            .iter()
            .find(|c| c.uuid == uuid)
            .ok_or_else(|| {
                let err =
                    BluetoothError::InvalidData(format!("Characteristic not found: {}", uuid));
                error!("{}Characteristic not found", ctx);
                err
            })?;

        // Unsubscribe
        match self.peripheral.unsubscribe(characteristic).await {
            Ok(()) => {
                debug!("{}Successfully unsubscribed from notifications", ctx);
                Ok(())
            }
            Err(e) => {
                error!("{}Failed to unsubscribe from notifications: {}", ctx, e);
                Err(BluetoothError::ApiError(format!(
                    "Failed to unsubscribe from notifications: {}",
                    e
                )))
            }
        }
    }

    /// Discover the characteristics of the device
    pub async fn discover_characteristics(&self) -> Result<Vec<Characteristic>, BluetoothError> {
        let ctx = ErrorContext::new("BlePeripheral", "discover_characteristics")
            .with_metadata("address", self.peripheral.address().to_string());

        // Check if we're connected
        if !self.is_connected {
            error!("{}Cannot discover characteristics: not connected", ctx);
            return Err(BluetoothError::ConnectionFailed(
                "Not connected".to_string(),
            ));
        }

        // Discover services (this ensures characteristics are discovered)
        match self.peripheral.discover_services().await {
            Ok(_) => {
                debug!("{}Successfully discovered services", ctx);
                Ok(self.peripheral.characteristics().into_iter().collect())
            }
            Err(e) => {
                error!("{}Failed to discover services: {}", ctx, e);
                Err(BluetoothError::ApiError(format!(
                    "Failed to discover services: {}",
                    e
                )))
            }
        }
    }

    /// Connect to a specific service
    pub async fn connect_service(&mut self, service_uuid: Uuid) -> Result<(), BluetoothError> {
        let ctx = ErrorContext::new("BlePeripheral", "connect_service")
            .with_metadata("address", self.peripheral.address().to_string())
            .with_metadata("service", service_uuid.to_string());

        // First connect to the device if not already connected
        if !self.is_connected {
            debug!("{}Not connected, connecting first", ctx);
            self.connect().await?;
        }

        // Discover services
        debug!("{}Discovering services", ctx);
        self.peripheral.discover_services().await.map_err(|e| {
            error!("{}Failed to discover services: {}", ctx, e);
            BluetoothError::ApiError(format!("Failed to discover services: {}", e))
        })?;

        // Get the services
        let services = self.peripheral.services();

        // Check if the service is available
        let _service = services
            .iter()
            .find(|s| s.uuid == service_uuid)
            .ok_or_else(|| {
                let err =
                    BluetoothError::InvalidData(format!("Service not found: {}", service_uuid));
                error!("{}Service not found", ctx);
                err
            })?;

        info!("{}Successfully connected to service", ctx);
        Ok(())
    }
}

impl Drop for BlePeripheral {
    fn drop(&mut self) {
        // We can't use async code in drop, so we just clear the handlers
        if let Ok(mut handlers) = self.notification_handlers.lock() {
            handlers.clear();
        }
        // In a real implementation we would want to properly unsubscribe from notifications,
        // but since await is not allowed in Drop, we can't do that here
    }
}
