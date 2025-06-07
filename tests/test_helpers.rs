//! Test helpers and utilities for RustPods tests
//!
//! This module provides common utilities, fixtures, and helper functions
//! that are used across multiple test modules to reduce code duplication
//! and improve test maintainability.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::Mutex;

// === PERFORMANCE TRACKING UTILITIES ===

/// Performance tracker for individual tests
pub struct TestPerformanceTracker {
    start_time: Instant,
    test_name: String,
    checkpoints: Vec<(String, Duration)>,
}

impl TestPerformanceTracker {
    /// Start tracking performance for a test
    pub fn start(test_name: impl Into<String>) -> Self {
        let test_name = test_name.into();
        println!("🚀 Starting test: {}", test_name);
        Self {
            start_time: Instant::now(),
            test_name,
            checkpoints: Vec::new(),
        }
    }
    
    /// Add a checkpoint with a label
    pub fn checkpoint(&mut self, label: impl Into<String>) {
        let elapsed = self.start_time.elapsed();
        let label = label.into();
        println!("  ⏱️  {} - {}ms", label, elapsed.as_millis());
        self.checkpoints.push((label, elapsed));
    }
    
    /// Finish tracking and print final results
    pub fn finish(self) {
        let total_time = self.start_time.elapsed();
        println!("✅ {} completed in {}ms", self.test_name, total_time.as_millis());
        
        // Show breakdown if there were checkpoints
        if !self.checkpoints.is_empty() {
            println!("   📊 Breakdown:");
            for (label, time) in &self.checkpoints {
                let percentage = (time.as_millis() as f64 / total_time.as_millis() as f64) * 100.0;
                println!("      - {}: {}ms ({:.1}%)", label, time.as_millis(), percentage);
            }
        }
        println!();
    }
}

/// Macro for easy performance tracking
#[macro_export]
macro_rules! track_performance {
    ($test_name:expr, $body:block) => {{
        let tracker = $crate::test_helpers::TestPerformanceTracker::start($test_name);
        let result = $body;
        tracker.finish();
        result
    }};
}

/// Async performance tracking
pub async fn track_async_performance<F, R>(test_name: &str, future: F) -> R
where
    F: std::future::Future<Output = R>,
{
    let tracker = TestPerformanceTracker::start(test_name);
    let result = future.await;
    tracker.finish();
    result
}

/// Global test performance registry
static PERF_REGISTRY: Mutex<Option<HashMap<String, Duration>>> = Mutex::new(None);

/// Initialize performance tracking registry
pub fn init_performance_tracking() {
    let mut registry = PERF_REGISTRY.lock().unwrap();
    *registry = Some(HashMap::new());
}

/// Record test performance in global registry
pub fn record_test_performance(test_name: &str, duration: Duration) {
    let mut registry = PERF_REGISTRY.lock().unwrap();
    if let Some(ref mut map) = *registry {
        map.insert(test_name.to_string(), duration);
    }
}

/// Print performance summary for all tracked tests
pub fn print_performance_summary() {
    let registry = PERF_REGISTRY.lock().unwrap();
    if let Some(ref map) = *registry {
        if !map.is_empty() {
            println!("\n📈 TEST PERFORMANCE SUMMARY:");
            println!("================================");
            
            let mut tests: Vec<_> = map.iter().collect();
            tests.sort_by_key(|(_, duration)| *duration);
            tests.reverse(); // Slowest first
            
            for (test_name, duration) in tests {
                let ms = duration.as_millis();
                let status = match ms {
                    0..=50 => "🟢 FAST",
                    51..=200 => "🟡 MODERATE", 
                    201..=1000 => "🟠 SLOW",
                    _ => "🔴 VERY SLOW"
                };
                println!("  {} {}: {}ms", status, test_name, ms);
            }
            println!();
        }
    }
}

// === EXISTING TEST HELPERS BELOW === 