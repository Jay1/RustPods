//! Integration tests for the state management system
//!
//! This file contains comprehensive tests that verify:
//! - State flow between components
//! - Animation and transition effects
//! - State persistence across app sessions
//! - Cross-component communications
//! - Error state handling

use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::collections::HashMap;

use tokio::sync::mpsc;
use tokio::time::timeout;

use rustpods::ui::state_manager::{StateManager, Action, ConnectionState, DeviceState, UiState};
use rustpods::ui::Message;
use rustpods::bluetooth::{AirPodsBatteryStatus, DiscoveredDevice};
use rustpods::airpods::{AirPodsBattery, AirPodsCharging};
use rustpods::config::{AppConfig, ConfigManager};
use btleplug::api::BDAddr;
use std::str::FromStr;

// SECTION: Helper Functions and Test Setup

/// Create a test device for integration testing
fn create_test_device(address: &str, name: &str, rssi: i32, is_airpods: bool) -> DiscoveredDevice {
    let bdaddr = BDAddr::from_str(address).unwrap_or_else(|_| BDAddr::default());
    let mut manufacturer_data = HashMap::new();
    
    if is_airpods {
        manufacturer_data.insert(0x004C, vec![0x07, 0x19, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06]);
    }
    
    DiscoveredDevice {
        address: bdaddr,
        name: Some(name.to_string()),
        rssi: Some(rssi as i16), // Convert i32 to i16 for the rssi field
        manufacturer_data,
        is_potential_airpods: is_airpods,
        last_seen: std::time::Instant::now(),
        is_connected: false,
        service_data: HashMap::new(),
        services: Vec::new(),
    }
}

/// Create a test battery status for testing
fn create_test_battery(left: Option<u8>, right: Option<u8>, case: Option<u8>) -> AirPodsBatteryStatus {
    AirPodsBatteryStatus {
        battery: AirPodsBattery {
            left,
            right,
            case,
            charging: AirPodsCharging {
                left: false,
                right: false,
                case: true,
            },
        },
        last_updated: std::time::Instant::now(),
    }
}

/// Create a message collector for testing state flow
struct MessageCollector {
    messages: Arc<Mutex<Vec<Message>>>,
    sender: mpsc::UnboundedSender<Message>,
    receiver: mpsc::UnboundedReceiver<Message>,
}

impl MessageCollector {
    fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self {
            messages: Arc::new(Mutex::new(Vec::new())),
            sender,
            receiver,
        }
    }
    
    fn sender(&self) -> mpsc::UnboundedSender<Message> {
        self.sender.clone()
    }
    
    async fn collect_messages(&mut self, timeout_ms: u64) -> Vec<Message> {
        let timeout_duration = Duration::from_millis(timeout_ms);
        
        // Start a task to collect messages with a timeout
        let messages_clone = self.messages.clone();
        let mut receiver = std::mem::replace(&mut self.receiver, mpsc::unbounded_channel().1);
        
        // Collect messages until timeout
        tokio::spawn(async move {
            loop {
                match timeout(timeout_duration, receiver.recv()).await {
                    Ok(Some(msg)) => {
                        let mut messages = messages_clone.lock().unwrap();
                        messages.push(msg);
                    },
                    _ => break,
                }
            }
        });
        
        // Wait for timeout
        tokio::time::sleep(timeout_duration).await;
        
        // Return collected messages
        let messages = self.messages.lock().unwrap().clone();
        messages
    }
}

// SECTION: State Flow Tests

/// Test that state flows correctly between components
#[tokio::test]
async fn test_state_flow_between_components() {
    // Create message collector
    let mut collector = MessageCollector::new();
    let sender = collector.sender();
    
    // Create state manager with the sender
    let state_manager = Arc::new(StateManager::new(sender));
    
    // Dispatch actions that should generate UI messages
    state_manager.dispatch(Action::ShowWindow);
    state_manager.dispatch(Action::StartScanning);
    state_manager.dispatch(Action::ShowSettings);
    
    // Add a device that should trigger UI updates
    let test_device = create_test_device("11:22:33:44:55:66", "Test AirPods", -60, true);
    state_manager.dispatch(Action::UpdateDevice(test_device));
    
    // Wait and collect messages that were sent to the UI
    let messages = collector.collect_messages(500).await;
    
    // Verify that expected messages were sent
    assert!(messages.contains(&Message::ShowWindow));
    assert!(messages.contains(&Message::StartScan));
    assert!(messages.contains(&Message::ShowSettings));
    assert!(messages.iter().any(|msg| matches!(msg, Message::UpdateDeviceList(_))));
    
    // Verify that state was also updated correctly
    let device_state = state_manager.get_device_state();
    assert_eq!(device_state.devices.len(), 1);
    
    let ui_state = state_manager.get_ui_state();
    assert!(ui_state.visible);
    assert!(ui_state.show_settings);
}

/// Test cross-component communication via state manager
#[tokio::test]
async fn test_cross_component_communication() {
    // Create message collector
    let mut collector = MessageCollector::new();
    let sender = collector.sender();
    
    // Create state manager with the sender
    let state_manager = Arc::new(StateManager::new(sender));
    
    // Simulate a component updating battery status
    let battery = create_test_battery(Some(80), Some(75), Some(90));
    state_manager.dispatch(Action::UpdateBatteryStatus(battery.clone()));
    
    // Simulate another component toggling UI visibility
    state_manager.dispatch(Action::ToggleVisibility);
    
    // Wait and collect messages
    let messages = collector.collect_messages(500).await;
    
    // Verify that the correct messages were sent to update other components
    assert!(messages.iter().any(|msg| matches!(msg, Message::UpdateBatteryStatus(_))));
    assert!(messages.contains(&Message::ToggleVisibility));
    
    // Verify that battery status is correctly stored in state
    let device_state = state_manager.get_device_state();
    assert!(device_state.battery_status.is_some());
    let stored_battery = device_state.battery_status.unwrap();
    assert_eq!(stored_battery.battery.left, battery.battery.left);
    assert_eq!(stored_battery.battery.right, battery.battery.right);
    assert_eq!(stored_battery.battery.case, battery.battery.case);
}

// SECTION: Animation and Transition Tests

/// Test animation and transition effects via state updates
#[tokio::test]
async fn test_animation_transitions() {
    // Create message collector
    let mut collector = MessageCollector::new();
    let sender = collector.sender();
    
    // Create state manager with the sender
    let state_manager = Arc::new(StateManager::new(sender.clone()));
    
    // Simulate animation progress updates
    for progress in [0.0, 0.25, 0.5, 0.75, 1.0].iter() {
        state_manager.dispatch(Action::UpdateAnimationProgress(*progress));
        
        // Verify progress is correctly updated in state
        let ui_state = state_manager.get_ui_state();
        assert_eq!(ui_state.animation_progress, *progress);
        
        // Sleep briefly to simulate animation frames
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    
    // Collect messages and verify animation updates were sent
    let messages = collector.collect_messages(300).await;
    let animation_updates = messages.iter()
        .filter(|msg| matches!(msg, Message::UpdateAnimationProgress(_)))
        .count();
    
    // Should have received animation progress messages
    assert!(animation_updates > 0);
}

// SECTION: State Persistence Tests

/// Test state persistence across app sessions
#[tokio::test]
async fn test_state_persistence() {
    // Create temporary directory for config file
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("test_config.json");
    
    // First "session"
    {
        // Create message collector
        let collector = MessageCollector::new();
        let sender = collector.sender();
        
        // Create state manager with the sender
        let state_manager = Arc::new(StateManager::new(sender));
        
        // Modify state that should be persisted
        let mut config = state_manager.get_config();
        config.settings_path = config_path.clone();
        config.bluetooth.auto_scan_on_startup = false;
        config.ui.show_notifications = false;
        state_manager.dispatch(Action::UpdateSettings(config));
        
        // Add a device that should be remembered (if applicable)
        let device = create_test_device("11:22:33:44:55:66", "Remembered AirPods", -60, true);
        state_manager.dispatch(Action::UpdateDevice(device.clone()));
        state_manager.dispatch(Action::SelectDevice("11:22:33:44:55:66".to_string()));
        
        // Save state
        state_manager.dispatch(Action::SavePersistentState);
        
        // Wait for save to complete
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        // Verify config file was created
        assert!(config_path.exists());
    }
    
    // Brief pause to ensure file is fully written
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Second "session" - reload state
    {
        // Create message collector
        let collector = MessageCollector::new();
        let sender = collector.sender();
        
        // Create state manager with the sender
        let state_manager = Arc::new(StateManager::new(sender));
        
        // Load saved state
        state_manager.dispatch(Action::LoadPersistentState);
        
        // Wait for load to complete
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        // Verify config was restored
        let config = state_manager.get_config();
        assert_eq!(config.settings_path, config_path);
        assert_eq!(config.bluetooth.auto_scan_on_startup, false);
        assert_eq!(config.ui.show_notifications, false);
    }
    
    // Clean up
    temp_dir.close().unwrap();
}

// SECTION: Error State Handling Tests

/// Test error state handling
#[tokio::test]
async fn test_error_state_handling() {
    // Create message collector
    let mut collector = MessageCollector::new();
    let sender = collector.sender();
    
    // Create state manager with the sender
    let state_manager = Arc::new(StateManager::new(sender));
    
    // Initially should have no error
    let ui_state = state_manager.get_ui_state();
    assert!(ui_state.error_message.is_none());
    
    // Set error message
    let error_message = "Test error message".to_string();
    state_manager.dispatch(Action::SetError(error_message.clone()));
    
    // Verify error state was updated
    let ui_state = state_manager.get_ui_state();
    assert!(ui_state.error_message.is_some());
    assert_eq!(ui_state.error_message.unwrap(), error_message);
    assert!(ui_state.show_error);
    
    // Clear error
    state_manager.dispatch(Action::ClearError);
    
    // Verify error was cleared
    let ui_state = state_manager.get_ui_state();
    assert!(ui_state.error_message.is_none());
    assert!(!ui_state.show_error);
}

/// Test connection state transitions and error handling
#[tokio::test]
async fn test_connection_state_transitions() {
    // Create message collector
    let mut collector = MessageCollector::new();
    let sender = collector.sender();
    
    // Create state manager with the sender
    let state_manager = Arc::new(StateManager::new(sender));
    
    // Initial state should be disconnected
    let device_state = state_manager.get_device_state();
    assert_eq!(device_state.connection_state, ConnectionState::Disconnected);
    
    // Transition to connecting
    state_manager.dispatch(Action::SetConnectionState(ConnectionState::Connecting));
    
    // Verify state update
    let device_state = state_manager.get_device_state();
    assert_eq!(device_state.connection_state, ConnectionState::Connecting);
    
    // Transition to connected
    state_manager.dispatch(Action::SetConnectionState(ConnectionState::Connected));
    
    // Verify state update
    let device_state = state_manager.get_device_state();
    assert_eq!(device_state.connection_state, ConnectionState::Connected);
    
    // Simulate connection failure
    let error_reason = "Device not found".to_string();
    state_manager.dispatch(Action::SetConnectionState(ConnectionState::Failed(error_reason.clone())));
    
    // Verify state update
    let device_state = state_manager.get_device_state();
    match device_state.connection_state {
        ConnectionState::Failed(ref reason) => assert_eq!(reason, &error_reason),
        _ => panic!("Expected Failed connection state"),
    }
    
    // Transition to reconnecting
    state_manager.dispatch(Action::SetConnectionState(ConnectionState::Reconnecting));
    
    // Verify state update
    let device_state = state_manager.get_device_state();
    assert_eq!(device_state.connection_state, ConnectionState::Reconnecting);
}

// SECTION: Advanced Tests for Edge Cases

/// Test that state updates are handled correctly when events occur rapidly
#[tokio::test]
async fn test_rapid_state_updates() {
    // Create message collector
    let collector = MessageCollector::new();
    let sender = collector.sender();
    
    // Create state manager with the sender
    let state_manager = Arc::new(StateManager::new(sender));
    
    // Dispatch multiple actions rapidly
    for i in 0..10 {
        // Toggle visibility rapidly
        state_manager.dispatch(Action::ToggleVisibility);
        
        // Update animation progress
        let progress = i as f32 / 10.0;
        state_manager.dispatch(Action::UpdateAnimationProgress(progress));
        
        // Minimal delay to simulate rapid updates
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    // Verify final state is as expected (should have toggled 10 times)
    let ui_state = state_manager.get_ui_state();
    assert_eq!(ui_state.animation_progress, 0.9);
    
    // If visibility starts as false, and toggles 10 times, it should end as false
    assert!(!ui_state.visible);
}

/// Test that system sleep/wake events are handled properly
#[tokio::test]
async fn test_system_sleep_wake_handling() {
    // Create message collector
    let mut collector = MessageCollector::new();
    let sender = collector.sender();
    
    // Create state manager with the sender
    let state_manager = Arc::new(StateManager::new(sender));
    
    // Add a test device and start scanning
    let test_device = create_test_device("11:22:33:44:55:66", "Test AirPods", -60, true);
    state_manager.dispatch(Action::UpdateDevice(test_device));
    state_manager.dispatch(Action::StartScanning);
    
    // Verify scanning is active
    let device_state = state_manager.get_device_state();
    assert!(device_state.is_scanning);
    
    // Simulate system sleep
    state_manager.dispatch(Action::SystemSleep);
    
    // Wait for processing
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Verify scanning was stopped
    let device_state = state_manager.get_device_state();
    assert!(!device_state.is_scanning);
    
    // Collect messages to verify system component notification
    let messages = collector.collect_messages(300).await;
    assert!(messages.contains(&Message::StopScan));
    
    // Simulate system wake
    state_manager.dispatch(Action::SystemWake);
    
    // Wait for processing
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // If auto_scan is true (default), scanning should have restarted
    let device_state = state_manager.get_device_state();
    assert!(device_state.is_scanning);
} 