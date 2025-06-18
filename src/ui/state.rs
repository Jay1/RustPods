use iced::{executor, Application, Command, Subscription};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use tokio::sync::{mpsc, Mutex};

use crate::airpods::battery::AirPodsBatteryInfo;
use crate::airpods::battery_estimator::BatteryEstimator;
use crate::airpods::battery_intelligence::BatteryIntelligence;
use crate::bluetooth::DiscoveredDevice;
use crate::config::{AppConfig, ConfigError, ConfigManager};
use crate::ui::{
    components::{BluetoothSetting, SystemSetting, UiSetting},
    system_tray::SystemTray,
    MainWindow, Message, SettingsWindow,
};

/// Device detection state for managing UI transitions
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceDetectionState {
    /// Initial state - no scanning has occurred
    Idle,
    /// Currently scanning for devices
    Scanning,
    /// Device found with details
    DeviceFound {
        device_name: String,
        device_address: String,
    },
    /// Devices found and displayed
    DevicesFound,
    /// No devices found (after tolerance period)
    NoDevicesFound,
    /// Connection error with retry information
    Error { message: String, retry_count: u32 },
    /// Successfully connected to device
    Connected {
        device_name: String,
        device_address: String,
    },
}

impl DeviceDetectionState {
    /// Check if there's an active device connection
    pub fn has_active_device(&self) -> bool {
        matches!(
            self,
            DeviceDetectionState::DeviceFound { .. }
                | DeviceDetectionState::DevicesFound
                | DeviceDetectionState::Connected { .. }
        )
    }
}

/// Main application state
#[derive(Debug, Clone)]
pub struct AppState {
    /// Whether the application window is visible
    pub visible: bool,

    /// Discovered Bluetooth devices
    pub devices: HashMap<String, DiscoveredDevice>,

    /// Currently selected device
    pub selected_device: Option<String>,

    /// Timestamp when the current device was connected
    pub connection_timestamp: Option<std::time::Instant>,

    /// Animation progress for refresh button (0.0-1.0)
    pub animation_progress: f32,

    /// Last known battery status
    pub battery_status: Option<crate::bluetooth::AirPodsBatteryStatus>,

    /// Application configuration
    pub config: AppConfig,

    /// Configuration manager
    pub config_manager: Option<ConfigManager>,

    /// Whether settings view is open
    pub show_settings: bool,

    /// Main window component
    pub main_window: MainWindow,

    /// Settings window component
    pub settings_window: SettingsWindow,

    /// Current settings error message (if any)
    pub settings_error: Option<String>,

    /// System tray component
    pub system_tray: Option<SystemTray>,

    /// Persistent status message (for status feedback)
    pub status_message: Option<String>,

    /// Current toast/notification message (temporary)
    pub toast_message: Option<String>,

    /// Channel to send messages to the controller
    pub controller_sender: mpsc::UnboundedSender<Message>,

    /// Merged Bluetooth devices
    pub merged_devices: Vec<MergedBluetoothDevice>,

    /// AirPods data loaded from the CLI scanner
    pub airpods_devices: Vec<AirPodsBatteryInfo>,

    /// Timestamp of the last AirPods data update
    pub last_update: std::time::Instant,

    /// Battery estimator for intelligent prediction between 10% increments
    pub battery_estimator: BatteryEstimator,

    /// New intelligent battery management system
    pub battery_intelligence: BatteryIntelligence,

    /// Device detection state
    pub device_detection_state: DeviceDetectionState,

    /// Consecutive scan failures counter (to prevent flashing on intermittent disconnections)
    pub consecutive_scan_failures: u32,
}

// Global receiver for controller messages (needed for subscription)
static CONTROLLER_RECEIVER: OnceLock<Arc<Mutex<Option<mpsc::UnboundedReceiver<Message>>>>> =
    OnceLock::new();

impl AppState {
    /// Create a new AppState with the given controller sender
    pub fn new(controller_sender: mpsc::UnboundedSender<Message>) -> Self {
        let config = AppConfig::default();
        let main_window = MainWindow::empty();
        let settings_window = SettingsWindow::new(config.clone());

        // Initialize battery estimator with saved history data
        let mut battery_estimator = BatteryEstimator::new();
        battery_estimator.left_history = config.battery.left_history.clone();
        battery_estimator.right_history = config.battery.right_history.clone();
        battery_estimator.case_history = config.battery.case_history.clone();

        // Create and initialize system tray
        let system_tray = match SystemTray::new(config.clone()) {
            Ok(mut tray) => {
                // Set the UI sender for fallback communication
                tray.set_ui_sender(controller_sender.clone());

                // Initialize the system tray (creates the actual icon and menu)
                match tray.initialize() {
                    Ok(_) => {
                        log::info!("System tray initialized successfully");
                        Some(tray)
                    }
                    Err(e) => {
                        log::error!("Failed to initialize system tray: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to create system tray: {}", e);
                None
            }
        };

        // Initialize the new BatteryIntelligence system
        let battery_intelligence_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("RustPods")
            .join("battery_intelligence");
        let mut battery_intelligence = BatteryIntelligence::new(battery_intelligence_dir);

        // Load existing device profiles
        if let Err(e) = battery_intelligence.load() {
            eprintln!(
                "Warning: Failed to load existing battery intelligence data: {}",
                e
            );
        }

        // Print initialization message
        println!("Battery Intelligence system initialized - profiles will be created when devices are detected");

        Self {
            visible: true,
            devices: HashMap::new(),
            selected_device: None,
            connection_timestamp: None,
            animation_progress: 0.0,
            battery_status: None,
            config,
            config_manager: None,
            show_settings: false,
            main_window,
            settings_window,
            settings_error: None,
            system_tray,
            status_message: None,
            toast_message: None,
            controller_sender,
            merged_devices: Vec::new(),
            airpods_devices: Vec::new(),
            last_update: std::time::Instant::now(),
            battery_estimator,
            battery_intelligence,
            device_detection_state: DeviceDetectionState::Idle,
            consecutive_scan_failures: 0,
        }
    }

    /// Create a new AppState for testing without CLI scanner integration
    #[cfg(test)]
    pub fn new_for_test(controller_sender: mpsc::UnboundedSender<Message>) -> Self {
        let config = AppConfig::default();
        let config_manager = None;
        let settings_window = SettingsWindow::new(config.clone());
        let main_window = MainWindow::empty();

        Self {
            visible: true,
            devices: HashMap::new(),
            selected_device: None,
            connection_timestamp: None,
            animation_progress: 0.0,
            battery_status: None,
            config,
            config_manager,
            show_settings: false,
            main_window,
            settings_window,
            settings_error: None,
            system_tray: None,
            status_message: None,
            toast_message: None,
            controller_sender,
            merged_devices: Vec::new(),
            airpods_devices: Vec::new(),
            last_update: std::time::Instant::now(),
            battery_estimator: BatteryEstimator::new(),
            battery_intelligence: BatteryIntelligence::new(std::path::PathBuf::from(
                "./test_battery_intelligence",
            )),
            device_detection_state: DeviceDetectionState::Idle,
            consecutive_scan_failures: 0,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        Self::new(tx)
    }
}

impl Application for AppState {
    type Message = Message;
    type Theme = crate::ui::theme::Theme;
    type Executor = executor::Default;
    type Flags = (
        mpsc::UnboundedSender<Message>,
        mpsc::UnboundedReceiver<Message>,
    );

    fn new((controller_sender, controller_receiver): Self::Flags) -> (Self, Command<Message>) {
        // Store the receiver in the global static for the subscription to use
        let receiver_arc = Arc::new(Mutex::new(Some(controller_receiver)));
        CONTROLLER_RECEIVER
            .set(receiver_arc.clone())
            .expect("Controller receiver already set");

        log::info!("AppState::new: Creating new application state with system tray communication");

        let app_state = Self::new(controller_sender);

        // Return a command that triggers initial AirPods scanning for immediate detection
        log::info!("Scheduling initial AirPods scan on startup");
        let initial_command = Command::perform(
            async {
                tokio::task::spawn_blocking(get_airpods_from_cli_scanner)
                    .await
                    .unwrap_or_else(|_| Vec::new())
            },
            Message::AirPodsDataLoaded,
        );

        (app_state, initial_command)
    }

    fn title(&self) -> String {
        String::from("RustPods - AirPods Battery Monitor")
    }

    fn theme(&self) -> Self::Theme {
        crate::ui::theme::Theme::CatppuccinMocha
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        crate::debug_log!("ui", "AppState::update: instance at {:p}", self as *const _);

        // Process system tray events
        if let Some(ref mut system_tray) = self.system_tray {
            if let Err(e) = system_tray.process_events() {
                log::error!("Failed to process system tray events: {}", e);
            }
        }

        match message {
            Message::ToggleVisibility => {
                self.toggle_visibility();
                Command::none()
            }
            Message::ToggleWindow => {
                self.toggle_visibility();
                Command::none()
            }
            Message::Exit => {
                // Check if minimize to tray is enabled
                if self.config.ui.minimize_to_tray_on_close {
                    log::info!("Minimizing to tray instead of exiting (hiding window but keeping event loop alive)");
                    self.visible = false;
                    // Save settings when minimizing to tray
                    if let Err(e) = self.config.save() {
                        log::error!("Failed to save settings: {}", e);
                    }
                    iced::window::change_mode(iced::window::Mode::Hidden)
                } else {
                    log::info!("Exiting application");
                    std::process::exit(0);
                }
            }
            Message::ForceQuit => {
                log::info!("ForceQuit message received - initiating graceful shutdown");

                // Use std::process::exit for force quit to avoid Tokio runtime shutdown issues
                // Graphics resources are properly cleaned up before this point (verified by testing)
                std::process::exit(0);
            }
            Message::NoOp => {
                // No operation - used for subscription management, do nothing
                Command::none()
            }
            Message::WindowCloseRequested => {
                log::info!("Window close requested - handling based on minimize to tray setting");
                if self.config.ui.minimize_to_tray_on_close {
                    log::info!("Minimizing to tray instead of closing");
                    self.visible = false;
                    // Save settings when minimizing to tray
                    if let Err(e) = self.config.save() {
                        log::error!("Failed to save settings on minimize: {}", e);
                    }
                    // Use Iced's proper window hiding command
                    iced::window::change_mode(iced::window::Mode::Hidden)
                } else {
                    // Exit the application normally
                    self.update(Message::Exit)
                }
            }
            Message::ShowWindow => {
                log::info!(
                    "ShowWindow message received from system tray, current visible: {}",
                    self.visible
                );
                if !self.visible {
                    // Window was hidden, need to restore it
                    self.visible = true;
                    log::info!("Window visibility set to true, restoring window from hidden/minimized state");
                    // Use Iced's window restoration command to properly show the window
                    iced::window::change_mode(iced::window::Mode::Windowed)
                } else {
                    // Window is already visible, but may be minimized - force restore
                    log::info!("Window already marked visible, forcing window restore and focus");
                    iced::window::change_mode(iced::window::Mode::Windowed)
                }
            }
            Message::HideWindow => {
                log::info!("HideWindow message received from system tray");
                self.visible = false;
                // Use Iced's proper window hiding command
                iced::window::change_mode(iced::window::Mode::Hidden)
            }
            Message::DeviceDiscovered(device) => {
                log::debug!("[UI] Device discovered: {:?}", device);
                self.update_device(device);
                Command::none()
            }
            Message::DeviceUpdated(device) => {
                self.update_device(device);
                Command::none()
            }
            Message::SelectDevice(address) => {
                self.select_device(address.clone());
                Command::none()
            }
            Message::BatteryStatusUpdated(status) => {
                let status_clone = status.clone();
                self.battery_status = Some(status);
                if let Some(_device) = self.get_selected_device() {
                    self.main_window = MainWindow::new()
                        .with_animation_progress(self.animation_progress)
                        .with_battery_status(status_clone.clone());
                }
                Command::none()
            }
            Message::AirPodsConnected(airpods) => {
                crate::debug_log!("bluetooth", "Connected to AirPods: {:?}", airpods);
                if let Some(address) = self.selected_device.as_ref() {
                    if let Some(device) = self.devices.get_mut(address) {
                        device.is_potential_airpods = true;
                    }
                }
                Command::none()
            }
            Message::BluetoothError(msg) => {
                crate::debug_log!("ui", "AppState::update: setting toast_message = {:?}", msg);
                self.toast_message = Some(msg);
                Command::none()
            }
            Message::UpdateBluetoothSetting(setting) => {
                self.settings_window.mark_changed();
                self.update_bluetooth_setting(setting);
                self.settings_window.update_config(self.config.clone());
                Command::none()
            }
            Message::UpdateUiSetting(setting) => {
                self.settings_window.mark_changed();
                self.update_ui_setting(setting);
                self.settings_window.update_config(self.config.clone());
                Command::none()
            }
            Message::UpdateSystemSetting(setting) => {
                self.settings_window.mark_changed();
                self.update_system_setting(setting);
                self.settings_window.update_config(self.config.clone());
                Command::none()
            }
            Message::OpenSettings => {
                self.settings_window.set_validation_error(None);
                self.settings_window.update_config(self.config.clone());
                self.show_settings = true;
                Command::none()
            }
            Message::CloseSettings => {
                self.show_settings = false;
                self.settings_window.update_config(self.config.clone());
                Command::none()
            }
            Message::SaveSettings => {
                let updated_config = self.settings_window.config();
                if let Err(e) = updated_config.validate() {
                    self.settings_window
                        .set_validation_error(Some(e.to_string()));
                    log::error!("Settings validation failed: {}", e);
                    return Command::none();
                }
                self.config = updated_config.clone();
                if let Err(e) = self.config.save() {
                    self.settings_window
                        .set_validation_error(Some(format!("Failed to save: {}", e)));
                    log::error!("Settings save failed: {}", e);
                    return Command::none();
                }
                self.apply_settings();
                self.show_settings = false;
                Command::none()
            }
            Message::SettingsChanged(config) => {
                self.config = config.clone();
                self.settings_window.update_config(config);
                Command::none()
            }
            Message::ShowToast(msg) => {
                self.toast_message = Some(msg);
                Command::perform(
                    async {
                        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    },
                    |_| Message::Tick,
                )
            }
            Message::MergedScanResult(devices) => {
                crate::debug_log!(
                    "ui",
                    "MergedScanResult received with {} devices",
                    devices.len()
                );

                // Clear existing devices and update with fresh data from async CLI scanner
                self.merged_devices.clear();
                self.merged_devices = devices.clone();

                // Update the main window with the new devices
                self.main_window.merged_devices = devices.clone();

                // Set status message only when no devices are found
                if devices.is_empty() {
                    self.status_message = Some("No AirPods devices found".to_string());
                } else {
                    // Clear status message when devices are found - only keep it for warnings/errors
                    self.status_message = None;
                }

                crate::debug_log!(
                    "ui",
                    "Total merged devices after async update: {}",
                    self.merged_devices.len()
                );
                Command::none()
            }
            Message::Tick => {
                crate::debug_log!("ui", "Tick message received - performing continuous scan");
                // Use the continuous scanning function for periodic updates
                Command::perform(
                    async {
                        tokio::task::spawn_blocking(get_airpods_from_cli_scanner_continuous)
                            .await
                            .unwrap_or_else(|_| Vec::new())
                    },
                    Message::AirPodsDataLoaded,
                )
            }
            Message::AirPodsDataLoaded(airpods_data) => {
                // Handle the result of the async AirPods data loading
                log::info!("AirPods data loaded: {} devices found", airpods_data.len());
                crate::debug_log!(
                    "airpods",
                    "AirPods data loaded: {} devices found",
                    airpods_data.len()
                );
                for (i, device) in airpods_data.iter().enumerate() {
                    crate::debug_log!(
                        "airpods",
                        "Device {}: {} - L:{}% R:{}% C:{}%",
                        i,
                        device.name,
                        device.left_battery,
                        device.right_battery,
                        device.case_battery
                    );
                }

                // Update device detection state based on scan results with tolerance mechanism
                if airpods_data.is_empty() {
                    // Increment consecutive failure counter
                    self.consecutive_scan_failures += 1;

                    crate::debug_log!(
                        "airpods",
                        "No AirPods found in scan #{} (consecutive failures: {})",
                        self.consecutive_scan_failures,
                        self.consecutive_scan_failures
                    );

                    // Only change to NoDevicesFound after 3 consecutive failures
                    // This prevents flashing when the scanner is temporarily intermittent
                    if self.consecutive_scan_failures >= 3 {
                        // Only change state if we're not already in NoDevicesFound
                        if self.device_detection_state != DeviceDetectionState::NoDevicesFound {
                            crate::debug_log!(
                                "airpods",
                                "Switching to NoDevicesFound state after {} consecutive failures",
                                self.consecutive_scan_failures
                            );
                            self.device_detection_state = DeviceDetectionState::NoDevicesFound;
                        }
                    } else {
                        // Still in tolerance period - keep current state if devices were previously found
                        if self.device_detection_state == DeviceDetectionState::DevicesFound {
                            crate::debug_log!(
                                "airpods",
                                "Keeping DevicesFound state during tolerance period (failure {}/3)",
                                self.consecutive_scan_failures
                            );
                        }
                    }
                } else {
                    // Reset consecutive failure counter when devices are found
                    if self.consecutive_scan_failures > 0 {
                        crate::debug_log!(
                            "airpods",
                            "Resetting consecutive failure counter (was {})",
                            self.consecutive_scan_failures
                        );
                        self.consecutive_scan_failures = 0;
                    }

                    // Update to DevicesFound state
                    if self.device_detection_state != DeviceDetectionState::DevicesFound {
                        crate::debug_log!("airpods", "Switching to DevicesFound state");
                        self.device_detection_state = DeviceDetectionState::DevicesFound;
                    }
                }

                // Update the state with the loaded AirPods data
                self.airpods_devices = airpods_data;
                self.last_update = std::time::Instant::now();

                // Update the merged devices to include the new AirPods data
                self.update_merged_devices();

                // Update the main window's device detection state to match the AppState
                self.main_window
                    .update_device_detection_state(self.device_detection_state.clone());

                Command::none()
            }
            // Window drag handling
            Message::WindowDragStart(_point) => {
                crate::debug_log!("ui", "Window drag started");
                iced::window::drag()
            }
            Message::WindowDragEnd => {
                crate::debug_log!("ui", "Window drag ended");
                Command::none()
            }
            Message::WindowDragMove(_point) => {
                // Handle window drag move if needed
                Command::none()
            }
            Message::SetDeviceName(name) => {
                self.config.bluetooth.paired_device_name = if name.trim().is_empty() {
                    None
                } else {
                    Some(name.trim().to_string())
                };
                if let Err(e) = self.config.save() {
                    log::error!("Failed to save device name: {}", e);
                } else {
                    // Update both the settings window and main window with the new config
                    self.settings_window.update_config(self.config.clone());
                    self.main_window.config = self.config.clone();

                    // Notify BatteryIntelligence about device name change for the selected device
                    if let Some(selected_device_id) = &self.selected_device {
                        let device_name = self
                            .config
                            .bluetooth
                            .paired_device_name
                            .as_deref()
                            .unwrap_or("AirPods Pro 2"); // Default name if none set

                        // This will trigger the file rename if the name changed
                        let _profile_updated = self
                            .battery_intelligence
                            .ensure_device_profile(selected_device_id, device_name);

                        // Save the updated profile
                        if let Err(e) = self.battery_intelligence.save() {
                            log::error!(
                                "Failed to save battery intelligence after device name change: {}",
                                e
                            );
                        } else {
                            crate::debug_log!(
                                "battery",
                                "Updated device name for {} to {}",
                                selected_device_id,
                                device_name
                            );
                        }
                    }

                    self.toast_message = Some("Device name saved".to_string());
                }
                Command::none()
            }
            Message::OpenProfileFolder => {
                let profile_dir =
                    crate::airpods::battery_intelligence::get_battery_intelligence_dir();

                #[cfg(target_os = "windows")]
                {
                    if let Err(e) = std::process::Command::new("explorer")
                        .arg(&profile_dir)
                        .spawn()
                    {
                        log::error!("Failed to open profile folder: {}", e);
                        self.toast_message = Some("Failed to open profile folder".to_string());
                    } else {
                        self.toast_message = Some("Profile folder opened".to_string());
                    }
                }

                #[cfg(not(target_os = "windows"))]
                {
                    self.toast_message = Some(format!("Profile folder: {}", profile_dir.display()));
                }

                Command::none()
            }
            Message::PurgeProfiles => {
                match self.battery_intelligence.purge_all_profiles() {
                    Ok(_) => {
                        self.toast_message =
                            Some("Purged battery intelligence profiles".to_string());
                        log::info!("Purged battery intelligence profiles");
                    }
                    Err(e) => {
                        self.toast_message = Some("Failed to purge profiles".to_string());
                        log::error!("Failed to purge battery profiles: {}", e);
                    }
                }
                Command::none()
            }
            _ => {
                crate::debug_log!(
                    "ui",
                    "Unhandled message in state.rs: {:?}",
                    std::any::type_name::<Message>()
                );
                Command::none()
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Message, iced::Renderer<crate::ui::theme::Theme>> {
        if !self.visible {
            iced::widget::text("").into()
        } else if self.show_settings {
            // Just show the settings content with full size - no overlays
            crate::ui::UiComponent::view(&self.settings_window)
        } else {
            // Just show the main window content with full size - no overlays
            crate::ui::UiComponent::view(&self.main_window)
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        use iced::time;
        use std::time::Duration;

        // Timer for periodic CLI scanner updates (every 10 seconds for good responsiveness)
        let timer = time::every(Duration::from_secs(10)).map(|_| Message::Tick);

        // Controller subscription for system tray communication
        let controller_subscription = iced::subscription::unfold(
            "controller-messages",
            (),
            |_state| async move {
                crate::debug_log!(
                    "ui",
                    "Controller subscription: Checking for messages from system tray"
                );
                // Access the global receiver safely
                if let Some(receiver_arc) = CONTROLLER_RECEIVER.get() {
                    let mut guard = receiver_arc.lock().await;
                    if let Some(ref mut receiver) = *guard {
                        match receiver.recv().await {
                            Some(message) => {
                                crate::debug_log!(
                                    "ui",
                                    "Controller subscription: Received message: {:?}",
                                    message
                                );
                                log::info!("Controller subscription received message from system tray: {:?}", message);
                                (message, ())
                            }
                            None => {
                                crate::debug_log!(
                                    "ui",
                                    "Controller subscription: Channel closed, no more messages"
                                );
                                log::warn!(
                                    "Controller channel closed - system tray communication lost"
                                );
                                // Channel closed, wait and try again
                                tokio::time::sleep(Duration::from_millis(100)).await;
                                (Message::NoOp, ())
                            }
                        }
                    } else {
                        crate::debug_log!("ui", "Controller subscription: No receiver available");
                        log::warn!("No controller receiver available");
                        // No receiver, wait and try again
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        (Message::NoOp, ())
                    }
                } else {
                    crate::debug_log!("ui", "Controller subscription: CONTROLLER_RECEIVER not set");
                    log::warn!("CONTROLLER_RECEIVER not initialized");
                    // Not initialized yet, wait and try again
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    (Message::NoOp, ())
                }
            },
        );

        Subscription::batch(vec![
            timer, // Add the timer subscription for periodic CLI scanner updates
            iced::subscription::events_with(|event, _status| {
                if let iced::Event::Window(iced::window::Event::CloseRequested) = event {
                    Some(Message::WindowCloseRequested)
                } else {
                    None
                }
            }),
            controller_subscription, // Add the controller subscription for system tray communication
        ])
    }
}

impl AppState {
    /// Toggle the visibility of the application
    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }

    /// Update a device in the devices list
    pub fn update_device(&mut self, device: DiscoveredDevice) {
        let address = device.address.to_string();
        self.devices.insert(address, device);
    }

    /// Remove a device and clear selection if it was selected
    pub fn remove_device(&mut self, address: &str) {
        self.devices.remove(address);
        self.check_selected_device();
    }

    /// Select a device by address
    pub fn select_device(&mut self, address: String) {
        // Only set if it exists
        if self.devices.contains_key(&address) {
            self.selected_device = Some(address);
            // Set the connection timestamp to now
            self.connection_timestamp = Some(std::time::Instant::now());
        }
    }

    /// Check if selected device exists and clear if not
    pub fn check_selected_device(&mut self) {
        if let Some(selected) = &self.selected_device {
            if !self.devices.contains_key(selected) {
                self.selected_device = None;
            }
        }
    }

    /// Get the currently selected device
    pub fn get_selected_device(&self) -> Option<&DiscoveredDevice> {
        self.selected_device
            .as_ref()
            .and_then(|addr| self.devices.get(addr))
    }

    /// Save the current settings
    #[allow(dead_code)]
    fn save_settings(&mut self) {
        // Validate settings first
        match self.config.validate() {
            Ok(_) => {
                // Save to file
                match self.config.save() {
                    Ok(_) => {
                        // Clear any previous error
                        self.settings_error = None;

                        // Log success
                        log::info!("Settings saved successfully");
                    }
                    Err(e) => {
                        // Set error message
                        self.settings_error = Some(format!("Failed to save settings: {}", e));

                        // Log error
                        log::error!("Failed to save settings: {}", e);
                    }
                }
            }
            Err(e) => {
                // Set validation error message
                self.settings_error = Some(format!("Invalid settings: {}", e));

                // Log error
                log::error!("Settings validation failed: {}", e);
            }
        }
    }

    /// Reset settings to defaults
    #[allow(dead_code)]
    fn reset_settings(&mut self) {
        // Create a new default config but preserve the settings path
        let settings_path = self.config.settings_path.clone();
        self.config = AppConfig::default();
        self.config.settings_path = settings_path;

        // Clear any error
        self.settings_error = None;

        log::info!("Settings reset to defaults");
    }

    /// Load settings from disk
    #[allow(dead_code)]
    fn load_settings(&mut self) -> Result<(), ConfigError> {
        match AppConfig::load() {
            Ok(config) => {
                self.config = config;
                Ok(())
            }
            Err(e) => {
                self.settings_error = Some(format!("Failed to load settings: {}", e));
                log::error!("Failed to load settings: {}", e);
                Err(e)
            }
        }
    }

    /// Apply settings to the application
    fn apply_settings(&mut self) {
        // Update all components with new settings

        // Update main window theme
        // (This would be implemented in a real app by applying theme settings
        // to all UI components)

        // Update system tray configuration if available
        /*
        if let Some(system_tray) = &mut self.system_tray {
            if let Err(e) = system_tray.update_config(self.config.clone()) {
                log::error!("Failed to update system tray config: {}", e);
            }
        }
        */

        log::info!("Settings applied");
    }

    /// Update a Bluetooth setting
    fn update_bluetooth_setting(&mut self, setting: BluetoothSetting) {
        match setting {
            BluetoothSetting::DeviceName(value) => {
                self.config.bluetooth.paired_device_name = if value.trim().is_empty() {
                    None
                } else {
                    Some(value.trim().to_string())
                };
            }
        }
    }

    /// Update a UI setting
    fn update_ui_setting(&mut self, setting: UiSetting) {
        match setting {
            UiSetting::ShowNotifications(value) => {
                self.config.ui.show_notifications = value;
            }
            UiSetting::StartMinimized(value) => {
                self.config.ui.start_minimized = value;
            }
            UiSetting::Theme(value) => {
                self.config.ui.theme = value.into();
            }
            UiSetting::ShowPercentageInTray(value) => {
                self.config.ui.show_percentage_in_tray = value;
            }
            UiSetting::ShowLowBatteryWarning(value) => {
                self.config.ui.show_low_battery_warning = value;
            }
            UiSetting::LowBatteryThreshold(value) => {
                self.config.ui.low_battery_threshold = value;
            }
            UiSetting::MinimizeToTrayOnClose(value) => {
                self.config.ui.minimize_to_tray_on_close = value;
            }
        }
    }

    /// Update a system setting
    fn update_system_setting(&mut self, setting: SystemSetting) {
        // Apply the system setting change
        match setting {
            SystemSetting::StartOnBoot(value) => {
                self.config.system.launch_at_startup = value;
            }
        }

        // Update system tray if available
        /*
        if let Some(system_tray) = &mut self.system_tray {
            if let Err(e) = system_tray.update_menu(&self.config) {
                log::error!("Failed to update system tray menu: {}", e);
            }
        }
        */
    }

    pub fn clear_status_message(&mut self) {
        crate::debug_log!(
            "ui",
            "AppState::clear_status_message: setting status_message = None"
        );
        self.status_message = None;
    }

    pub fn clear_toast_message(&mut self) {
        crate::debug_log!(
            "ui",
            "AppState::clear_toast_message: setting toast_message = None"
        );
        self.toast_message = None;
    }

    /// Refresh device data from the CLI scanner (fast synchronous call in async command)
    #[allow(dead_code)]
    fn refresh_device_data(&mut self) {
        crate::debug_log!(
            "ui",
            "refresh_device_data called at {:?}",
            std::time::SystemTime::now()
        );

        // Use Command::perform to run the CLI scanner without blocking the UI
        // This is much faster than tokio::spawn and still prevents UI freezing
    }

    /// Create a command to refresh device data from CLI scanner (now uses continuous mode for reliability)
    pub fn refresh_device_data_command() -> Command<Message> {
        Command::perform(
            async {
                // Always use continuous scanning mode for maximum reliability
                // This ensures we find AirPods regardless of timing quirks
                let airpods_data = get_airpods_from_cli_scanner();
                crate::debug_log!(
                    "bluetooth",
                    "CLI scanner returned {} AirPods devices",
                    airpods_data.len()
                );

                // Convert to MergedBluetoothDevice format
                airpods_data
                    .into_iter()
                    .map(|airpods| MergedBluetoothDevice {
                        name: if airpods.name.is_empty() {
                            "AirPods".to_string()
                        } else {
                            airpods.name
                        },
                        address: airpods.canonical_address.clone(),
                        paired: true,
                        connected: true,
                        device_type: DeviceType::AirPods,
                        battery: if airpods.left_battery >= 0 {
                            Some(airpods.left_battery as u8)
                        } else {
                            None
                        }
                        .or(if airpods.right_battery >= 0 {
                            Some(airpods.right_battery as u8)
                        } else {
                            None
                        }),
                        left_battery: if airpods.left_battery >= 0 {
                            Some(airpods.left_battery as u8)
                        } else {
                            None
                        },
                        right_battery: if airpods.right_battery >= 0 {
                            Some(airpods.right_battery as u8)
                        } else {
                            None
                        },
                        case_battery: if airpods.case_battery >= 0 {
                            Some(airpods.case_battery as u8)
                        } else {
                            None
                        },
                        left_battery_fractional: None, // No estimation for this path
                        right_battery_fractional: None,
                        case_battery_fractional: None,
                        device_subtype: Some("earbud".to_string()),
                        left_in_ear: airpods.left_in_ear,
                        right_in_ear: airpods.right_in_ear,
                        case_lid_open: airpods.case_lid_open,
                        side: airpods.side.map(|s| s.to_string()),
                        both_in_case: airpods.both_in_case,
                        color: airpods.color.map(|c| c.to_string()),
                        switch_count: airpods.switch_count.map(|s| s as u8),
                        is_connected: true,
                        last_seen: std::time::SystemTime::now(),
                        rssi: airpods.rssi.map(|r| r as i16),
                        manufacturer_data: airpods
                            .raw_manufacturer_data
                            .clone()
                            .map(|s| s.into_bytes())
                            .unwrap_or_default(),
                    })
                    .collect()
            },
            Message::MergedScanResult,
        )
    }

    /// Update the merged devices with the loaded AirPods data
    pub fn update_merged_devices(&mut self) {
        crate::debug_log!(
            "ui",
            "Updating merged devices with {} AirPods devices",
            self.airpods_devices.len()
        );

        // Only update merged devices if we have actual AirPods data
        // This prevents clearing devices when the CLI scanner temporarily returns empty results
        if !self.airpods_devices.is_empty() {
            // Clear existing merged devices only when we have new data to replace them
            self.merged_devices.clear();

            // Auto-select first device if none selected
            if self.selected_device.is_none() && !self.airpods_devices.is_empty() {
                let first_device_id = self.generate_stable_device_id(&self.airpods_devices[0]);
                crate::debug_log!(
                    "battery",
                    "Auto-selecting first available device: {}",
                    first_device_id
                );
                self.selected_device = Some(first_device_id);
            }

            // Update battery intelligence system ONLY for the selected device if estimation is enabled
            if self.config.battery.enable_estimation {
                // Only create and update profiles for the selected device (keep existing profiles for other devices)
                if let Some(selected_device_id) = &self.selected_device {
                    if let Some(selected_airpods) = self.airpods_devices.iter().find(|airpods| {
                        self.generate_stable_device_id(airpods) == *selected_device_id
                    }) {
                        crate::debug_log!(
                            "battery",
                            "Updating BatteryIntelligence for selected device: {}",
                            selected_airpods.name
                        );

                        // Generate stable device ID for this device (singleton pattern)
                        let stable_device_id = self.generate_stable_device_id(selected_airpods);

                        // Ensure device profile exists (singleton pattern - always updates the one profile)
                        let _is_new_device = self
                            .battery_intelligence
                            .ensure_device_profile(&stable_device_id, &selected_airpods.name);

                        // Update the BatteryIntelligence system with device data (singleton pattern)
                        self.battery_intelligence.update_device_battery(
                            &stable_device_id,
                            &selected_airpods.name,
                            Some(selected_airpods.left_battery.max(0).min(100) as u8),
                            Some(selected_airpods.right_battery.max(0).min(100) as u8),
                            Some(selected_airpods.case_battery.max(0).min(100) as u8),
                            selected_airpods.left_charging,
                            selected_airpods.right_charging,
                            selected_airpods.case_charging,
                            selected_airpods.left_in_ear.unwrap_or(false),
                            selected_airpods.right_in_ear.unwrap_or(false),
                            selected_airpods.rssi.map(|r| r as i16),
                        );

                        // Save the BatteryIntelligence data after updates
                        if let Err(e) = self.battery_intelligence.save() {
                            eprintln!("Warning: Failed to save battery intelligence data: {}", e);
                        }

                        // Also update the old estimator for backward compatibility during transition
                        self.battery_estimator.update_real_data(
                            Some(selected_airpods.left_battery),
                            Some(selected_airpods.right_battery),
                            Some(selected_airpods.case_battery),
                        );
                    } else {
                        crate::debug_log!(
                            "battery",
                            "Selected device {} not found in current AirPods scan results",
                            selected_device_id
                        );
                    }
                } else {
                    crate::debug_log!(
                        "battery",
                        "No device selected, skipping BatteryIntelligence update"
                    );
                }

                // Save updated battery estimator data to config (for backward compatibility)
                let (_left_est, _right_est, _case_est) =
                    self.battery_estimator.get_estimated_levels();
                self.config.battery.left_history = self.battery_estimator.left_history.clone();
                self.config.battery.right_history = self.battery_estimator.right_history.clone();
                self.config.battery.case_history = self.battery_estimator.case_history.clone();
            }

            // Get estimated battery levels with fractional precision for 1% increments using BatteryIntelligence
            let (
                left_estimate,
                right_estimate,
                case_estimate,
                left_fractional,
                right_fractional,
                case_fractional,
            ) = if self.config.battery.enable_estimation && !self.airpods_devices.is_empty() {
                // Get estimates from the singleton battery intelligence if available
                if let Some(_selected_device_id) = &self.selected_device {
                    if let Some((left_est, right_est, case_est)) =
                        self.battery_intelligence.get_battery_estimates()
                    {
                        (
                            Some(left_est.level.round().max(0.0).min(100.0) as u8),
                            Some(right_est.level.round().max(0.0).min(100.0) as u8),
                            Some(case_est.level.round().max(0.0).min(100.0) as u8),
                            Some(left_est.level.round()), // Round fractional values to whole percentages
                            Some(right_est.level.round()), // Round fractional values to whole percentages
                            Some(case_est.level.round()), // Round fractional values to whole percentages
                        )
                    } else {
                        // Fallback to old estimator if BatteryIntelligence doesn't have data yet
                        let (left_est, right_est, case_est) =
                            self.battery_estimator.get_estimated_levels();
                        (
                            if left_est.level >= 0.0 {
                                Some(left_est.level.round() as u8)
                            } else {
                                None
                            },
                            if right_est.level >= 0.0 {
                                Some(right_est.level.round() as u8)
                            } else {
                                None
                            },
                            if case_est.level >= 0.0 {
                                Some(case_est.level.round() as u8)
                            } else {
                                None
                            },
                            if left_est.level >= 0.0 {
                                Some(left_est.level.round())
                            } else {
                                None
                            }, // Round fractional values
                            if right_est.level >= 0.0 {
                                Some(right_est.level.round())
                            } else {
                                None
                            }, // Round fractional values
                            if case_est.level >= 0.0 {
                                Some(case_est.level.round())
                            } else {
                                None
                            }, // Round fractional values
                        )
                    }
                } else {
                    // No selected device, use fallback estimator
                    let (left_est, right_est, case_est) =
                        self.battery_estimator.get_estimated_levels();
                    (
                        if left_est.level >= 0.0 {
                            Some(left_est.level.round() as u8)
                        } else {
                            None
                        },
                        if right_est.level >= 0.0 {
                            Some(right_est.level.round() as u8)
                        } else {
                            None
                        },
                        if case_est.level >= 0.0 {
                            Some(case_est.level.round() as u8)
                        } else {
                            None
                        },
                        if left_est.level >= 0.0 {
                            Some(left_est.level.round())
                        } else {
                            None
                        }, // Round fractional values
                        if right_est.level >= 0.0 {
                            Some(right_est.level.round())
                        } else {
                            None
                        }, // Round fractional values
                        if case_est.level >= 0.0 {
                            Some(case_est.level.round())
                        } else {
                            None
                        }, // Round fractional values
                    )
                }
            } else {
                (None, None, None, None, None, None)
            };

            // Add AirPods devices to the merged devices
            self.merged_devices
                .extend(self.airpods_devices.iter().map(|airpods| {
                    crate::debug_log!(
                        "airpods",
                        "Converting AirPods device: {} - L:{}% R:{}% C:{}%",
                        airpods.name,
                        airpods.left_battery,
                        airpods.right_battery,
                        airpods.case_battery
                    );

                    // Use estimated levels if available and enabled, otherwise use raw data
                    let left_battery = if self.config.battery.enable_estimation {
                        left_estimate.unwrap_or(airpods.left_battery as u8)
                    } else {
                        airpods.left_battery as u8
                    };

                    let right_battery = if self.config.battery.enable_estimation {
                        right_estimate.unwrap_or(airpods.right_battery as u8)
                    } else {
                        airpods.right_battery as u8
                    };

                    let case_battery = if self.config.battery.enable_estimation {
                        case_estimate.unwrap_or(airpods.case_battery as u8)
                    } else {
                        airpods.case_battery as u8
                    };

                    crate::debug_log!(
                        "airpods",
                        "Final merged device - L:{}% R:{}% C:{}%",
                        left_battery,
                        right_battery,
                        case_battery
                    );

                    // NOTE: Removed old logging system call - BatteryIntelligence handles all logging now
                    // The old log_battery_data() created file spam and is replaced by smart significance filtering

                    MergedBluetoothDevice {
                        name: airpods.name.clone(),
                        address: airpods.canonical_address.clone(),
                        paired: true,
                        connected: true,
                        device_type: DeviceType::AirPods,
                        battery: Some(left_battery).or(Some(right_battery)),
                        left_battery: Some(left_battery),
                        right_battery: Some(right_battery),
                        case_battery: Some(case_battery),
                        left_battery_fractional: left_fractional,
                        right_battery_fractional: right_fractional,
                        case_battery_fractional: case_fractional,
                        device_subtype: Some("earbud".to_string()),
                        left_in_ear: airpods.left_in_ear,
                        right_in_ear: airpods.right_in_ear,
                        case_lid_open: airpods.case_lid_open,
                        side: airpods.side.map(|s| s.to_string()),
                        both_in_case: airpods.both_in_case,
                        color: airpods.color.map(|c| c.to_string()),
                        switch_count: airpods.switch_count.map(|s| s as u8),
                        is_connected: true,
                        last_seen: std::time::SystemTime::now(),
                        rssi: airpods.rssi.map(|r| r as i16),
                        manufacturer_data: airpods
                            .raw_manufacturer_data
                            .clone()
                            .map(|s| s.into_bytes())
                            .unwrap_or_default(),
                    }
                }));

            // Update the main window with the new merged devices
            self.main_window.merged_devices = self.merged_devices.clone();
            crate::debug_log!(
                "ui",
                "Updated main_window.merged_devices count: {}",
                self.main_window.merged_devices.len()
            );

            // Update settings window with connected device names
            let connected_device_names: Vec<String> = self
                .merged_devices
                .iter()
                .filter(|device| device.is_connected || device.connected)
                .map(|device| device.name.clone())
                .collect();
            self.settings_window
                .update_connected_devices(connected_device_names);

            // Clear status message when devices are found - only keep it for warnings/errors
            self.status_message = None;
        } else {
            // If no AirPods data, keep existing merged devices but update status
            crate::debug_log!(
                "ui",
                "No AirPods data available, preserving existing {} merged devices",
                self.merged_devices.len()
            );

            // Only set "no devices" message if we don't have any existing devices
            if self.merged_devices.is_empty() {
                self.status_message = Some("No AirPods devices found".to_string());
            } else {
                // Keep existing devices and status, this might be a temporary scan failure
                crate::debug_log!(
                    "ui",
                    "Preserving existing devices during temporary scan failure"
                );
            }
        }

        let estimation_note =
            if self.config.battery.enable_estimation && !self.merged_devices.is_empty() {
                " (with smart estimation)"
            } else {
                ""
            };

        crate::debug_log!(
            "ui",
            "Total merged devices after async update: {}{}",
            self.merged_devices.len(),
            estimation_note
        );
    }

    /// Generate a stable device identifier that handles MAC address randomization
    /// This uses device model and user preferences to create consistent identifiers
    /// across MAC address changes due to privacy randomization
    fn generate_stable_device_id(&self, airpods: &AirPodsBatteryInfo) -> String {
        // Priority 1: If user has set a custom device name, use that as the stable identifier
        if let Some(custom_name) = &self.config.bluetooth.paired_device_name {
            if !custom_name.trim().is_empty()
                && !custom_name.starts_with("AirPods")
                && !custom_name.starts_with("Beats")
                && custom_name != "Unknown Device"
            {
                // Use sanitized custom name as primary identifier
                let sanitized_name = custom_name
                    .chars()
                    .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
                    .collect::<String>()
                    .replace(' ', "_")
                    .to_lowercase();
                return format!("custom_{}", sanitized_name);
            }
        }

        // Priority 2: For default device names, use a model-based identifier
        // This assumes the user typically has one AirPods device of each model
        // If they have multiple of the same model, they should use custom names to distinguish them
        let model_id = airpods.name.replace(" ", "_").to_lowercase();
        format!("model_{}", model_id)
    }
}

#[derive(Debug, Clone)]
pub struct MergedBluetoothDevice {
    pub name: String,
    pub address: String,
    pub paired: bool,
    pub connected: bool,
    pub device_type: DeviceType,
    pub battery: Option<u8>,
    pub left_battery: Option<u8>,
    pub right_battery: Option<u8>,
    pub case_battery: Option<u8>,
    /// Battery levels rounded to whole percentages (no fractional display)
    pub left_battery_fractional: Option<f32>,
    pub right_battery_fractional: Option<f32>,
    pub case_battery_fractional: Option<f32>,
    pub device_subtype: Option<String>, // "earbud" or "case"
    pub left_in_ear: Option<bool>,
    pub right_in_ear: Option<bool>,
    pub case_lid_open: Option<bool>,
    pub side: Option<String>,
    pub both_in_case: Option<bool>,
    pub color: Option<String>,
    pub switch_count: Option<u8>,
    pub is_connected: bool,
    pub last_seen: std::time::SystemTime,
    pub rssi: Option<i16>,
    pub manufacturer_data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceType {
    AirPods,
    Other,
}

impl Default for MergedBluetoothDevice {
    fn default() -> Self {
        Self {
            name: String::new(),
            address: String::new(),
            paired: false,
            connected: false,
            device_type: DeviceType::Other,
            battery: None,
            left_battery: None,
            right_battery: None,
            case_battery: None,
            left_battery_fractional: None,
            right_battery_fractional: None,
            case_battery_fractional: None,
            device_subtype: None,
            left_in_ear: None,
            right_in_ear: None,
            case_lid_open: None,
            side: None,
            both_in_case: None,
            color: None,
            switch_count: None,
            is_connected: false,
            last_seen: std::time::SystemTime::UNIX_EPOCH,
            rssi: None,
            manufacturer_data: Vec::new(),
        }
    }
}

/// Async function to scan for AirPods without blocking the UI
#[allow(dead_code)]
async fn async_scan_for_airpods() -> Vec<AirPodsBatteryInfo> {
    use tokio::task;

    // Run the CLI scanner in a blocking task to avoid blocking the async runtime
    task::spawn_blocking(get_airpods_from_cli_scanner)
        .await
        .unwrap_or_else(|_| {
            log::error!("Failed to execute CLI scanner task");
            Vec::new()
        })
}

/// Get AirPods data from the CLI scanner
#[allow(dead_code)]
fn get_airpods_from_cli_scanner() -> Vec<AirPodsBatteryInfo> {
    use std::process::Command as ProcessCommand;

    // Get the executable path and its directory
    let exe_path =
        std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("./rustpods.exe"));
    let exe_dir = exe_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

    crate::debug_log!("bluetooth", "CLI Scanner Path Resolution Debug");
    crate::debug_log!("bluetooth", "Executable path: {}", exe_path.display());
    crate::debug_log!("bluetooth", "Executable directory: {}", exe_dir.display());
    crate::debug_log!(
        "bluetooth",
        "Current working directory: {}",
        current_dir.display()
    );

    // Try multiple possible locations for the CLI scanner
    let cli_paths = vec![
        // 1. Same directory as the executable (most likely when running from target/release)
        exe_dir.join("airpods_battery_cli.exe"),
        // 2. bin folder relative to current working directory
        current_dir.join("bin").join("airpods_battery_cli.exe"),
        // 3. bin folder relative to executable directory (if exe is in subdir)
        exe_dir.join("bin").join("airpods_battery_cli.exe"),
        // 4. Project root if we're in target/release (go up 2 levels)
        exe_dir
            .parent()
            .and_then(|p| p.parent())
            .map(|project_root| project_root.join("bin").join("airpods_battery_cli.exe"))
            .unwrap_or_default(),
        // 5. Development location relative to current working directory
        current_dir
            .join("scripts")
            .join("airpods_battery_cli")
            .join("build")
            .join("Release")
            .join("airpods_battery_cli.exe"),
    ];

    crate::debug_log!(
        "bluetooth",
        "Trying {} possible CLI scanner locations",
        cli_paths.len()
    );
    for (i, path) in cli_paths.iter().enumerate() {
        crate::debug_log!("bluetooth", "Path {}: {}", i + 1, path.display());
        crate::debug_log!("bluetooth", "Path {} exists: {}", i + 1, path.exists());
    }

    // Find the first existing CLI scanner
    let cli_path = cli_paths.into_iter().find(|path| path.exists());

    let cli_path = match cli_path {
        Some(path) => {
            crate::debug_log!("bluetooth", "Found CLI scanner at: {}", path.display());
            path
        }
        None => {
            log::error!("No CLI scanner found in any of the expected locations!");
            return Vec::new();
        }
    };

    // Execute CLI scanner
    let mut command = ProcessCommand::new(&cli_path);
    command.arg("--fast");

    // Hide console window on Windows in release builds
    #[cfg(all(windows, not(debug_assertions)))]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    match command.output() {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                crate::debug_log!(
                    "bluetooth",
                    "CLI scanner output length: {} chars",
                    stdout.len()
                );

                // Parse the JSON output
                if let Ok(cli_result) =
                    serde_json::from_str::<crate::bluetooth::cli_scanner::CliScannerResult>(&stdout)
                {
                    let mut airpods_devices = Vec::new();

                    for device in &cli_result.devices {
                        if let Some(airpods_data) = &device.airpods_data {
                            // Create canonical address (lowercased, colon-free MAC address)
                            let canonical_address = device.address.replace(":", "").to_lowercase();

                            // Use canonical address as the primary identifier (no decimal conversion)
                            let address = u64::from_str_radix(&canonical_address, 16).unwrap_or(0);

                            let airpods_info = crate::airpods::battery::AirPodsBatteryInfo {
                                address,
                                canonical_address,
                                name: airpods_data.model.clone(),
                                model_id: 0, // Not provided by CLI scanner
                                left_battery: airpods_data.left_battery,
                                right_battery: airpods_data.right_battery,
                                case_battery: airpods_data.case_battery,
                                left_charging: airpods_data.left_charging,
                                right_charging: airpods_data.right_charging,
                                case_charging: airpods_data.case_charging,
                                left_in_ear: None,   // Not provided by CLI scanner
                                right_in_ear: None,  // Not provided by CLI scanner
                                case_lid_open: None, // Not provided by CLI scanner
                                side: None,          // Not provided by CLI scanner
                                both_in_case: None,  // Not provided by CLI scanner
                                color: None,         // Not provided by CLI scanner
                                switch_count: None,  // Not provided by CLI scanner
                                rssi: None,          // Not provided by CLI scanner
                                timestamp: None,     // Not provided by CLI scanner
                                raw_manufacturer_data: None, // Not provided by CLI scanner
                            };

                            airpods_devices.push(airpods_info);
                        }
                    }

                    crate::debug_log!(
                        "bluetooth",
                        "Parsed {} AirPods devices from CLI scanner",
                        airpods_devices.len()
                    );
                    airpods_devices
                } else {
                    log::error!("Failed to parse CLI scanner JSON output");
                    log::error!(
                        "Raw output preview: {}",
                        stdout.chars().take(200).collect::<String>()
                    );
                    Vec::new()
                }
            } else {
                log::error!(
                    "CLI scanner failed with exit code: {:?}",
                    output.status.code()
                );
                log::error!(
                    "CLI scanner stderr: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                Vec::new()
            }
        }
        Err(e) => {
            log::error!("Failed to execute CLI scanner: {}", e);
            // Already logged with log::error! above
            Vec::new()
        }
    }
}

/// Continuous CLI scanner for periodic updates every 10 seconds
/// This function is called by the timer subscription to maintain fresh data
#[allow(dead_code)]
fn get_airpods_from_cli_scanner_continuous() -> Vec<AirPodsBatteryInfo> {
    use std::process::Command as ProcessCommand;

    // Get the executable path and its directory
    let exe_path =
        std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("./rustpods.exe"));
    let exe_dir = exe_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

    crate::debug_log!("bluetooth", "Continuous CLI Scanner (every 10s)");
    crate::debug_log!("bluetooth", "Executable path: {}", exe_path.display());
    crate::debug_log!("bluetooth", "Executable directory: {}", exe_dir.display());
    crate::debug_log!(
        "bluetooth",
        "Current working directory: {}",
        current_dir.display()
    );

    // Try multiple possible locations for the CLI scanner
    let cli_paths = vec![
        // 1. Same directory as the executable (most likely when running from target/release)
        exe_dir.join("airpods_battery_cli.exe"),
        // 2. bin folder relative to current working directory
        current_dir.join("bin").join("airpods_battery_cli.exe"),
        // 3. bin folder relative to executable directory (if exe is in subdir)
        exe_dir.join("bin").join("airpods_battery_cli.exe"),
        // 4. Project root if we're in target/release (go up 2 levels)
        exe_dir
            .parent()
            .and_then(|p| p.parent())
            .map(|project_root| project_root.join("bin").join("airpods_battery_cli.exe"))
            .unwrap_or_default(),
        // 5. Development location relative to current working directory
        current_dir
            .join("scripts")
            .join("airpods_battery_cli")
            .join("build")
            .join("Release")
            .join("airpods_battery_cli.exe"),
    ];

    crate::debug_log!(
        "bluetooth",
        "Continuous scan - Trying {} possible CLI scanner locations",
        cli_paths.len()
    );
    for (i, path) in cli_paths.iter().enumerate() {
        crate::debug_log!(
            "bluetooth",
            "Continuous scan - Path {}: {}",
            i + 1,
            path.display()
        );
        crate::debug_log!(
            "bluetooth",
            "Continuous scan - Path {} exists: {}",
            i + 1,
            path.exists()
        );
    }

    // Find the first existing CLI scanner
    let cli_path = cli_paths.into_iter().find(|path| path.exists());

    let cli_path = match cli_path {
        Some(path) => {
            crate::debug_log!(
                "bluetooth",
                "Continuous scan - Found CLI scanner at: {}",
                path.display()
            );
            path
        }
        None => {
            log::error!("Continuous scan - No CLI scanner found in any of the expected locations!");
            return Vec::new();
        }
    };

    // Execute CLI scanner with fast argument for 2-second scan
    let mut command = ProcessCommand::new(&cli_path);
    command.arg("--fast");

    // Hide console window on Windows in release builds
    #[cfg(all(windows, not(debug_assertions)))]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    match command.output() {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                crate::debug_log!(
                    "bluetooth",
                    "Continuous scan output length: {} chars",
                    stdout.len()
                );

                // Parse the JSON output
                if let Ok(cli_result) =
                    serde_json::from_str::<crate::bluetooth::cli_scanner::CliScannerResult>(&stdout)
                {
                    let mut airpods_devices = Vec::new();

                    for device in &cli_result.devices {
                        if let Some(airpods_data) = &device.airpods_data {
                            // Create canonical address (lowercased, colon-free MAC address)
                            let canonical_address = device.address.replace(":", "").to_lowercase();

                            // Use canonical address as the primary identifier (no decimal conversion)
                            let address = u64::from_str_radix(&canonical_address, 16).unwrap_or(0);

                            let airpods_info = crate::airpods::battery::AirPodsBatteryInfo {
                                address,
                                canonical_address,
                                name: airpods_data.model.clone(),
                                model_id: 0, // Not provided by CLI scanner
                                left_battery: airpods_data.left_battery,
                                right_battery: airpods_data.right_battery,
                                case_battery: airpods_data.case_battery,
                                left_charging: airpods_data.left_charging,
                                right_charging: airpods_data.right_charging,
                                case_charging: airpods_data.case_charging,
                                left_in_ear: None,   // Not provided by CLI scanner
                                right_in_ear: None,  // Not provided by CLI scanner
                                case_lid_open: None, // Not provided by CLI scanner
                                side: None,          // Not provided by CLI scanner
                                both_in_case: None,  // Not provided by CLI scanner
                                color: None,         // Not provided by CLI scanner
                                switch_count: None,  // Not provided by CLI scanner
                                rssi: None,          // Not provided by CLI scanner
                                timestamp: None,     // Not provided by CLI scanner
                                raw_manufacturer_data: None, // Not provided by CLI scanner
                            };

                            airpods_devices.push(airpods_info);
                        }
                    }

                    crate::debug_log!(
                        "bluetooth",
                        "Continuous scan found {} AirPods devices",
                        airpods_devices.len()
                    );
                    airpods_devices
                } else {
                    log::error!("Continuous scan - Failed to parse CLI scanner JSON output");
                    log::error!(
                        "Continuous scan - Raw output preview: {}",
                        stdout.chars().take(200).collect::<String>()
                    );
                    Vec::new()
                }
            } else {
                log::error!(
                    "Continuous scan - CLI scanner failed with exit code: {:?}",
                    output.status.code()
                );
                log::error!(
                    "Continuous scan - CLI scanner stderr: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                Vec::new()
            }
        }
        Err(e) => {
            log::error!("Continuous scan - Failed to execute CLI scanner: {}", e);
            // Already logged with log::error! above
            Vec::new()
        }
    }
}
