//! Custom error types for RustPodsMon

use crate::bluetooth::BleError;

#[derive(Debug)]
pub enum RustPodsError {
    BluetoothError(BleError),
    AirPodsError,
    UiError,
    ConfigError,
    AppError,
}

impl std::fmt::Display for RustPodsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RustPodsError::BluetoothError(e) => write!(f, "Bluetooth error: {}", e),
            RustPodsError::AirPodsError => write!(f, "AirPods error"),
            RustPodsError::UiError => write!(f, "UI error"),
            RustPodsError::ConfigError => write!(f, "Config error"),
            RustPodsError::AppError => write!(f, "App error"),
        }
    }
}

impl std::error::Error for RustPodsError {}

impl From<BleError> for RustPodsError {
    fn from(err: BleError) -> Self {
        RustPodsError::BluetoothError(err)
    }
} 