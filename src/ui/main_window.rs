//! Main application window for RustPods
//!
//! Implements the main UI window component with device list and battery status display.

use iced::widget::svg::Handle;
use iced::{
    alignment::Horizontal,
    widget::{button, column, container, row, text, Space, Svg},
    Alignment, Command, Element, Length,
};

use crate::airpods::DetectedAirPods;
use crate::bluetooth::AirPodsBatteryStatus;
use crate::config::AppConfig;
use crate::ui::theme;
use crate::ui::Message;
use crate::ui::UiComponent;

use crate::ui::state::MergedBluetoothDevice;
use crate::ui::theme::Theme;

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

    // Update the view method to use the helper methods
    fn view_content(&self) -> Element<'_, Message, iced::Renderer<Theme>> {
        crate::debug_log!(
            "ui",
            "MainWindow::view_content: merged_devices.len() = {}",
            self.merged_devices.len()
        );

        // Always show the battery UI - get device data or use defaults (Left and Right only)
        let (left_battery, right_battery) = if let Some(device) = self.merged_devices.first() {
            crate::debug_log!("ui", "Showing popup for device: {}", device.name);
            (device.left_battery.unwrap_or(0), device.right_battery.unwrap_or(0))
        } else {
            crate::debug_log!("ui", "No devices found, showing default battery UI with 0% values");
            (0, 0)
        };

        // Custom title bar header (Discord-style)
        let header_row = container(
            row![
                // App title/brand
                text("RustPods")
                    .size(20.0)
                    .style(crate::ui::theme::TEXT),
                Space::with_width(Length::Fill),
                // Window controls
                button(
                    Svg::new(Handle::from_memory(crate::assets::ui::SETTINGS_ICON))
                        .width(Length::Fixed(21.0))
                        .height(Length::Fixed(21.0))
                )
                .on_press(Message::OpenSettings)
                .style(crate::ui::theme::settings_button_style())
                .padding(5),
                button(
                    Svg::new(Handle::from_memory(crate::assets::ui::CLOSE_ICON))
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
        .style(iced::theme::Container::Box);

        // Two-column layout: each battery centered in its half of the window
        let content_row = row![
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
        .width(Length::Fill);

        // Main layout: title bar at top, battery widgets centered in remaining space
        container(
            column![
                // Title bar stays at the top with proper background
                header_row,
                
                // Battery widgets centered in the remaining space
                container(content_row)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .center_y()
            ]
            .width(Length::Fill)
            .height(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(iced::theme::Container::Box)
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
