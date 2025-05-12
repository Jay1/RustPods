//! Enhanced battery status monitoring for Bluetooth devices

use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::collections::VecDeque;
use tokio::task::JoinHandle;
use tokio::time::interval;
use log;

use crate::airpods::{AirPodsBattery, DetectedAirPods};
use crate::bluetooth::{AirPodsBatteryStatus, BleError};
use crate::config::AppConfig;

/// Size of the battery reading buffer for smoothing
const BATTERY_BUFFER_SIZE: usize = 3;

/// Default polling interval in seconds
const DEFAULT_POLLING_INTERVAL: u64 = 5;

/// Minimum polling interval in seconds
const MIN_POLLING_INTERVAL: u64 = 2;

/// Maximum polling interval in seconds
const MAX_POLLING_INTERVAL: u64 = 30;

/// Low battery threshold (percentage)
const LOW_BATTERY_THRESHOLD: u8 = 20;

/// Default level change threshold for adaptive polling
const DEFAULT_CHANGE_THRESHOLD: u8 = 5;

/// Options for battery monitoring
#[derive(Debug, Clone)]
pub struct BatteryMonitorOptions {
    /// Initial polling interval in seconds
    pub polling_interval: u64,
    
    /// Whether to use adaptive polling interval
    pub adaptive_polling: bool,
    
    /// Minimum level change to trigger faster polling (percentage)
    pub change_threshold: u8,
    
    /// Whether to use buffer for smoothing readings
    pub use_smoothing: bool,
    
    /// Low battery threshold in percentage
    pub low_battery_threshold: u8,
    
    /// Whether to notify on low battery
    pub notify_low_battery: bool,
}

impl Default for BatteryMonitorOptions {
    fn default() -> Self {
        Self {
            polling_interval: DEFAULT_POLLING_INTERVAL,
            adaptive_polling: true,
            change_threshold: DEFAULT_CHANGE_THRESHOLD,
            use_smoothing: true,
            low_battery_threshold: LOW_BATTERY_THRESHOLD,
            notify_low_battery: true,
        }
    }
}

impl BatteryMonitorOptions {
    /// Create options from app config
    pub fn from_config(config: &AppConfig) -> Self {
        Self {
            polling_interval: config.bluetooth.battery_refresh_interval,
            adaptive_polling: config.bluetooth.adaptive_polling,
            change_threshold: config.battery.change_threshold,
            use_smoothing: config.battery.smoothing_enabled,
            low_battery_threshold: config.battery.low_threshold,
            notify_low_battery: config.battery.notify_low,
        }
    }
}

/// Battery reading buffer for smoothing readings
#[derive(Debug, Clone)]
struct BatteryBuffer {
    /// Buffer for left earbud readings
    left_buffer: VecDeque<u8>,
    
    /// Buffer for right earbud readings
    right_buffer: VecDeque<u8>,
    
    /// Buffer for case readings
    case_buffer: VecDeque<u8>,
    
    /// Maximum buffer size
    max_size: usize,
}

impl BatteryBuffer {
    /// Create a new battery buffer
    pub fn new(max_size: usize) -> Self {
        Self {
            left_buffer: VecDeque::with_capacity(max_size),
            right_buffer: VecDeque::with_capacity(max_size),
            case_buffer: VecDeque::with_capacity(max_size),
            max_size,
        }
    }
    
    /// Add a battery reading to the buffer
    pub fn add_reading(&mut self, battery: &AirPodsBattery) {
        // Add left reading if available
        if let Some(left) = battery.left {
            self.left_buffer.push_back(left);
            if self.left_buffer.len() > self.max_size {
                self.left_buffer.pop_front();
            }
        }
        
        // Add right reading if available
        if let Some(right) = battery.right {
            self.right_buffer.push_back(right);
            if self.right_buffer.len() > self.max_size {
                self.right_buffer.pop_front();
            }
        }
        
        // Add case reading if available
        if let Some(case) = battery.case {
            self.case_buffer.push_back(case);
            if self.case_buffer.len() > self.max_size {
                self.case_buffer.pop_front();
            }
        }
    }
    
    /// Get the smoothed battery reading
    pub fn get_smoothed_reading(&self, current: &AirPodsBattery) -> AirPodsBattery {
        // Smoothed readings
        let left = if !self.left_buffer.is_empty() {
            Some(self.calculate_average(&self.left_buffer))
        } else {
            current.left
        };
        
        let right = if !self.right_buffer.is_empty() {
            Some(self.calculate_average(&self.right_buffer))
        } else {
            current.right
        };
        
        let case = if !self.case_buffer.is_empty() {
            Some(self.calculate_average(&self.case_buffer))
        } else {
            current.case
        };
        
        // Create smoothed battery with current charging status
        AirPodsBattery {
            left,
            right,
            case,
            charging: current.charging.clone(),
        }
    }
    
    /// Calculate the average of a buffer
    fn calculate_average(&self, buffer: &VecDeque<u8>) -> u8 {
        if buffer.is_empty() {
            return 0;
        }
        
        let sum: u32 = buffer.iter().map(|&x| x as u32).sum();
        (sum / buffer.len() as u32) as u8
    }
    
    /// Clear the buffer
    pub fn clear(&mut self) {
        self.left_buffer.clear();
        self.right_buffer.clear();
        self.case_buffer.clear();
    }
}

/// Enhanced battery monitor for AirPods devices
pub struct BatteryMonitor {
    /// Options for the battery monitor
    options: BatteryMonitorOptions,
    
    /// Buffer for smoothing readings
    buffer: BatteryBuffer,
    
    /// Last valid battery reading
    last_valid_reading: Option<AirPodsBattery>,
    
    /// Current polling interval
    current_interval: Duration,
    
    /// Whether any low battery notifications have been sent
    low_battery_notified: bool,
    
    /// Last notification time for each component
    last_notification: std::collections::HashMap<String, chrono::DateTime<chrono::Utc>>,
}

impl BatteryMonitor {
    /// Create a new battery monitor with default options
    pub fn new() -> Self {
        Self::with_options(BatteryMonitorOptions::default())
    }
    
    /// Create a new battery monitor with the specified options
    pub fn with_options(options: BatteryMonitorOptions) -> Self {
        Self {
            current_interval: Duration::from_secs(options.polling_interval),
            buffer: BatteryBuffer::new(BATTERY_BUFFER_SIZE),
            last_valid_reading: None,
            low_battery_notified: false,
            last_notification: std::collections::HashMap::new(),
            options,
        }
    }
    
    /// Start monitoring battery status for a device
    pub fn start_monitoring<F>(
        self,
        device: Arc<Mutex<Option<DetectedAirPods>>>,
        callback: F,
        runtime_handle: Arc<tokio::runtime::Handle>,
    ) -> JoinHandle<()>
    where
        F: Fn(AirPodsBatteryStatus, Option<BatteryAlert>) + Send + 'static,
    {
        // Create Arc for shared state
        let monitor = Arc::new(Mutex::new(self));
        
        // Create task to monitor battery status
        runtime_handle.spawn(async move {
            // Create interval timer with initial polling interval
            let initial_interval = {
                let guard = monitor.lock().unwrap();
                guard.current_interval
            };
            
            let mut interval_timer = interval(initial_interval);
            
            loop {
                // Wait for next interval
                interval_timer.tick().await;
                
                // Get the connected device if available
                let device_opt = {
                    let guard = device.lock().unwrap();
                    guard.clone()
                };
                
                // Skip this cycle if no device is connected
                if device_opt.is_none() {
                    // Reset the battery buffer when no device is connected
                    let mut monitor_guard = monitor.lock().unwrap();
                    monitor_guard.buffer.clear();
                    monitor_guard.last_valid_reading = None;
                    monitor_guard.low_battery_notified = false;
                    continue;
                }
                
                let airpods = device_opt.unwrap();
                
                // Process battery data from device
                let (status, alert) = {
                    let mut monitor_guard = monitor.lock().unwrap();
                    
                    // Get raw battery data from device
                    let raw_battery = airpods.battery.clone();
                    
                    // Validate the battery reading
                    if !monitor_guard.is_valid_battery(&raw_battery) {
                        // If we have a previous valid reading, use that instead
                        if let Some(last_valid) = &monitor_guard.last_valid_reading {
                            // Create battery status with last valid reading
                            let status = AirPodsBatteryStatus::new(last_valid.clone());
                            (status, None)
                        } else {
                            // No valid reading available, skip this cycle
                            continue;
                        }
                    } else {
                        // Store valid reading
                        monitor_guard.last_valid_reading = Some(raw_battery.clone());
                        
                        // Add to buffer for smoothing
                        monitor_guard.buffer.add_reading(&raw_battery);
                        
                        // Get smoothed reading if enabled
                        let battery = if monitor_guard.options.use_smoothing {
                            monitor_guard.buffer.get_smoothed_reading(&raw_battery)
                        } else {
                            raw_battery.clone()
                        };
                        
                        // Create battery status
                        let status = AirPodsBatteryStatus::new(battery.clone());
                        
                        // Check for low battery
                        let alert = if monitor_guard.options.notify_low_battery {
                            monitor_guard.check_low_battery(&battery)
                        } else {
                            None
                        };
                        
                        // Adjust polling interval based on battery data
                        if monitor_guard.options.adaptive_polling {
                            let new_interval = monitor_guard.calculate_adaptive_interval(&battery);
                            if new_interval != monitor_guard.current_interval {
                                monitor_guard.current_interval = new_interval;
                                // Update the interval timer
                                interval_timer = interval(new_interval);
                            }
                        }
                        
                        (status, alert)
                    }
                };
                
                // Call the callback with battery status and alert
                callback(status, alert);
            }
        })
    }
    
    /// Check if a battery reading is valid
    fn is_valid_battery(&self, battery: &AirPodsBattery) -> bool {
        // Basic validation: at least one component should have a reading
        let has_data = battery.left.is_some() || battery.right.is_some() || battery.case.is_some();
        
        // Check for unreasonable values (should be 0-100)
        let valid_range = battery.left.map_or(true, |v| v <= 100) &&
                          battery.right.map_or(true, |v| v <= 100) &&
                          battery.case.map_or(true, |v| v <= 100);
        
        // Check for impossible charging status
        let valid_charging = !battery.charging.left || battery.left.is_some();
        let valid_charging_right = !battery.charging.right || battery.right.is_some();
        
        has_data && valid_range && valid_charging && valid_charging_right
    }
    
    /// Calculate adaptive polling interval based on battery status
    fn calculate_adaptive_interval(&self, battery: &AirPodsBattery) -> Duration {
        // Default to current interval
        let mut new_interval = self.current_interval;
        
        // Check if we have a previous reading to compare
        if let Some(last) = &self.last_valid_reading {
            // Check if there has been a significant change in any component
            let significant_change = self.has_significant_change(last, battery);
            
            // Adjust interval based on changes and charging status
            if significant_change || battery.charging.is_any_charging() {
                // Faster polling if there are significant changes or device is charging
                new_interval = Duration::from_secs(MIN_POLLING_INTERVAL);
            } else {
                // Get lowest battery level (if available)
                let min_level = self.get_minimum_battery_level(battery);
                
                // Adjust interval based on lowest battery level
                if let Some(level) = min_level {
                    if level <= self.options.low_battery_threshold {
                        // More frequent updates for low battery
                        new_interval = Duration::from_secs(MIN_POLLING_INTERVAL);
                    } else if level < 50 {
                        // Medium frequency for mid-range battery
                        new_interval = Duration::from_secs(self.options.polling_interval);
                    } else {
                        // Slower updates for high battery
                        new_interval = Duration::from_secs(MAX_POLLING_INTERVAL);
                    }
                } else {
                    // No battery level available, use default
                    new_interval = Duration::from_secs(self.options.polling_interval);
                }
            }
        }
        
        new_interval
    }
    
    /// Check if there has been a significant change in battery levels
    fn has_significant_change(&self, last: &AirPodsBattery, current: &AirPodsBattery) -> bool {
        // Check left earbud
        let left_change = match (last.left, current.left) {
            (Some(l1), Some(l2)) => (l1 as i32 - l2 as i32).abs() >= self.options.change_threshold as i32,
            (None, Some(_)) | (Some(_), None) => true, // Component appeared or disappeared
            _ => false,
        };
        
        // Check right earbud
        let right_change = match (last.right, current.right) {
            (Some(r1), Some(r2)) => (r1 as i32 - r2 as i32).abs() >= self.options.change_threshold as i32,
            (None, Some(_)) | (Some(_), None) => true, // Component appeared or disappeared
            _ => false,
        };
        
        // Check case
        let case_change = match (last.case, current.case) {
            (Some(c1), Some(c2)) => (c1 as i32 - c2 as i32).abs() >= self.options.change_threshold as i32,
            (None, Some(_)) | (Some(_), None) => true, // Component appeared or disappeared
            _ => false,
        };
        
        // Check charging status changes
        let charging_change = last.charging.left != current.charging.left ||
                               last.charging.right != current.charging.right ||
                               last.charging.case != current.charging.case;
        
        // Return true if any component has changed significantly
        left_change || right_change || case_change || charging_change
    }
    
    /// Get the minimum battery level across all components
    fn get_minimum_battery_level(&self, battery: &AirPodsBattery) -> Option<u8> {
        let mut levels = Vec::with_capacity(3);
        
        if let Some(left) = battery.left {
            levels.push(left);
        }
        
        if let Some(right) = battery.right {
            levels.push(right);
        }
        
        if let Some(case) = battery.case {
            levels.push(case);
        }
        
        if levels.is_empty() {
            None
        } else {
            Some(*levels.iter().min().unwrap())
        }
    }
    
    /// Check for low battery conditions
    fn check_low_battery(&mut self, battery: &AirPodsBattery) -> Option<BatteryAlert> {
        // Get current time for rate limiting notifications
        let now = chrono::Utc::now();
        
        // Check left earbud
        if let Some(left) = battery.left {
            if left <= self.options.low_battery_threshold && !battery.charging.left {
                // Check if we've already notified about left earbud recently
                let should_notify = match self.last_notification.get("left") {
                    Some(last_time) => {
                        // Only notify again after 30 minutes
                        (now - *last_time).num_minutes() >= 30
                    },
                    None => true,
                };
                
                if should_notify {
                    // Update last notification time
                    self.last_notification.insert("left".to_string(), now);
                    return Some(BatteryAlert::LowBattery("Left AirPod".to_string(), left));
                }
            }
        }
        
        // Check right earbud
        if let Some(right) = battery.right {
            if right <= self.options.low_battery_threshold && !battery.charging.right {
                // Check if we've already notified about right earbud recently
                let should_notify = match self.last_notification.get("right") {
                    Some(last_time) => {
                        // Only notify again after 30 minutes
                        (now - *last_time).num_minutes() >= 30
                    },
                    None => true,
                };
                
                if should_notify {
                    // Update last notification time
                    self.last_notification.insert("right".to_string(), now);
                    return Some(BatteryAlert::LowBattery("Right AirPod".to_string(), right));
                }
            }
        }
        
        // Check case
        if let Some(case) = battery.case {
            if case <= self.options.low_battery_threshold && !battery.charging.case {
                // Check if we've already notified about case recently
                let should_notify = match self.last_notification.get("case") {
                    Some(last_time) => {
                        // Only notify again after 30 minutes
                        (now - *last_time).num_minutes() >= 30
                    },
                    None => true,
                };
                
                if should_notify {
                    // Update last notification time
                    self.last_notification.insert("case".to_string(), now);
                    return Some(BatteryAlert::LowBattery("AirPods Case".to_string(), case));
                }
            }
        }
        
        None
    }
}

/// Battery alert types
#[derive(Debug, Clone)]
pub enum BatteryAlert {
    /// Low battery alert with component name and level
    LowBattery(String, u8),
    
    /// Charging complete alert with component name
    ChargingComplete(String),
    
    /// Battery level increased significantly
    BatteryIncreased(String, u8),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::airpods::ChargingStatus;
    
    #[test]
    fn test_battery_buffer() {
        let mut buffer = BatteryBuffer::new(3);
        
        // Add first reading
        let battery1 = AirPodsBattery {
            left: Some(50),
            right: Some(60),
            case: Some(70),
            charging: ChargingStatus { left: false, right: false, case: false },
        };
        
        buffer.add_reading(&battery1);
        
        // Check buffer contents
        assert_eq!(buffer.left_buffer.len(), 1);
        assert_eq!(buffer.right_buffer.len(), 1);
        assert_eq!(buffer.case_buffer.len(), 1);
        
        // Add second reading
        let battery2 = AirPodsBattery {
            left: Some(52),
            right: Some(58),
            case: Some(72),
            charging: ChargingStatus { left: false, right: false, case: false },
        };
        
        buffer.add_reading(&battery2);
        
        // Check buffer contents
        assert_eq!(buffer.left_buffer.len(), 2);
        assert_eq!(buffer.right_buffer.len(), 2);
        assert_eq!(buffer.case_buffer.len(), 2);
        
        // Get smoothed reading
        let smoothed = buffer.get_smoothed_reading(&battery2);
        
        // Verify smoothed values are averages
        assert_eq!(smoothed.left, Some(51)); // (50 + 52) / 2
        assert_eq!(smoothed.right, Some(59)); // (60 + 58) / 2
        assert_eq!(smoothed.case, Some(71)); // (70 + 72) / 2
        
        // Add more readings to test buffer size limit
        buffer.add_reading(&AirPodsBattery {
            left: Some(54),
            right: Some(56),
            case: Some(74),
            charging: ChargingStatus { left: false, right: false, case: false },
        });
        
        buffer.add_reading(&AirPodsBattery {
            left: Some(56),
            right: Some(54),
            case: Some(76),
            charging: ChargingStatus { left: false, right: false, case: false },
        });
        
        // Check buffer size is limited to max_size
        assert_eq!(buffer.left_buffer.len(), 3);
        assert_eq!(buffer.right_buffer.len(), 3);
        assert_eq!(buffer.case_buffer.len(), 3);
        
        // First reading should be removed
        assert_eq!(buffer.left_buffer[0], 52);
    }
    
    #[test]
    fn test_battery_validation() {
        let monitor = BatteryMonitor::new();
        
        // Valid battery
        let valid = AirPodsBattery {
            left: Some(50),
            right: Some(60),
            case: Some(70),
            charging: ChargingStatus { left: false, right: false, case: false },
        };
        
        assert!(monitor.is_valid_battery(&valid));
        
        // Invalid: no data
        let no_data = AirPodsBattery {
            left: None,
            right: None,
            case: None,
            charging: ChargingStatus { left: false, right: false, case: false },
        };
        
        assert!(!monitor.is_valid_battery(&no_data));
        
        // Invalid: out of range
        let out_of_range = AirPodsBattery {
            left: Some(120),
            right: Some(60),
            case: Some(70),
            charging: ChargingStatus { left: false, right: false, case: false },
        };
        
        assert!(!monitor.is_valid_battery(&out_of_range));
        
        // Invalid: impossible charging
        let impossible_charging = AirPodsBattery {
            left: None,
            right: Some(60),
            case: Some(70),
            charging: ChargingStatus { left: true, right: false, case: false },
        };
        
        assert!(!monitor.is_valid_battery(&impossible_charging));
    }
    
    #[test]
    fn test_significant_change_detection() {
        let options = BatteryMonitorOptions {
            change_threshold: 5,
            ..Default::default()
        };
        
        let monitor = BatteryMonitor::with_options(options);
        
        // Initial battery
        let battery1 = AirPodsBattery {
            left: Some(50),
            right: Some(60),
            case: Some(70),
            charging: ChargingStatus { left: false, right: false, case: false },
        };
        
        // Small change (below threshold)
        let battery2 = AirPodsBattery {
            left: Some(52),
            right: Some(58),
            case: Some(72),
            charging: ChargingStatus { left: false, right: false, case: false },
        };
        
        assert!(!monitor.has_significant_change(&battery1, &battery2));
        
        // Large change (above threshold)
        let battery3 = AirPodsBattery {
            left: Some(60),
            right: Some(60),
            case: Some(70),
            charging: ChargingStatus { left: false, right: false, case: false },
        };
        
        assert!(monitor.has_significant_change(&battery1, &battery3));
        
        // Change in charging status
        let battery4 = AirPodsBattery {
            left: Some(50),
            right: Some(60),
            case: Some(70),
            charging: ChargingStatus { left: true, right: false, case: false },
        };
        
        assert!(monitor.has_significant_change(&battery1, &battery4));
        
        // Component disappearance
        let battery5 = AirPodsBattery {
            left: None,
            right: Some(60),
            case: Some(70),
            charging: ChargingStatus { left: false, right: false, case: false },
        };
        
        assert!(monitor.has_significant_change(&battery1, &battery5));
    }
    
    #[test]
    fn test_adaptive_interval() {
        let options = BatteryMonitorOptions {
            polling_interval: 10,
            adaptive_polling: true,
            ..Default::default()
        };
        
        let mut monitor = BatteryMonitor::with_options(options);
        
        // Set initial reading
        let battery1 = AirPodsBattery {
            left: Some(80),
            right: Some(85),
            case: Some(90),
            charging: ChargingStatus { left: false, right: false, case: false },
        };
        
        monitor.last_valid_reading = Some(battery1.clone());
        
        // High battery level should result in slower polling
        let interval = monitor.calculate_adaptive_interval(&battery1);
        assert_eq!(interval, Duration::from_secs(MAX_POLLING_INTERVAL));
        
        // Low battery should result in faster polling
        let low_battery = AirPodsBattery {
            left: Some(15),
            right: Some(85),
            case: Some(90),
            charging: ChargingStatus { left: false, right: false, case: false },
        };
        
        let interval = monitor.calculate_adaptive_interval(&low_battery);
        assert_eq!(interval, Duration::from_secs(MIN_POLLING_INTERVAL));
        
        // Charging should result in faster polling
        let charging_battery = AirPodsBattery {
            left: Some(80),
            right: Some(85),
            case: Some(90),
            charging: ChargingStatus { left: true, right: false, case: false },
        };
        
        let interval = monitor.calculate_adaptive_interval(&charging_battery);
        assert_eq!(interval, Duration::from_secs(MIN_POLLING_INTERVAL));
    }
} 