// Common test utilities shared across all test modules
use futures::Stream;
use futures::StreamExt;
use std::collections::HashMap;
use std::pin::Pin;
use std::str::FromStr;
use std::time::Duration;
use tokio::sync::mpsc::Receiver;
use tokio::time::{self, timeout};
use tokio_stream::wrappers::ReceiverStream;

// Add imports for RustPods types
use btleplug::api::BDAddr;
use rustpods::airpods::{AirPodsBattery, AirPodsChargingState, AirPodsType, DetectedAirPods};
use rustpods::bluetooth::{BleEvent, DiscoveredDevice};
use rustpods::config::{AppConfig, LogLevel, Theme};
use rustpods::ui::state::AppState;

/// Helper to convert tokio receiver to stream for testing
pub fn receiver_to_stream<T>(rx: Receiver<T>) -> impl Stream<Item = T> {
    futures::stream::unfold(rx, |mut rx| async move {
        rx.recv().await.map(|value| (value, rx))
    })
}

/// Utility function to wait for a specific duration
pub async fn wait_ms(ms: u64) {
    tokio::time::sleep(Duration::from_millis(ms)).await;
}

/// Sets up test environment variables if needed
pub fn setup_test_env() {
    std::env::set_var("RUST_BACKTRACE", "1");
    std::env::set_var("RUST_LOG", "debug");
}

/// Helper function to create a temporary test directory and clean it up after the test
#[cfg(test)]
pub fn with_temp_dir<F, R>(test_fn: F) -> R
where
    F: FnOnce(&std::path::Path) -> R,
{
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    let result = test_fn(temp_dir.path());
    // Temp directory will be automatically cleaned up when it goes out of scope
    result
}

/// Execute an async operation with a timeout
pub async fn with_timeout<T>(
    duration: Duration,
    operation: impl std::future::Future<Output = T>,
) -> T {
    match time::timeout(duration, operation).await {
        Ok(result) => result,
        Err(_) => panic!("Operation timed out after {:?}", duration),
    }
}

/// Create a helper to wait for events with a default timeout
pub async fn wait_for_event<S, T>(stream: &mut S) -> Option<T>
where
    S: StreamExt<Item = T> + Unpin,
{
    with_timeout(Duration::from_millis(500), stream.next()).await
}

/// Create a helper to assert that no more events are received
pub async fn assert_no_more_events<S, T>(stream: &mut S, timeout_ms: u64)
where
    S: StreamExt<Item = T> + Unpin,
{
    let result = time::timeout(Duration::from_millis(timeout_ms), stream.next()).await;
    assert!(result.is_err(), "Should not receive any more events");
}

/// Sleep for a short period to allow async operations to complete
pub async fn short_delay() {
    wait_ms(100).await;
}

/// Sleep for a longer period when dealing with complex event chains
pub async fn medium_delay() {
    wait_ms(250).await;
}

/// Sleep for a long period to allow for significant operations
pub async fn long_delay() {
    wait_ms(500).await;
}

/// Sleep for an extra long period to allow for substantial operations
pub async fn very_long_delay() {
    wait_ms(1000).await;
}

/// Helper to create temporary test directory
pub fn create_temp_dir() -> tempfile::TempDir {
    // Temp directory will be automatically cleaned up when it goes out of scope
    tempfile::tempdir().expect("Failed to create temporary directory")
}

/// Convert a tokio mpsc Receiver into a Stream
/// This makes it easier to work with receivers in async code using Stream combinators
pub fn receiver_to_stream_boxed<T: Send + 'static>(
    receiver: Receiver<T>,
) -> Pin<Box<dyn Stream<Item = T> + Send>> {
    Box::pin(ReceiverStream::new(receiver))
}

/// Receive the next event from a channel with a timeout
pub async fn receive_with_timeout<T>(rx: &mut Receiver<T>, duration: Duration) -> Option<T> {
    match timeout(duration, rx.recv()).await {
        Ok(Some(event)) => Some(event),
        _ => None,
    }
}

/// Create a test configuration
pub fn create_test_config() -> AppConfig {
    let mut config = AppConfig::default();
    config.bluetooth.scan_interval = Duration::from_secs(30);
    config.bluetooth.auto_reconnect = true;
    config.bluetooth.min_rssi = Some(-70);
    config.ui.theme = Theme::System;
    config.ui.show_notifications = true;
    config.system.launch_at_startup = true;
    config.system.log_level = LogLevel::Info;
    config.system.enable_telemetry = true;
    config.system.auto_save_interval = Some(60);
    config.system.enable_crash_recovery = true;
    config.battery.low_threshold = 20;
    config
}

/// Create a test AppState
pub fn create_test_app_state() -> AppState {
    let mut state = AppState::default();
    state.visible = true;
    state.config = create_test_config();
    state
}

/// Create a test AirPods device
pub fn create_test_airpods(device_type: AirPodsType, address: Option<&str>) -> DetectedAirPods {
    let addr = match address {
        Some(addr_str) => BDAddr::from_str(addr_str)
            .unwrap_or_else(|_| BDAddr::from([0x11, 0x22, 0x33, 0x44, 0x55, 0x66])),
        None => BDAddr::from([0x11, 0x22, 0x33, 0x44, 0x55, 0x66]),
    };

    let name = match device_type {
        AirPodsType::AirPods1 => "AirPods",
        AirPodsType::AirPods2 => "AirPods",
        AirPodsType::AirPods3 => "AirPods (3rd generation)",
        AirPodsType::AirPodsPro => "AirPods Pro",
        AirPodsType::AirPodsPro2 => "AirPods Pro",
        AirPodsType::AirPodsMax => "AirPods Max",
        AirPodsType::Unknown => "Unknown AirPods",
    };

    DetectedAirPods {
        address: addr,
        name: Some(name.to_string()),
        device_type,
        battery: Some(AirPodsBattery {
            left: Some(70),
            right: Some(70),
            case: None,
            charging: Some(AirPodsChargingState::NotCharging),
        }),
        rssi: Some(-60),
        last_seen: std::time::Instant::now(),
        is_connected: false,
    }
}

/// Create a test discovered device with manufacturer data
pub fn create_test_device_with_data(
    address: &str,
    name: Option<&str>,
    manufacturer_id: u16,
    data: Vec<u8>,
) -> DiscoveredDevice {
    let mut manufacturer_data = HashMap::new();
    manufacturer_data.insert(manufacturer_id, data);

    DiscoveredDevice {
        address: BDAddr::from_str(address)
            .unwrap_or_else(|_| BDAddr::from([0x11, 0x22, 0x33, 0x44, 0x55, 0x66])),
        name: name.map(String::from),
        rssi: Some(-60),
        manufacturer_data,
        is_potential_airpods: false,
        last_seen: std::time::Instant::now(),
        is_connected: false,
        service_data: HashMap::new(),
        services: vec![],
        tx_power_level: None,
    }
}

/// Create a test Apple device with manufacturer data
pub fn create_test_apple_device(
    address: &str,
    name: Option<&str>,
    data: Vec<u8>,
) -> DiscoveredDevice {
    create_test_device_with_data(address, name, 76, data) // 76 is Apple's manufacturer ID
}

/// Create a simple channel for BLE events
pub fn create_ble_event_channel() -> (
    tokio::sync::mpsc::Sender<BleEvent>,
    tokio::sync::mpsc::Receiver<BleEvent>,
) {
    tokio::sync::mpsc::channel(100)
}

/// Helper to get sample manufacturer data for different AirPods models
pub fn get_sample_airpods_data(model: AirPodsType) -> Vec<u8> {
    match model {
        AirPodsType::AirPods1 => {
            vec![
                0x01, 0x19, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0xb0,
            ]
        }
        AirPodsType::AirPods2 => {
            vec![
                0x02, 0x19, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0xb0,
            ]
        }
        AirPodsType::AirPods3 => {
            vec![
                0x05, 0x19, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0xb0,
            ]
        }
        AirPodsType::AirPodsPro => {
            vec![
                0x03, 0x19, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0xb0,
            ]
        }
        AirPodsType::AirPodsPro2 => {
            vec![
                0x06, 0x19, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0xb0,
            ]
        }
        AirPodsType::AirPodsMax => {
            vec![
                0x04, 0x19, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0xb0,
            ]
        }
        AirPodsType::Unknown => {
            vec![
                0xFF, 0x19, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0xb0,
            ]
        }
    }
}

/// Create a UI test setup with a specific theme
pub fn create_ui_test_with_theme(theme: Theme) -> AppState {
    let mut state = AppState::default();
    state.visible = true;

    // Create a test config
    let mut config = create_test_config();
    config.ui.theme = theme;

    // Apply the config
    state.config = config;

    state
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc::channel;

    #[tokio::test]
    async fn test_receiver_to_stream() {
        let (tx, rx) = channel::<i32>(10);
        let stream = receiver_to_stream(rx);
        tokio::pin!(stream);

        tx.send(42).await.unwrap();

        let result = wait_for_event(&mut stream).await;
        assert_eq!(result, Some(42));
    }

    #[tokio::test]
    async fn test_with_timeout_success() {
        let result = with_timeout(Duration::from_millis(100), async { 42 }).await;
        assert_eq!(result, 42);
    }

    #[tokio::test]
    #[should_panic(expected = "Operation timed out")]
    async fn test_with_timeout_failure() {
        let _ = with_timeout(Duration::from_millis(10), async {
            time::sleep(Duration::from_millis(100)).await;
            42
        })
        .await;
    }

    #[tokio::test]
    async fn test_create_test_config() {
        let config = create_test_config();

        // Verify basic properties
        assert_eq!(config.bluetooth.scan_duration, Duration::from_secs(5));
        assert_eq!(config.ui.low_battery_threshold, 20);
        assert_eq!(config.system.log_level, LogLevel::Info);
    }

    #[tokio::test]
    async fn test_create_test_airpods() {
        let airpods = create_test_airpods(AirPodsType::AirPodsPro, None);

        assert_eq!(airpods.device_type, AirPodsType::AirPodsPro);
        assert_eq!(airpods.name, Some("AirPods Pro".to_string()));
    }
}
