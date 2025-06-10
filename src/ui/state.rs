use iced::{executor, Application, Command, Subscription};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};

use crate::airpods::battery::AirPodsBatteryInfo;
use crate::airpods::battery_estimator::BatteryEstimator;
use crate::bluetooth::DiscoveredDevice;
use crate::config::{AppConfig, ConfigError, ConfigManager};
use crate::ui::{
    components::{BluetoothSetting, SystemSetting, UiSetting},
    system_tray::{SystemTray, SystemTrayError},
    MainWindow, Message, SettingsWindow,
};

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
                tokio::task::spawn_blocking(get_airpods_from_cli_scanner).await.unwrap_or_else(|_| Vec::new())
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
        crate::debug_log!(
            "ui",
            "AppState::update: instance at {:p}",
            self as *const _
        );
        match message {
            Message::ToggleVisibility => {
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
                if let Some(tray) = &mut self.system_tray {
                    if let Err(e) = tray.update_icon(true) {
                        log::warn!("Failed to update tray icon: {}", e);
                    }
                    if let Err(e) = tray.update_tooltip_with_battery(
                        status_clone.battery.left,
                        status_clone.battery.right,
                        status_clone.battery.case,
                    ) {
                        log::warn!("Failed to update tray tooltip: {}", e);
                    }
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
                crate::debug_log!(
                    "ui",
                    "AppState::update: setting toast_message = {:?}",
                    msg
                );
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
                        tokio::task::spawn_blocking(get_airpods_from_cli_scanner_continuous).await.unwrap_or_else(|_| Vec::new())
                    },
                    Message::AirPodsDataLoaded,
                )
            }
            Message::AirPodsDataLoaded(airpods_data) => {
                // Handle the result of the async AirPods data loading
                log::info!("AirPods data loaded: {} devices found", airpods_data.len());
                crate::debug_log!("airpods", "AirPods data loaded: {} devices found", airpods_data.len());
                for (i, device) in airpods_data.iter().enumerate() {
                    crate::debug_log!("airpods", "Device {}: {} - L:{}% R:{}% C:{}%", 
                        i, device.name, device.left_battery, device.right_battery, device.case_battery);
                }

                // Update the state with the loaded AirPods data
                self.airpods_devices = airpods_data;
                self.last_update = std::time::Instant::now();

                // Update the merged devices to include the new AirPods data
                self.update_merged_devices();

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
            _ => {
                crate::debug_log!("ui", "Unhandled message in state.rs: {:?}", std::any::type_name::<Message>());
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
                crate::debug_log!("ui", "Controller subscription: Checking for messages from system tray");
                // Access the global receiver safely
                if let Some(receiver_arc) = CONTROLLER_RECEIVER.get() {
                    let mut guard = receiver_arc.lock().await;
                    if let Some(ref mut receiver) = *guard {
                        match receiver.recv().await {
                            Some(message) => {
                                crate::debug_log!("ui", "Controller subscription: Received message: {:?}", message);
                                log::info!("Controller subscription received message from system tray: {:?}", message);
                                (message, ())
                            }
                            None => {
                                crate::debug_log!("ui", "Controller subscription: Channel closed, no more messages");
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

    /// Initialize the system tray component
    pub fn initialize_system_tray(
        &mut self,
        _tx: std::sync::mpsc::Sender<Message>,
    ) -> Result<(), SystemTrayError> {
        // Create the system tray
        let tray = SystemTray::new(self.config.clone())?;
        self.system_tray = Some(tray);
        Ok(())
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

        // Update system tray configuration
        if let Some(system_tray) = &mut self.system_tray {
            if let Err(e) = system_tray.update_config(self.config.clone()) {
                log::error!("Failed to update system tray config: {}", e);
            } else {
                log::debug!("System tray configuration updated");
            }

            // Special handling for startup registration on Windows
            #[cfg(target_os = "windows")]
            {
                // Ensure startup registration matches config
                let should_register = self.config.system.launch_at_startup;
                let is_registered = system_tray.is_startup_registered();

                if should_register != is_registered {
                    if let Err(e) = system_tray.set_startup_enabled(should_register) {
                        log::error!("Failed to update startup registration: {}", e);
                    } else {
                        log::info!("Updated startup registration to: {}", should_register);
                    }
                }
            }
        }

        log::info!("Settings applied");
    }

    /// Update a Bluetooth setting
    fn update_bluetooth_setting(&mut self, setting: BluetoothSetting) {
        match setting {
            BluetoothSetting::AutoScanOnStartup(value) => {
                self.config.bluetooth.auto_scan_on_startup = value;
            }
            BluetoothSetting::ScanDuration(value) => {
                self.config.bluetooth.scan_duration = std::time::Duration::from_secs(value as u64);
            }
            BluetoothSetting::ScanInterval(value) => {
                self.config.bluetooth.scan_interval = std::time::Duration::from_secs(value as u64);
            }
            BluetoothSetting::MinRssi(value) => {
                self.config.bluetooth.min_rssi = Some(value.try_into().unwrap_or(-70));
            }
            BluetoothSetting::BatteryRefreshInterval(value) => {
                self.config.bluetooth.battery_refresh_interval = Duration::from_secs(value as u64);
            }
            BluetoothSetting::AutoReconnect(value) => {
                self.config.bluetooth.auto_reconnect = value;
            }
            BluetoothSetting::ReconnectAttempts(value) => {
                self.config.bluetooth.reconnect_attempts = value.try_into().unwrap_or(3);
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
        match setting {
            SystemSetting::StartOnBoot(value) => {
                self.config.system.launch_at_startup = value;
            }
            SystemSetting::StartMinimized(value) => {
                self.config.ui.start_minimized = value;
            }
            SystemSetting::LogLevel(value) => {
                self.config.system.log_level = value;
            }
            SystemSetting::EnableTelemetry(value) => {
                self.config.system.enable_telemetry = value;
            }
        }
    }

    pub fn clear_status_message(&mut self) {
        crate::debug_log!("ui", "AppState::clear_status_message: setting status_message = None");
        self.status_message = None;
    }

    pub fn clear_toast_message(&mut self) {
        crate::debug_log!("ui", "AppState::clear_toast_message: setting toast_message = None");
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
                        address: airpods.address.to_string(),
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
    fn update_merged_devices(&mut self) {
        crate::debug_log!("ui", "Updating merged devices with {} AirPods devices", self.airpods_devices.len());
        
        // Only update merged devices if we have actual AirPods data
        // This prevents clearing devices when the CLI scanner temporarily returns empty results
        if !self.airpods_devices.is_empty() {
            // Clear existing merged devices only when we have new data to replace them
            self.merged_devices.clear();

            // Update battery estimator with new real data if estimation is enabled
            if self.config.battery.enable_estimation {
                for airpods in &self.airpods_devices {
                    self.battery_estimator.update_real_data(
                        Some(airpods.left_battery),
                        Some(airpods.right_battery),
                        Some(airpods.case_battery),
                    );
                }

                // Save updated battery estimator data to config
                let (_left_est, _right_est, _case_est) = self.battery_estimator.get_estimated_levels();
                self.config.battery.left_history = self.battery_estimator.left_history.clone();
                self.config.battery.right_history = self.battery_estimator.right_history.clone();
                self.config.battery.case_history = self.battery_estimator.case_history.clone();
            }

            // Get estimated battery levels
            let (left_estimate, right_estimate, case_estimate) = if self.config.battery.enable_estimation {
                self.battery_estimator.get_display_levels()
            } else {
                (None, None, None)
            };

            // Add AirPods devices to the merged devices
            self.merged_devices
                .extend(self.airpods_devices.iter().map(|airpods| {
                    crate::debug_log!("airpods", "Converting AirPods device: {} - L:{}% R:{}% C:{}%", 
                        airpods.name, airpods.left_battery, airpods.right_battery, airpods.case_battery);
                    
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

                    crate::debug_log!("airpods", "Final merged device - L:{}% R:{}% C:{}%", 
                        left_battery, right_battery, case_battery);

                    MergedBluetoothDevice {
                        name: airpods.name.clone(),
                        address: airpods.address.to_string(),
                        paired: true,
                        connected: true,
                        device_type: DeviceType::AirPods,
                        battery: Some(left_battery).or(Some(right_battery)),
                        left_battery: Some(left_battery),
                        right_battery: Some(right_battery),
                        case_battery: Some(case_battery),
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
            crate::debug_log!("ui", "Updated main_window.merged_devices count: {}", self.main_window.merged_devices.len());

            // Clear status message when devices are found - only keep it for warnings/errors
            self.status_message = None;
        } else {
            // If no AirPods data, keep existing merged devices but update status
            crate::debug_log!("ui", "No AirPods data available, preserving existing {} merged devices", self.merged_devices.len());
            
            // Only set "no devices" message if we don't have any existing devices
            if self.merged_devices.is_empty() {
                self.status_message = Some("No AirPods devices found".to_string());
            } else {
                // Keep existing devices and status, this might be a temporary scan failure
                crate::debug_log!("ui", "Preserving existing devices during temporary scan failure");
            }
        }

        let estimation_note = if self.config.battery.enable_estimation && !self.merged_devices.is_empty() {
            " (with smart estimation)"
        } else {
            ""
        };

        crate::debug_log!("ui", "Total merged devices after async update: {}{}", 
            self.merged_devices.len(), estimation_note);
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
    let exe_path = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("./rustpods.exe"));
    let exe_dir = exe_path.parent().unwrap_or_else(|| std::path::Path::new("."));
    let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    
    crate::debug_log!("bluetooth", "CLI Scanner Path Resolution Debug");
    crate::debug_log!("bluetooth", "Executable path: {}", exe_path.display());
    crate::debug_log!("bluetooth", "Executable directory: {}", exe_dir.display());
    crate::debug_log!("bluetooth", "Current working directory: {}", current_dir.display());
    
    // Try multiple possible locations for the CLI scanner
    let cli_paths = vec![
        // 1. Same directory as the executable (most likely when running from target/release)
        exe_dir.join("airpods_battery_cli.exe"),
        // 2. bin folder relative to current working directory 
        current_dir.join("bin").join("airpods_battery_cli.exe"),
        // 3. bin folder relative to executable directory (if exe is in subdir)
        exe_dir.join("bin").join("airpods_battery_cli.exe"),
        // 4. Project root if we're in target/release (go up 2 levels)
        exe_dir.parent().and_then(|p| p.parent()).map(|project_root| project_root.join("bin").join("airpods_battery_cli.exe")).unwrap_or_default(),
        // 5. Development location relative to current working directory
        current_dir.join("scripts").join("airpods_battery_cli").join("build").join("Release").join("airpods_battery_cli.exe"),
    ];

    crate::debug_log!("bluetooth", "Trying {} possible CLI scanner locations", cli_paths.len());
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
    match ProcessCommand::new(&cli_path)
        .arg("--fast")
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                crate::debug_log!("bluetooth", "CLI scanner output length: {} chars", stdout.len());

                // Parse the JSON output
                if let Ok(cli_result) = serde_json::from_str::<crate::bluetooth::cli_scanner::CliScannerResult>(&stdout) {
                    let mut airpods_devices = Vec::new();

                    for device in &cli_result.devices {
                        if let Some(airpods_data) = &device.airpods_data {
                            // Parse address from MAC address string (e.g. "56:4F:9A:2E:2B:96") to u64
                            let address = device.address
                                .replace(":", "")
                                .chars()
                                .collect::<String>()
                                .parse::<u64>()
                                .or_else(|_| {
                                    // If direct parsing fails, try parsing as hex
                                    u64::from_str_radix(&device.address.replace(":", ""), 16)
                                })
                                .unwrap_or(0);

                            let airpods_info = crate::airpods::battery::AirPodsBatteryInfo {
                                address,
                                name: airpods_data.model.clone(),
                                model_id: 0, // Not provided by CLI scanner
                                left_battery: airpods_data.left_battery,
                                right_battery: airpods_data.right_battery,
                                case_battery: airpods_data.case_battery,
                                left_charging: airpods_data.left_charging,
                                right_charging: airpods_data.right_charging,
                                case_charging: airpods_data.case_charging,
                                left_in_ear: None, // Not provided by CLI scanner
                                right_in_ear: None, // Not provided by CLI scanner  
                                case_lid_open: None, // Not provided by CLI scanner
                                side: None, // Not provided by CLI scanner
                                both_in_case: None, // Not provided by CLI scanner
                                color: None, // Not provided by CLI scanner
                                switch_count: None, // Not provided by CLI scanner
                                rssi: None, // Not provided by CLI scanner
                                timestamp: None, // Not provided by CLI scanner
                                raw_manufacturer_data: None, // Not provided by CLI scanner
                            };

                            airpods_devices.push(airpods_info);
                        }
                    }

                    crate::debug_log!("bluetooth", "Parsed {} AirPods devices from CLI scanner", airpods_devices.len());
                    airpods_devices
                } else {
                    log::error!("Failed to parse CLI scanner JSON output");
                    log::error!("Raw output preview: {}", stdout.chars().take(200).collect::<String>());
                    Vec::new()
                }
            } else {
                log::error!("CLI scanner failed with exit code: {:?}", output.status.code());
                log::error!("CLI scanner stderr: {}", String::from_utf8_lossy(&output.stderr));
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
    let exe_path = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("./rustpods.exe"));
    let exe_dir = exe_path.parent().unwrap_or_else(|| std::path::Path::new("."));
    let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    
    crate::debug_log!("bluetooth", "Continuous CLI Scanner (every 10s)");
    crate::debug_log!("bluetooth", "Executable path: {}", exe_path.display());
    crate::debug_log!("bluetooth", "Executable directory: {}", exe_dir.display());
    crate::debug_log!("bluetooth", "Current working directory: {}", current_dir.display());
    
    // Try multiple possible locations for the CLI scanner
    let cli_paths = vec![
        // 1. Same directory as the executable (most likely when running from target/release)
        exe_dir.join("airpods_battery_cli.exe"),
        // 2. bin folder relative to current working directory 
        current_dir.join("bin").join("airpods_battery_cli.exe"),
        // 3. bin folder relative to executable directory (if exe is in subdir)
        exe_dir.join("bin").join("airpods_battery_cli.exe"),
        // 4. Project root if we're in target/release (go up 2 levels)
        exe_dir.parent().and_then(|p| p.parent()).map(|project_root| project_root.join("bin").join("airpods_battery_cli.exe")).unwrap_or_default(),
        // 5. Development location relative to current working directory
        current_dir.join("scripts").join("airpods_battery_cli").join("build").join("Release").join("airpods_battery_cli.exe"),
    ];

    crate::debug_log!("bluetooth", "Continuous scan - Trying {} possible CLI scanner locations", cli_paths.len());
    for (i, path) in cli_paths.iter().enumerate() {
        crate::debug_log!("bluetooth", "Continuous scan - Path {}: {}", i + 1, path.display());
        crate::debug_log!("bluetooth", "Continuous scan - Path {} exists: {}", i + 1, path.exists());
    }

    // Find the first existing CLI scanner
    let cli_path = cli_paths.into_iter().find(|path| path.exists());
    
    let cli_path = match cli_path {
        Some(path) => {
            crate::debug_log!("bluetooth", "Continuous scan - Found CLI scanner at: {}", path.display());
            path
        }
        None => {
            log::error!("Continuous scan - No CLI scanner found in any of the expected locations!");
            return Vec::new();
        }
    };

    // Execute CLI scanner with fast argument for 2-second scan
    match ProcessCommand::new(&cli_path)
        .arg("--fast")
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                crate::debug_log!("bluetooth", "Continuous scan output length: {} chars", stdout.len());

                // Parse the JSON output
                if let Ok(cli_result) = serde_json::from_str::<crate::bluetooth::cli_scanner::CliScannerResult>(&stdout) {
                    let mut airpods_devices = Vec::new();

                    for device in &cli_result.devices {
                        if let Some(airpods_data) = &device.airpods_data {
                            // Parse address from MAC address string (e.g. "56:4F:9A:2E:2B:96") to u64
                            let address = device.address
                                .replace(":", "")
                                .chars()
                                .collect::<String>()
                                .parse::<u64>()
                                .or_else(|_| {
                                    // If direct parsing fails, try parsing as hex
                                    u64::from_str_radix(&device.address.replace(":", ""), 16)
                                })
                                .unwrap_or(0);

                            let airpods_info = crate::airpods::battery::AirPodsBatteryInfo {
                                address,
                                name: airpods_data.model.clone(),
                                model_id: 0, // Not provided by CLI scanner
                                left_battery: airpods_data.left_battery,
                                right_battery: airpods_data.right_battery,
                                case_battery: airpods_data.case_battery,
                                left_charging: airpods_data.left_charging,
                                right_charging: airpods_data.right_charging,
                                case_charging: airpods_data.case_charging,
                                left_in_ear: None, // Not provided by CLI scanner
                                right_in_ear: None, // Not provided by CLI scanner  
                                case_lid_open: None, // Not provided by CLI scanner
                                side: None, // Not provided by CLI scanner
                                both_in_case: None, // Not provided by CLI scanner
                                color: None, // Not provided by CLI scanner
                                switch_count: None, // Not provided by CLI scanner
                                rssi: None, // Not provided by CLI scanner
                                timestamp: None, // Not provided by CLI scanner
                                raw_manufacturer_data: None, // Not provided by CLI scanner
                            };

                            airpods_devices.push(airpods_info);
                        }
                    }

                    crate::debug_log!("bluetooth", "Continuous scan found {} AirPods devices", airpods_devices.len());
                    airpods_devices
                } else {
                    log::error!("Continuous scan - Failed to parse CLI scanner JSON output");
                    log::error!("Continuous scan - Raw output preview: {}", stdout.chars().take(200).collect::<String>());
                    Vec::new()
                }
            } else {
                log::error!("Continuous scan - CLI scanner failed with exit code: {:?}", output.status.code());
                log::error!("Continuous scan - CLI scanner stderr: {}", String::from_utf8_lossy(&output.stderr));
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
