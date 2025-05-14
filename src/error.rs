//! Error management for the RustPods application
//!
//! This module defines the core error types and utilities for the application,
//! providing a consistent error handling pattern throughout the codebase.
//! It uses thiserror for defining error types and provides utilities for
//! tracking, reporting, and recovering from errors.

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use thiserror::Error;
use crate::bluetooth::BleError;
use std::path::PathBuf;
use std::io;
use std::fmt;
use std::sync::PoisonError;
use std::num::ParseIntError;
use btleplug::Error as BtlePlugError;
use std::sync::Mutex;
use std::time::Duration;

/// Maximum number of errors to keep in history
const MAX_ERROR_HISTORY: usize = 100;

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorSeverity {
    /// Critical error that requires immediate attention
    Critical,
    
    /// Major error that significantly impacts functionality
    Major,
    
    /// Standard error
    Error,
    
    /// Minor error that doesn't significantly impact functionality
    Minor,
    
    /// Warning
    Warning,
    
    /// Informational message
    Info,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Critical => write!(f, "CRITICAL"),
            ErrorSeverity::Major => write!(f, "MAJOR"),
            ErrorSeverity::Minor => write!(f, "MINOR"),
            ErrorSeverity::Warning => write!(f, "WARNING"),
            ErrorSeverity::Error => write!(f, "ERROR"),
            ErrorSeverity::Info => write!(f, "INFO"),
        }
    }
}

/// Main error type for the application
#[derive(Debug, Error)]
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
    
    /// System/OS related errors
    #[error("System error: {0}")]
    System(String),
    
    /// State/data related errors
    #[error("State error: {0}")]
    State(String),
    
    /// Device related errors
    #[error("Device error: {0}")]
    Device(String),
    
    /// General application errors
    #[error("Application error: {0}")]
    General(String),
    
    /// Configuration error with debug info
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    /// UI error with no message
    #[error("UI error")]
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
    
    /// AirPods related errors
    #[error("AirPods error: {0}")]
    AirPods(#[from] AirPodsError),
    
    /// Battery monitoring errors
    #[error("Battery monitoring error: {0}")]
    BatteryMonitor(String),
    
    /// Battery monitoring error
    #[error("Battery monitoring error: {0}")]
    BatteryMonitorError(String),
    
    /// State persistence error
    #[error("State persistence error: {0}")]
    StatePersistence(String),
    
    /// Lifecycle error
    #[error("Lifecycle error: {0}")]
    Lifecycle(String),
    
    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    ParseError(String),
    
    /// File I/O error
    #[error("File I/O error: {0}")]
    IoError(String),
    
    /// Path error
    #[error("Path error: {0}")]
    Path(String),
    
    /// File not found error
    #[error("File not found: {0}")]
    FileNotFound(std::path::PathBuf),
    
    /// Permission denied error
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),
    
    /// Parse error
    #[error("Parse error: {0}")]
    Parse(String),
    
    /// Timeout error
    #[error("Timeout error: {0}")]
    Timeout(String),
    
    /// Context error
    #[error("{context}: {source}")]
    Context {
        context: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    /// Invalid data error
    #[error("Invalid data: {0}")]
    InvalidData(String),
    
    /// Bluetooth error
    #[error("Bluetooth error: {0}")]
    BluetoothError(#[from] BluetoothError),
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
    /// Reconnect to Bluetooth device
    ReconnectBluetooth,
    /// Reload configuration
    ReloadConfig,
    /// Clear cache
    ClearCache,
    /// Prompt user for action
    PromptUser,
    /// Select a different Bluetooth adapter
    SelectDifferentAdapter,
    /// Restart the entire application
    RestartApplication,
    /// Custom action with description
    Custom(String),
}

impl fmt::Display for RecoveryAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecoveryAction::None => write!(f, "None"),
            RecoveryAction::Retry => write!(f, "Retry"),
            RecoveryAction::Restart => write!(f, "Restart"),
            RecoveryAction::ResetConfig => write!(f, "Reset Config"),
            RecoveryAction::NotifyUser => write!(f, "Notify User"),
            RecoveryAction::ReconnectBluetooth => write!(f, "Reconnect Bluetooth"),
            RecoveryAction::ReloadConfig => write!(f, "Reload Config"),
            RecoveryAction::ClearCache => write!(f, "Clear Cache"),
            RecoveryAction::PromptUser => write!(f, "Prompt User"),
            RecoveryAction::SelectDifferentAdapter => write!(f, "Select Different Adapter"),
            RecoveryAction::RestartApplication => write!(f, "Restart Application"),
            RecoveryAction::Custom(desc) => write!(f, "Custom: {}", desc),
        }
    }
}

impl RecoveryAction {
    /// Get a user-friendly description of the recovery action
    pub fn description(&self) -> &'static str {
        match self {
            RecoveryAction::None => "No action needed",
            RecoveryAction::Retry => "Retry the operation",
            RecoveryAction::Restart => "Restart component",
            RecoveryAction::ResetConfig => "Reset configuration",
            RecoveryAction::NotifyUser => "Notify user",
            RecoveryAction::ReconnectBluetooth => "Reconnect Bluetooth",
            RecoveryAction::ReloadConfig => "Reload configuration",
            RecoveryAction::ClearCache => "Clear cache",
            RecoveryAction::PromptUser => "Prompt user for action",
            RecoveryAction::SelectDifferentAdapter => "Select a different Bluetooth adapter",
            RecoveryAction::RestartApplication => "Restart the application",
            RecoveryAction::Custom(_) => "Custom action",
        }
    }
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

/// Error context to enrich error information
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Component where the error occurred
    pub component: String,
    /// Operation being performed when error occurred
    pub operation: String,
    /// Additional metadata about the error
    pub metadata: HashMap<String, String>,
    /// Time when the error occurred
    pub timestamp: DateTime<Utc>,
    /// User-friendly message
    pub user_message: Option<String>,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(component: impl Into<String>, operation: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            operation: operation.into(),
            metadata: HashMap::new(),
            timestamp: Utc::now(),
            user_message: None,
        }
    }
    
    /// Add metadata to the context
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Add a user-friendly message
    pub fn with_user_message(mut self, message: impl Into<String>) -> Self {
        self.user_message = Some(message.into());
        self
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}:{}] ", self.component, self.operation)?;
        if !self.metadata.is_empty() {
            write!(f, "(")?;
            let mut first = true;
            for (key, value) in &self.metadata {
                if !first {
                    write!(f, ", ")?;
                }
                write!(f, "{}={}", key, value)?;
                first = false;
            }
            write!(f, ") ")?;
        }
        Ok(())
    }
}

/// Entry in the error history
#[derive(Debug)]
pub struct ErrorEntry {
    /// The error type as a string
    pub error_type: String,
    /// The error message
    pub error_message: String,
    /// When the error occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Context information about the error
    pub context: Option<ErrorContext>,
    /// Recovery action attempted
    pub recovery: Option<RecoveryAction>,
}

/// Error manager for tracking and reporting errors
#[derive(Debug)]
pub struct ErrorManager {
    /// Error history
    history: Vec<ErrorEntry>,
    /// Error statistics
    stats: ErrorStats,
    /// Detailed error records with context
    detailed_history: Vec<ErrorRecord>,
}

/// Error record for the history
#[derive(Debug)]
struct ErrorRecord {
    /// Error type as string
    error_type: String,
    /// Error message
    error_message: String,
    /// Severity of the error
    severity: ErrorSeverity,
    /// Timestamp when the error occurred
    timestamp: DateTime<Utc>,
    /// Component where the error occurred
    component: String,
    /// Recovery action to take
    recovery_action: RecoveryAction,
    /// Context information
    context: Option<ErrorContext>,
}

impl ErrorManager {
    /// Create a new ErrorManager
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            stats: ErrorStats::default(),
            detailed_history: Vec::new(),
        }
    }
    
    /// Add an error to the history
    pub fn add_to_history(&mut self, error: &RustPodsError) {
        // Get the current time
        let now = chrono::Utc::now();
        
        // Update the statistics
        self.stats.total += 1;
        
        // Get the error type as a string
        let error_type = match error {
            RustPodsError::Bluetooth(_) => "bluetooth",
            RustPodsError::BluetoothApiError(_) => "bluetooth_api",
            RustPodsError::AirPods(_) => "airpods",
            RustPodsError::Ui(_) => "ui",
            RustPodsError::UiError => "ui",
            RustPodsError::Config(_) => "config",
            RustPodsError::ConfigError(_) => "config",
            RustPodsError::Application(_) => "application",
            RustPodsError::DeviceNotFound => "device",
            RustPodsError::Device(_) => "device",
            RustPodsError::BatteryMonitor(_) => "battery",
            RustPodsError::BatteryMonitorError(_) => "battery",
            RustPodsError::State(_) => "state",
            RustPodsError::StatePersistence(_) => "state",
            RustPodsError::Lifecycle(_) => "lifecycle",
            RustPodsError::System(_) => "system",
            RustPodsError::General(_) => "general",
            RustPodsError::IoError(_) => "file_io",
            RustPodsError::ParseError(_) => "json",
            RustPodsError::Path(_) => "path",
            RustPodsError::FileNotFound(_) => "file_not_found",
            RustPodsError::PermissionDenied(_) => "permission_denied",
            RustPodsError::Validation(_) => "validation",
            RustPodsError::Parse(_) => "parse",
            RustPodsError::Timeout(_) => "timeout",
            RustPodsError::Context { .. } => "context",
            RustPodsError::InvalidData(_) => "invalid_data",
            RustPodsError::BluetoothError(_) => "bluetooth_error",
        };
        
        // Update type counts
        *self.stats.by_type.entry(error_type.to_string()).or_insert(0) += 1;
        
        // Update severity counts
        let severity = error.severity();
        *self.stats.by_severity.entry(severity).or_insert(0) += 1;
        
        // Update first and last error timestamps
        if self.stats.first_error.is_none() {
            self.stats.first_error = Some(now);
        }
        self.stats.last_error = Some(now);
        
        // Create error entry
        let entry = ErrorEntry {
            error_type: error_type.to_string(),
            error_message: error.to_string(),
            timestamp: now,
            context: None,
            recovery: Some(error.recovery_action()),
        };
        
        // Add to history, keeping the max size
        self.history.push(entry);
        if self.history.len() > MAX_ERROR_HISTORY {
            self.history.remove(0);
        }
    }
    
    /// Record an error with context
    pub fn record_error_with_context(&mut self, error: RustPodsError, context: ErrorContext, recovery_action: RecoveryAction) {
        // Get the current time
        let now = chrono::Utc::now();
        
        // Update the statistics
        self.stats.total += 1;
        
        // Get the error type as a string
        let error_type = match &error {
            RustPodsError::Bluetooth(_) => "bluetooth",
            RustPodsError::BluetoothApiError(_) => "bluetooth_api",
            RustPodsError::AirPods(_) => "airpods",
            RustPodsError::Ui(_) => "ui",
            RustPodsError::UiError => "ui",
            RustPodsError::Config(_) => "config",
            RustPodsError::ConfigError(_) => "config",
            RustPodsError::Application(_) => "application",
            RustPodsError::DeviceNotFound => "device",
            RustPodsError::Device(_) => "device",
            RustPodsError::BatteryMonitor(_) => "battery",
            RustPodsError::BatteryMonitorError(_) => "battery",
            RustPodsError::State(_) => "state",
            RustPodsError::StatePersistence(_) => "state",
            RustPodsError::Lifecycle(_) => "lifecycle",
            RustPodsError::System(_) => "system",
            RustPodsError::General(_) => "general",
            RustPodsError::IoError(_) => "file_io",
            RustPodsError::ParseError(_) => "json",
            RustPodsError::Path(_) => "path",
            RustPodsError::FileNotFound(_) => "file_not_found",
            RustPodsError::PermissionDenied(_) => "permission_denied",
            RustPodsError::Validation(_) => "validation",
            RustPodsError::Parse(_) => "parse",
            RustPodsError::Timeout(_) => "timeout",
            RustPodsError::Context { .. } => "context",
            RustPodsError::InvalidData(_) => "invalid_data",
            RustPodsError::BluetoothError(_) => "bluetooth_error",
        };
        
        // Update type counts
        *self.stats.by_type.entry(error_type.to_string()).or_insert(0) += 1;
        
        // Update severity counts
        let severity = error.severity();
        *self.stats.by_severity.entry(severity).or_insert(0) += 1;
        
        // Update first and last error timestamps
        if self.stats.first_error.is_none() {
            self.stats.first_error = Some(now);
        }
        self.stats.last_error = Some(now);
        
        // Create error entry
        let entry = ErrorEntry {
            error_type: error_type.to_string(),
            error_message: error.to_string(),
            timestamp: now,
            context: Some(context.clone()),
            recovery: Some(recovery_action.clone()),
        };
        
        // Create detailed record
        let record = ErrorRecord {
            error_type: error_type.to_string(),
            error_message: error.to_string(),
            severity,
            timestamp: now,
            component: context.component.clone(),
            recovery_action,
            context: Some(context),
        };
        
        // Add to history, keeping the max size
        self.history.push(entry);
        if self.history.len() > MAX_ERROR_HISTORY {
            self.history.remove(0);
        }
        
        // Add to detailed history
        self.detailed_history.push(record);
        if self.detailed_history.len() > MAX_ERROR_HISTORY {
            self.detailed_history.remove(0);
        }
        
        // Log the error
        log::error!("{}", error);
    }
    
    /// Record an error
    pub fn record_error(&mut self, error: &RustPodsError) {
        self.add_to_history(error);
    }
    
    /// Get error history
    pub fn get_error_history(&self) -> &Vec<ErrorEntry> {
        &self.history
    }
    
    /// Get error statistics
    pub fn get_stats(&self) -> ErrorStats {
        self.stats.clone()
    }
    
    /// Clear error history
    pub fn clear_history(&mut self) {
        self.history.clear();
        self.detailed_history.clear();
    }
    
    /// Reset error statistics
    pub fn reset_stats(&mut self) {
        self.stats = ErrorStats::default();
    }
    
    /// Get detailed error history
    pub fn get_detailed_history(&self) -> &Vec<ErrorRecord> {
        &self.detailed_history
    }
    
    /// Get the most recent error
    pub fn get_latest_error(&self) -> Option<String> {
        self.history.last().map(|entry| entry.error_message.clone())
    }
    
    /// Get the most recent detailed error record
    pub fn get_latest_detailed_error(&self) -> Option<&ErrorRecord> {
        self.detailed_history.last()
    }
}

impl Default for ErrorManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Result type alias for application results
pub type Result<T> = std::result::Result<T, RustPodsError>;

impl RustPodsError {
    /// Get a user-friendly message for this error
    pub fn user_message(&self) -> String {
        match self {
            RustPodsError::Bluetooth(msg) => format!("Bluetooth issue: {}", msg),
            RustPodsError::BluetoothApiError(_) => "Bluetooth API issue. Please check your Bluetooth adapter.".to_string(),
            RustPodsError::AirPods(msg) => format!("AirPods issue: {}", msg),
            RustPodsError::Ui(msg) => format!("User interface issue: {}", msg),
            RustPodsError::UiError => "User interface issue".to_string(),
            RustPodsError::Config(msg) => format!("Configuration issue: {}", msg),
            RustPodsError::ConfigError(msg) => format!("Configuration issue: {}", msg),
            RustPodsError::Application(msg) => format!("Application error: {}", msg),
            RustPodsError::DeviceNotFound => "Device not found".to_string(),
            RustPodsError::Device(msg) => format!("Device issue: {}", msg),
            RustPodsError::BatteryMonitor(msg) => format!("Battery monitoring issue: {}", msg),
            RustPodsError::BatteryMonitorError(msg) => format!("Battery monitoring issue: {}", msg),
            RustPodsError::State(msg) => format!("State management issue: {}", msg),
            RustPodsError::StatePersistence(msg) => format!("State persistence issue: {}", msg),
            RustPodsError::Lifecycle(msg) => format!("Lifecycle issue: {}", msg),
            RustPodsError::System(msg) => format!("System issue: {}", msg),
            RustPodsError::General(msg) => msg.clone(),
            RustPodsError::IoError(e) => format!("File access issue: {}", e),
            RustPodsError::ParseError(e) => format!("Data format issue: {}", e),
            RustPodsError::Path(msg) => format!("File path issue: {}", msg),
            RustPodsError::FileNotFound(path) => format!("File not found: {}", path.display()),
            RustPodsError::PermissionDenied(msg) => format!("Permission denied: {}", msg),
            RustPodsError::Validation(msg) => format!("Validation issue: {}", msg),
            RustPodsError::Parse(msg) => format!("Parsing issue: {}", msg),
            RustPodsError::Timeout(msg) => format!("Operation timed out: {}", msg),
            RustPodsError::Context { context, source } => format!("{}: {}", context, source),
            RustPodsError::InvalidData(msg) => format!("Invalid data: {}", msg),
            RustPodsError::BluetoothError(_) => format!("Bluetooth error: {}", self),
        }
    }

    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            RustPodsError::Bluetooth(_) => true,
            RustPodsError::BluetoothApiError(_) => true,
            RustPodsError::AirPods(_) => true,
            RustPodsError::Ui(_) => true,
            RustPodsError::UiError => true,
            RustPodsError::Config(_) => true,
            RustPodsError::ConfigError(_) => true,
            RustPodsError::Application(_) => false,
            RustPodsError::DeviceNotFound => true,
            RustPodsError::Device(_) => true,
            RustPodsError::BatteryMonitor(_) => true,
            RustPodsError::BatteryMonitorError(_) => true,
            RustPodsError::State(_) => false,
            RustPodsError::StatePersistence(_) => false,
            RustPodsError::Lifecycle(_) => false,
            RustPodsError::System(_) => false,
            RustPodsError::General(_) => false,
            RustPodsError::IoError(_) => true,
            RustPodsError::ParseError(_) => false,
            RustPodsError::Path(_) => false,
            RustPodsError::FileNotFound(_) => true,
            RustPodsError::PermissionDenied(_) => false,
            RustPodsError::Validation(_) => true,
            RustPodsError::Parse(_) => false,
            RustPodsError::Timeout(_) => true,
            RustPodsError::Context { .. } => false,
            RustPodsError::InvalidData(_) => false,
            RustPodsError::BluetoothError(_) => true,
        }
    }
    
    /// Get the recommended recovery action for this error
    pub fn recovery_action(&self) -> RecoveryAction {
        match self {
            RustPodsError::Bluetooth(_) => RecoveryAction::ReconnectBluetooth,
            RustPodsError::BluetoothApiError(_) => RecoveryAction::ReconnectBluetooth,
            RustPodsError::AirPods(_) => RecoveryAction::ReconnectBluetooth,
            RustPodsError::Ui(_) => RecoveryAction::Restart,
            RustPodsError::UiError => RecoveryAction::Restart,
            RustPodsError::Config(_) => RecoveryAction::ReloadConfig,
            RustPodsError::ConfigError(_) => RecoveryAction::ReloadConfig,
            RustPodsError::Application(_) => RecoveryAction::Restart,
            RustPodsError::DeviceNotFound => RecoveryAction::ReconnectBluetooth,
            RustPodsError::Device(_) => RecoveryAction::ReconnectBluetooth,
            RustPodsError::BatteryMonitor(_) => RecoveryAction::Retry,
            RustPodsError::BatteryMonitorError(_) => RecoveryAction::Retry,
            RustPodsError::State(_) => RecoveryAction::Restart,
            RustPodsError::StatePersistence(_) => RecoveryAction::ResetConfig,
            RustPodsError::Lifecycle(_) => RecoveryAction::Restart,
            RustPodsError::System(_) => RecoveryAction::Restart,
            RustPodsError::General(_) => RecoveryAction::NotifyUser,
            RustPodsError::IoError(_) => RecoveryAction::Retry,
            RustPodsError::ParseError(_) => RecoveryAction::ResetConfig,
            RustPodsError::Path(_) => RecoveryAction::NotifyUser,
            RustPodsError::FileNotFound(_) => RecoveryAction::NotifyUser,
            RustPodsError::PermissionDenied(_) => RecoveryAction::PromptUser,
            RustPodsError::Validation(_) => RecoveryAction::NotifyUser,
            RustPodsError::Parse(_) => RecoveryAction::NotifyUser,
            RustPodsError::Timeout(_) => RecoveryAction::Retry,
            RustPodsError::Context { .. } => RecoveryAction::NotifyUser,
            RustPodsError::InvalidData(_) => RecoveryAction::NotifyUser,
            RustPodsError::BluetoothError(_) => RecoveryAction::ReconnectBluetooth,
        }
    }

    /// Get the severity level for this error
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            RustPodsError::Bluetooth(_) => ErrorSeverity::Major,
            RustPodsError::BluetoothApiError(_) => ErrorSeverity::Major,
            RustPodsError::AirPods(_) => ErrorSeverity::Major,
            RustPodsError::Ui(_) => ErrorSeverity::Minor,
            RustPodsError::UiError => ErrorSeverity::Minor,
            RustPodsError::Config(_) => ErrorSeverity::Major,
            RustPodsError::ConfigError(_) => ErrorSeverity::Major,
            RustPodsError::Application(_) => ErrorSeverity::Critical,
            RustPodsError::DeviceNotFound => ErrorSeverity::Major,
            RustPodsError::Device(_) => ErrorSeverity::Major,
            RustPodsError::BatteryMonitor(_) => ErrorSeverity::Minor,
            RustPodsError::BatteryMonitorError(_) => ErrorSeverity::Minor,
            RustPodsError::State(_) => ErrorSeverity::Major,
            RustPodsError::StatePersistence(_) => ErrorSeverity::Major,
            RustPodsError::Lifecycle(_) => ErrorSeverity::Critical,
            RustPodsError::System(_) => ErrorSeverity::Critical,
            RustPodsError::General(_) => ErrorSeverity::Error,
            RustPodsError::IoError(_) => ErrorSeverity::Major,
            RustPodsError::ParseError(_) => ErrorSeverity::Major,
            RustPodsError::Path(_) => ErrorSeverity::Major,
            RustPodsError::FileNotFound(_) => ErrorSeverity::Major,
            RustPodsError::PermissionDenied(_) => ErrorSeverity::Critical,
            RustPodsError::Validation(_) => ErrorSeverity::Error,
            RustPodsError::Parse(_) => ErrorSeverity::Error,
            RustPodsError::Timeout(_) => ErrorSeverity::Major,
            RustPodsError::Context { .. } => ErrorSeverity::Error,
            RustPodsError::InvalidData(_) => ErrorSeverity::Major,
            RustPodsError::BluetoothError(_) => ErrorSeverity::Major,
        }
    }

    /// Get the category of the error
    pub fn get_category(&self) -> &'static str {
        match self {
            RustPodsError::Bluetooth(_) => "bluetooth",
            RustPodsError::BluetoothApiError(_) => "bluetooth_api",
            RustPodsError::BluetoothError(_) => "bluetooth_error",
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
            RustPodsError::IoError(_) => "file_io",
            RustPodsError::ParseError(_) => "json",
            RustPodsError::Path(_) => "path",
            RustPodsError::FileNotFound(_) => "file_not_found",
            RustPodsError::PermissionDenied(_) => "permission",
            RustPodsError::Validation(_) => "validation",
            RustPodsError::Parse(_) => "parse",
            RustPodsError::Timeout(_) => "timeout",
            RustPodsError::Context { .. } => "context",
            RustPodsError::InvalidData(_) => "invalid_data",
        }
    }
    
    /// Get the specific type of the error
    pub fn get_type(&self) -> &'static str {
        match self {
            RustPodsError::Bluetooth(_) => "bluetooth_generic",
            RustPodsError::BluetoothApiError(_) => "bluetooth_api",
            RustPodsError::BluetoothError(_) => "bluetooth_error",
            RustPodsError::Config(_) => "config_generic",
            RustPodsError::ConfigError(_) => "config_error",
            RustPodsError::Ui(_) => "ui_generic",
            RustPodsError::UiError => "ui_error",
            RustPodsError::System(_) => "system_generic",
            RustPodsError::State(_) => "state_generic",
            RustPodsError::Device(_) => "device_generic",
            RustPodsError::DeviceNotFound => "device_not_found",
            RustPodsError::General(_) => "general",
            RustPodsError::Application(_) => "application",
            RustPodsError::AirPods(_) => "airpods",
            RustPodsError::BatteryMonitor(_) => "battery_monitor",
            RustPodsError::BatteryMonitorError(_) => "battery_error",
            RustPodsError::StatePersistence(_) => "state",
            RustPodsError::Lifecycle(_) => "lifecycle",
            RustPodsError::IoError(_) => "file_io",
            RustPodsError::ParseError(_) => "json",
            RustPodsError::Path(_) => "path",
            RustPodsError::FileNotFound(_) => "file_not_found",
            RustPodsError::PermissionDenied(_) => "permission",
            RustPodsError::Validation(_) => "validation",
            RustPodsError::Parse(_) => "parse",
            RustPodsError::Timeout(_) => "timeout",
            RustPodsError::Context { .. } => "context",
            RustPodsError::InvalidData(_) => "invalid_data",
        }
    }
    
    /// Create a UI error with a message and severity
    pub fn ui(message: impl Into<String>, severity: ErrorSeverity) -> Self {
        Self::Ui(message.into())
    }
    
    /// Create a system error with a message and severity
    pub fn system(message: impl Into<String>, severity: ErrorSeverity) -> Self {
        Self::System(message.into())
    }
    
    /// Create an application error with a message
    pub fn application(message: impl Into<String>) -> Self {
        Self::Application(message.into())
    }
    
    /// Add context to an error
    pub fn with_context(error: impl Into<Box<dyn std::error::Error + Send + Sync + 'static>>, context: impl Into<String>) -> Self {
        Self::Context {
            context: context.into(),
            source: error.into(),
        }
    }
}

/// Bluetooth-specific error type
#[derive(Debug)]
pub enum BluetoothError {
    /// Connection to device failed
    ConnectionFailed(String),
    
    /// Device not found
    DeviceNotFound(String),
    
    /// Scan operation failed
    ScanFailed(String),
    
    /// Device disconnected unexpectedly
    DeviceDisconnected(String),
    
    /// No suitable adapter found
    NoAdapter,
    
    /// Permission error
    PermissionDenied(String),
    
    /// Invalid data received from device
    InvalidData(String),
    
    /// Operation timed out
    Timeout(Duration),
    
    /// Raw btleplug API error
    ApiError(String),
    
    /// Failed to refresh the Bluetooth adapter
    AdapterRefreshFailed {
        /// The error that occurred
        error: String,
        /// Recommended recovery action
        recovery: RecoveryAction,
        /// Number of retries attempted
        retries: u32,
    },
    
    /// Bluetooth adapter not available
    AdapterNotAvailable {
        /// Reason adapter is not available
        reason: String,
        /// Recommended recovery action
        recovery: RecoveryAction,
    },
    
    /// Adapter failed to scan
    AdapterScanFailed {
        /// The error that occurred
        error: String,
        /// Recommended recovery action
        recovery: RecoveryAction, 
    },
    
    /// Other error
    Other(String),
}

impl std::fmt::Display for BluetoothError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BluetoothError::ConnectionFailed(s) => write!(f, "Connection failed: {}", s),
            BluetoothError::DeviceNotFound(s) => write!(f, "Device not found: {}", s),
            BluetoothError::ScanFailed(s) => write!(f, "Scan failed: {}", s),
            BluetoothError::DeviceDisconnected(s) => write!(f, "Device disconnected: {}", s),
            BluetoothError::NoAdapter => write!(f, "No suitable Bluetooth adapter found"),
            BluetoothError::PermissionDenied(s) => write!(f, "Bluetooth permission denied: {}", s),
            BluetoothError::InvalidData(s) => write!(f, "Invalid data received: {}", s),
            BluetoothError::ApiError(e) => write!(f, "Bluetooth API error: {}", e),
            BluetoothError::Timeout(d) => write!(f, "Operation timed out after {:?}", d),
            BluetoothError::AdapterRefreshFailed { error, recovery, retries } => write!(f, "Failed to refresh adapter: {} ({} retries attempted)", error, retries),
            BluetoothError::AdapterNotAvailable { reason, recovery } => write!(f, "Adapter not available: {} (recommended recovery: {})", reason, recovery),
            BluetoothError::AdapterScanFailed { error, recovery } => write!(f, "Adapter scan failed: {} (recommended recovery: {})", error, recovery),
            BluetoothError::Other(s) => write!(f, "Bluetooth error: {}", s),
        }
    }
}

impl std::error::Error for BluetoothError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<btleplug::Error> for BluetoothError {
    fn from(error: btleplug::Error) -> Self {
        // Use the generic converter from bluetooth module
        crate::bluetooth::convert_btleplug_error(error, "BluetoothModule", "operation")
    }
}

/// UI-specific error type
#[derive(Debug, Clone, Error)]
pub enum UiError {
    /// Rendering error
    #[error("Rendering error: {0}")]
    RenderingError(String),
    
    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    /// Layout error
    #[error("Layout error: {0}")]
    LayoutError(String),
    
    /// Resource loading error
    #[error("Failed to load resource: {0}")]
    ResourceLoadingError(String),
    
    /// Update error
    #[error("UI update error: {0}")]
    UpdateError(String),
    
    /// Thread error
    #[error("UI thread error: {0}")]
    ThreadError(String),
    
    /// Other error
    #[error("{0}")]
    Other(String),
}

/// Configuration-specific error type
#[derive(Debug, Clone, Error)]
pub enum ConfigError {
    /// Failed to read config file
    #[error("Failed to read config file: {0}")]
    ReadError(String),
    
    /// Failed to write config file
    #[error("Failed to write config file: {0}")]
    WriteError(String),
    
    /// Failed to parse config
    #[error("Failed to parse config: {0}")]
    ParseError(String),
    
    /// IO error
    #[error("IO error: {0}")]
    IoError(String),
    
    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    /// Invalid value for field
    #[error("Invalid value for {0}: {1}")]
    InvalidValue(String, String),
    
    /// Other error
    #[error("{0}")]
    Other(String),
}

impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> Self {
        ConfigError::ParseError(err.to_string())
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::IoError(err.to_string())
    }
}

/// AirPods-specific error type
#[derive(Debug, Error)]
pub enum AirPodsError {
    /// Failed to parse AirPods data
    #[error("Failed to parse AirPods data: {0}")]
    ParseError(String),
    
    /// Invalid data format
    #[error("Invalid data format: {0}")]
    InvalidFormat(String),
    
    /// Invalid data
    #[error("Invalid data: {0}")]
    InvalidData(String),
    
    /// Missing required data
    #[error("Missing required data: {0}")]
    MissingData(String),
    
    /// Device compatibility error
    #[error("Device compatibility error: {0}")]
    DeviceCompatibility(String),
    
    /// Connection error
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    /// Manufacturer data is missing
    #[error("Manufacturer data is missing")]
    ManufacturerDataMissing,
    
    /// Detection failed
    #[error("AirPods detection failed: {0}")]
    DetectionFailed(String),
    
    /// Other error
    #[error("{0}")]
    Other(String),
}

impl Clone for AirPodsError {
    fn clone(&self) -> Self {
        match self {
            AirPodsError::ParseError(s) => AirPodsError::ParseError(s.clone()),
            AirPodsError::InvalidFormat(s) => AirPodsError::InvalidFormat(s.clone()),
            AirPodsError::InvalidData(s) => AirPodsError::InvalidData(s.clone()),
            AirPodsError::MissingData(s) => AirPodsError::MissingData(s.clone()),
            AirPodsError::DeviceCompatibility(s) => AirPodsError::DeviceCompatibility(s.clone()),
            AirPodsError::ConnectionError(s) => AirPodsError::ConnectionError(s.clone()),
            AirPodsError::ManufacturerDataMissing => AirPodsError::ManufacturerDataMissing,
            AirPodsError::DetectionFailed(s) => AirPodsError::DetectionFailed(s.clone()),
            AirPodsError::Other(s) => AirPodsError::Other(s.clone()),
        }
    }
}

impl Clone for BluetoothError {
    fn clone(&self) -> Self {
        match self {
            BluetoothError::ConnectionFailed(s) => BluetoothError::ConnectionFailed(s.clone()),
            BluetoothError::DeviceNotFound(s) => BluetoothError::DeviceNotFound(s.clone()),
            BluetoothError::ScanFailed(s) => BluetoothError::ScanFailed(s.clone()),
            BluetoothError::DeviceDisconnected(s) => BluetoothError::DeviceDisconnected(s.clone()),
            BluetoothError::NoAdapter => BluetoothError::NoAdapter,
            BluetoothError::PermissionDenied(s) => BluetoothError::PermissionDenied(s.clone()),
            BluetoothError::InvalidData(s) => BluetoothError::InvalidData(s.clone()),
            BluetoothError::Timeout(d) => BluetoothError::Timeout(*d),
            BluetoothError::ApiError(s) => BluetoothError::ApiError(s.clone()),
            BluetoothError::AdapterRefreshFailed { error, recovery, retries } => BluetoothError::AdapterRefreshFailed {
                error: error.clone(),
                recovery: recovery.clone(),
                retries: *retries,
            },
            BluetoothError::AdapterNotAvailable { reason, recovery } => BluetoothError::AdapterNotAvailable {
                reason: reason.clone(),
                recovery: recovery.clone(),
            },
            BluetoothError::AdapterScanFailed { error, recovery } => BluetoothError::AdapterScanFailed {
                error: error.clone(),
                recovery: recovery.clone(),
            },
            BluetoothError::Other(s) => BluetoothError::Other(s.clone()),
        }
    }
}

impl Clone for RustPodsError {
    fn clone(&self) -> Self {
        match self {
            Self::Bluetooth(s) => Self::Bluetooth(s.clone()),
            Self::BluetoothApiError(e) => Self::BluetoothApiError(e.clone()),
            Self::Config(s) => Self::Config(s.clone()),
            Self::ConfigError(s) => Self::ConfigError(s.clone()),
            Self::Ui(s) => Self::Ui(s.clone()),
            Self::UiError => Self::UiError,
            Self::System(s) => Self::System(s.clone()),
            Self::State(s) => Self::State(s.clone()),
            Self::Device(s) => Self::Device(s.clone()),
            Self::DeviceNotFound => Self::DeviceNotFound,
            Self::General(s) => Self::General(s.clone()),
            Self::Application(s) => Self::Application(s.clone()),
            Self::AirPods(e) => Self::AirPods(e.clone()),
            Self::BatteryMonitor(s) => Self::BatteryMonitor(s.clone()),
            Self::BatteryMonitorError(s) => Self::BatteryMonitorError(s.clone()),
            Self::StatePersistence(s) => Self::StatePersistence(s.clone()),
            Self::Lifecycle(s) => Self::Lifecycle(s.clone()),
            Self::ParseError(e) => Self::Parse(format!("JSON parse error: {}", e)),
            Self::IoError(e) => Self::General(format!("I/O error: {}", e)),
            Self::Path(s) => Self::Path(s.clone()),
            Self::FileNotFound(p) => Self::FileNotFound(p.clone()),
            Self::PermissionDenied(s) => Self::PermissionDenied(s.clone()),
            Self::Validation(s) => Self::Validation(s.clone()),
            Self::Parse(s) => Self::Parse(s.clone()),
            Self::Timeout(s) => Self::Timeout(s.clone()),
            Self::Context { context, source } => Self::General(format!("{}: {}", context, source)),
            Self::InvalidData(s) => Self::InvalidData(s.clone()),
            Self::BluetoothError(e) => Self::BluetoothError(e.clone()),
        }
    }
}

// Add these additional From implementations to handle non-cloneable errors
impl From<serde_json::Error> for RustPodsError {
    fn from(err: serde_json::Error) -> Self {
        RustPodsError::ParseError(err.to_string())
    }
}

impl From<std::io::Error> for RustPodsError {
    fn from(err: std::io::Error) -> Self {
        RustPodsError::IoError(err.to_string())
    }
} 