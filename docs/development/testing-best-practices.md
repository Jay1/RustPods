# Advanced Rust Testing Best Practices for RustPods

This guide covers advanced testing techniques, performance optimization, and cybersecurity-focused testing patterns for the RustPods project.

## Performance Monitoring & Optimization

### 1. Built-in Performance Tracking

```rust
// Use the custom performance tracker
use crate::test_helpers::TestPerformanceTracker;

#[test]
fn test_with_performance_tracking() {
    let mut tracker = TestPerformanceTracker::start("test_bluetooth_scanning");
    
    tracker.checkpoint("Initializing scanner");
    let scanner = BluetoothScanner::new();
    
    tracker.checkpoint("Starting scan");
    let result = scanner.start_scan();
    
    tracker.checkpoint("Processing results");
    assert!(result.is_ok());
    
    tracker.finish(); // Shows breakdown with percentages
}
```

### 2. Cargo Test Optimization Flags

```bash
# Fast development testing (recommended daily workflow)
cargo test --lib --no-default-features  # 0.07s - unit tests only

# Skip compilation of C++ CLI scanner when not needed
SKIP_CLI_BUILD=1 cargo test --lib

# Parallel test execution (default, but can be tuned)
cargo test -- --test-threads=8

# Run only specific test patterns
cargo test bluetooth::scanner --lib  # Test specific modules

# Profile-guided optimization for release builds
cargo test --release  # Use for performance benchmarks
```

### 3. Advanced Test Organization

```rust
// Conditional compilation for different test suites
#[cfg(test)]
mod unit_tests {
    use super::*;
    
    // Fast unit tests
    #[test]
    fn test_core_logic() { /* ... */ }
}

#[cfg(all(test, feature = "integration-tests"))]
mod integration_tests {
    use super::*;
    
    // Slower integration tests, only run with feature flag
    #[tokio::test]
    async fn test_full_bluetooth_flow() { /* ... */ }
}

#[cfg(all(test, feature = "performance-tests"))]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[test]
    fn test_performance_requirements() {
        let start = Instant::now();
        // Test critical path
        assert!(start.elapsed().as_millis() < 100, "Operation too slow");
    }
}
```

## Cybersecurity-Focused Testing Patterns

### 1. Security Property Testing

```rust
use proptest::prelude::*;

// Property-based testing for security invariants
proptest! {
    #[test]
    fn test_bluetooth_address_validation(addr in "([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}") {
        // Ensure all valid MAC addresses are accepted
        let result = validate_bluetooth_address(&addr);
        prop_assert!(result.is_ok());
    }
    
    #[test]
    fn test_no_buffer_overflow_in_parsing(data in prop::collection::vec(any::<u8>(), 0..1000)) {
        // Ensure parsing random data never panics or corrupts memory
        let result = std::panic::catch_unwind(|| {
            parse_manufacturer_data(&data)
        });
        prop_assert!(result.is_ok()); // Should not panic
    }
}
```

### 2. Timing Attack Resistance

```rust
#[test]
fn test_constant_time_operations() {
    use std::time::Instant;
    
    // Test that authentication operations take constant time
    let test_cases = vec![
        "valid_key_12345",
        "invalid_key_12345", 
        "short",
        "very_long_invalid_key_that_might_cause_timing_differences"
    ];
    
    let mut timings = Vec::new();
    
    for key in test_cases {
        let start = Instant::now();
        let _ = validate_api_key(key); // Should be constant time
        timings.push(start.elapsed());
    }
    
    // Verify timing variance is minimal (< 10% difference)
    let max_time = timings.iter().max().unwrap();
    let min_time = timings.iter().min().unwrap();
    let variance = (max_time.as_nanos() - min_time.as_nanos()) as f64 / min_time.as_nanos() as f64;
    
    assert!(variance < 0.1, "Timing variance too high: {}%", variance * 100.0);
}
```

### 3. Input Sanitization Testing

```rust
#[test]
fn test_input_sanitization() {
    let malicious_inputs = vec![
        "'; DROP TABLE devices; --",
        "<script>alert('xss')</script>",
        "\x00\x01\x02\x03", // Null bytes
        "A".repeat(10000),   // Buffer overflow attempts
        "../../../etc/passwd", // Path traversal
    ];
    
    for input in malicious_inputs {
        let result = process_device_name(input);
        assert!(result.is_err() || is_sanitized(&result.unwrap()));
    }
}
```

## Advanced Async Testing Patterns

### 1. Timeout Testing for Network Operations

```rust
#[tokio::test(flavor = "multi_thread")]
async fn test_bluetooth_scan_timeout() {
    use tokio::time::{timeout, Duration};
    
    let scanner = BluetoothScanner::new();
    
    // Ensure operations complete within reasonable time
    let result = timeout(Duration::from_secs(5), scanner.scan()).await;
    
    assert!(result.is_ok(), "Bluetooth scan should complete within 5 seconds");
}
```

### 2. Resource Cleanup Testing

```rust
#[tokio::test]
async fn test_resource_cleanup() {
    let initial_handles = count_file_handles();
    
    {
        let scanner = BluetoothScanner::new().await?;
        let _scan_result = scanner.start_scanning().await?;
        // Scanner dropped here
    }
    
    tokio::time::sleep(Duration::from_millis(100)).await; // Allow cleanup
    
    let final_handles = count_file_handles();
    assert_eq!(initial_handles, final_handles, "Resource leak detected");
}
```

### 3. Concurrent Access Testing

```rust
#[tokio::test]
async fn test_concurrent_bluetooth_access() {
    let scanner = Arc::new(BluetoothScanner::new());
    let mut handles = Vec::new();
    
    // Spawn multiple concurrent scan operations
    for i in 0..10 {
        let scanner_clone = Arc::clone(&scanner);
        let handle = tokio::spawn(async move {
            scanner_clone.scan_once().await
        });
        handles.push(handle);
    }
    
    // All should complete successfully
    for handle in handles {
        assert!(handle.await.is_ok());
    }
}
```

## Mock Testing for External Dependencies

### 1. Bluetooth Hardware Mocking

```rust
#[cfg(test)]
pub struct MockBluetoothAdapter {
    devices: Vec<MockDevice>,
    scan_active: bool,
    pub call_log: Vec<String>,
}

impl MockBluetoothAdapter {
    pub fn with_devices(devices: Vec<MockDevice>) -> Self {
        Self { devices, scan_active: false, call_log: Vec::new() }
    }
    
    pub fn simulate_device_discovery(&mut self, device: MockDevice) {
        self.devices.push(device);
        self.call_log.push("device_discovered".to_string());
    }
    
    pub fn simulate_connection_failure(&mut self) {
        self.call_log.push("connection_failed".to_string());
    }
}

#[async_trait]
impl BluetoothAdapterTrait for MockBluetoothAdapter {
    async fn start_scan(&mut self) -> Result<(), BluetoothError> {
        self.call_log.push("start_scan".to_string());
        self.scan_active = true;
        Ok(())
    }
    
    async fn get_devices(&self) -> Vec<Device> {
        self.call_log.push("get_devices".to_string());
        self.devices.iter().map(|d| d.to_device()).collect()
    }
}
```

## Performance Benchmarking

### 1. Criterion.rs Integration

```toml
# Add to Cargo.toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "bluetooth_performance"
harness = false
```

```rust
// benches/bluetooth_performance.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rustpods::bluetooth::BluetoothScanner;

fn benchmark_device_parsing(c: &mut Criterion) {
    let sample_data = include_bytes!("../test_data/airpods_sample.bin");
    
    c.bench_function("parse_airpods_data", |b| {
        b.iter(|| parse_airpods_data(black_box(sample_data)))
    });
}

fn benchmark_filtering(c: &mut Criterion) {
    let devices = create_test_devices(1000);
    
    c.bench_function("filter_airpods", |b| {
        b.iter(|| filter_airpods_devices(black_box(&devices)))
    });
}

criterion_group!(benches, benchmark_device_parsing, benchmark_filtering);
criterion_main!(benches);
```

## Test Data Management

### 1. Fixture Management

```rust
// tests/fixtures/mod.rs
use std::sync::OnceLock;

static TEST_CONFIG: OnceLock<AppConfig> = OnceLock::new();

pub fn get_test_config() -> &'static AppConfig {
    TEST_CONFIG.get_or_init(|| {
        AppConfig {
            bluetooth: BluetoothConfig {
                scan_duration: Duration::from_secs(5),
                auto_scan_on_startup: false,
                ..Default::default()
            },
            ui: UiConfig {
                show_notifications: false,
                ..Default::default()
            },
            ..Default::default()
        }
    })
}

pub fn create_test_airpods() -> AirPodsDevice {
    AirPodsDevice {
        address: "AA:BB:CC:DD:EE:FF".parse().unwrap(),
        name: "Test AirPods Pro".to_string(),
        battery: AirPodsBattery {
            left: Some(75),
            right: Some(80),
            case: Some(90),
            charging: Some(AirPodsChargingState::NotCharging),
        },
        last_seen: Instant::now(),
    }
}
```

## CI/CD Performance Integration

### 1. GitHub Actions Performance Testing

```yaml
# .github/workflows/performance.yml
name: Performance Tests

on: [push, pull_request]

jobs:
  performance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Run fast tests
        run: |
          time cargo test --lib --no-default-features
          
      - name: Performance benchmark
        run: |
          cargo bench --bench bluetooth_performance
          
      - name: Check performance regression
        run: |
          # Compare with baseline performance metrics
          python scripts/check_performance_regression.py
```

## Memory Safety & Leak Detection

### 1. Valgrind Integration (Linux)

```bash
# Install valgrind
sudo apt-get install valgrind

# Run tests with memory checking
cargo test --target x86_64-unknown-linux-gnu
valgrind --tool=memcheck --leak-check=full ./target/debug/deps/rustpods-*

# For more detailed analysis
cargo install cargo-valgrind
cargo valgrind test
```

### 2. AddressSanitizer (when available)

```bash
# Set environment for address sanitizer
export RUSTFLAGS="-Zsanitizer=address"
export RUSTDOCFLAGS="-Zsanitizer=address"
cargo +nightly test -Zbuild-std --target x86_64-unknown-linux-gnu
```

## Test Metrics & Reporting

### 1. Coverage with Performance Tracking

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run with performance and coverage
cargo tarpaulin --out Html --output-dir coverage/ --timeout 120 --features "test-utils"
```

### 2. Custom Test Reporter

```rust
// tests/test_reporter.rs
pub struct PerformanceTestReporter {
    results: Vec<TestResult>,
    start_time: Instant,
}

impl PerformanceTestReporter {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            start_time: Instant::now(),
        }
    }
    
    pub fn report_test(&mut self, name: &str, duration: Duration, success: bool) {
        self.results.push(TestResult {
            name: name.to_string(),
            duration,
            success,
        });
    }
    
    pub fn generate_report(&self) -> String {
        let total_time = self.start_time.elapsed();
        let success_rate = self.results.iter().filter(|r| r.success).count() as f64 / self.results.len() as f64;
        
        format!(
            "Performance Report:\n\
            Total Time: {}ms\n\
            Success Rate: {:.1}%\n\
            Fastest Test: {}ms\n\
            Slowest Test: {}ms",
            total_time.as_millis(),
            success_rate * 100.0,
            self.results.iter().map(|r| r.duration.as_millis()).min().unwrap_or(0),
            self.results.iter().map(|r| r.duration.as_millis()).max().unwrap_or(0)
        )
    }
}
```

## Recommendations for Daily Development

1. **Use `cargo test --lib --no-default-features`** for fastest feedback (0.07s)
2. **Run performance script**: `./scripts/test_performance.ps1 -Fast` 
3. **Enable feature flags** for comprehensive testing: `cargo test --features "integration-tests"`
4. **Use property-based testing** for security-critical code
5. **Mock external dependencies** to isolate unit tests
6. **Benchmark performance-critical paths** with Criterion.rs
7. **Monitor resource usage** in async tests
8. **Set up CI performance baselines** to catch regressions

This testing strategy provides both rapid development feedback and comprehensive verification while maintaining security-focused testing practices suitable for cybersecurity research and development. 