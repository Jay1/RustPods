//! Structured logging for RustPods
//!
//! This module provides a structured logging setup that integrates with the
//! error handling system and provides context-aware logs with selective debug categories.

use chrono::Local;
use log::Level;
use log::{LevelFilter, Metadata, Record};
use std::fmt::Debug;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::{Once, RwLock};

use crate::config::LogLevel;
use crate::error::ErrorContext;

/// Timestamp format for log entries
const TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S%.3f";

/// Global initialization guard
static INIT_LOGGER: Once = Once::new();

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

/// Configure logging with the specified level and optionally a log file
pub fn configure_logging(
    level: LogLevel,
    log_file: Option<PathBuf>,
    console_output: bool,
) -> Result<(), String> {
    // Initialize only once
    let mut result = Ok(());

    INIT_LOGGER.call_once(|| {
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

        // Check module path against debug categories
        if module_path.contains("::ui")
            || module_path.contains("system_tray")
            || module_path.contains("window")
        {
            return flags.ui;
        }
        if module_path.contains("::bluetooth")
            || module_path.contains("cli_scanner")
            || module_path.contains("adapter")
        {
            return flags.bluetooth;
        }
        if module_path.contains("::airpods") || module_path.contains("battery") {
            return flags.airpods;
        }
        if module_path.contains("::config") || module_path.contains("validation") {
            return flags.config;
        }
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
