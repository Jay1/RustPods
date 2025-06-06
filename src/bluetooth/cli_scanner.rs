//! CLI Scanner Integration for AirPods Battery Monitoring
//!
//! This module provides efficient integration with the native C++ CLI scanner,
//! including smart polling intervals, JSON parsing, and resource management.

use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;
use tokio::time::interval;
use serde::{Deserialize, Serialize};

use crate::airpods::{AirPodsBattery, DetectedAirPods, AirPodsType, AirPodsChargingState};
use crate::bluetooth::BluetoothError;
use btleplug::api::BDAddr;
use crate::config::{AppConfig, LogLevel};

/// Default polling interval for CLI scanner (30 seconds)
const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(30);

/// Fast polling interval when changes are detected (10 seconds)
const FAST_POLL_INTERVAL: Duration = Duration::from_secs(10);

/// Minimum polling interval to prevent resource exhaustion (5 seconds)
const MIN_POLL_INTERVAL: Duration = Duration::from_secs(5);

/// Maximum polling interval for battery conservation (120 seconds)
const MAX_POLL_INTERVAL: Duration = Duration::from_secs(120);

/// Number of fast polls before returning to normal interval
const FAST_POLL_COUNT: u32 = 3;

/// CLI scanner timeout in seconds
const CLI_TIMEOUT: Duration = Duration::from_secs(10);

/// JSON structures for CLI scanner output
#[derive(Debug, Clone, Deserialize)]
pub struct CliScannerResult {
    pub scanner_version: String,
    pub scan_timestamp: String,
    pub total_devices: i32,
    pub devices: Vec<CliDeviceInfo>,
    pub airpods_count: i32,
    pub status: String,
    pub note: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CliDeviceInfo {
    pub device_id: String,
    pub address: String,
    pub rssi: i32,
    pub manufacturer_data_hex: String,
    pub airpods_data: Option<CliAirPodsData>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CliAirPodsData {
    pub model: String,
    pub model_id: String,
    pub left_battery: i32,
    pub right_battery: i32,
    pub case_battery: i32,
    pub left_charging: bool,
    pub right_charging: bool,
    pub case_charging: bool,
    pub left_in_ear: bool,
    pub right_in_ear: bool,
    pub both_in_case: bool,
    pub lid_open: bool,
    pub broadcasting_ear: String,
}

/// CLI Scanner configuration options
#[derive(Debug, Clone)]
pub struct CliScannerConfig {
    /// Path to the CLI scanner executable
    pub scanner_path: PathBuf,
    
    /// Base polling interval
    pub poll_interval: Duration,
    
    /// Whether to use adaptive polling
    pub adaptive_polling: bool,
    
    /// Maximum number of consecutive errors before stopping
    pub max_errors: u32,
    
    /// Whether to enable detailed logging
    pub verbose_logging: bool,
}

impl Default for CliScannerConfig {
    fn default() -> Self {
        Self {
            scanner_path: PathBuf::from("scripts/airpods_battery_cli/build/Release/airpods_battery_cli_v5.exe"),
            poll_interval: DEFAULT_POLL_INTERVAL,
            adaptive_polling: true,
            max_errors: 5,
            verbose_logging: false,
        }
    }
}

impl CliScannerConfig {
    /// Create configuration from app config
    pub fn from_app_config(config: &AppConfig) -> Self {
        Self {
            scanner_path: Self::resolve_scanner_path(&std::env::current_dir().unwrap_or_else(|_| ".".into())),
            poll_interval: Duration::from_secs(config.bluetooth.battery_refresh_interval.max(MIN_POLL_INTERVAL.as_secs())),
            adaptive_polling: config.bluetooth.adaptive_polling,
            max_errors: 5,
            verbose_logging: config.system.log_level == crate::config::LogLevel::Debug || config.system.log_level == crate::config::LogLevel::Trace,
        }
    }
    
    /// Resolve the scanner path based on build configuration
    fn resolve_scanner_path(project_root: &std::path::Path) -> PathBuf {
        let mut path = project_root.to_path_buf();
        path.push("scripts");
        path.push("airpods_battery_cli");
        path.push("build");
        
        // Try Release first, then Debug (v5 is the working implementation)
        let release_path = path.join("Release").join("airpods_battery_cli_v5.exe");
        if release_path.exists() {
            return release_path;
        }
        
        let debug_path = path.join("Debug").join("airpods_battery_cli_v5.exe");
        if debug_path.exists() {
            return debug_path;
        }
        
        // Fallback to relative path
        PathBuf::from("scripts/airpods_battery_cli/build/Release/airpods_battery_cli_v5.exe")
    }
}

/// CLI Scanner state for adaptive polling
#[derive(Debug)]
struct ScannerState {
    current_interval: Duration,
    fast_polls_remaining: u32,
    last_scan_time: Option<Instant>,
    last_airpods_data: Option<CliAirPodsData>,
    consecutive_errors: u32,
    total_scans: u64,
    successful_scans: u64,
}

impl Default for ScannerState {
    fn default() -> Self {
        Self {
            current_interval: DEFAULT_POLL_INTERVAL,
            fast_polls_remaining: 0,
            last_scan_time: None,
            last_airpods_data: None,
            consecutive_errors: 0,
            total_scans: 0,
            successful_scans: 0,
        }
    }
}

/// CLI Scanner integration for AirPods battery monitoring
pub struct CliScanner {
    config: CliScannerConfig,
    state: Arc<Mutex<ScannerState>>,
    #[allow(dead_code)]
    runtime_handle: Arc<tokio::runtime::Handle>,
}

impl CliScanner {
    /// Create a new CLI scanner
    pub fn new(config: CliScannerConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(ScannerState::default())),
            runtime_handle: Arc::new(tokio::runtime::Handle::current()),
        }
    }
    
    /// Start continuous monitoring with callback
    pub fn start_monitoring<F>(&self, callback: F) -> JoinHandle<()>
    where
        F: Fn(Result<Vec<DetectedAirPods>, BluetoothError>) + Send + 'static,
    {
        let config = self.config.clone();
        let state = Arc::clone(&self.state);
        
        tokio::spawn(async move {
            let mut interval_timer = interval(config.poll_interval);
            
            loop {
                interval_timer.tick().await;
                
                // Perform scan
                let scan_result = Self::execute_scan(&config).await;
                
                // Update state and determine next interval
                let next_interval = {
                    let mut state_guard = state.lock().unwrap();
                    state_guard.total_scans += 1;
                    state_guard.last_scan_time = Some(Instant::now());
                    
                    match &scan_result {
                        Ok(airpods_list) => {
                            state_guard.consecutive_errors = 0;
                            state_guard.successful_scans += 1;
                            
                            // Check for changes and update adaptive polling
                            let should_use_fast_polling = if config.adaptive_polling {
                                Self::detect_significant_changes(&mut state_guard, airpods_list)
                            } else {
                                false
                            };
                            
                            if should_use_fast_polling {
                                state_guard.fast_polls_remaining = FAST_POLL_COUNT;
                                state_guard.current_interval = FAST_POLL_INTERVAL;
                            } else if state_guard.fast_polls_remaining > 0 {
                                state_guard.fast_polls_remaining -= 1;
                                state_guard.current_interval = FAST_POLL_INTERVAL;
                            } else {
                                state_guard.current_interval = config.poll_interval;
                            }
                        }
                        Err(error) => {
                            state_guard.consecutive_errors += 1;
                            
                            if config.verbose_logging {
                                log::warn!("CLI scanner error (attempt {}): {}", 
                                         state_guard.consecutive_errors, error);
                            }
                            
                            // Exponential backoff on errors, but cap at max interval
                            let backoff_multiplier = (state_guard.consecutive_errors as u64).min(4);
                            let error_interval = config.poll_interval.mul_f64(1.5_f64.powi(backoff_multiplier as i32));
                            state_guard.current_interval = error_interval.min(MAX_POLL_INTERVAL);
                            
                            // Stop if too many consecutive errors
                            if state_guard.consecutive_errors >= config.max_errors {
                                log::error!("CLI scanner stopped after {} consecutive errors", 
                                           state_guard.consecutive_errors);
                                break;
                            }
                        }
                    }
                    
                    state_guard.current_interval
                };
                
                // Call the callback with results
                callback(scan_result);
                
                // Update interval timer
                interval_timer = interval(next_interval);
                
                // Log statistics periodically
                if config.verbose_logging {
                    let state_guard = state.lock().unwrap();
                    if state_guard.total_scans % 10 == 0 {
                        let success_rate = (state_guard.successful_scans as f64 / state_guard.total_scans as f64) * 100.0;
                        log::info!("CLI scanner stats: {} total scans, {:.1}% success rate, current interval: {:?}",
                                 state_guard.total_scans, success_rate, state_guard.current_interval);
                    }
                }
            }
        })
    }
    
    /// Execute a single scan
    async fn execute_scan(config: &CliScannerConfig) -> Result<Vec<DetectedAirPods>, BluetoothError> {
        let start_time = Instant::now();
        
        // Spawn the CLI process
        let output = tokio::process::Command::new(&config.scanner_path)
            .output()
            .await
            .map_err(|e| BluetoothError::Other(format!("Failed to execute CLI scanner: {}", e)))?;
        
        let execution_time = start_time.elapsed();
        
        // Check if execution took too long
        if execution_time > CLI_TIMEOUT {
            return Err(BluetoothError::Timeout(execution_time));
        }
        
        // Check exit status
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BluetoothError::Other(format!("CLI scanner failed: {}", stderr)));
        }
        
        // Parse JSON output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let scanner_result: CliScannerResult = serde_json::from_str(&stdout)
            .map_err(|e| BluetoothError::InvalidData(format!("Failed to parse CLI output: {}", e)))?;
        
        // Convert CLI results to DetectedAirPods
        let mut airpods_list = Vec::new();
        
        for device in scanner_result.devices {
            if let Some(airpods_data) = device.airpods_data {
                let detected_airpods = Self::convert_cli_data_to_airpods(device.device_id, airpods_data)?;
                airpods_list.push(detected_airpods);
            }
        }
        
        if config.verbose_logging {
            log::debug!("CLI scanner found {} AirPods devices in {:?}", 
                       airpods_list.len(), execution_time);
        }
        
        Ok(airpods_list)
    }
    
    /// Convert CLI data to DetectedAirPods
    fn convert_cli_data_to_airpods(device_id: String, cli_data: CliAirPodsData) -> Result<DetectedAirPods, BluetoothError> {
        // Map model string to AirPodsType
        let airpods_type = match cli_data.model.as_str() {
            "AirPods 1" => AirPodsType::AirPods1,
            "AirPods 2" => AirPodsType::AirPods2,
            "AirPods 3" => AirPodsType::AirPods3,
            "AirPods Pro" => AirPodsType::AirPodsPro,
            "AirPods Pro 2" => AirPodsType::AirPodsPro2,
            "AirPods Max" => AirPodsType::AirPodsMax,
            _ => AirPodsType::Unknown,
        };
        
        // Convert battery levels (CLI uses -1 for unavailable, we use None)
        let charging_state = if cli_data.left_charging && cli_data.right_charging {
            AirPodsChargingState::BothBudsCharging
        } else if cli_data.left_charging {
            AirPodsChargingState::LeftCharging
        } else if cli_data.right_charging {
            AirPodsChargingState::RightCharging
        } else if cli_data.case_charging {
            AirPodsChargingState::CaseCharging
        } else {
            AirPodsChargingState::NotCharging
        };

        let battery = AirPodsBattery {
            left: if cli_data.left_battery >= 0 { Some(cli_data.left_battery as u8) } else { None },
            right: if cli_data.right_battery >= 0 { Some(cli_data.right_battery as u8) } else { None },
            case: if cli_data.case_battery >= 0 { Some(cli_data.case_battery as u8) } else { None },
            charging: Some(charging_state),
        };
        
        // Create a BDAddr from device_id (using a simple conversion for now)
        let address = BDAddr::from([0; 6]); // TODO: Parse actual address from device_id
        
        Ok(DetectedAirPods {
            address,
            name: Some(format!("{} ({})", cli_data.model, device_id)),
            rssi: None, // CLI scanner doesn't provide RSSI
            device_type: airpods_type,
            battery: Some(battery),
            last_seen: std::time::Instant::now(),
            is_connected: false, // CLI scanner doesn't provide connection status
        })
    }
    
    /// Detect significant changes that warrant faster polling
    fn detect_significant_changes(state: &mut ScannerState, current_airpods: &[DetectedAirPods]) -> bool {
        // If this is the first scan, no changes to detect
        let previous_data = match &state.last_airpods_data {
            Some(data) => data,
            None => {
                // Store current data for next comparison
                if let Some(airpods) = current_airpods.first() {
                    state.last_airpods_data = Some(Self::airpods_to_cli_data(airpods));
                }
                return false;
            }
        };
        
        // Check if we found new AirPods or lost existing ones
        if current_airpods.is_empty() {
            return !current_airpods.is_empty(); // Change if we had data before
        }
        
        if let Some(current) = current_airpods.first() {
            let current_cli_data = Self::airpods_to_cli_data(current);
            
            // Detect significant changes
            let battery_change = Self::significant_battery_change(previous_data, &current_cli_data);
            let charging_change = Self::charging_state_change(previous_data, &current_cli_data);
            let usage_change = Self::usage_state_change(previous_data, &current_cli_data);
            
            // Update stored data
            state.last_airpods_data = Some(current_cli_data);
            
            battery_change || charging_change || usage_change
        } else {
            false
        }
    }
    
    /// Convert DetectedAirPods back to CliAirPodsData for comparison
    fn airpods_to_cli_data(airpods: &DetectedAirPods) -> CliAirPodsData {
        let default_battery = AirPodsBattery::default();
        let battery = airpods.battery.as_ref().unwrap_or(&default_battery);
        let charging = battery.charging.unwrap_or(AirPodsChargingState::NotCharging);
        
        CliAirPodsData {
            model: format!("{:?}", airpods.device_type),
            model_id: "".to_string(),
            left_battery: battery.left.map(|b| b as i32).unwrap_or(-1),
            right_battery: battery.right.map(|b| b as i32).unwrap_or(-1),
            case_battery: battery.case.map(|b| b as i32).unwrap_or(-1),
            left_charging: charging.is_left_charging(),
            right_charging: charging.is_right_charging(),
            case_charging: charging.is_case_charging(),
            left_in_ear: false, // Not available in DetectedAirPods
            right_in_ear: false, // Not available in DetectedAirPods
            both_in_case: false, // Not available in DetectedAirPods
            lid_open: false, // Not available in DetectedAirPods
            broadcasting_ear: "unknown".to_string(),
        }
    }
    
    /// Check for significant battery level changes (>10% change)
    fn significant_battery_change(prev: &CliAirPodsData, curr: &CliAirPodsData) -> bool {
        const THRESHOLD: i32 = 10;
        
        let left_change = (prev.left_battery - curr.left_battery).abs();
        let right_change = (prev.right_battery - curr.right_battery).abs();
        let case_change = (prev.case_battery - curr.case_battery).abs();
        
        left_change >= THRESHOLD || right_change >= THRESHOLD || case_change >= THRESHOLD
    }
    
    /// Check for charging state changes
    fn charging_state_change(prev: &CliAirPodsData, curr: &CliAirPodsData) -> bool {
        prev.left_charging != curr.left_charging ||
        prev.right_charging != curr.right_charging ||
        prev.case_charging != curr.case_charging
    }
    
    /// Check for usage state changes (in-ear, case lid)
    fn usage_state_change(prev: &CliAirPodsData, curr: &CliAirPodsData) -> bool {
        prev.left_in_ear != curr.left_in_ear ||
        prev.right_in_ear != curr.right_in_ear ||
        prev.both_in_case != curr.both_in_case ||
        prev.lid_open != curr.lid_open
    }
    
    /// Get current scanner statistics
    pub fn get_stats(&self) -> ScannerStats {
        let state = self.state.lock().unwrap();
        ScannerStats {
            total_scans: state.total_scans,
            successful_scans: state.successful_scans,
            consecutive_errors: state.consecutive_errors,
            current_interval: state.current_interval,
            last_scan_time: state.last_scan_time,
            success_rate: if state.total_scans > 0 {
                (state.successful_scans as f64 / state.total_scans as f64) * 100.0
            } else {
                0.0
            },
        }
    }
}

/// Scanner statistics for monitoring and debugging
#[derive(Debug, Clone)]
pub struct ScannerStats {
    pub total_scans: u64,
    pub successful_scans: u64,
    pub consecutive_errors: u32,
    pub current_interval: Duration,
    pub last_scan_time: Option<Instant>,
    pub success_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_scanner_config_default() {
        let config = CliScannerConfig::default();
        assert_eq!(config.poll_interval, DEFAULT_POLL_INTERVAL);
        assert!(config.adaptive_polling);
        assert_eq!(config.max_errors, 5);
    }
    
    #[test]
    fn test_significant_battery_change_detection() {
        let prev = CliAirPodsData {
            model: "AirPods Pro 2".to_string(),
            model_id: "0x2014".to_string(),
            left_battery: 80,
            right_battery: 75,
            case_battery: 60,
            left_charging: false,
            right_charging: false,
            case_charging: false,
            left_in_ear: false,
            right_in_ear: false,
            both_in_case: true,
            lid_open: false,
            broadcasting_ear: "left".to_string(),
        };
        
        let curr = CliAirPodsData {
            left_battery: 65, // 15% drop - significant
            right_battery: 70, // 5% drop - not significant
            ..prev.clone()
        };
        
        assert!(CliScanner::significant_battery_change(&prev, &curr));
        
        let curr_small = CliAirPodsData {
            left_battery: 75, // 5% drop - not significant
            right_battery: 70, // 5% drop - not significant
            ..prev.clone()
        };
        
        assert!(!CliScanner::significant_battery_change(&prev, &curr_small));
    }
    
    #[test]
    fn test_charging_state_change_detection() {
        let prev = CliAirPodsData {
            model: "AirPods Pro 2".to_string(),
            model_id: "0x2014".to_string(),
            left_battery: 80,
            right_battery: 75,
            case_battery: 60,
            left_charging: false,
            right_charging: false,
            case_charging: false,
            left_in_ear: false,
            right_in_ear: false,
            both_in_case: true,
            lid_open: false,
            broadcasting_ear: "left".to_string(),
        };
        
        let curr = CliAirPodsData {
            case_charging: true, // Started charging
            ..prev.clone()
        };
        
        assert!(CliScanner::charging_state_change(&prev, &curr));
        
        let curr_same = prev.clone();
        assert!(!CliScanner::charging_state_change(&prev, &curr_same));
    }
} 