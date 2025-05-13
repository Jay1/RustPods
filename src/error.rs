//! Error management for the RustPods application

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use thiserror::Error;
use crate::bluetooth::BleError;

/// Maximum number of errors to keep in history
const MAX_ERROR_HISTORY: usize = 100;

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorSeverity {
    /// Critical errors that prevent core functionality
    Critical,
    /// Major errors that significantly impact functionality
    Major,
    /// Minor errors with limited impact
    Minor,
    /// Warnings that don't impact functionality
    Warning,
    /// General error level
    Error,
    /// Recoverable errors
    Recoverable,
}

/// Main error type for the application
#[derive(Debug, Clone, Error)]
pub enum RustPodsError {
    /// Bluetooth related errors
    #[error("Bluetooth error: {0}")]
    Bluetooth(String),
    
    /// Configuration related errors
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// UI related errors
    #[error("UI error: {0}")]
    Ui(String),
    
    /// System related errors
    #[error("System error: {0}")]
    System(String),
    
    /// State management errors
    #[error("State error: {0}")]
    State(String),
    
    /// Device related errors
    #[error("Device error: {0}")]
    Device(String),
    
    /// General errors
    #[error("{0}")]
    General(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    /// UI error without message
    #[error("UI error occurred")]
    UiError,
    
    /// Bluetooth API error
    #[error("Bluetooth API error: {0}")]
    BluetoothApiError(BleError),
    
    /// Device not found error
    #[error("Device not found")]
    DeviceNotFound,
    
    /// Application error
    #[error("Application error: {0}")]
    Application(String),
    
    /// AirPods specific error
    #[error("AirPods error: {0}")]
    AirPods(String),
    
    /// Battery monitor error
    #[error("Battery monitoring error: {0}")]
    BatteryMonitor(String),
    
    /// Battery monitor error
    #[error("Battery monitoring error: {0}")]
    BatteryMonitorError(String),
    
    /// State persistence error
    #[error("State persistence error: {0}")]
    StatePersistence(String),
    
    /// Lifecycle management error
    #[error("Lifecycle error: {0}")]
    Lifecycle(String),
}

/// Recovery action to take for an error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryAction {
    /// No recovery action is needed
    None,
    /// Retry the operation
    Retry,
    /// Restart the application component
    Restart,
    /// Reset the configuration
    ResetConfig,
    /// Notify the user
    NotifyUser,
}

/// Error statistics
#[derive(Debug, Clone, Default)]
pub struct ErrorStats {
    /// Total number of errors
    pub total: usize,
    /// Errors by severity
    pub by_severity: HashMap<ErrorSeverity, usize>,
    /// Errors by type
    pub by_type: HashMap<String, usize>,
    /// First error timestamp
    pub first_error: Option<DateTime<Utc>>,
    /// Last error timestamp
    pub last_error: Option<DateTime<Utc>>,
    /// Total errors count
    pub total_errors: usize,
    /// Bluetooth specific errors
    pub bluetooth_errors: usize,
    /// AirPods specific errors
    pub airpods_errors: usize,
    /// UI related errors
    pub ui_errors: usize,
    /// Configuration errors
    pub config_errors: usize,
    /// Application errors
    pub app_errors: usize,
    /// Device errors
    pub device_errors: usize,
    /// Battery errors
    pub battery_errors: usize,
    /// System errors
    pub system_errors: usize,
    /// Persistence errors
    pub persistence_errors: usize,
    /// Lifecycle errors
    pub lifecycle_errors: usize,
    /// Critical errors
    pub critical_errors: usize,
    /// Error level errors
    pub error_level_errors: usize,
    /// Recoverable errors
    pub recoverable_errors: usize,
    /// Warnings
    pub warnings: usize,
}

/// Error record for the history
#[derive(Debug, Clone)]
struct ErrorRecord {
    /// The error that occurred
    error: RustPodsError,
    /// Severity of the error
    severity: ErrorSeverity,
    /// Timestamp when the error occurred
    timestamp: DateTime<Utc>,
    /// Component where the error occurred
    component: String,
    /// Recovery action to take
    recovery_action: RecoveryAction,
}

/// Error manager for tracking and reporting errors
pub struct ErrorManager {
    /// Error history
    history: Vec<RustPodsError>,
    /// Error statistics
    stats: ErrorStats,
}

impl ErrorManager {
    /// Create a new ErrorManager
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            stats: ErrorStats::default(),
        }
    }
    
    /// Add an error to the history
    pub fn add_to_history(&mut self, error: RustPodsError) {
        // Update statistics
        self.stats.total += 1;
        
        // Update by severity
        let count = self.stats.by_severity
            .entry(error.severity())
            .or_insert(0);
        *count += 1;
        
        // Update by type
        let error_type = match &error {
            RustPodsError::Bluetooth(_) => "bluetooth",
            RustPodsError::BluetoothApiError(_) => "bluetooth",
            RustPodsError::Config(_) => "config",
            RustPodsError::ConfigError(_) => "config",
            RustPodsError::Ui(_) => "ui",
            RustPodsError::UiError => "ui",
            RustPodsError::System(_) => "system",
            RustPodsError::State(_) => "state",
            RustPodsError::Device(_) => "device",
            RustPodsError::DeviceNotFound => "device",
            RustPodsError::General(_) => "general",
            RustPodsError::Application(_) => "application",
            RustPodsError::AirPods(_) => "airpods",
            RustPodsError::BatteryMonitor(_) => "battery",
            RustPodsError::BatteryMonitorError(_) => "battery",
            RustPodsError::StatePersistence(_) => "state",
            RustPodsError::Lifecycle(_) => "lifecycle",
        };
        
        let count = self.stats.by_type.entry(error_type.to_string()).or_insert(0);
        *count += 1;
        
        // Update timestamps
        if self.stats.first_error.is_none() {
            self.stats.first_error = Some(Utc::now());
        }
        self.stats.last_error = Some(Utc::now());
        
        // Add to history (limiting size to prevent memory bloat)
        const MAX_HISTORY_SIZE: usize = 100;
        if self.history.len() >= MAX_HISTORY_SIZE {
            self.history.remove(0); // Remove oldest error
        }
        self.history.push(error);
    }
    
    /// Record an error
    pub fn record_error(&mut self, error: &RustPodsError) {
        self.add_to_history(error.clone());
    }
    
    /// Get error history
    pub fn get_error_history(&self) -> &Vec<RustPodsError> {
        &self.history
    }
    
    /// Get error statistics
    pub fn get_stats(&self) -> ErrorStats {
        self.stats.clone()
    }
    
    /// Clear error history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
    
    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = ErrorStats::default();
    }
}

impl Default for ErrorManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RustPodsError {
    /// Get the severity level of the error
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            RustPodsError::Bluetooth(_) => ErrorSeverity::Major,
            RustPodsError::Config(_) => ErrorSeverity::Minor,
            RustPodsError::Ui(_) => ErrorSeverity::Minor,
            RustPodsError::System(_) => ErrorSeverity::Critical,
            RustPodsError::State(_) => ErrorSeverity::Major,
            RustPodsError::Device(_) => ErrorSeverity::Major,
            RustPodsError::General(_) => ErrorSeverity::Minor,
            RustPodsError::ConfigError(_) => ErrorSeverity::Minor,
            RustPodsError::UiError => ErrorSeverity::Minor,
            RustPodsError::BluetoothApiError(_) => ErrorSeverity::Major,
            RustPodsError::DeviceNotFound => ErrorSeverity::Minor,
            RustPodsError::Application(_) => ErrorSeverity::Major,
            RustPodsError::AirPods(_) => ErrorSeverity::Minor,
            RustPodsError::BatteryMonitor(_) => ErrorSeverity::Minor,
            RustPodsError::BatteryMonitorError(_) => ErrorSeverity::Minor,
            RustPodsError::StatePersistence(_) => ErrorSeverity::Major,
            RustPodsError::Lifecycle(_) => ErrorSeverity::Critical,
        }
    }
    
    /// Create a UI error
    pub fn ui(message: impl Into<String>, severity: ErrorSeverity) -> Self {
        RustPodsError::Ui(message.into())
    }
    
    /// Create a system error
    pub fn system(message: impl Into<String>, severity: ErrorSeverity) -> Self {
        RustPodsError::System(message.into())
    }
    
    /// Create an application error
    pub fn application(message: impl Into<String>) -> Self {
        RustPodsError::Application(message.into())
    }
}

// Add From implementation for BleError
impl From<BleError> for RustPodsError {
    fn from(error: BleError) -> Self {
        match error {
            BleError::AdapterNotFound => 
                RustPodsError::Bluetooth("Bluetooth adapter not found".to_string()),
            BleError::ScanInProgress => 
                RustPodsError::Bluetooth("Bluetooth scan already in progress".to_string()),
            BleError::ScanNotStarted => 
                RustPodsError::Bluetooth("Bluetooth scan not started".to_string()),
            BleError::AdapterNotInitialized => 
                RustPodsError::Bluetooth("Bluetooth adapter not initialized".to_string()),
            BleError::DeviceNotFound => 
                RustPodsError::DeviceNotFound,
            BleError::InvalidData => 
                RustPodsError::Bluetooth("Invalid Bluetooth data received".to_string()),
            BleError::Timeout => 
                RustPodsError::Bluetooth("Bluetooth operation timed out".to_string()),
            BleError::BtlePlugError(msg) => 
                RustPodsError::Bluetooth(format!("Bluetooth API error: {}", msg)),
            BleError::Other(msg) => 
                RustPodsError::Bluetooth(msg),
        }
    }
} 