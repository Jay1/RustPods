//! Structured logging for RustPods
//!
//! This module provides a structured logging setup that integrates with the
//! error handling system and provides context-aware logs with selective debug categories.

use chrono::Local;
use log::Level;
use log::{LevelFilter, Metadata, Record};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::{Once, RwLock};
use serde_json;

use crate::config::LogLevel;
use crate::error::ErrorContext;

/// Timestamp format for log entries
const TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S%.3f";

/// Maximum number of log files to keep (for cleanup)
const MAX_LOG_FILES: usize = 1;

/// Maximum number of battery profile files to keep
const MAX_BATTERY_FILES: usize = 50;

/// Global initialization guard
static INIT_LOGGER: Once = Once::new();

/// Get the application data directory
fn get_app_data_dir() -> Result<PathBuf, String> {
    let data_dir = dirs::data_local_dir()
        .ok_or_else(|| "Failed to get local data directory".to_string())?
        .join("RustPods")
        .join("logs");
    
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;
    
    Ok(data_dir)
}

/// Debug flag categories for selective logging
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct DebugFlags {
    pub ui: bool,        // UI events, window management, system tray
    pub bluetooth: bool, // Bluetooth scanning, device discovery, CLI scanner
    pub airpods: bool,   // AirPods detection, battery parsing
    pub config: bool,    // Configuration loading, saving, validation
    pub system: bool,    // System-level operations, lifecycle, persistence
    pub all: bool,       // Enable all debug output
}

impl DebugFlags {
    /// Check if any debug flags are enabled
    pub fn any_enabled(&self) -> bool {
        self.ui || self.bluetooth || self.airpods || self.config || self.system || self.all
    }
}


/// Global debug flags storage
static DEBUG_FLAGS: RwLock<DebugFlags> = RwLock::new(DebugFlags {
    ui: false,
    bluetooth: false,
    airpods: false,
    config: false,
    system: false,
    all: false,
});

/// Custom logger implementation for RustPods
pub struct RustPodsLogger {
    /// File output for logs
    file: Option<Mutex<File>>,
    /// Log level filter
    level: LevelFilter,
    /// Whether to output to stderr
    console_output: bool,
}

impl log::Log for RustPodsLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // Always allow warn and error
        if metadata.level() <= Level::Warn {
            return metadata.level() <= self.level;
        }

        // For debug/info/trace levels, check if enabled by log level
        if metadata.level() > self.level {
            return false;
        }

        // For debug level, also check debug flags
        if metadata.level() == Level::Debug {
            let module_path = metadata.target();
            return should_log_debug(module_path);
        }

        // Info and trace follow normal level filtering
        true
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        // Format timestamp
        let timestamp = Local::now().format(TIMESTAMP_FORMAT);

        // Format the log message
        let level_str = match record.level() {
            Level::Error => "\x1B[31mERROR\x1B[0m", // Red
            Level::Warn => "\x1B[33mWARN \x1B[0m",  // Yellow
            Level::Info => "\x1B[32mINFO \x1B[0m",  // Green
            Level::Debug => "\x1B[36mDEBUG\x1B[0m", // Cyan
            Level::Trace => "\x1B[90mTRACE\x1B[0m", // Gray
        };

        // Extract module/component and file/line info
        let module = record.module_path().unwrap_or("<unknown>");
        let file_info = format!(
            "{}:{}",
            record.file().unwrap_or("<unknown>"),
            record.line().unwrap_or(0)
        );

        // Format log entry with colors for console
        let console_entry = format!(
            "[{}] {} [{}] [{}] {}\n",
            timestamp,
            level_str,
            module,
            file_info,
            record.args()
        );

        // Plain format for file
        let file_entry = format!(
            "[{}] {} [{}] [{}] {}\n",
            timestamp,
            record.level(),
            module,
            file_info,
            record.args()
        );

        // Output to console if enabled
        if self.console_output {
            let _ = io::stderr().write_all(console_entry.as_bytes());
        }

        // Output to file if configured
        if let Some(file) = &self.file {
            if let Ok(mut file) = file.lock() {
                let _ = file.write_all(file_entry.as_bytes());
                let _ = file.flush();
            }
        }
    }

    fn flush(&self) {
        if let Some(file) = &self.file {
            if let Ok(mut file) = file.lock() {
                let _ = file.flush();
            }
        }
    }
}

/// Clean up old log files, keeping only the most recent MAX_LOG_FILES
fn cleanup_old_log_files(log_dir: &std::path::Path) -> Result<(), String> {
    if !log_dir.exists() {
        return Ok(());
    }

    // Get all log files in the directory
    let mut log_files: Vec<_> = std::fs::read_dir(log_dir)
        .map_err(|e| format!("Failed to read log directory: {}", e))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().is_file() && 
            entry.file_name().to_string_lossy().starts_with("rustpods_") &&
            entry.file_name().to_string_lossy().ends_with(".log")
        })
        .collect();

    // If we don't have too many files, nothing to clean up
    if log_files.len() <= MAX_LOG_FILES {
        return Ok(());
    }

    // Sort by modification time (newest first)
    log_files.sort_by(|a, b| {
        let a_modified = a.metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        let b_modified = b.metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        
        b_modified.cmp(&a_modified)
    });

    // Remove files beyond MAX_LOG_FILES
    let _files_to_remove = log_files.len() - MAX_LOG_FILES;
    let mut removed_count = 0;

    for file_entry in log_files.iter().skip(MAX_LOG_FILES) {
        match std::fs::remove_file(file_entry.path()) {
            Ok(()) => {
                removed_count += 1;
                log::debug!("Removed old log file: {}", file_entry.path().display());
            }
            Err(e) => {
                log::warn!("Failed to remove log file {}: {}", file_entry.path().display(), e);
            }
        }
    }

    if removed_count > 0 {
        log::info!("Cleaned up {} old log files, keeping {} most recent", removed_count, MAX_LOG_FILES);
    }

    Ok(())
}

/// Configure logging with the specified level and optionally a log file
pub fn configure_logging(
    level: LogLevel,
    log_file: Option<PathBuf>,
    console_output: bool,
) -> Result<(), String> {
    // Initialize only once
    let mut result = Ok(());

    INIT_LOGGER.call_once(|| {
        // Clean up old log files if we have a log file path
        if let Some(ref path) = log_file {
            if let Some(log_dir) = path.parent() {
                if let Err(e) = cleanup_old_log_files(log_dir) {
                    eprintln!("Warning: Failed to cleanup old log files: {}", e);
                }
            }
        }

        // Convert LogLevel to LevelFilter
        let level_filter = match level {
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        };

        // Open log file if path is provided
        let file = if let Some(path) = log_file.clone() {
            // Ensure parent directory exists
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    if let Err(e) = std::fs::create_dir_all(parent) {
                        result = Err(format!("Failed to create log directory: {}", e));
                        return;
                    }
                }
            }

            // Open the file for appending, create if it doesn't exist
            match OpenOptions::new().create(true).append(true).open(&path) {
                Ok(file) => Some(Mutex::new(file)),
                Err(e) => {
                    result = Err(format!("Failed to open log file: {}", e));
                    return;
                }
            }
        } else {
            None
        };

        // Create and set the logger
        let logger = Box::new(RustPodsLogger {
            file,
            level: level_filter,
            console_output,
        });

        if let Err(e) = log::set_boxed_logger(logger) {
            result = Err(format!("Failed to set logger: {}", e));
            return;
        }

        log::set_max_level(level_filter);

        // Log the initialization
        log::info!("Logging initialized at level: {}", level);
        if let Some(path) = log_file {
            log::info!("Log file: {}", path.display());
        }
    });

    result
}

/// Set global debug flags for selective logging
pub fn set_debug_flags(flags: DebugFlags) {
    if let Ok(mut debug_flags) = DEBUG_FLAGS.write() {
        *debug_flags = flags;
    }
}

/// Check if a debug category should log based on the module path and global flags
pub fn should_log_debug(module_path: &str) -> bool {
    if let Ok(flags) = DEBUG_FLAGS.read() {
        if flags.all {
            return true;
        }

        // Check module path against debug categories - improved matching
        // UI category: ui module, system_tray, window management
        if module_path.contains("ui") 
            || module_path.contains("system_tray")
            || module_path.contains("window")
            || module_path.contains("main_window")
            || module_path.contains("state")  // state.rs is in ui module
        {
            return flags.ui;
        }
        
        // Bluetooth category: bluetooth module, CLI scanner, adapters
        if module_path.contains("bluetooth")
            || module_path.contains("cli_scanner")
            || module_path.contains("adapter")
            || module_path.contains("peripheral")
        {
            return flags.bluetooth;
        }
        
        // AirPods category: airpods module, battery info
        if module_path.contains("airpods") 
            || module_path.contains("battery")
        {
            return flags.airpods;
        }
        
        // Config category: config module, validation
        if module_path.contains("config") 
            || module_path.contains("validation")
        {
            return flags.config;
        }
        
        // System category: lifecycle, persistence, telemetry, diagnostics
        if module_path.contains("lifecycle")
            || module_path.contains("persistence")
            || module_path.contains("telemetry")
            || module_path.contains("diagnostics")
        {
            return flags.system;
        }
    }
    false
}

/// Conditional debug logging macro that respects debug flags
#[macro_export]
macro_rules! debug_log {
    ($category:expr, $($arg:tt)*) => {
        if $crate::logging::should_log_debug(module_path!()) {
            log::debug!($($arg)*);
        }
    };
}

/// Conditional info logging for selected debug categories
#[macro_export]
macro_rules! debug_info {
    ($category:expr, $($arg:tt)*) => {
        if $crate::logging::should_log_debug(module_path!()) {
            log::info!($($arg)*);
        }
    };
}

/// Log an error with context
pub fn log_error<E: Debug>(error: &E, context: &ErrorContext) {
    let component = &context.component;
    let operation = &context.operation;

    // Build the metadata string
    let metadata = if context.metadata.is_empty() {
        String::new()
    } else {
        let pairs = context
            .metadata
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(", ");
        format!(" | {}", pairs)
    };

    // Log the error with context
    log::error!(
        "[{}::{}] Error: {:?}{}{}",
        component,
        operation,
        error,
        if context.metadata.is_empty() {
            ""
        } else {
            " | "
        },
        metadata
    );

    // If user message is provided, log it separately
    if let Some(msg) = &context.user_message {
        log::error!("[{}::{}] User message: {}", component, operation, msg);
    }
}

/// Log an error with recovery action
pub fn log_error_with_recovery<E: Debug>(error: &E, context: &ErrorContext, recovery: &str) {
    // Log the basic error first
    log_error(error, context);

    // Log the recovery action
    log::info!(
        "[{}::{}] Recovery action: {}",
        context.component,
        context.operation,
        recovery
    );
}

/// Helper for performance logging
pub struct PerformanceLogger {
    /// Operation being timed
    operation: String,
    /// Component performing the operation
    component: String,
    /// Start time
    start_time: std::time::Instant,
}

impl PerformanceLogger {
    /// Create a new performance logger
    pub fn new<S: Into<String>>(component: S, operation: S) -> Self {
        Self {
            component: component.into(),
            operation: operation.into(),
            start_time: std::time::Instant::now(),
        }
    }

    /// Finish timing and log result
    pub fn finish(self) {
        let duration = self.start_time.elapsed();
        log::debug!(
            "[{}::{}] Operation completed in {:?}",
            self.component,
            self.operation,
            duration
        );
    }

    /// Finish timing with additional context
    pub fn finish_with_context(self, context: &str) {
        let duration = self.start_time.elapsed();
        log::debug!(
            "[{}::{}] Operation '{}' completed in {:?}",
            self.component,
            self.operation,
            context,
            duration
        );
    }
}

/// Battery profile entry for detailed battery monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryProfileEntry {
    pub timestamp: String,
    pub device_name: String,
    pub device_address: String,
    pub left_battery: Option<u8>,
    pub right_battery: Option<u8>,
    pub left_charging: Option<bool>,
    pub right_charging: Option<bool>,
    pub left_in_ear: Option<bool>,
    pub right_in_ear: Option<bool>,
    pub estimated_left: Option<u8>,
    pub estimated_right: Option<u8>,
    pub discharge_rate_left: Option<f32>,  // %/hour
    pub discharge_rate_right: Option<f32>, // %/hour
    pub usage_session_minutes: Option<u32>,
    pub rssi: Option<i16>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatteryProfile {
    pub device_name: String,
    pub device_address: String,
    pub session_start: String,
    pub entries: Vec<BatteryProfileEntry>,
    pub summary: BatterySessionSummary,
}

/// Summary statistics for a battery monitoring session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatterySessionSummary {
    pub total_duration_minutes: u32,
    pub average_discharge_rate_left: f32,
    pub average_discharge_rate_right: f32,
    pub min_battery_left: u8,
    pub min_battery_right: u8,
    pub max_battery_left: u8,
    pub max_battery_right: u8,
    pub total_entries: usize,
    pub charging_events: u32,
    pub usage_patterns: Vec<String>, // e.g., "Heavy usage", "Light usage", "Charging session"
}

/// Battery logger for dedicated battery profiling
pub struct BatteryLogger {
    current_session: Option<BatteryProfile>,
    battery_log_dir: PathBuf,
}

impl BatteryLogger {
    pub fn new() -> Result<Self, String> {
        let data_dir = get_app_data_dir()?;
        let battery_log_dir = data_dir.join("battery");
        
        // Create battery directory if it doesn't exist
        std::fs::create_dir_all(&battery_log_dir)
            .map_err(|e| format!("Failed to create battery log directory: {}", e))?;
        
        // Clean up old battery files
        cleanup_old_battery_files(&battery_log_dir)?;
        
        Ok(Self {
            current_session: None,
            battery_log_dir,
        })
    }
    
    pub fn start_session(&mut self, device_name: &str, device_address: &str) {
        let session_start = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        self.current_session = Some(BatteryProfile {
            device_name: device_name.to_string(),
            device_address: device_address.to_string(),
            session_start,
            entries: Vec::new(),
            summary: BatterySessionSummary {
                total_duration_minutes: 0,
                average_discharge_rate_left: 0.0,
                average_discharge_rate_right: 0.0,
                min_battery_left: 100,
                min_battery_right: 100,
                max_battery_left: 0,
                max_battery_right: 0,
                total_entries: 0,
                charging_events: 0,
                usage_patterns: Vec::new(),
            },
        });
    }
    
    pub fn log_battery_data(&mut self, entry: BatteryProfileEntry) {
        if let Some(ref mut session) = self.current_session {
            // Update summary statistics
            if let Some(left) = entry.left_battery {
                session.summary.min_battery_left = session.summary.min_battery_left.min(left);
                session.summary.max_battery_left = session.summary.max_battery_left.max(left);
            }
            if let Some(right) = entry.right_battery {
                session.summary.min_battery_right = session.summary.min_battery_right.min(right);
                session.summary.max_battery_right = session.summary.max_battery_right.max(right);
            }
            
            // Detect charging events
            if entry.left_charging == Some(true) || entry.right_charging == Some(true) {
                session.summary.charging_events += 1;
            }
            
            session.entries.push(entry);
            session.summary.total_entries = session.entries.len();
            
            // Auto-save every 5 entries to prevent data loss
            if session.entries.len() % 5 == 0 {
                let _ = self.save_current_session();
            }
        }
    }
    
    pub fn end_session(&mut self) -> Result<(), String> {
        if let Some(mut session) = self.current_session.take() {
            // Calculate final summary statistics
            self.calculate_session_summary(&mut session);
            
            // Save the complete session
            self.save_session(&session)?;
        }
        Ok(())
    }
    
    fn save_current_session(&self) -> Result<(), String> {
        if let Some(ref session) = self.current_session {
            self.save_session(session)
        } else {
            Ok(())
        }
    }
    
    fn save_session(&self, session: &BatteryProfile) -> Result<(), String> {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let filename = format!("battery_profile_{}_{}.json", 
            session.device_address.replace(":", ""), timestamp);
        let file_path = self.battery_log_dir.join(filename);
        
        let json_data = serde_json::to_string_pretty(session)
            .map_err(|e| format!("Failed to serialize battery profile: {}", e))?;
        
        std::fs::write(&file_path, json_data)
            .map_err(|e| format!("Failed to write battery profile: {}", e))?;
        
        println!("Battery profile saved: {}", file_path.display());
        Ok(())
    }
    
    fn calculate_session_summary(&self, session: &mut BatteryProfile) {
        if session.entries.is_empty() {
            return;
        }
        
        // Calculate total duration
        if let (Some(_first), Some(_last)) = (session.entries.first(), session.entries.last()) {
            // Parse timestamps and calculate duration
            // For now, use entry count as a proxy (each entry ~10 seconds)
            session.summary.total_duration_minutes = (session.entries.len() as u32 * 10) / 60;
        }
        
        // Calculate average discharge rates
        let discharge_rates_left: Vec<f32> = session.entries
            .iter()
            .filter_map(|e| e.discharge_rate_left)
            .collect();
        if !discharge_rates_left.is_empty() {
            session.summary.average_discharge_rate_left = 
                discharge_rates_left.iter().sum::<f32>() / discharge_rates_left.len() as f32;
        }
        
        let discharge_rates_right: Vec<f32> = session.entries
            .iter()
            .filter_map(|e| e.discharge_rate_right)
            .collect();
        if !discharge_rates_right.is_empty() {
            session.summary.average_discharge_rate_right = 
                discharge_rates_right.iter().sum::<f32>() / discharge_rates_right.len() as f32;
        }
        
        // Analyze usage patterns
        self.analyze_usage_patterns(session);
    }
    
    fn analyze_usage_patterns(&self, session: &mut BatteryProfile) {
        let mut patterns = Vec::new();
        
        // Detect heavy usage periods (high discharge rate)
        let avg_discharge = (session.summary.average_discharge_rate_left + 
                           session.summary.average_discharge_rate_right) / 2.0;
        
        if avg_discharge > 15.0 {
            patterns.push("Heavy usage session".to_string());
        } else if avg_discharge > 8.0 {
            patterns.push("Moderate usage session".to_string());
        } else {
            patterns.push("Light usage session".to_string());
        }
        
        // Detect charging patterns
        if session.summary.charging_events > (session.summary.total_entries / 4) as u32 {
            patterns.push("Frequent charging".to_string());
        }
        
        // Detect low battery warnings
        if session.summary.min_battery_left < 20 || session.summary.min_battery_right < 20 {
            patterns.push("Low battery reached".to_string());
        }
        
        session.summary.usage_patterns = patterns;
    }
}

/// Clean up old battery profile files, keeping only the most recent MAX_BATTERY_FILES
fn cleanup_old_battery_files(battery_dir: &std::path::Path) -> Result<(), String> {
    if !battery_dir.exists() {
        return Ok(());
    }
    
    // Get all battery profile files
    let mut battery_files: Vec<_> = std::fs::read_dir(battery_dir)
        .map_err(|e| format!("Failed to read battery directory: {}", e))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().is_file() && 
            entry.file_name().to_string_lossy().starts_with("battery_profile_") &&
            entry.file_name().to_string_lossy().ends_with(".json")
        })
        .collect();
    
    // If we don't have too many files, nothing to clean up
    if battery_files.len() <= MAX_BATTERY_FILES {
        return Ok(());
    }
    
    // Sort by modification time (newest first)
    battery_files.sort_by(|a, b| {
        let a_time = a.metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        let b_time = b.metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        b_time.cmp(&a_time) // Newest first
    });
    
    // Remove the oldest files
    for file_entry in battery_files.iter().skip(MAX_BATTERY_FILES) {
        if let Err(e) = std::fs::remove_file(file_entry.path()) {
            eprintln!("Warning: Failed to remove old battery file {}: {}", 
                     file_entry.path().display(), e);
        } else {
            println!("Cleaned up old battery file: {}", file_entry.path().display());
        }
    }
    
    Ok(())
}

/// Global battery logger instance
static BATTERY_LOGGER: Mutex<Option<BatteryLogger>> = Mutex::new(None);

/// Initialize the battery logger
pub fn init_battery_logger() -> Result<(), String> {
    let mut logger = BATTERY_LOGGER.lock().unwrap();
    if logger.is_none() {
        *logger = Some(BatteryLogger::new()?);
    }
    Ok(())
}

/// Log battery data to the dedicated battery profiling system
pub fn log_battery_data(
    device_name: &str,
    device_address: &str,
    left_battery: Option<u8>,
    right_battery: Option<u8>,
    left_charging: Option<bool>,
    right_charging: Option<bool>,
    left_in_ear: Option<bool>,
    right_in_ear: Option<bool>,
    estimated_left: Option<u8>,
    estimated_right: Option<u8>,
    discharge_rate_left: Option<f32>,
    discharge_rate_right: Option<f32>,
    rssi: Option<i16>,
) {
    let mut logger = BATTERY_LOGGER.lock().unwrap();
    if let Some(ref mut logger) = logger.as_mut() {
        // Start a new session if we don't have one
        if logger.current_session.is_none() {
            logger.start_session(device_name, device_address);
        }
        
        let entry = BatteryProfileEntry {
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
            device_name: device_name.to_string(),
            device_address: device_address.to_string(),
            left_battery,
            right_battery,
            left_charging,
            right_charging,
            left_in_ear,
            right_in_ear,
            estimated_left,
            estimated_right,
            discharge_rate_left,
            discharge_rate_right,
            usage_session_minutes: None, // Will be calculated in summary
            rssi,
        };
        
        logger.log_battery_data(entry);
    }
}

/// End the current battery logging session
pub fn end_battery_session() {
    let mut logger = BATTERY_LOGGER.lock().unwrap();
    if let Some(ref mut logger) = logger.as_mut() {
        let _ = logger.end_session();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    #[ignore]
    // This test is ignored by default because the global logger can only be initialized once per process.
    // Run manually if you need to verify logger file creation. In CI or multi-test runs, this will always fail.
    fn test_logger_creation() {
        let temp_dir = tempdir().unwrap();
        let log_path = temp_dir.path().join("test.log");

        // Configure logging
        let result = configure_logging(LogLevel::Debug, Some(log_path.clone()), false);
        assert!(result.is_ok());

        // Log a test message
        log::debug!("Test debug message");
        log::info!("Test info message");

        // Verify file exists and has content
        assert!(log_path.exists());
    }

    #[test]
    fn test_performance_logger() {
        // Setup logger
        let _ = configure_logging(LogLevel::Debug, None, false);

        // Use the performance logger
        {
            let perf = PerformanceLogger::new("Test", "performance_logging");
            // Simulate some work
            std::thread::sleep(std::time::Duration::from_millis(10));
            perf.finish();
        }

        // With context
        {
            let perf = PerformanceLogger::new("Test", "performance_logging");
            // Simulate some work
            std::thread::sleep(std::time::Duration::from_millis(10));
            perf.finish_with_context("with extra info");
        }
    }
}
