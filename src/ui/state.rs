use std::collections::HashMap;
#[cfg(test)]
use std::time::Instant;
use iced::{Subscription, Application, Command};
use std::convert::TryInto;
use iced::executor;
use tokio::sync::mpsc;
use std::sync::OnceLock;
use tokio::sync::Mutex;
use serde::Deserialize;
use std::time::Duration;

use crate::bluetooth::DiscoveredDevice;
use crate::bluetooth::cli_scanner::{CliScannerResult, CliAirPodsData};
use crate::airpods::battery::AirPodsBatteryInfo;
use crate::config::{AppConfig, ConfigManager, ConfigError};
use crate::ui::Message;
use crate::ui::components::{BluetoothSetting, UiSetting, SystemSetting};
use crate::ui::{MainWindow, SettingsWindow};
use crate::ui::SystemTray;
use crate::ui::system_tray::SystemTrayError;

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
}

impl AppState {
    /// Create a new AppState with the given controller sender
    pub fn new(controller_sender: mpsc::UnboundedSender<Message>) -> Self {
        // Load config or use default
        let config = AppConfig::load().unwrap_or_default();
        let config_manager = Some(ConfigManager::default());
        let main_window = MainWindow::empty();
        let settings_window = SettingsWindow::new(config.clone());
        // --- BEGIN: AirPods helper integration ---
        #[cfg(target_os = "windows")]
        {
            use crate::bluetooth::scanner::DiscoveredDevice;
            use btleplug::api::BDAddr;
            use std::collections::HashMap;
            use crate::airpods::battery::AirPodsBatteryInfo;
use crate::bluetooth::cli_scanner::{CliScannerResult, CliDeviceInfo};
use crate::bluetooth::cli_scanner::{CliScannerConfig, CliScanner};
            use crate::airpods::{AirPodsBattery, AirPodsChargingState};
            use crate::bluetooth::AirPodsBatteryStatus;
            // Use the new CLI scanner to get actual device data
            let infos: Vec<AirPodsBatteryInfo> = get_airpods_from_cli_scanner();
            println!("[DEBUG] AirPods helper output: {infos:?}");
            log::info!("Raw AirPodsBatteryInfo from helper: {infos:?}");
            let mut devices = HashMap::new();
            let mut merged_devices = Vec::new();
            let mut address_map: HashMap<u64, &AirPodsBatteryInfo> = HashMap::new();
            // Clamp battery values to 0-100, None if out of range
            let clamp = |v: i32| -> Option<u8> {
                if v > 0 && v <= 100 { Some(v as u8) } else if v > 100 { Some(100) } else { None }
            };
            for info in infos.iter() {
                if address_map.contains_key(&info.address) {
                    println!("[WARN] Duplicate AirPods address in helper output: {}", info.address);
                    continue;
                }
                if info.left_battery <= 0 && info.right_battery <= 0 && info.case_battery <= 0 {
                    println!("[WARN] Skipping AirPods entry with no battery info: {}", info.address);
                    continue;
                }
                address_map.insert(info.address, info);
            }
            // Deduplicate: keep only the entry with the highest sum of battery values for each name
            let mut deduped: HashMap<String, &AirPodsBatteryInfo> = HashMap::new();
            for info in address_map.values() {
                let name = if info.name.is_empty() { "AirPods".to_string() } else { info.name.clone() };
                let sum = info.left_battery.max(0) + info.right_battery.max(0) + info.case_battery.max(0);
                if let Some(existing) = deduped.get(&name) {
                    let existing_sum = existing.left_battery.max(0) + existing.right_battery.max(0) + existing.case_battery.max(0);
                    if sum > existing_sum {
                        println!("[INFO] Replacing AirPods entry for '{}' with higher battery sum ({} > {})", name, sum, existing_sum);
                        deduped.insert(name, info);
                    } else {
                        println!("[INFO] Filtering out ghost/duplicate AirPods entry for '{}' with lower battery sum ({} <= {})", name, sum, existing_sum);
                    }
                } else {
                    deduped.insert(name, info);
                }
            }
            let mut selected_device = None;
            let mut battery_status = None;
            for info in deduped.values() {
                let bytes = info.address.to_le_bytes();
                let address = BDAddr::from([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]]);
                let left_battery = clamp(info.left_battery);
                let right_battery = clamp(info.right_battery);
                let case_battery = clamp(info.case_battery);
                let left_in_ear = info.left_in_ear.unwrap_or(false);
                let right_in_ear = info.right_in_ear.unwrap_or(false);
                let case_lid_open = info.case_lid_open.unwrap_or(false);
                let side = info.side.map(|v| v.to_string());
                let both_in_case = info.both_in_case;
                let color = info.color.map(|v| v.to_string());
                let switch_count = info.switch_count.map(|v| v as u8);
                // Robustly determine charging state
                let charging_state = match (info.left_charging, info.right_charging, info.case_charging) {
                    (true, true, _) => Some(AirPodsChargingState::BothBudsCharging),
                    (true, false, _) => Some(AirPodsChargingState::LeftCharging),
                    (false, true, _) => Some(AirPodsChargingState::RightCharging),
                    (false, false, true) => Some(AirPodsChargingState::CaseCharging),
                    _ => Some(AirPodsChargingState::NotCharging),
                };
                // Convert to AirPodsBattery and AirPodsBatteryStatus
                let battery = AirPodsBattery {
                    left: left_battery,
                    right: right_battery,
                    case: case_battery,
                    charging: charging_state,
                };
                let status = AirPodsBatteryStatus::new(battery.clone());
                // Store the first device's battery status for UI display
                if battery_status.is_none() {
                    battery_status = Some(status.clone());
                }
                let device = DiscoveredDevice {
                    address,
                    name: if info.name.is_empty() { Some("AirPods".to_string()) } else { Some(info.name.clone()) },
                    rssi: None,
                    manufacturer_data: HashMap::new(),
                    is_potential_airpods: true,
                    last_seen: std::time::Instant::now(),
                    is_connected: true,
                    service_data: HashMap::new(),
                    services: Vec::new(),
                    tx_power_level: None,
                };
                if selected_device.is_none() {
                    selected_device = Some(address.to_string());
                }
                devices.insert(address.to_string(), device);
                merged_devices.push(crate::ui::state::MergedBluetoothDevice {
                    name: if info.name.is_empty() { "AirPods".to_string() } else { info.name.clone() },
                    address: address.to_string(),
                    paired: false,
                    connected: true,
                    device_type: None,
                    battery: [left_battery, right_battery, case_battery].iter().filter_map(|&b| b).max(),
                    left_battery,
                    right_battery,
                    case_battery,
                    device_subtype: None,
                    left_in_ear: Some(left_in_ear),
                    right_in_ear: Some(right_in_ear),
                    case_lid_open: Some(case_lid_open),
                    side,
                    both_in_case,
                    color,
                    switch_count,
                });
            }
            let mut main_window = MainWindow::empty();
            main_window.merged_devices = merged_devices;
            // --- Store the battery status for UI and state ---
            let this = Self {
                visible: true,
                devices,
                selected_device,
                connection_timestamp: None,
                animation_progress: 0.0,
                battery_status, // Set robustly from helper output
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
            };
            println!("[DEBUG] AppState::new: instance at {:p}", &this as *const _);
            this
        }
        // --- END: AirPods helper integration ---
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
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        panic!("AppState::default() should not be used. Use AppState::new(controller_sender) instead.");
    }
}

impl Application for AppState {
    type Message = Message;
    type Theme = crate::ui::theme::Theme;
    type Executor = executor::Default;
    type Flags = (mpsc::UnboundedSender<Message>, mpsc::UnboundedReceiver<Message>);

    fn new((controller_sender, controller_receiver): Self::Flags) -> (Self, Command<Message>) {
        // Store the receiver in the subscription state, not in AppState
        (AppState::new(controller_sender), Command::none())
    }
    
    fn title(&self) -> String {
        String::from("RustPods - AirPods Battery Monitor")
    }

    fn theme(&self) -> Self::Theme {
        crate::ui::theme::Theme::CatppuccinMocha
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        println!("[DEBUG] AppState::update: instance at {:p}", self as *const _);
        match message {
            Message::ToggleVisibility => {
                self.toggle_visibility();
            }
            Message::Exit => {
                #[cfg(target_os = "windows")]
                if let Some(tray) = &mut self.system_tray {
                    if let Err(e) = tray.cleanup() {
                        log::error!("Failed to clean up system tray: {}", e);
                    } else {
                        log::info!("System tray resources cleaned up");
                    }
                }
                if let Err(e) = self.config.save() {
                    log::error!("Failed to save settings on exit: {}", e);
                }
                std::process::exit(0);
            }
            Message::DeviceDiscovered(device) => {
                log::debug!("[UI] Device discovered: {:?}", device);
                self.update_device(device);
            }
            Message::DeviceUpdated(device) => {
                self.update_device(device);
            }
            Message::SelectDevice(address) => {
                self.select_device(address.clone());
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
                        status_clone.battery.case
                    ) {
                        log::warn!("Failed to update tray tooltip: {}", e);
                    }
                }
            }
            Message::AirPodsConnected(airpods) => {
                println!("Connected to AirPods: {:?}", airpods);
                if let Some(address) = self.selected_device.as_ref() {
                    if let Some(device) = self.devices.get_mut(address) {
                        device.is_potential_airpods = true;
                    }
                }
            }
            Message::BluetoothError(msg) => {
                println!("[DEBUG] AppState::update: setting toast_message = {:?}", msg);
                self.toast_message = Some(msg);
            }
            Message::UpdateBluetoothSetting(setting) => {
                self.settings_window.mark_changed();
                self.update_bluetooth_setting(setting);
                self.settings_window.update_config(self.config.clone());
            }
            Message::UpdateUiSetting(setting) => {
                self.settings_window.mark_changed();
                self.update_ui_setting(setting);
                self.settings_window.update_config(self.config.clone());
            }
            Message::UpdateSystemSetting(setting) => {
                self.settings_window.mark_changed();
                self.update_system_setting(setting);
                self.settings_window.update_config(self.config.clone());
            }
            Message::OpenSettings => {
                self.settings_window.set_validation_error(None);
                self.settings_window.update_config(self.config.clone());
                self.show_settings = true;
            }
            Message::CloseSettings => {
                self.show_settings = false;
                self.settings_window.update_config(self.config.clone());
            }
            Message::SaveSettings => {
                let updated_config = self.settings_window.config();
                if let Err(e) = updated_config.validate() {
                    self.settings_window.set_validation_error(Some(e.to_string()));
                    log::error!("Settings validation failed: {}", e);
                    return Command::none();
                }
                self.config = updated_config.clone();
                if let Err(e) = self.config.save() {
                    self.settings_window.set_validation_error(Some(format!("Failed to save: {}", e)));
                    log::error!("Settings save failed: {}", e);
                    return Command::none();
                }
                self.apply_settings();
                self.show_settings = false;
            }
            Message::SettingsChanged(config) => {
                self.config = config.clone();
                self.settings_window.update_config(config);
            }
            Message::ShowToast(msg) => {
                self.toast_message = Some(msg);
                return Command::perform(async {
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    ()
                }, |_| Message::Tick);
            }
            Message::MergedScanResult(devices) => {
                self.status_message = Some(format!("Found {} devices", devices.len()));
                self.merged_devices = devices.clone();
                self.main_window.merged_devices = devices;
            }
            Message::Tick => {
                println!("[DEBUG] Tick message received - refreshing device data");
                // Refresh device data from CLI scanner on each tick
                self.refresh_device_data();
            }
            _ => {
                log::debug!("Unhandled message in state.rs");
            }
        }
        Command::none()
    }

    fn view(&self) -> iced::Element<'_, Message, iced::Renderer<crate::ui::theme::Theme>> {
        use iced::widget::{container, text};
        use iced::Length;
        println!("[DEBUG] AppState::view: status_message = {:?}, toast_message = {:?}", self.status_message, self.toast_message);
        let status_message = self.status_message.as_ref();
        let toast_message = self.toast_message.as_ref();
        if !self.visible {
            iced::widget::text("").into()
        } else if self.show_settings {
            let mut column = iced::widget::Column::new();
            column = column.push(crate::ui::UiComponent::view(&self.settings_window));
            // Persistent status bar
            if let Some(status) = status_message {
                let status_bar = container(
                    text(status)
                        .size(20)
                        .style(crate::ui::theme::MAUVE)
                )
                .width(Length::Fill)
                .padding(8)
                .style(iced::theme::Container::Box);
                column = column.push(status_bar);
            }
            // Toast bar (bottom)
            if let Some(toast) = toast_message {
                let toast_bar = container(
                    text(toast)
                        .size(20)
                        .style(crate::ui::theme::ROSEWATER)
                )
                .width(Length::Fill)
                .padding(8)
                .style(iced::theme::Container::Box);
                column = column.push(toast_bar);
            }
            container(column).width(Length::Fill).height(Length::Fill).into()
        } else {
            let mut column = iced::widget::Column::new();
            column = column.push(crate::ui::UiComponent::view(&self.main_window));
            // Persistent status bar
            if let Some(status) = status_message {
                let status_bar = container(
                    text(status)
                        .size(20)
                        .style(crate::ui::theme::MAUVE)
                )
                .width(Length::Fill)
                .padding(8)
                .style(iced::theme::Container::Box);
                column = column.push(status_bar);
            }
            // Toast bar (bottom)
            if let Some(toast) = toast_message {
                let toast_bar = container(
                    text(toast)
                        .size(20)
                        .style(crate::ui::theme::ROSEWATER)
                )
                .width(Length::Fill)
                .padding(8)
                .style(iced::theme::Container::Box);
                column = column.push(toast_bar);
            }
            container(column).width(Length::Fill).height(Length::Fill).into()
        }
    }
    
    fn subscription(&self) -> Subscription<Message> {
        use iced::time;
        use std::time::Duration;
        
        // Timer for periodic CLI scanner updates (every 30 seconds)
        let timer = time::every(Duration::from_secs(30)).map(|_| Message::Tick);
        
        // The controller_receiver is passed in as part of the Flags tuple, so we need to capture it in the unfold state.
        // We'll use a static mut to store the receiver for the lifetime of the app (safe for this diagnostic purpose).
        static RECEIVER: OnceLock<Mutex<Option<mpsc::UnboundedReceiver<Message>>>> = OnceLock::new();
        let _receiver = RECEIVER.get_or_init(|| Mutex::new(None));
        // The first time, move the receiver from the flags into the static
        // (This is a hack for demo purposes; in production, use a better state management approach)
        Subscription::batch(vec![
            timer, // Add the timer subscription for periodic CLI scanner updates
            iced::subscription::events_with(|event, _status| {
                if let iced::Event::Window(iced::window::Event::CloseRequested) = event {
                    Some(Message::Exit)
                } else {
                    None
                }
            }),
            iced::subscription::unfold("controller-messages", (), move |()| async move {
                let mut guard = RECEIVER.get().unwrap().lock().await;
                if let Some(ref mut rx) = *guard {
                    // Await the next message (wait until a message is available)
                    if let Some(msg) = rx.recv().await {
                        println!("[DEBUG] AppState::subscription: received message from controller: {:?}", msg);
                        (msg, ())
                    } else {
                        // Channel closed, just await forever
                        futures::future::pending().await
                    }
                } else {
                    // No receiver, just await forever
                    futures::future::pending().await
                }
            }),
        ])
    }
}

impl AppState {
    /// Toggle the visibility of the application
    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }
    
    /// Initialize the system tray component
    pub fn initialize_system_tray(&mut self, tx: std::sync::mpsc::Sender<Message>) -> Result<(), SystemTrayError> {
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
        self.selected_device.as_ref().and_then(|addr| self.devices.get(addr))
    }
    
    /// Save the current settings
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
                    },
                    Err(e) => {
                        // Set error message
                        self.settings_error = Some(format!("Failed to save settings: {}", e));
                        
                        // Log error
                        log::error!("Failed to save settings: {}", e);
                    }
                }
            },
            Err(e) => {
                // Set validation error message
                self.settings_error = Some(format!("Invalid settings: {}", e));
                
                // Log error
                log::error!("Settings validation failed: {}", e);
            }
        }
    }
    
    /// Reset settings to defaults
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
    fn load_settings(&mut self) -> Result<(), ConfigError> {
        match AppConfig::load() {
            Ok(config) => {
                self.config = config;
                Ok(())
            },
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

    /// Refresh device data from the CLI scanner
    fn refresh_device_data(&mut self) {
        println!("[DEBUG] refresh_device_data called at {:?}", std::time::SystemTime::now());
        
        // Call the CLI scanner to get AirPods data
        let airpods_data = get_airpods_from_cli_scanner();
        
        println!("[DEBUG] CLI scanner returned {} AirPods devices", airpods_data.len());
        
        // Clear existing devices and update with fresh data from CLI scanner
        self.merged_devices.clear();
        
        // Update the merged devices list with AirPods data
        for airpods in airpods_data {
            println!("[DEBUG] Processing AirPods device: {:?}", airpods.address);
            
            let device = MergedBluetoothDevice {
                name: if airpods.name.is_empty() { "AirPods".to_string() } else { airpods.name },
                address: airpods.address.to_string(),
                paired: true,  // AirPods from CLI scanner are already paired
                connected: true, // Assume connected if detected by CLI scanner
                device_type: Some("AirPods".to_string()),
                battery: None, // Overall battery not used for AirPods
                left_battery: if airpods.left_battery >= 0 { Some(airpods.left_battery as u8) } else { None },
                right_battery: if airpods.right_battery >= 0 { Some(airpods.right_battery as u8) } else { None },
                case_battery: if airpods.case_battery >= 0 { Some(airpods.case_battery as u8) } else { None },
                device_subtype: Some("earbud".to_string()),
                left_in_ear: airpods.left_in_ear,
                right_in_ear: airpods.right_in_ear,
                case_lid_open: airpods.case_lid_open,
                side: airpods.side.map(|s| s.to_string()),
                both_in_case: airpods.both_in_case,
                color: airpods.color.map(|c| c.to_string()),
                switch_count: airpods.switch_count.map(|s| s as u8),
            };
            
            self.merged_devices.push(device);
        }
        
        println!("[DEBUG] Total merged devices after update: {}", self.merged_devices.len());
        
        // Update the main window with the new devices
        self.main_window.merged_devices = self.merged_devices.clone();
        
        // Set status message based on device count
        if self.merged_devices.is_empty() {
            self.status_message = Some("No AirPods devices found".to_string());
        } else {
            self.status_message = Some(format!("Updated at {} - {} device(s) found", 
                chrono::Local::now().format("%H:%M:%S"), 
                self.merged_devices.len()));
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PythonBleDevice {
    pub name: String,
    pub address: String,
    pub battery: Option<u8>,
}

#[derive(Debug, Clone)]
pub struct MergedBluetoothDevice {
    pub name: String,
    pub address: String,
    pub paired: bool,
    pub connected: bool,
    pub device_type: Option<String>,
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
}

/// Get AirPods data from the CLI scanner
fn get_airpods_from_cli_scanner() -> Vec<AirPodsBatteryInfo> {
    use std::process::Command;
    use crate::bluetooth::cli_scanner::{CliScannerResult, CliDeviceInfo};
    
    // Path to the CLI scanner executable (v5 is the working implementation)
    let scanner_path = "scripts/airpods_battery_cli/build/Release/airpods_battery_cli_v5.exe";
    
    println!("[DEBUG] Attempting to call CLI scanner at: {}", scanner_path);
    
    // Execute the CLI scanner
    match Command::new(scanner_path).output() {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                println!("[DEBUG] AirPods helper output: {}", stdout);
                
                // Extract JSON from the output (look for opening and closing braces)
                let json_str = if let (Some(start), Some(end)) = (stdout.find('{'), stdout.rfind('}')) {
                    &stdout[start..=end]
                } else {
                    println!("[DEBUG] No valid JSON found in output");
                    return Vec::new();
                };
                
                println!("[DEBUG] Extracted JSON: {}", json_str);
                
                // Try to parse the JSON output from the CLI scanner
                match serde_json::from_str::<CliScannerResult>(json_str) {
                    Ok(scanner_result) => {
                        println!("[DEBUG] Successfully parsed CLI scanner result with {} devices", scanner_result.devices.len());
                        let mut airpods_infos = Vec::new();
                        
                                // Convert CLI scanner results to AirPodsBatteryInfo format
        for device in scanner_result.devices {
            println!("[DEBUG] Processing device: {} (address: {}, rssi: {})", device.device_id, device.address, device.rssi);
            println!("[DEBUG] Manufacturer data: '{}'", device.manufacturer_data_hex);
            
            if let Some(airpods_data) = device.airpods_data {
                println!("[DEBUG] Found AirPods data: {} - L:{}% R:{}% C:{}%", 
                    airpods_data.model, airpods_data.left_battery, airpods_data.right_battery, airpods_data.case_battery);
                
                // Parse device ID as address (simplified for now)
                let address = device.device_id.chars()
                    .filter(|c| c.is_ascii_hexdigit())
                    .take(12)
                    .collect::<String>()
                    .parse::<u64>()
                    .unwrap_or(0x123456789ABC); // Default mock address
                
                let info = AirPodsBatteryInfo {
                    address,
                    name: airpods_data.model.clone(),
                    model_id: airpods_data.model_id.trim_start_matches("0x").parse::<u16>().unwrap_or(0x2014),
                    left_battery: airpods_data.left_battery,
                    left_charging: airpods_data.left_charging,
                    right_battery: airpods_data.right_battery,
                    right_charging: airpods_data.right_charging,
                    case_battery: airpods_data.case_battery,
                    case_charging: airpods_data.case_charging,
                    left_in_ear: Some(airpods_data.left_in_ear),
                    right_in_ear: Some(airpods_data.right_in_ear),
                    case_lid_open: Some(airpods_data.lid_open),
                    side: Some(0), // Default value
                    both_in_case: Some(airpods_data.both_in_case),
                    color: Some(0), // Default value
                    switch_count: Some(0), // Default value
                    rssi: None,
                    timestamp: Some(chrono::Local::now().timestamp() as u64),
                    raw_manufacturer_data: Some(device.manufacturer_data_hex),
                };
                
                airpods_infos.push(info);
            } else {
                println!("[DEBUG] Device {} is not AirPods (no manufacturer data or not Apple device)", device.device_id);
            }
        }
                        
                        println!("[DEBUG] Converted {} CLI devices to AirPodsBatteryInfo", airpods_infos.len());
                        airpods_infos
                    }
                    Err(e) => {
                        println!("[DEBUG] JSON parsing failed: {}", e);
                        println!("[DEBUG] No valid device data - returning empty list");
                        Vec::new()
                    }
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                log::error!("CLI scanner failed: {}", stderr);
                Vec::new()
            }
        }
        Err(e) => {
            log::error!("Failed to execute CLI scanner: {}", e);
            Vec::new()
        }
    }
} 