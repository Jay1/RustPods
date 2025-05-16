//! Telemetry module for RustPods
//!
//! This module provides optional telemetry functionality for error reporting
//! and anonymous usage analytics with strong privacy protections.
//! Telemetry is always opt-in and can be disabled at any time.

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::config::AppConfig;
use crate::error::{RustPodsError, ErrorSeverity, ErrorStats, RecoveryAction, ErrorContext};

/// Error entry for telemetry tracking
#[derive(Debug, Clone)]
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

/// Telemetry manager
pub struct TelemetryManager {
    /// Whether telemetry is enabled
    enabled: bool,
    
    /// Installation ID (anonymous)
    installation_id: String,
    
    /// Error statistics
    error_stats: ErrorStats,
    
    /// Last telemetry upload time
    last_upload: Instant,
    
    /// Usage metrics
    usage_metrics: Arc<Mutex<UsageMetrics>>,
    
    /// Application version
    app_version: String,
    
    /// Error handler
    error_handler: Option<Box<dyn FnMut(&RustPodsError)>>,

    /// Error queue for diagnostic triggering
    error_queue: Vec<ErrorEntry>,

    /// Diagnostics information
    diagnostics: DiagnosticsInfo,
}

/// Usage metrics collected for telemetry
#[derive(Debug, Default, Clone)]
pub struct UsageMetrics {
    /// Number of application starts
    pub app_starts: u32,
    
    /// Total app runtime in seconds
    pub total_runtime_seconds: u64,
    
    /// Number of successful device connections
    pub successful_connections: u32,
    
    /// Number of failed connection attempts
    pub failed_connections: u32,
    
    /// Number of successful battery readings
    pub battery_readings: u32,
    
    /// Features used (with count)
    pub features_used: HashMap<String, u32>,
}

/// Diagnostic information tracking
#[derive(Debug, Default, Clone)]
struct DiagnosticsInfo {
    /// Last time diagnostics were collected
    last_collection: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Number of times diagnostics have been collected
    collection_count: u32,
}

impl TelemetryManager {
    /// Create a new telemetry manager
    pub fn new(config: &AppConfig) -> Self {
        // Generate installation ID if it doesn't exist
        let installation_id = Self::get_or_create_installation_id();
        
        Self {
            enabled: config.system.enable_telemetry,
            installation_id,
            error_stats: ErrorStats::default(),
            last_upload: Instant::now(),
            usage_metrics: Arc::new(Mutex::new(UsageMetrics::default())),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            error_handler: None,
            error_queue: Vec::new(),
            diagnostics: DiagnosticsInfo::default(),
        }
    }
    
    /// Get or create an installation ID
    fn get_or_create_installation_id() -> String {
        // Try to read from config dir first
        let config_dir = dirs::config_dir()
            .map(|d| d.join("rustpods"))
            .unwrap_or_default();
        
        let id_path = config_dir.join("installation_id");
        
        // Try to read existing ID
        if id_path.exists() {
            if let Ok(id) = std::fs::read_to_string(&id_path) {
                return id.trim().to_string();
            }
        }
        
        // Generate new ID
        let new_id = Uuid::new_v4().to_string();
        
        // Try to save it
        if std::fs::create_dir_all(&config_dir).is_ok() {
            let _ = std::fs::write(&id_path, &new_id);
        }
        
        new_id
    }
    
    /// Set whether telemetry is enabled
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Record an application event for telemetry
    pub fn record_event(&self, event_type: TelemetryEvent) {
        if !self.enabled {
            return;
        }
        
        // Track the event locally
        if let Ok(mut metrics) = self.usage_metrics.lock() {
            match event_type {
                TelemetryEvent::AppStart => {
                    metrics.app_starts += 1;
                },
                TelemetryEvent::DeviceConnection { success } => {
                    if success {
                        metrics.successful_connections += 1;
                    } else {
                        metrics.failed_connections += 1;
                    }
                },
                TelemetryEvent::BatteryReading => {
                    metrics.battery_readings += 1;
                },
                TelemetryEvent::FeatureUsed(feature) => {
                    *metrics.features_used.entry(feature).or_insert(0) += 1;
                },
            }
        }
    }
    
    /// Record an error occurrence
    pub fn record_error(&mut self, error: &RustPodsError) {
        if !self.enabled {
            return;
        }
        
        // Update local error stats
        match error {
            RustPodsError::Bluetooth(_) => self.error_stats.bluetooth_errors += 1,
            RustPodsError::BluetoothApiError(_) => self.error_stats.bluetooth_errors += 1,
            RustPodsError::BluetoothError(_) => self.error_stats.bluetooth_errors += 1,
            RustPodsError::AirPods(_) => self.error_stats.airpods_errors += 1,
            RustPodsError::Ui(_) => self.error_stats.ui_errors += 1,
            RustPodsError::UiError => self.error_stats.ui_errors += 1,
            RustPodsError::Config(_) => self.error_stats.config_errors += 1,
            RustPodsError::ConfigError(_) => self.error_stats.config_errors += 1,
            RustPodsError::Application(_) => self.error_stats.app_errors += 1,
            RustPodsError::General(_) => self.error_stats.app_errors += 1,
            RustPodsError::System(_) => self.error_stats.system_errors += 1,
            RustPodsError::DeviceNotFound => self.error_stats.bluetooth_errors += 1,
            RustPodsError::Device(_) => self.error_stats.bluetooth_errors += 1,
            RustPodsError::BatteryMonitor(_) => self.error_stats.battery_errors += 1,
            RustPodsError::BatteryMonitorError(_) => self.error_stats.battery_errors += 1,
            RustPodsError::StatePersistence(_) => self.error_stats.app_errors += 1,
            RustPodsError::State(_) => self.error_stats.app_errors += 1,
            RustPodsError::Lifecycle(_) => self.error_stats.app_errors += 1,
            RustPodsError::IoError(_) => self.error_stats.system_errors += 1,
            RustPodsError::ParseError(_) => self.error_stats.config_errors += 1,
            RustPodsError::Path(_) => self.error_stats.system_errors += 1,
            RustPodsError::FileNotFound(_) => self.error_stats.system_errors += 1,
            RustPodsError::PermissionDenied(_) => self.error_stats.system_errors += 1,
            RustPodsError::Validation(_) => self.error_stats.app_errors += 1,
            RustPodsError::Parse(_) => self.error_stats.config_errors += 1,
            RustPodsError::Timeout(_) => self.error_stats.bluetooth_errors += 1,
            RustPodsError::Context { .. } => self.error_stats.app_errors += 1,
            RustPodsError::InvalidData(_) => self.error_stats.app_errors += 1,
        }
        
        // Add to error queue for diagnostic monitoring
        self.error_queue.push(ErrorEntry {
            error_type: error.get_category().to_string(),
            error_message: error.to_string(),
            timestamp: chrono::Utc::now(),
            context: None,
            recovery: Some(error.recovery_action()),
        });
        
        // Limit error queue size
        if self.error_queue.len() > 100 {  // Assuming MAX_ERROR_HISTORY is 100
            self.error_queue.remove(0);
        }
        
        // Update severity counts
        match error.severity() {
            ErrorSeverity::Critical => self.error_stats.critical_errors += 1,
            ErrorSeverity::Error => self.error_stats.error_level_errors += 1,
            ErrorSeverity::Major => self.error_stats.error_level_errors += 1,
            ErrorSeverity::Minor => self.error_stats.recoverable_errors += 1,
            ErrorSeverity::Warning => self.error_stats.warnings += 1,
            ErrorSeverity::Info => {}, // No action for info
        }
        
        // Call the handler if provided
        if let Some(handler) = &mut self.error_handler {
            handler(error);
        }
        
        // Log the error
        match error.severity() {
            ErrorSeverity::Critical => log::error!("CRITICAL: {}", error),
            ErrorSeverity::Major => log::error!("{}", error),
            ErrorSeverity::Error => log::error!("{}", error),
            ErrorSeverity::Minor => log::warn!("{}", error),
            ErrorSeverity::Warning => log::warn!("{}", error),
            ErrorSeverity::Info => log::info!("{}", error),
        }
        
        // Check if we need to trigger data collection
        if self.should_collect_diagnostics() {
            self.collect_diagnostics();
        }
    }
    
    /// Check if telemetry should be uploaded
    ///
    /// Based on time since last upload and queue size
    pub fn should_upload(&self) -> bool {
        if !self.enabled {
            return false;
        }
        
        // Upload at most once per day
        self.last_upload.elapsed() > Duration::from_secs(24 * 60 * 60)
    }
    
    /// Upload telemetry data
    pub async fn upload_telemetry(&mut self) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }
        
        // Create telemetry payload
        let payload = self.create_telemetry_payload();
        
        // In a real implementation, we would send this payload to a server
        log::debug!("Would upload telemetry payload: {:?}", payload);
        
        // Update last upload time
        self.last_upload = Instant::now();
        
        // Reset local counters after successful upload
        self.error_stats = ErrorStats::default();
        
        if let Ok(mut metrics) = self.usage_metrics.lock() {
            // Reset some metrics (but keep cumulative ones)
            metrics.features_used.clear();
        }
        
        Ok(())
    }
    
    /// Create telemetry payload
    fn create_telemetry_payload(&self) -> TelemetryPayload {
        let usage_metrics = if let Ok(metrics) = self.usage_metrics.lock() {
            metrics.clone()
        } else {
            UsageMetrics::default()
        };
        
        TelemetryPayload {
            installation_id: self.installation_id.clone(),
            app_version: self.app_version.clone(),
            os_name: std::env::consts::OS.to_string(),
            os_arch: std::env::consts::ARCH.to_string(),
            error_stats: self.error_stats.clone(),
            usage_metrics,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
    
    /// Update usage time
    pub fn update_usage_time(&self, seconds: u64) {
        if !self.enabled {
            return;
        }
        
        if let Ok(mut metrics) = self.usage_metrics.lock() {
            metrics.total_runtime_seconds += seconds;
        }
    }
    
    /// Check if diagnostic data collection should be triggered
    fn should_collect_diagnostics(&self) -> bool {
        // If we've hit a critical error threshold or high error rate in a short time
        let recent_errors = self.error_queue.len();
        let critical_errors = self.error_queue.iter()
            .filter(|err| matches!(err.recovery, Some(RecoveryAction::RestartApplication)))
            .count();
        
        // Criteria for collecting diagnostics:
        // 1. Any critical error
        // 2. More than 10 errors in a short timeframe
        // 3. More than 5 errors of the same type
        if critical_errors > 0 || recent_errors > 10 {
            return true;
        }
        
        // Check for patterns of errors
        let mut error_types = std::collections::HashMap::new();
        for error in &self.error_queue {
            *error_types.entry(error.error_type.clone()).or_insert(0) += 1;
        }
        
        error_types.values().any(|&count| count > 5)
    }
    
    /// Collect diagnostic data about the system state
    fn collect_diagnostics(&mut self) {
        // Log that we're collecting diagnostics
        log::info!("Collecting diagnostic data due to error patterns");
        
        // In a real implementation, we'd collect data about:
        // - System state
        // - Bluetooth adapter state
        // - Device connection state
        // - Configuration
        // - Memory usage
        // - etc.
        
        // For now, just update the last collection time
        self.diagnostics.last_collection = Some(chrono::Utc::now());
        self.diagnostics.collection_count += 1;
    }
}

/// Telemetry events
#[derive(Debug, Clone)]
pub enum TelemetryEvent {
    /// Application start
    AppStart,
    
    /// Device connection attempt
    DeviceConnection {
        /// Whether connection was successful
        success: bool,
    },
    
    /// Battery reading obtained
    BatteryReading,
    
    /// Feature used
    FeatureUsed(String),
}

/// Telemetry payload for upload
#[derive(Debug, Clone)]
struct TelemetryPayload {
    /// Anonymous installation ID
    installation_id: String,
    
    /// Application version
    app_version: String,
    
    /// Operating system
    os_name: String,
    
    /// Architecture
    os_arch: String,
    
    /// Error statistics
    error_stats: ErrorStats,
    
    /// Usage metrics
    usage_metrics: UsageMetrics,
    
    /// Timestamp
    timestamp: String,
}

/// Initialize telemetry
pub fn init_telemetry(config: &AppConfig) -> Arc<Mutex<TelemetryManager>> {
    let manager = TelemetryManager::new(config);
    
    // Record app start
    if manager.enabled {
        manager.record_event(TelemetryEvent::AppStart);
    }
    
    Arc::new(Mutex::new(manager))
} 