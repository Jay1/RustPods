//! Main application window for RustPods
//!
//! Implements the main UI window component with device list and battery status display.

use iced::widget::svg::Handle as SvgHandle;
use iced::{
    alignment::Horizontal,
    widget::{button, column, container, row, text, Space, Svg, mouse_area},
    Alignment, Command, Element, Length,
};

use crate::airpods::DetectedAirPods;
use crate::bluetooth::AirPodsBatteryStatus;
use crate::config::AppConfig;
use crate::ui::theme;
use crate::ui::Message;
use crate::ui::UiComponent;

use crate::ui::state::{MergedBluetoothDevice, DeviceDetectionState};
use crate::ui::theme::Theme;
use crate::ui::components::WaitingMode;

/// Main window component
#[derive(Debug, Clone)]
pub struct MainWindow {
    /// Selected device (if any)
    pub selected_device: Option<DetectedAirPods>,

    /// List of discovered devices
    pub merged_devices: Vec<MergedBluetoothDevice>,

    /// Whether a scan is currently in progress
    pub is_scanning: bool,

    /// Animation progress (0.0-1.0)
    pub animation_progress: f32,

    /// Application configuration
    pub config: AppConfig,

    /// Whether advanced display mode is enabled
    pub advanced_display_mode: bool,

    /// AirPods popup component
    pub show_airpods_dialog: bool,

    /// Device detection state to track AirPods connectivity
    pub device_detection_state: DeviceDetectionState,

    /// Waiting mode component for when no devices are detected
    pub waiting_mode: WaitingMode,
}

impl Default for MainWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl MainWindow {
    /// Create an empty main window
    pub fn empty() -> Self {
        Self::new()
    }

    /// Create a new main window
    pub fn new() -> Self {
        Self {
            selected_device: None,
            merged_devices: Vec::new(),
            is_scanning: false,
            animation_progress: 0.0,
            config: AppConfig::default(),
            advanced_display_mode: false,
            show_airpods_dialog: false,
            device_detection_state: DeviceDetectionState::Scanning,
            waiting_mode: WaitingMode::new(),
        }
    }

    /// Set the animation progress and return a new instance
    pub fn with_animation_progress(mut self, progress: f32) -> Self {
        self.animation_progress = progress;
        self
    }

    /// Set the window size and return a new instance
    pub fn with_window_size(self, _size: (u32, u32)) -> Self {
        // Window size not directly used in MainWindow, but needed for compatibility
        self
    }

    /// Toggle advanced display mode
    pub fn toggle_advanced_display(&mut self) -> Command<Message> {
        self.advanced_display_mode = !self.advanced_display_mode;
        Command::none()
    }

    /// Toggle advanced display mode and return a command
    pub fn toggle_display_mode(&mut self) -> Command<Message> {
        let _ = self.toggle_advanced_display();
        Command::none()
    }

    /// Connect to a device
    pub fn connect_to_device(&mut self, airpods: DetectedAirPods) -> Command<Message> {
        self.selected_device = Some(airpods);
        Command::none()
    }

    /// Disconnect from the current device
    pub fn disconnect(&mut self) -> Command<Message> {
        self.selected_device = None;
        Command::none()
    }

    /// Update scan status
    pub fn set_scanning(&mut self, scanning: bool) -> Command<Message> {
        self.is_scanning = scanning;
        Command::none()
    }

    /// Update animation progress
    pub fn update_animation(&mut self, progress: f32) -> Command<Message> {
        self.animation_progress = progress;
        Command::none()
    }

    /// Update battery status
    pub fn update_battery(&mut self, battery_status: AirPodsBatteryStatus) -> Command<Message> {
        if let Some(device) = &mut self.selected_device {
            device.battery = Some(battery_status.battery);
        }
        Command::none()
    }

    /// Set connection transition animation progress and return a new instance
    pub fn with_connection_transition(mut self, progress: f32) -> Self {
        self.animation_progress = progress;
        self
    }

    /// Set battery status and return a new instance
    pub fn with_battery_status(mut self, battery_status: AirPodsBatteryStatus) -> Self {
        if let Some(device) = &mut self.selected_device {
            device.battery = Some(battery_status.battery);
        }
        self
    }

    /// Update device detection state
    pub fn update_device_detection_state(&mut self, state: DeviceDetectionState) {
        self.device_detection_state = state.clone();
        self.waiting_mode.update_detection_state(state);
    }

    /// Update waiting mode animation
    pub fn update_waiting_mode_animation(&mut self, progress: f32) {
        self.waiting_mode.update_animation(progress);
    }

    // Update the view method to use the helper methods
    fn view_content(&self) -> Element<'_, Message, iced::Renderer<Theme>> {
        crate::debug_log!("ui", "MainWindow::view_content - merged_devices.len() = {}", self.merged_devices.len());

        // Custom title bar header (Discord-style) - make it draggable
        let header_row = mouse_area(
            container(
                row![
                    // App logo and title - this area should be draggable
                    mouse_area(
                        row![
                            // App logo icon
                            container(
                                Svg::new(SvgHandle::from_memory(crate::assets::app::LOGO_SVG))
                                    .width(Length::Fixed(24.0))
                                    .height(Length::Fixed(24.0))
                            )
                            .padding([0, 8, 0, 0]), // Right padding to separate from text
                            // App title/brand
                            text("RustPods")
                                .size(20.0)
                                .style(crate::ui::theme::TEXT)
                        ]
                        .align_items(Alignment::Center)
                        .spacing(0)
                    )
                    .on_press(Message::WindowDragStart(iced::Point::new(0.0, 0.0))),
                    Space::with_width(Length::Fill),
                    // Window controls
                    button(
                        Svg::new(SvgHandle::from_memory(crate::assets::ui::SETTINGS_ICON))
                            .width(Length::Fixed(21.0))
                            .height(Length::Fixed(21.0))
                    )
                    .on_press(Message::OpenSettings)
                    .style(crate::ui::theme::settings_button_style())
                    .padding(5),
                    button(
                        Svg::new(SvgHandle::from_memory(crate::assets::ui::CLOSE_ICON))
                            .width(Length::Fixed(21.0))
                            .height(Length::Fixed(21.0))
                    )
                    .on_press(Message::Exit)
                    .style(crate::ui::theme::close_button_style())
                    .padding(5)
                ]
                .spacing(6)
                .align_items(Alignment::Center)
            )
            .width(Length::Fill)
            .padding([8, 12, 8, 12]) // Custom title bar padding
        )
        .on_press(Message::WindowDragStart(iced::Point::new(0.0, 0.0))); // Make entire title bar draggable

        // Determine what content to show based on device detection state
        let main_content = if self.merged_devices.is_empty() || !self.device_detection_state.has_active_device() {
            // Show waiting mode when no devices are detected or not connected
            crate::debug_log!("ui", "No devices detected, showing waiting mode");
            self.waiting_mode.view()
        } else if let Some(device) = self.merged_devices.first() {
            // Show battery widgets when devices are connected
            // Use fractional battery levels if available, otherwise fall back to integer levels
            let left_battery = device.left_battery_fractional
                .unwrap_or(device.left_battery.unwrap_or(0) as f32);
            let right_battery = device.right_battery_fractional
                .unwrap_or(device.right_battery.unwrap_or(0) as f32);
            
            crate::debug_log!("ui", "Showing battery UI for device: {} - L:{:.1}% R:{:.1}%", 
                device.name, left_battery, right_battery);

            // Get custom device name from config if available
            let display_name = self.config.bluetooth.paired_device_name
                .as_ref()
                .unwrap_or(&device.name);

            // Main layout with device name at top and battery widgets below
            container(
                column![
                    // Device name at the top
                    container(
                        text(display_name)
                            .size(18)
                            .style(theme::TEXT)
                            .horizontal_alignment(Horizontal::Center)
                    )
                    .width(Length::Fill)
                    .center_x()
                    .padding([0, 0, 15, 0]), // Bottom padding to separate from battery widgets
                    
                    // Two-column layout: each battery centered in its half of the window
                    container(
                        row![
                            // Left column - Left earbud centered in left half
                            container(
                                column![
                                    crate::ui::components::view_circular_battery_widget(
                                        left_battery,
                                        false // TODO: Add charging status when available
                                    ),
                                    text("Left")
                                        .size(14)
                                        .style(theme::TEXT)
                                        .horizontal_alignment(Horizontal::Center)
                                ]
                                .align_items(Alignment::Center)
                                .spacing(5)
                            )
                            .width(Length::FillPortion(1))
                            .center_x(),
                            
                            // Right column - Right earbud centered in right half
                            container(
                                column![
                                    crate::ui::components::view_circular_battery_widget(
                                        right_battery,
                                        false // TODO: Add charging status when available
                                    ),
                                    text("Right")
                                        .size(14)
                                        .style(theme::TEXT)
                                        .horizontal_alignment(Horizontal::Center)
                                ]
                                .align_items(Alignment::Center)
                                .spacing(5)
                            )
                            .width(Length::FillPortion(1))
                            .center_x()
                        ]
                        .width(Length::Fill)
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                ]
                .align_items(Alignment::Center)
                .spacing(0)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
        } else {
            // Fallback to waiting mode
            crate::debug_log!("ui", "Fallback to waiting mode");
            self.waiting_mode.view()
        };

        // Main layout: title bar at top, main content centered in remaining space
        container(
            column![
                // Title bar stays at the top with proper background
                header_row,
                
                // Main content (battery widgets or waiting mode) centered in the remaining space
                container(main_content)
                    .width(Length::Fill)
                    .height(Length::Fill)
            ]
            .width(Length::Fill)
            .height(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    /// Create a simple battery bar indicator
    #[allow(dead_code)]
    fn create_simple_battery_bar(
        &self,
        battery_level: Option<u8>,
    ) -> Element<'_, Message, iced::Renderer<Theme>> {
        let level = battery_level.unwrap_or(0);

        // Simple battery indicator using text
        let battery_text = "█".repeat((level / 10) as usize);
        text(battery_text)
            .size(12)
            .style(if level > 20 {
                theme::GREEN
            } else if level > 10 {
                theme::YELLOW
            } else {
                theme::RED
            })
            .into()
    }
}

impl UiComponent for MainWindow {
    fn view(&self) -> Element<'_, Message, iced::Renderer<Theme>> {
        // Return content directly without any wrapper - use full window space
        self.view_content()
    }
}
