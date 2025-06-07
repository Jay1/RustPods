//! Enhanced battery status monitoring for Bluetooth devices

use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::collections::VecDeque;
use tokio::task::JoinHandle;
use tokio::time::interval;
use std::time::Instant;

use crate::airpods::{AirPodsBattery, DetectedAirPods, AirPodsChargingState};
use crate::bluetooth::AirPodsBatteryStatus;
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
    
    /// Runtime handle to spawn tasks on
    pub _runtime_handle: Arc<tokio::runtime::Handle>,
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
            _runtime_handle: Arc::new(tokio::runtime::Handle::current()),
        }
    }
}

impl BatteryMonitorOptions {
    /// Create options from app config
    pub fn from_config(config: &AppConfig) -> Self {
        Self {
            polling_interval: config.bluetooth.battery_refresh_interval.as_secs(),
            adaptive_polling: config.bluetooth.adaptive_polling,
            change_threshold: config.battery.change_threshold,
            use_smoothing: config.battery.smoothing_enabled,
            low_battery_threshold: config.battery.low_threshold,
            notify_low_battery: config.battery.notify_low,
            _runtime_handle: Arc::new(tokio::runtime::Handle::current()),
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
            left_buffer: VecDeque::new(),
            right_buffer: VecDeque::new(),
            case_buffer: VecDeque::new(),
            max_size,
        }
    }
    
    /// Add a battery reading to the buffer
    pub fn add_reading(&mut self, battery: &AirPodsBattery) {
        // Add left battery reading
        if let Some(left) = battery.left {
            self.left_buffer.push_back(left);
            // Trim if too large
            if self.left_buffer.len() > self.max_size {
                self.left_buffer.pop_front();
            }
        }
        
        // Add right battery reading
        if let Some(right) = battery.right {
            self.right_buffer.push_back(right);
            // Trim if too large
            if self.right_buffer.len() > self.max_size {
                self.right_buffer.pop_front();
            }
        }
        
        // Add case battery reading
        if let Some(case) = battery.case {
            self.case_buffer.push_back(case);
            // Trim if too large
            if self.case_buffer.len() > self.max_size {
                self.case_buffer.pop_front();
            }
        }
    }
    
    /// Get the average left earbud battery level
    pub fn get_average_left(&self) -> Option<u8> {
        self.calculate_average(&self.left_buffer)
    }
    
    /// Get the average right earbud battery level
    pub fn get_average_right(&self) -> Option<u8> {
        self.calculate_average(&self.right_buffer)
    }
    
    /// Get the average case battery level
    pub fn get_average_case(&self) -> Option<u8> {
        self.calculate_average(&self.case_buffer)
    }
    
    /// Calculate the average battery level from a buffer
    fn calculate_average(&self, buffer: &VecDeque<u8>) -> Option<u8> {
        if buffer.is_empty() {
            return None;
        }
        
        let sum: u32 = buffer.iter().map(|&v| v as u32).sum();
        Some((sum / buffer.len() as u32) as u8)
    }
    
    /// Clear the buffer
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.left_buffer.clear();
        self.right_buffer.clear();
        self.case_buffer.clear();
    }
    
    /// Get a smoothed reading based on the buffer contents and current reading
    pub fn get_smoothed_reading(&self, current: &AirPodsBattery) -> AirPodsBattery {
        // Return smoothed battery levels
        AirPodsBattery {
            left: if let Some(current_left) = current.left {
                if self.left_buffer.is_empty() {
                    Some(current_left)
                } else {
                    self.get_average_left()
                }
            } else {
                None
            },
            right: if let Some(current_right) = current.right {
                if self.right_buffer.is_empty() {
                    Some(current_right)
                } else {
                    self.get_average_right()
                }
            } else {
                None
            },
            case: if let Some(current_case) = current.case {
                if self.case_buffer.is_empty() {
                    Some(current_case)
                } else {
                    self.get_average_case()
                }
            } else {
                None
            },
            // Keep the current charging status
            charging: current.charging,
        }
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
    #[allow(dead_code)]
    low_battery_notified: bool,
    
    /// Last notification time for each component
    last_notification: std::collections::HashMap<String, Instant>,
}

impl Default for BatteryMonitor {
    fn default() -> Self {
        Self::new()
    }
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
        _runtime_handle: Arc<tokio::runtime::Handle>,
    ) -> JoinHandle<()>
    where
        F: Fn(AirPodsBatteryStatus, Option<BatteryAlert>) + Send + 'static,
    {
        // Clone options for use in closure
        let options = self.options.clone();
        
        // Create monitor (tricky to move self into the async closure)
        let monitor = Arc::new(Mutex::new(self));
        
        // Start the monitoring task
        tokio::spawn(async move {
            // Create an interval based on the polling interval
            let mut timer = interval(Duration::from_secs(options.polling_interval));
            
            loop {
                // Wait for the next interval tick
                timer.tick().await;
                
                // Get the current device, if any
                let current_device = {
                    // Lock the device mutex for a minimum time
                    let device_guard = device.lock().unwrap();
                    device_guard.clone()
                };
                
                // Only continue if there's a device
                if let Some(current_device) = current_device {
                    // Check if device has battery info
                    if let Some(battery) = &current_device.battery {
                        // Update the monitor with the new battery reading
                        let mut _alert_result = None;
                        
                        {
                            let mut monitor_guard = monitor.lock().unwrap();
                            
                            if options.use_smoothing {
                                // Add the reading to the buffer
                                monitor_guard.buffer.add_reading(battery);
                                
                                // Get the smoothed reading
                                let smoothed = monitor_guard.buffer.get_smoothed_reading(battery);
                                
                                // Check if the reading is valid
                                if monitor_guard.is_valid_battery(&smoothed) {
                                    // Calculate adaptive interval if needed
                                    if options.adaptive_polling {
                                        let new_interval = monitor_guard.calculate_adaptive_interval(&smoothed);
                                        if new_interval != monitor_guard.current_interval {
                                            // Update the interval
                                            monitor_guard.current_interval = new_interval;
                                            
                                            // Recreate the interval if it changed significantly
                                            if (new_interval.as_secs_f32() / monitor_guard.current_interval.as_secs_f32() - 1.0).abs() > 0.2 {
                                                timer = interval(new_interval);
                                            }
                                        }
                                    }
                                    
                                    // Check for significant changes
                                    if let Some(last_reading) = &monitor_guard.last_valid_reading {
                                        // Only report significant changes
                                        if monitor_guard.has_significant_change(last_reading, &smoothed) {
                                            // Create battery status
                                            let status = AirPodsBatteryStatus {
                                                battery: smoothed.clone(),
                                                last_updated: std::time::Instant::now(),
                                            };
                                            
                                            // Check for low battery alerts
                                            if options.notify_low_battery {
                                                _alert_result = monitor_guard.check_low_battery(&smoothed);
                                                
                                                // Call the callback
                                                callback(status, _alert_result.clone());
                                            } else {
                                                // Call the callback without low battery check
                                                callback(status, None);
                                            }
                                        }
                                    } else {
                                        // First reading, always report
                                        // Create battery status
                                        let status = AirPodsBatteryStatus {
                                            battery: smoothed.clone(),
                                            last_updated: std::time::Instant::now(),
                                        };
                                        
                                        // Check for low battery alerts
                                        if options.notify_low_battery {
                                            _alert_result = monitor_guard.check_low_battery(&smoothed);
                                            
                                            // Call the callback
                                            callback(status, _alert_result.clone());
                                        } else {
                                            // Call the callback without low battery check
                                            callback(status, None);
                                        }
                                    }
                                    
                                    // Update last valid reading
                                    monitor_guard.last_valid_reading = Some(smoothed);
                                }
                            } else {
                                // Not using smoothing, just use the raw reading
                                
                                // Check if the reading is valid
                                if monitor_guard.is_valid_battery(battery) {
                                    // Calculate adaptive interval if needed
                                    if options.adaptive_polling {
                                        let new_interval = monitor_guard.calculate_adaptive_interval(battery);
                                        if new_interval != monitor_guard.current_interval {
                                            // Update the interval
                                            monitor_guard.current_interval = new_interval;
                                            
                                            // Recreate the interval if it changed significantly
                                            if (new_interval.as_secs_f32() / monitor_guard.current_interval.as_secs_f32() - 1.0).abs() > 0.2 {
                                                timer = interval(new_interval);
                                            }
                                        }
                                    }
                                    
                                    // Check for significant changes
                                    if let Some(last_reading) = &monitor_guard.last_valid_reading {
                                        // Only report significant changes
                                        if monitor_guard.has_significant_change(last_reading, battery) {
                                            // Create battery status
                                            let status = AirPodsBatteryStatus {
                                                battery: battery.clone(),
                                                last_updated: std::time::Instant::now(),
                                            };
                                            
                                            // Check for low battery alerts
                                            if options.notify_low_battery {
                                                _alert_result = monitor_guard.check_low_battery(battery);
                                                
                                                // Call the callback
                                                callback(status, _alert_result.clone());
                                            } else {
                                                // Call the callback without low battery check
                                                callback(status, None);
                                            }
                                        }
                                    } else {
                                        // First reading, always report
                                        // Create battery status
                                        let status = AirPodsBatteryStatus {
                                            battery: battery.clone(),
                                            last_updated: std::time::Instant::now(),
                                        };
                                        
                                        // Check for low battery alerts
                                        if options.notify_low_battery {
                                            _alert_result = monitor_guard.check_low_battery(battery);
                                            
                                            // Call the callback
                                            callback(status, _alert_result.clone());
                                        } else {
                                            // Call the callback without low battery check
                                            callback(status, None);
                                        }
                                    }
                                    
                                    // Update last valid reading
                                    monitor_guard.last_valid_reading = Some(battery.clone());
                                }
                            }
                        }
                    }
                }
                
                // Sleep for a short time to avoid busy-waiting
                // This is not strictly necessary with interval, but can help reduce CPU usage
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        })
    }
    
    /// Check if a battery reading is valid
    fn is_valid_battery(&self, battery: &AirPodsBattery) -> bool {
        // Check if we have at least one valid reading
        let has_any_reading = battery.left.is_some() || battery.right.is_some() || battery.case.is_some();
        
        // Check if all readings are within valid range (0-100%)
        let valid_left = battery.left.map_or(true, |level| level <= 100);
        let valid_right = battery.right.map_or(true, |level| level <= 100);
        let valid_case = battery.case.map_or(true, |level| level <= 100);
        
        // Check charging status validity
        let valid_charging = battery.charging.is_some();
        
        has_any_reading && valid_left && valid_right && valid_case && valid_charging
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
            if significant_change || battery.charging.as_ref().is_some_and(|c| c.is_any_charging()) {
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
        // Get the thresholds from options
        let threshold = self.options.change_threshold as i16;
        
        // Check if any level has changed significantly
        let level_change = if let Some(last_left) = last.left {
            if let Some(current_left) = current.left {
                (current_left as i16 - last_left as i16).abs() > threshold
            } else {
                false
            }
        } else {
            false
        };
        
        let right_change = if let Some(last_right) = last.right {
            if let Some(current_right) = current.right {
                (current_right as i16 - last_right as i16).abs() > threshold
            } else {
                false
            }
        } else {
            false
        };
        
        let case_change = if let Some(last_case) = last.case {
            if let Some(current_case) = current.case {
                (current_case as i16 - last_case as i16).abs() > threshold
            } else {
                false
            }
        } else {
            false
        };
        
        // Check charging status change - any change is significant
        let charging_change = match (&last.charging, &current.charging) {
            (Some(last_state), Some(current_state)) => last_state != current_state,
            (None, Some(_)) | (Some(_), None) => true,
            (None, None) => false,
        };
        
        // Any significant change triggers more frequent polling
        level_change || right_change || case_change || charging_change
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
    
    /// Check for low battery and generate alerts if needed
    fn check_low_battery(&mut self, battery: &AirPodsBattery) -> Option<BatteryAlert> {
        // If notifications are disabled, don't generate alerts
        if !self.options.notify_low_battery {
            return None;
        }
        
        // Use current time for notifications
        let now = Instant::now();
        
        // Check left earbud
        if let Some(left) = battery.left {
            // Check if left is charging
            let is_charging = matches!(&battery.charging, 
                Some(AirPodsChargingState::LeftCharging) | 
                Some(AirPodsChargingState::BothBudsCharging));
            
            if left <= self.options.low_battery_threshold && !is_charging {
                // Check if we've already alerted for this component recently
                if !self.should_throttle_notification("left") {
                    self.last_notification.insert("left".to_string(), now);
                    return Some(BatteryAlert::LowBattery("Left AirPod".to_string(), left));
                }
            }
        }
        
        // Check right earbud
        if let Some(right) = battery.right {
            // Check if right is charging
            let is_charging = matches!(&battery.charging, 
                Some(AirPodsChargingState::RightCharging) | 
                Some(AirPodsChargingState::BothBudsCharging));
            
            if right <= self.options.low_battery_threshold && !is_charging {
                // Check if we've already alerted for this component recently
                if !self.should_throttle_notification("right") {
                    self.last_notification.insert("right".to_string(), now);
                    return Some(BatteryAlert::LowBattery("Right AirPod".to_string(), right));
                }
            }
        }
        
        // Check case
        if let Some(case) = battery.case {
            // Check if case is charging
            let is_charging = matches!(&battery.charging, 
                Some(AirPodsChargingState::CaseCharging));
            
            if case <= self.options.low_battery_threshold && !is_charging {
                // Check if we've already alerted for this component recently
                if !self.should_throttle_notification("case") {
                    self.last_notification.insert("case".to_string(), now);
                    return Some(BatteryAlert::LowBattery("AirPods Case".to_string(), case));
                }
            }
        }
        
        None
    }

    /// Check if a notification for a component should be throttled
    fn should_throttle_notification(&self, component: &str) -> bool {
        if let Some(last_time) = self.last_notification.get(component) {
            // Calculate time since last notification
            let duration = last_time.elapsed();
            
            // Throttle if less than cooldown period (default 30 minutes)
            duration.as_secs() < 30 * 60
        } else {
            // No previous notification, don't throttle
            false
        }
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
    use crate::airpods::AirPodsChargingState;
    
    #[test]
    fn test_battery_buffer() {
        let mut buffer = BatteryBuffer::new(3);
        
        // Test initial empty state
        assert!(buffer.get_average_left().is_none());
        assert!(buffer.get_average_right().is_none());
        assert!(buffer.get_average_case().is_none());
        
        // Add first reading
        let battery1 = AirPodsBattery {
            left: Some(50),
            right: Some(60),
            case: Some(70),
            charging: Some(AirPodsChargingState::NotCharging),
        };
        
        buffer.add_reading(&battery1);
        
        // Check buffer contents
        assert_eq!(buffer.left_buffer.len(), 1);
        assert_eq!(buffer.right_buffer.len(), 1);
        assert_eq!(buffer.case_buffer.len(), 1);
        
        // Add second reading
        let battery2 = AirPodsBattery {
            left: Some(60),
            right: Some(70),
            case: Some(80),
            charging: Some(AirPodsChargingState::LeftCharging),
        };
        
        buffer.add_reading(&battery2);
        
        // Test averages with two readings
        assert_eq!(buffer.get_average_left(), Some(55));
        assert_eq!(buffer.get_average_right(), Some(65));
        assert_eq!(buffer.get_average_case(), Some(75));
    }
    
    #[tokio::test]
    async fn test_battery_validation() {
        let monitor = BatteryMonitor::new();
        
        // Valid battery
        let valid = AirPodsBattery {
            left: Some(50),
            right: Some(60),
            case: Some(70),
            charging: Some(AirPodsChargingState::NotCharging),
        };
        
        assert!(monitor.is_valid_battery(&valid));
        
        // Invalid: no data
        let no_data = AirPodsBattery {
            left: None,
            right: None,
            case: None,
            charging: Some(AirPodsChargingState::NotCharging),
        };
        
        assert!(!monitor.is_valid_battery(&no_data));
        
        // Invalid: out of range
        let out_of_range = AirPodsBattery {
            left: Some(120),
            right: Some(60),
            case: Some(70),
            charging: Some(AirPodsChargingState::NotCharging),
        };
        
        assert!(!monitor.is_valid_battery(&out_of_range));
    }
    
    #[tokio::test]
    async fn test_significant_change_detection() {
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
            charging: Some(AirPodsChargingState::NotCharging),
        };
        
        // Small change (below threshold)
        let battery2 = AirPodsBattery {
            left: Some(52),
            right: Some(58),
            case: Some(72),
            charging: Some(AirPodsChargingState::NotCharging),
        };
        
        assert!(!monitor.has_significant_change(&battery1, &battery2));
        
        // Large change (above threshold)
        let battery3 = AirPodsBattery {
            left: Some(60),
            right: Some(70),
            case: Some(80),
            charging: Some(AirPodsChargingState::LeftCharging),
        };
        
        assert!(monitor.has_significant_change(&battery1, &battery3));
        
        // Change in charging state only
        let battery4 = AirPodsBattery {
            left: Some(50),
            right: Some(60),
            case: Some(70),
            charging: Some(AirPodsChargingState::CaseCharging),
        };
        
        assert!(monitor.has_significant_change(&battery1, &battery4));
    }
    
    #[tokio::test]
    async fn test_adaptive_interval() {
        let options = BatteryMonitorOptions {
            polling_interval: 10,
            adaptive_polling: true,
            ..Default::default()
        };
        
        let monitor = BatteryMonitor::with_options(options);
        
        // Check that the current interval is set to the polling_interval
        assert_eq!(monitor.current_interval, Duration::from_secs(10));
        
        // We're just testing initialization - the adaptive calculation
        // occurs within the start_monitoring method during runtime,
        // so we can't directly test update_polling_interval here
    }
} 