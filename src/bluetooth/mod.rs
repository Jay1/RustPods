//! BLE scanning and device management

pub mod scanner;
pub mod adapter;
pub mod examples;
pub mod scanner_config;
mod filter;
mod peripheral;
pub mod events;
pub mod battery;
pub mod battery_monitor;

// Import error types from crate root
use crate::error::{BluetoothError, RustPodsError, ErrorContext, RecoveryAction};
use std::fmt::Debug;

// Re-export all necessary types from scanner
pub use scanner::{
    BleScanner, 
    DiscoveredDevice,
    BleScannerConfig,
    parse_bdaddr,
};

// Re-export ScanConfig
pub use scanner_config::ScanConfig;

pub use adapter::{
    AdapterManager, AdapterInfo,
};

pub use events::{
    EventBroker, EventFilter, SubscriberId, receiver_to_stream, BleEvent
};

pub use battery::{
    AirPodsBatteryStatus, extract_battery_status, start_battery_monitoring
};

pub use battery_monitor::{
    BatteryMonitor, BatteryMonitorOptions, BatteryAlert
};

// Export examples for testing
pub use examples::{
    discover_adapters, scan_with_adapter, interval_scanning,
    airpods_filtering
};

pub struct BluetoothManager;

pub trait BluetoothDevice {}

pub use adapter::BluetoothAdapter;
pub use adapter::BleAdapterEvent;
pub use adapter::AdapterCapabilities;
pub use adapter::AdapterStatus;
pub use filter::*;
pub use peripheral::*;

/// Helper function to create error context for Bluetooth operations
pub fn create_error_context(component: &str, operation: &str) -> ErrorContext {
    ErrorContext::new(component, operation)
}

/// Create a BluetoothError with recovery action
pub fn bluetooth_error_with_recovery(
    error_type: BluetoothError,
    recovery: RecoveryAction,
) -> BluetoothError {
    match error_type {
        BluetoothError::ApiError(e) => BluetoothError::ApiError(e),
        BluetoothError::NoAdapter => BluetoothError::NoAdapter,
        BluetoothError::ScanFailed(msg) => BluetoothError::ScanFailed(msg),
        BluetoothError::DeviceNotFound(msg) => BluetoothError::DeviceNotFound(msg),
        BluetoothError::InvalidData(msg) => BluetoothError::InvalidData(msg),
        BluetoothError::Timeout(duration) => BluetoothError::Timeout(duration),
        BluetoothError::ConnectionFailed(msg) => BluetoothError::ConnectionFailed(msg),
        BluetoothError::DeviceDisconnected(msg) => BluetoothError::DeviceDisconnected(msg),
        BluetoothError::PermissionDenied(msg) => BluetoothError::PermissionDenied(msg),
        BluetoothError::AdapterRefreshFailed { error, .. } => BluetoothError::AdapterRefreshFailed {
            error,
            recovery,
            retries: 0,
        },
        BluetoothError::AdapterNotAvailable { reason, .. } => BluetoothError::AdapterNotAvailable {
            reason,
            recovery,
        },
        BluetoothError::AdapterScanFailed { error, .. } => BluetoothError::AdapterScanFailed {
            error,
            recovery,
        },
        BluetoothError::Other(msg) => BluetoothError::Other(msg),
    }
}

/// Enhanced error handling for Bluetooth operations with context
/// 
/// This function takes an error and adds appropriate context information
/// and recovery actions based on the type of error and operation.
pub fn handle_bluetooth_error<T, E>(
    result: Result<T, E>,
    component: &str,
    operation: &str,
    recovery: Option<RecoveryAction>
) -> Result<T, BluetoothError>
where
    E: Into<BluetoothError>
{
    result.map_err(|err| {
        let bluetooth_err = err.into();
        // Log the error with context
        log::error!("[{}:{}] {}", component, operation, bluetooth_err);
        
        // Apply recommended recovery action or use the provided one
        let recovery_action = recovery.unwrap_or(match &bluetooth_err {
            BluetoothError::ConnectionFailed(_) => RecoveryAction::Retry,
            BluetoothError::DeviceNotFound(_) => RecoveryAction::Retry,
            BluetoothError::ScanFailed(_) => RecoveryAction::RestartApplication,
            BluetoothError::NoAdapter => RecoveryAction::SelectDifferentAdapter,
            BluetoothError::Timeout(_) => RecoveryAction::Retry,
            _ => RecoveryAction::NotifyUser,
        });
        
        // Return error with context and recovery action
        match bluetooth_err {
            BluetoothError::AdapterRefreshFailed { error, .. } => BluetoothError::AdapterRefreshFailed {
                error,
                recovery: recovery_action,
                retries: 0,
            },
            BluetoothError::AdapterNotAvailable { reason, .. } => BluetoothError::AdapterNotAvailable {
                reason,
                recovery: recovery_action,
            },
            BluetoothError::AdapterScanFailed { error, .. } => BluetoothError::AdapterScanFailed {
                error,
                recovery: recovery_action,
            },
            _ => bluetooth_error_with_recovery(bluetooth_err, recovery_action)
        }
    })
}

/// Convert a btleplug Error to our custom BluetoothError
pub fn convert_btleplug_error(error: btleplug::Error, _component: &str, operation: &str) -> BluetoothError {
    use btleplug::Error as BtlePlugError;
    
    match error {
        BtlePlugError::PermissionDenied => BluetoothError::PermissionDenied(
            format!("Permission denied during {}", operation)
        ),
        BtlePlugError::DeviceNotFound => BluetoothError::DeviceNotFound(
            format!("Device not found during {}", operation)
        ),
        BtlePlugError::NotConnected => BluetoothError::ConnectionFailed(
            format!("Device not connected during {}", operation)
        ),
        BtlePlugError::InvalidBDAddr(_) => BluetoothError::InvalidData(
            "Invalid Bluetooth address format".to_string()
        ),
        BtlePlugError::Uuid(_) => BluetoothError::InvalidData(
            "Invalid UUID format".to_string()
        ),
        BtlePlugError::NotSupported(_) => BluetoothError::Other(
            format!("Operation not supported: {}", operation)
        ),
        BtlePlugError::Other(msg) => BluetoothError::Other(msg.to_string()),
        _ => BluetoothError::Other(
            format!("Unknown Bluetooth error during {}", operation)
        ),
    }
}

/// Create a BluetoothError from a btleplug error
pub fn create_bluetooth_error<E: std::fmt::Display + Debug>(
    error: E, 
    _component: &str, 
    _operation: &str
) -> BluetoothError {
    // Convert to string and create a generic error
    BluetoothError::Other(error.to_string())
} 