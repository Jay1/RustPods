//! Battery level estimation between Apple's 10% increments
//!
//! This module provides intelligent battery estimation by tracking discharge patterns
//! over time. Since Apple's BLE protocol only reports battery in 10% increments,
//! we can estimate intermediate values (41%, 42%, etc.) based on time elapsed
//! and historical discharge rates.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{Duration, SystemTime};

/// Maximum number of discharge rate samples to keep in history
const MAX_HISTORY_ENTRIES: usize = 20;

/// Minimum time between readings to calculate discharge rate (30 seconds)
const MIN_READING_INTERVAL: Duration = Duration::from_secs(30);

/// Maximum realistic discharge rate (% per minute) - prevents unrealistic estimates
const MAX_DISCHARGE_RATE: f32 = 2.0; // 2% per minute = 120% per hour (unrealistic, for safety)

/// Default discharge rate when no history available (% per minute)
const DEFAULT_DISCHARGE_RATE: f32 = 0.16; // ~10% per hour, reasonable default

/// Battery estimator for predicting levels between Apple's 10% increments
#[derive(Debug, Clone)]
pub struct BatteryEstimator {
    pub left_history: DischargeHistory,
    pub right_history: DischargeHistory,
    pub case_history: DischargeHistory,
}

/// Historical discharge data for a single battery (left, right, or case)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DischargeHistory {
    /// Rolling history of discharge rates (limited to MAX_HISTORY_ENTRIES)
    pub discharge_rates: VecDeque<DischargeRate>,
    /// Last known battery level from Apple BLE (source of truth)
    pub last_known_level: Option<u8>,
    /// Time when last_known_level was recorded
    pub last_known_time: Option<SystemTime>,
}

/// A single discharge rate measurement
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DischargeRate {
    /// Percentage lost per minute
    pub percentage_per_minute: f32,
    /// When this rate was calculated
    pub timestamp: SystemTime,
}

/// Estimated battery level with confidence indicator
#[derive(Debug, Clone)]
pub struct EstimatedBattery {
    /// Estimated battery percentage (can be fractional)
    pub level: f32,
    /// True if this is real Apple data, false if estimated
    pub is_real_data: bool,
    /// Confidence in estimate (0.0 to 1.0, only relevant for estimates)
    pub confidence: f32,
}

impl BatteryEstimator {
    /// Create a new battery estimator
    pub fn new() -> Self {
        Self {
            left_history: DischargeHistory::new(),
            right_history: DischargeHistory::new(),
            case_history: DischargeHistory::new(),
        }
    }

    /// Update with new real battery data from Apple BLE
    pub fn update_real_data(&mut self, left: Option<i32>, right: Option<i32>, case: Option<i32>) {
        let now = SystemTime::now();

        if let Some(left_level) = left {
            self.left_history.update_real_reading(left_level as u8, now);
        }

        if let Some(right_level) = right {
            self.right_history.update_real_reading(right_level as u8, now);
        }

        if let Some(case_level) = case {
            self.case_history.update_real_reading(case_level as u8, now);
        }
    }

    /// Get estimated battery levels (uses real data when available, estimates otherwise)
    pub fn get_estimated_levels(&self) -> (EstimatedBattery, EstimatedBattery, EstimatedBattery) {
        (
            self.left_history.get_estimated_level(),
            self.right_history.get_estimated_level(),
            self.case_history.get_estimated_level(),
        )
    }

    /// Get just the estimated percentages as integers for display
    pub fn get_display_levels(&self) -> (Option<u8>, Option<u8>, Option<u8>) {
        let (left, right, case) = self.get_estimated_levels();
        
        (
            if left.level >= 0.0 { Some(left.level.round() as u8) } else { None },
            if right.level >= 0.0 { Some(right.level.round() as u8) } else { None },
            if case.level >= 0.0 { Some(case.level.round() as u8) } else { None },
        )
    }

    /// Check if we have any recent real data
    pub fn has_recent_data(&self) -> bool {
        let cutoff = SystemTime::now() - Duration::from_secs(300); // 5 minutes
        
        [&self.left_history, &self.right_history, &self.case_history]
            .iter()
            .any(|history| {
                history.last_known_time
                    .map(|time| time > cutoff)
                    .unwrap_or(false)
            })
    }
}

impl DischargeHistory {
    /// Create a new discharge history
    pub fn new() -> Self {
        Self {
            discharge_rates: VecDeque::new(),
            last_known_level: None,
            last_known_time: None,
        }
    }

    /// Update with a new real battery reading from Apple BLE
    pub fn update_real_reading(&mut self, level: u8, timestamp: SystemTime) {
        // If we have previous data, calculate discharge rate
        if let (Some(prev_level), Some(prev_time)) = (self.last_known_level, self.last_known_time) {
            if let Ok(elapsed) = timestamp.duration_since(prev_time) {
                // Only calculate if enough time has passed and level has actually changed
                if elapsed >= MIN_READING_INTERVAL && prev_level != level {
                    let minutes_elapsed = elapsed.as_secs_f32() / 60.0;
                    let level_change = prev_level as f32 - level as f32; // Positive = discharge
                    let rate = level_change / minutes_elapsed;

                    // Only store realistic discharge rates (positive = discharging)
                    if rate > 0.0 && rate <= MAX_DISCHARGE_RATE {
                        self.add_discharge_rate(rate, timestamp);
                    }
                }
            }
        }

        // Update last known data
        self.last_known_level = Some(level);
        self.last_known_time = Some(timestamp);
    }

    /// Add a new discharge rate to history
    fn add_discharge_rate(&mut self, rate: f32, timestamp: SystemTime) {
        self.discharge_rates.push_back(DischargeRate {
            percentage_per_minute: rate,
            timestamp,
        });

        // Keep only the most recent entries
        while self.discharge_rates.len() > MAX_HISTORY_ENTRIES {
            self.discharge_rates.pop_front();
        }
    }

    /// Get estimated current battery level
    pub fn get_estimated_level(&self) -> EstimatedBattery {
        // If no data available, return unknown
        let (last_level, last_time) = match (self.last_known_level, self.last_known_time) {
            (Some(level), Some(time)) => (level, time),
            _ => return EstimatedBattery {
                level: -1.0, // Unknown
                is_real_data: false,
                confidence: 0.0,
            },
        };

        let now = SystemTime::now();
        let elapsed = match now.duration_since(last_time) {
            Ok(duration) => duration,
            Err(_) => return EstimatedBattery {
                level: last_level as f32,
                is_real_data: true,
                confidence: 1.0,
            },
        };

        // If reading is very recent (< 1 minute), treat as real data
        if elapsed.as_secs() < 60 {
            return EstimatedBattery {
                level: last_level as f32,
                is_real_data: true,
                confidence: 1.0,
            };
        }

        // Calculate estimated level based on discharge rate
        let minutes_elapsed = elapsed.as_secs_f32() / 60.0;
        let avg_discharge_rate = self.get_average_discharge_rate();
        let estimated_discharge = avg_discharge_rate * minutes_elapsed;
        let estimated_level = (last_level as f32 - estimated_discharge).max(0.0);

        // Calculate confidence based on data quality and time elapsed
        let confidence = self.calculate_confidence(elapsed);

        EstimatedBattery {
            level: estimated_level,
            is_real_data: false,
            confidence,
        }
    }

    /// Get average discharge rate from recent history
    fn get_average_discharge_rate(&self) -> f32 {
        if self.discharge_rates.is_empty() {
            return DEFAULT_DISCHARGE_RATE;
        }

        // Weight recent rates more heavily
        let mut total_weight = 0.0;
        let mut weighted_sum = 0.0;
        let now = SystemTime::now();

        for rate in &self.discharge_rates {
            // Calculate age in hours
            let age_hours = now.duration_since(rate.timestamp)
                .unwrap_or(Duration::from_secs(3600))
                .as_secs_f32() / 3600.0;

            // Recent data gets higher weight (exponential decay)
            let weight = (-age_hours / 24.0).exp(); // Half-life of 24 hours
            
            weighted_sum += rate.percentage_per_minute * weight;
            total_weight += weight;
        }

        if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            DEFAULT_DISCHARGE_RATE
        }
    }

    /// Calculate confidence in estimate based on data quality and elapsed time
    fn calculate_confidence(&self, elapsed: Duration) -> f32 {
        let mut confidence = 1.0;

        // Confidence decreases over time
        let hours_elapsed = elapsed.as_secs_f32() / 3600.0;
        confidence *= (-hours_elapsed / 6.0).exp(); // Drops to ~37% after 6 hours

        // Confidence increases with more historical data
        let data_quality = (self.discharge_rates.len() as f32 / MAX_HISTORY_ENTRIES as f32).min(1.0);
        confidence *= 0.3 + 0.7 * data_quality; // Range: 30% to 100%

        confidence.max(0.1).min(1.0) // Clamp between 10% and 100%
    }
}

impl Default for BatteryEstimator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for DischargeHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimator_creation() {
        let estimator = BatteryEstimator::new();
        assert_eq!(estimator.left_history.discharge_rates.len(), 0);
        assert_eq!(estimator.right_history.discharge_rates.len(), 0);
        assert_eq!(estimator.case_history.discharge_rates.len(), 0);
    }

    #[test]
    fn test_real_data_update() {
        let mut estimator = BatteryEstimator::new();
        estimator.update_real_data(Some(80), Some(75), Some(90));

        assert_eq!(estimator.left_history.last_known_level, Some(80));
        assert_eq!(estimator.right_history.last_known_level, Some(75));
        assert_eq!(estimator.case_history.last_known_level, Some(90));
    }

    #[test]
    fn test_discharge_rate_calculation() {
        let mut history = DischargeHistory::new();
        let now = SystemTime::now();
        let earlier = now - Duration::from_secs(600); // 10 minutes ago

        // First reading: 80%
        history.update_real_reading(80, earlier);
        
        // Second reading: 70% (10% drop in 10 minutes = 1% per minute)
        history.update_real_reading(70, now);

        assert_eq!(history.discharge_rates.len(), 1);
        assert!((history.discharge_rates[0].percentage_per_minute - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_estimation_confidence() {
        let mut history = DischargeHistory::new();
        let now = SystemTime::now();
        
        // Add some data
        history.update_real_reading(80, now - Duration::from_secs(60));
        
        let estimate = history.get_estimated_level();
        assert!(estimate.confidence > 0.0);
        assert!(estimate.confidence <= 1.0);
    }
} 