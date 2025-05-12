//! Enhanced logging system for RustPods
//!
//! This module provides a structured logging system with file rotation,
//! configurable verbosity levels, and diagnostic features.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Local};
use log::{Level, LevelFilter, Metadata, Record};

use crate::config::AppConfig;

/// Maximum log file size in bytes (5 MB)
const MAX_LOG_FILE_SIZE: u64 = 5 * 1024 * 1024;

/// Maximum number of backup log files
const MAX_BACKUP_LOG_FILES: usize = 5;

/// Custom logger implementation
pub struct RustPodsLogger {
    /// File logger configuration
    file_config: Arc<Mutex<FileLoggerConfig>>,
    
    /// Console logger configuration  
    console_config: ConsoleLoggerConfig,
    
    /// Diagnostic mode
    diagnostic_mode: bool,
    
    /// Send log data for telemetry (if enabled)
    telemetry_enabled: bool,
    
    /// Current log file path
    log_file_path: Arc<Mutex<Option<PathBuf>>>,
    
    /// Current log file handle
    log_file: Arc<Mutex<Option<File>>>,
}

/// File logger configuration
#[derive(Debug, Clone)]
struct FileLoggerConfig {
    /// Log directory
    log_dir: PathBuf,
    
    /// Log level filter
    level: LevelFilter,
    
    /// Log file name pattern
    file_pattern: String,
    
    /// Enable file rotation
    rotation_enabled: bool,
}

/// Console logger configuration
#[derive(Debug, Clone)]
struct ConsoleLoggerConfig {
    /// Log level filter
    level: LevelFilter,
    
    /// Include timestamp in console output
    include_timestamp: bool,
    
    /// Include source location in console output
    include_source_location: bool,
    
    /// Whether to use ANSI colors
    colored_output: bool,
}

impl RustPodsLogger {
    /// Create a new logger
    pub fn new(config: &AppConfig) -> Self {
        // Determine log directory
        let log_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("RustPods")
            .join("logs");
        
        // Create log directory if it doesn't exist
        if !log_dir.exists() {
            if let Err(e) = fs::create_dir_all(&log_dir) {
                eprintln!("Failed to create log directory: {}", e);
            }
        }
        
        // Convert log level from config
        let log_level = match config.system.log_level {
            crate::config::LogLevel::Error => LevelFilter::Error,
            crate::config::LogLevel::Warn => LevelFilter::Warn,
            crate::config::LogLevel::Info => LevelFilter::Info,
            crate::config::LogLevel::Debug => LevelFilter::Debug,
            crate::config::LogLevel::Trace => LevelFilter::Trace,
        };
        
        // Create file logger config
        let file_config = FileLoggerConfig {
            log_dir,
            level: log_level,
            file_pattern: "rustpods_{}.log".to_string(),
            rotation_enabled: true,
        };
        
        // Create console logger config
        let console_config = ConsoleLoggerConfig {
            level: log_level,
            include_timestamp: true,
            include_source_location: log_level >= LevelFilter::Debug,
            colored_output: true,
        };
        
        // Create logger instance
        let mut logger = Self {
            file_config: Arc::new(Mutex::new(file_config)),
            console_config,
            diagnostic_mode: false,
            telemetry_enabled: config.system.enable_telemetry,
            log_file_path: Arc::new(Mutex::new(None)),
            log_file: Arc::new(Mutex::new(None)),
        };
        
        // Initialize log file
        if let Err(e) = logger.initialize_log_file() {
            eprintln!("Failed to initialize log file: {}", e);
        }
        
        logger
    }
    
    /// Initialize the log file
    fn initialize_log_file(&mut self) -> io::Result<()> {
        let file_config = self.file_config.lock().unwrap();
        
        // Generate log file name with timestamp
        let now = Local::now();
        let timestamp = now.format("%Y%m%d_%H%M%S");
        let file_name = format!("rustpods_{}.log", timestamp);
        let file_path = file_config.log_dir.join(&file_name);
        
        // Open log file
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)?;
        
        // Store file path and handle
        {
            let mut log_file_path = self.log_file_path.lock().unwrap();
            *log_file_path = Some(file_path);
        }
        {
            let mut log_file = self.log_file.lock().unwrap();
            *log_file = Some(file);
        }
        
        // Write initial log entry
        self.log_to_file(&format!(
            "=== RustPods Log Started at {} ===\n", 
            now.format("%Y-%m-%d %H:%M:%S")
        ))?;
        
        Ok(())
    }
    
    /// Log a message to the current log file
    fn log_to_file(&self, message: &str) -> io::Result<()> {
        if let Ok(mut log_file) = self.log_file.lock() {
            if let Some(file) = log_file.as_mut() {
                // Write message to file
                file.write_all(message.as_bytes())?;
                file.flush()?;
                
                // Check if rotation is needed
                if self.is_rotation_needed()? {
                    self.rotate_logs()?;
                    
                    // Re-initialize log file after rotation
                    drop(log_file); // Release mutex lock
                    let file_config = self.file_config.lock().unwrap();
                    
                    // Generate new log file name with timestamp
                    let now = Local::now();
                    let timestamp = now.format("%Y%m%d_%H%M%S");
                    let file_name = format!("rustpods_{}.log", timestamp);
                    let file_path = file_config.log_dir.join(&file_name);
                    
                    // Open new log file
                    let file = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&file_path)?;
                    
                    // Store new file path and handle
                    {
                        let mut log_file_path = self.log_file_path.lock().unwrap();
                        *log_file_path = Some(file_path);
                    }
                    {
                        let mut log_file = self.log_file.lock().unwrap();
                        *log_file = Some(file);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Check if log rotation is needed
    fn is_rotation_needed(&self) -> io::Result<bool> {
        if let Ok(log_file_path) = self.log_file_path.lock() {
            if let Some(path) = log_file_path.as_ref() {
                if let Ok(metadata) = fs::metadata(path) {
                    return Ok(metadata.len() > MAX_LOG_FILE_SIZE);
                }
            }
        }
        
        Ok(false)
    }
    
    /// Rotate log files
    fn rotate_logs(&self) -> io::Result<()> {
        if let Ok(file_config) = self.file_config.lock() {
            // List log files
            let entries = fs::read_dir(&file_config.log_dir)?;
            
            // Collect and sort log files by creation time (newest first)
            let mut log_files: Vec<PathBuf> = entries
                .filter_map(|entry| {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.is_file() && path.extension().map_or(false, |ext| ext == "log") {
                            return Some(path);
                        }
                    }
                    None
                })
                .collect();
            
            log_files.sort_by(|a, b| {
                let a_time = fs::metadata(a).and_then(|m| m.modified()).unwrap_or_else(|_| SystemTime::now());
                let b_time = fs::metadata(b).and_then(|m| m.modified()).unwrap_or_else(|_| SystemTime::now());
                b_time.cmp(&a_time)
            });
            
            // Keep only a maximum number of backup log files
            if log_files.len() > MAX_BACKUP_LOG_FILES {
                for old_file in log_files.iter().skip(MAX_BACKUP_LOG_FILES) {
                    if let Err(e) = fs::remove_file(old_file) {
                        eprintln!("Failed to delete old log file {}: {}", old_file.display(), e);
                    } else {
                        println!("Deleted old log file: {}", old_file.display());
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Set the diagnostic mode
    pub fn set_diagnostic_mode(&mut self, enabled: bool) {
        self.diagnostic_mode = enabled;
        
        // In diagnostic mode, increase logging level
        if enabled {
            self.set_level(LevelFilter::Debug);
        }
    }
    
    /// Set logging level
    pub fn set_level(&mut self, level: LevelFilter) {
        if let Ok(mut file_config) = self.file_config.lock() {
            file_config.level = level;
        }
        self.console_config.level = level;
    }
    
    /// Enable or disable telemetry
    pub fn set_telemetry_enabled(&mut self, enabled: bool) {
        self.telemetry_enabled = enabled;
    }
    
    /// Get current log file path
    pub fn log_file_path(&self) -> Option<PathBuf> {
        if let Ok(path) = self.log_file_path.lock() {
            path.clone()
        } else {
            None
        }
    }
    
    /// Format a record for console output
    fn format_console_record(&self, record: &Record) -> String {
        let level_str = match record.level() {
            Level::Error => {
                if self.console_config.colored_output {
                    "\x1B[1;31mERROR\x1B[0m"  // Bold Red
                } else {
                    "ERROR"
                }
            },
            Level::Warn => {
                if self.console_config.colored_output {
                    "\x1B[1;33mWARN \x1B[0m"  // Bold Yellow
                } else {
                    "WARN "
                }
            },
            Level::Info => {
                if self.console_config.colored_output {
                    "\x1B[1;32mINFO \x1B[0m"  // Bold Green
                } else {
                    "INFO "
                }
            },
            Level::Debug => {
                if self.console_config.colored_output {
                    "\x1B[1;34mDEBUG\x1B[0m"  // Bold Blue
                } else {
                    "DEBUG"
                }
            },
            Level::Trace => {
                if self.console_config.colored_output {
                    "\x1B[1;35mTRACE\x1B[0m"  // Bold Magenta
                } else {
                    "TRACE"
                }
            },
        };

        let mut result = String::new();
        
        // Add timestamp if configured
        if self.console_config.include_timestamp {
            let now = Local::now();
            result.push_str(&format!("[{}] ", now.format("%H:%M:%S")));
        }
        
        // Add level and message
        result.push_str(&format!("[{}] {}", level_str, record.args()));
        
        // Add source location if configured
        if self.console_config.include_source_location {
            if let Some(file) = record.file() {
                if let Some(line) = record.line() {
                    result.push_str(&format!(" ({}:{})", file, line));
                }
            }
        }
        
        result
    }
    
    /// Format a record for file output
    fn format_file_record(&self, record: &Record) -> String {
        let now = Local::now();
        let timestamp = now.format("%Y-%m-%d %H:%M:%S%.3f");
        
        let mut result = format!(
            "[{}] [{}] {}",
            timestamp,
            record.level(),
            record.args()
        );
        
        // Add source location for file logs
        if let Some(file) = record.file() {
            if let Some(line) = record.line() {
                result.push_str(&format!(" ({}:{})", file, line));
            }
        }
        
        // Add module path for better context
        if let Some(module) = record.module_path() {
            result.push_str(&format!(" [{}]", module));
        }
        
        // Add newline
        result.push('\n');
        
        result
    }
    
    /// Save diagnostic information to a file
    pub fn save_diagnostics(&self) -> io::Result<PathBuf> {
        let file_config = self.file_config.lock().unwrap();
        let diagnostics_dir = file_config.log_dir.join("diagnostics");
        
        // Create diagnostics directory if it doesn't exist
        if !diagnostics_dir.exists() {
            fs::create_dir_all(&diagnostics_dir)?;
        }
        
        // Generate diagnostic file name with timestamp
        let now = Local::now();
        let timestamp = now.format("%Y%m%d_%H%M%S");
        let file_name = format!("rustpods_diagnostic_{}.log", timestamp);
        let file_path = diagnostics_dir.join(&file_name);
        
        // Create diagnostic file
        let mut file = File::create(&file_path)?;
        
        // Write system information
        writeln!(file, "=== RustPods Diagnostic Report ===")?;
        writeln!(file, "Generated: {}", now.format("%Y-%m-%d %H:%M:%S"))?;
        writeln!(file, "=== System Information ===")?;
        
        // OS information
        writeln!(file, "OS: {}", std::env::consts::OS)?;
        writeln!(file, "Architecture: {}", std::env::consts::ARCH)?;
        
        // Process information
        if let Ok(current_exe) = std::env::current_exe() {
            writeln!(file, "Executable: {}", current_exe.display())?;
        }
        
        writeln!(file, "Working directory: {}", std::env::current_dir()?.display())?;
        
        // Collect recent logs
        if let Ok(log_path) = self.log_file_path.lock() {
            if let Some(path) = log_path.as_ref() {
                writeln!(file, "\n=== Recent Logs ===")?;
                if let Ok(log_content) = fs::read_to_string(path) {
                    // Take the last 100 lines
                    let lines: Vec<&str> = log_content.lines().collect();
                    let start = if lines.len() > 100 { lines.len() - 100 } else { 0 };
                    
                    for line in &lines[start..] {
                        writeln!(file, "{}", line)?;
                    }
                } else {
                    writeln!(file, "Could not read log file")?;
                }
            }
        }
        
        // Flush file
        file.flush()?;
        
        // Return the path to the diagnostic file
        Ok(file_path)
    }
}

impl log::Log for RustPodsLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let console_enabled = metadata.level() <= self.console_config.level;
        let file_enabled = if let Ok(file_config) = self.file_config.lock() {
            metadata.level() <= file_config.level
        } else {
            false
        };
        
        console_enabled || file_enabled
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        
        // Console logging
        if record.level() <= self.console_config.level {
            let formatted = self.format_console_record(record);
            println!("{}", formatted);
        }
        
        // File logging
        if let Ok(file_config) = self.file_config.lock() {
            if record.level() <= file_config.level {
                let formatted = self.format_file_record(record);
                if let Err(e) = self.log_to_file(&formatted) {
                    eprintln!("Failed to write to log file: {}", e);
                }
            }
        }
        
        // Telemetry logging (for critical errors and important events)
        if self.telemetry_enabled && record.level() <= Level::Error {
            if let Some(module) = record.module_path() {
                if !module.contains("tokio") && !module.contains("runtime") {
                    // This is where we'd send telemetry data if implemented
                    // telemetry::send(record);
                }
            }
        }
    }

    fn flush(&self) {
        // Flush log file
        if let Ok(mut log_file) = self.log_file.lock() {
            if let Some(file) = log_file.as_mut() {
                let _ = file.flush();
            }
        }
    }
}

/// Initialize the logging system
pub fn init_logger(config: &AppConfig) -> Result<(), log::SetLoggerError> {
    let max_level = match config.system.log_level {
        crate::config::LogLevel::Error => LevelFilter::Error,
        crate::config::LogLevel::Warn => LevelFilter::Warn,
        crate::config::LogLevel::Info => LevelFilter::Info,
        crate::config::LogLevel::Debug => LevelFilter::Debug,
        crate::config::LogLevel::Trace => LevelFilter::Trace,
    };
    
    let logger = Box::new(RustPodsLogger::new(config));
    log::set_boxed_logger(logger)?;
    log::set_max_level(max_level);
    
    Ok(())
}

/// Determine if diagnostics should be collected
/// Returns true if too many errors occurred recently
pub fn should_collect_diagnostics(error_rate: usize, time_window: std::time::Duration) -> bool {
    // This is a placeholder for a more complex error rate detection algorithm
    // In a real implementation, we would track errors over time and detect spikes
    error_rate > 5
} 