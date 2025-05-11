//! Custom error types for RustPodsMon

#[derive(Debug)]
pub enum RustPodsError {
    BluetoothError,
    AirPodsError,
    UiError,
    ConfigError,
    AppError,
}

impl std::fmt::Display for RustPodsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RustPodsError::BluetoothError => write!(f, "Bluetooth error"),
            RustPodsError::AirPodsError => write!(f, "AirPods error"),
            RustPodsError::UiError => write!(f, "UI error"),
            RustPodsError::ConfigError => write!(f, "Config error"),
            RustPodsError::AppError => write!(f, "App error"),
        }
    }
}

impl std::error::Error for RustPodsError {} 