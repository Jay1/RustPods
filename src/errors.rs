//! Error types for the RustPods application

use std::fmt;
use std::error::Error;
use thiserror::Error;

/// Application error type
#[derive(Debug, Error)]
pub enum AppError {
    /// Bluetooth error
    #[error("Bluetooth error: {0}")]
    Bluetooth(#[from] crate::bluetooth::BleError),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(#[from] crate::config::ConfigError),
    
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// UI error
    #[error("UI error: {0}")]
    Ui(String),
    
    /// State error
    #[error("State error: {0}")]
    State(String),
    
    /// Initialization error
    #[error("Initialization error: {0}")]
    Init(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// Other error
    #[error("{0}")]
    Other(String),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, AppError>;

impl From<String> for AppError {
    fn from(s: String) -> Self {
        Self::Other(s)
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        Self::Other(s.to_string())
    }
}

/// UI error type
#[derive(Debug)]
pub enum UiError {
    /// Window creation error
    WindowCreation(String),
    
    /// Rendering error
    Rendering(String),
    
    /// Event handling error
    EventHandling(String),
    
    /// System tray error
    SystemTray(String),
}

impl fmt::Display for UiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WindowCreation(msg) => write!(f, "Window creation error: {}", msg),
            Self::Rendering(msg) => write!(f, "Rendering error: {}", msg),
            Self::EventHandling(msg) => write!(f, "Event handling error: {}", msg),
            Self::SystemTray(msg) => write!(f, "System tray error: {}", msg),
        }
    }
}

impl Error for UiError {}

impl From<UiError> for AppError {
    fn from(err: UiError) -> Self {
        Self::Ui(err.to_string())
    }
}

/// State error type
#[derive(Debug)]
pub enum StateError {
    /// Invalid state transition
    InvalidTransition(String),
    
    /// Missing state
    MissingState(String),
    
    /// State lock error
    LockError,
    
    /// Persistence error
    Persistence(String),
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTransition(msg) => write!(f, "Invalid state transition: {}", msg),
            Self::MissingState(msg) => write!(f, "Missing state: {}", msg),
            Self::LockError => write!(f, "Failed to lock state"),
            Self::Persistence(msg) => write!(f, "State persistence error: {}", msg),
        }
    }
}

impl Error for StateError {}

impl From<StateError> for AppError {
    fn from(err: StateError) -> Self {
        Self::State(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_app_error_display() {
        let err = AppError::Init("Failed to initialize".to_string());
        assert_eq!(err.to_string(), "Initialization error: Failed to initialize");
    }
    
    #[test]
    fn test_ui_error_display() {
        let err = UiError::WindowCreation("Failed to create window".to_string());
        assert_eq!(err.to_string(), "Window creation error: Failed to create window");
    }
    
    #[test]
    fn test_state_error_display() {
        let err = StateError::InvalidTransition("Cannot transition from A to B".to_string());
        assert_eq!(err.to_string(), "Invalid state transition: Cannot transition from A to B");
    }
} 