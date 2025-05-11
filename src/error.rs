//! Custom error types for RustPods

use crate::bluetooth::BleError;

#[derive(Debug)]
pub enum RustPodsError {
    BluetoothError(String),
    BluetoothApiError(BleError),
    AirPodsError,
    UiError,
    ConfigError(String),
    AppError,
    DeviceNotFound,
    BatteryMonitorError(String),
}

impl std::fmt::Display for RustPodsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RustPodsError::BluetoothError(msg) => write!(f, "Bluetooth error: {}", msg),
            RustPodsError::BluetoothApiError(e) => write!(f, "Bluetooth API error: {}", e),
            RustPodsError::AirPodsError => write!(f, "AirPods error"),
            RustPodsError::UiError => write!(f, "UI error"),
            RustPodsError::ConfigError(msg) => write!(f, "Config error: {}", msg),
            RustPodsError::AppError => write!(f, "App error"),
            RustPodsError::DeviceNotFound => write!(f, "Device not found"),
            RustPodsError::BatteryMonitorError(msg) => write!(f, "Battery monitoring error: {}", msg),
        }
    }
}

impl std::error::Error for RustPodsError {}

impl From<BleError> for RustPodsError {
    fn from(err: BleError) -> Self {
        RustPodsError::BluetoothApiError(err)
    }
} 