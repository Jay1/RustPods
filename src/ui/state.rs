use iced::{executor, Application, Command, Subscription};
use std::collections::HashMap;
use std::process::Command as ProcessCommand;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};

use crate::airpods::battery::AirPodsBatteryInfo;
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

        // Return a command that triggers initial continuous scanning for immediate AirPods detection
        // This scans until AirPods are found, providing instant responsiveness
        log::info!("Scheduling initial continuous scanning for immediate AirPods detection");
        let initial_command = Self::refresh_device_data_command(); // Use the same unified scanning approach

        (app_state, initial_command)
    }

    fn title(&self) -> String {
        String::from("RustPods - AirPods Battery Monitor")
    }

    fn theme(&self) -> Self::Theme {
        crate::ui::theme::Theme::CatppuccinMocha
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        println!(
            "[DEBUG] AppState::update: instance at {:p}",
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
                println!("Connected to AirPods: {:?}", airpods);
                if let Some(address) = self.selected_device.as_ref() {
                    if let Some(device) = self.devices.get_mut(address) {
                        device.is_potential_airpods = true;
                    }
                }
                Command::none()
            }
            Message::BluetoothError(msg) => {
                println!(
                    "[DEBUG] AppState::update: setting toast_message = {:?}",
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
                println!(
                    "[DEBUG] MergedScanResult received with {} devices",
                    devices.len()
                );

                // Clear existing devices and update with fresh data from async CLI scanner
                self.merged_devices.clear();
                self.merged_devices = devices.clone();

                // Update the main window with the new devices
                self.main_window.merged_devices = devices.clone();

                // Set status message based on device count with timing info
                if devices.is_empty() {
                    self.status_message = Some("No AirPods devices found".to_string());
                } else {
                    self.status_message = Some(format!(
                        "Updated at {} - {} device(s) found",
                        chrono::Local::now().format("%H:%M:%S"),
                        devices.len()
                    ));
                }

                println!(
                    "[DEBUG] Total merged devices after async update: {}",
                    self.merged_devices.len()
                );
                Command::none()
            }
            Message::Tick => {
                println!("[DEBUG] Tick message received - refreshing device data");
                // Refresh device data from CLI scanner on each tick using fast command approach
                Self::refresh_device_data_command()
            }
            Message::AirPodsDataLoaded(airpods_data) => {
                // Handle the result of the async AirPods data loading
                log::info!("AirPods data loaded: {} devices found", airpods_data.len());

                // Update the state with the loaded AirPods data
                self.airpods_devices = airpods_data;
                self.last_update = std::time::Instant::now();

                // Update the merged devices to include the new AirPods data
                self.update_merged_devices();

                Command::none()
            }
            _ => {
                log::debug!("Unhandled message in state.rs");
                Command::none()
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Message, iced::Renderer<crate::ui::theme::Theme>> {
        use iced::widget::{container, text};
        use iced::Length;
        println!(
            "[DEBUG] AppState::view: status_message = {:?}, toast_message = {:?}",
            self.status_message, self.toast_message
        );
        let status_message = self.status_message.as_ref();
        let toast_message = self.toast_message.as_ref();
        if !self.visible {
            iced::widget::text("").into()
        } else if self.show_settings {
            let mut column = iced::widget::Column::new();
            column = column.push(crate::ui::UiComponent::view(&self.settings_window));
            // Persistent status bar
            if let Some(status) = status_message {
                let status_bar = container(text(status).size(20).style(crate::ui::theme::MAUVE))
                    .width(Length::Fill)
                    .padding(8)
                    .style(iced::theme::Container::Box);
                column = column.push(status_bar);
            }
            // Toast bar (bottom)
            if let Some(toast) = toast_message {
                let toast_bar = container(text(toast).size(20).style(crate::ui::theme::ROSEWATER))
                    .width(Length::Fill)
                    .padding(8)
                    .style(iced::theme::Container::Box);
                column = column.push(toast_bar);
            }
            container(column)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            let mut column = iced::widget::Column::new();
            column = column.push(crate::ui::UiComponent::view(&self.main_window));
            // Persistent status bar
            if let Some(status) = status_message {
                let status_bar = container(text(status).size(20).style(crate::ui::theme::MAUVE))
                    .width(Length::Fill)
                    .padding(8)
                    .style(iced::theme::Container::Box);
                column = column.push(status_bar);
            }
            // Toast bar (bottom)
            if let Some(toast) = toast_message {
                let toast_bar = container(text(toast).size(20).style(crate::ui::theme::ROSEWATER))
                    .width(Length::Fill)
                    .padding(8)
                    .style(iced::theme::Container::Box);
                column = column.push(toast_bar);
            }
            container(column)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        use iced::time;
        use std::time::Duration;

        // Timer for periodic CLI scanner updates (every 30 seconds)
        let timer = time::every(Duration::from_secs(30)).map(|_| Message::Tick);

        // Controller subscription for system tray communication
        let controller_subscription = iced::subscription::unfold(
            "controller-messages",
            (),
            |_state| async move {
                println!("[DEBUG] Controller subscription: Checking for messages from system tray");
                // Access the global receiver safely
                if let Some(receiver_arc) = CONTROLLER_RECEIVER.get() {
                    let mut guard = receiver_arc.lock().await;
                    if let Some(ref mut receiver) = *guard {
                        match receiver.recv().await {
                            Some(message) => {
                                println!(
                                    "[DEBUG] Controller subscription: Received message: {:?}",
                                    message
                                );
                                log::info!("Controller subscription received message from system tray: {:?}", message);
                                (message, ())
                            }
                            None => {
                                println!("[DEBUG] Controller subscription: Channel closed, no more messages");
                                log::warn!(
                                    "Controller channel closed - system tray communication lost"
                                );
                                // Channel closed, wait and try again
                                tokio::time::sleep(Duration::from_millis(100)).await;
                                (Message::NoOp, ())
                            }
                        }
                    } else {
                        println!("[DEBUG] Controller subscription: No receiver available");
                        log::warn!("No controller receiver available");
                        // No receiver, wait and try again
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        (Message::NoOp, ())
                    }
                } else {
                    println!("[DEBUG] Controller subscription: CONTROLLER_RECEIVER not set");
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
        println!("[DEBUG] AppState::clear_status_message: setting status_message = None");
        self.status_message = None;
    }

    pub fn clear_toast_message(&mut self) {
        println!("[DEBUG] AppState::clear_toast_message: setting toast_message = None");
        self.toast_message = None;
    }

    /// Refresh device data from the CLI scanner (fast synchronous call in async command)
    #[allow(dead_code)]
    fn refresh_device_data(&mut self) {
        println!(
            "[DEBUG] refresh_device_data called at {:?}",
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
                let airpods_data = get_airpods_from_cli_scanner_continuous();
                println!(
                    "[DEBUG] CLI scanner returned {} AirPods devices",
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
        // Clear existing merged devices
        self.merged_devices.clear();

        // Add AirPods devices to the merged devices
        self.merged_devices
            .extend(self.airpods_devices.iter().map(|airpods| {
                MergedBluetoothDevice {
                    name: airpods.name.clone(),
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
                }
            }));

        // Update the main window with the new merged devices
        self.main_window.merged_devices = self.merged_devices.clone();

        // Set status message based on device count with timing info
        if self.merged_devices.is_empty() {
            self.status_message = Some("No AirPods devices found".to_string());
        } else {
            self.status_message = Some(format!(
                "Updated at {} - {} device(s) found",
                chrono::Local::now().format("%H:%M:%S"),
                self.merged_devices.len()
            ));
        }

        println!(
            "[DEBUG] Total merged devices after async update: {}",
            self.merged_devices.len()
        );
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
    // Path to the v6 modular CLI scanner executable
    let cli_path = "scripts/airpods_battery_cli/build/Release/airpods_battery_cli.exe";

    println!("[DEBUG] Calling CLI scanner at: {}", cli_path);

    match ProcessCommand::new(cli_path)
        .arg("--fast") // Use ultra-fast 2-second scan with early exit
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                println!("[DEBUG] CLI scanner output length: {} chars", stdout.len());

                // Parse the JSON output from the CLI scanner
                match serde_json::from_str::<crate::bluetooth::cli_scanner::CliScannerResult>(
                    &stdout,
                ) {
                    Ok(scan_result) => {
                        // Extract AirPods devices from the scan result and convert them
                        let airpods_devices: Vec<AirPodsBatteryInfo> = scan_result
                            .devices
                            .into_iter()
                            .filter_map(|device| {
                                if let Some(airpods_data) = device.airpods_data {
                                    // Parse model_id from hex string (e.g. "0x2014" -> 0x2014)
                                    let model_id = if let Some(stripped) =
                                        airpods_data.model_id.strip_prefix("0x")
                                    {
                                        u16::from_str_radix(stripped, 16).unwrap_or(0)
                                    } else {
                                        airpods_data.model_id.parse::<u16>().unwrap_or(0)
                                    };

                                    // Parse address from string to u64
                                    let address = device.address.parse::<u64>().unwrap_or(0);

                                    Some(AirPodsBatteryInfo {
                                        address,
                                        name: airpods_data.model,
                                        model_id,
                                        left_battery: airpods_data.left_battery,
                                        left_charging: airpods_data.left_charging,
                                        right_battery: airpods_data.right_battery,
                                        right_charging: airpods_data.right_charging,
                                        case_battery: airpods_data.case_battery,
                                        case_charging: airpods_data.case_charging,
                                        left_in_ear: Some(airpods_data.left_in_ear),
                                        right_in_ear: Some(airpods_data.right_in_ear),
                                        case_lid_open: Some(airpods_data.lid_open),
                                        side: None, // Not provided in current CLI output format
                                        both_in_case: Some(airpods_data.both_in_case),
                                        color: None, // Not provided in current CLI output format
                                        switch_count: None, // Not provided in current CLI output format
                                        rssi: Some(device.rssi),
                                        timestamp: Some(
                                            scan_result.scan_timestamp.parse::<u64>().unwrap_or(0),
                                        ),
                                        raw_manufacturer_data: Some(device.manufacturer_data_hex),
                                    })
                                } else {
                                    None
                                }
                            })
                            .collect();

                        println!(
                            "[DEBUG] Parsed {} AirPods devices from CLI scanner",
                            airpods_devices.len()
                        );
                        airpods_devices
                    }
                    Err(e) => {
                        log::error!("Failed to parse CLI scanner JSON output: {}", e);
                        println!("[DEBUG] CLI scanner JSON parse error: {}", e);
                        println!(
                            "[DEBUG] Raw output preview: {}",
                            stdout.chars().take(200).collect::<String>()
                        );
                        Vec::new()
                    }
                }
            } else {
                log::error!(
                    "CLI scanner failed with exit code: {:?}",
                    output.status.code()
                );
                println!(
                    "[DEBUG] CLI scanner stderr: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                Vec::new()
            }
        }
        Err(e) => {
            log::error!("Failed to execute CLI scanner: {}", e);
            println!("[DEBUG] CLI scanner execution error: {}", e);
            Vec::new()
        }
    }
}

/// Get AirPods data from the CLI scanner using continuous scanning mode
fn get_airpods_from_cli_scanner_continuous() -> Vec<AirPodsBatteryInfo> {
    // Path to the v6 modular CLI scanner executable
    let cli_path = "scripts/airpods_battery_cli/build/Release/airpods_battery_cli.exe";

    println!("[DEBUG] Calling continuous CLI scanner at: {}", cli_path);

    match ProcessCommand::new(cli_path)
        .arg("--continuous") // Use continuous scanning mode until AirPods found
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                println!(
                    "[DEBUG] Continuous CLI scanner output length: {} chars",
                    stdout.len()
                );

                // Parse the JSON output from the CLI scanner
                match serde_json::from_str::<crate::bluetooth::cli_scanner::CliScannerResult>(
                    &stdout,
                ) {
                    Ok(scan_result) => {
                        // Extract AirPods devices from the scan result and convert them
                        let airpods_devices: Vec<AirPodsBatteryInfo> = scan_result
                            .devices
                            .into_iter()
                            .filter_map(|device| {
                                if let Some(airpods_data) = device.airpods_data {
                                    // Parse model_id from hex string (e.g. "0x2014" -> 0x2014)
                                    let model_id = if let Some(stripped) =
                                        airpods_data.model_id.strip_prefix("0x")
                                    {
                                        u16::from_str_radix(stripped, 16).unwrap_or(0)
                                    } else {
                                        airpods_data.model_id.parse::<u16>().unwrap_or(0)
                                    };

                                    // Parse address from string to u64
                                    let address = device.address.parse::<u64>().unwrap_or(0);

                                    Some(AirPodsBatteryInfo {
                                        address,
                                        name: airpods_data.model,
                                        model_id,
                                        left_battery: airpods_data.left_battery,
                                        left_charging: airpods_data.left_charging,
                                        right_battery: airpods_data.right_battery,
                                        right_charging: airpods_data.right_charging,
                                        case_battery: airpods_data.case_battery,
                                        case_charging: airpods_data.case_charging,
                                        left_in_ear: Some(airpods_data.left_in_ear),
                                        right_in_ear: Some(airpods_data.right_in_ear),
                                        case_lid_open: Some(airpods_data.lid_open),
                                        side: None, // Not provided in current CLI output format
                                        both_in_case: Some(airpods_data.both_in_case),
                                        color: None, // Not provided in current CLI output format
                                        switch_count: None, // Not provided in current CLI output format
                                        rssi: Some(device.rssi),
                                        timestamp: Some(
                                            scan_result.scan_timestamp.parse::<u64>().unwrap_or(0),
                                        ),
                                        raw_manufacturer_data: Some(device.manufacturer_data_hex),
                                    })
                                } else {
                                    None
                                }
                            })
                            .collect();

                        println!(
                            "[DEBUG] Parsed {} AirPods devices from continuous CLI scanner",
                            airpods_devices.len()
                        );
                        airpods_devices
                    }
                    Err(e) => {
                        log::error!("Failed to parse continuous CLI scanner JSON output: {}", e);
                        println!("[DEBUG] Continuous CLI scanner JSON parse error: {}", e);
                        println!(
                            "[DEBUG] Raw output preview: {}",
                            stdout.chars().take(200).collect::<String>()
                        );
                        Vec::new()
                    }
                }
            } else {
                log::error!(
                    "Continuous CLI scanner failed with exit code: {:?}",
                    output.status.code()
                );
                println!(
                    "[DEBUG] Continuous CLI scanner stderr: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                Vec::new()
            }
        }
        Err(e) => {
            log::error!("Failed to execute continuous CLI scanner: {}", e);
            println!("[DEBUG] Continuous CLI scanner execution error: {}", e);
            Vec::new()
        }
    }
}
