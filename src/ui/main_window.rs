//! Main application window for RustPods
//! 
//! Implements the main UI window component with device list and battery status display.

use iced::{
    widget::{button, column, container, row, text, scrollable, Svg, Space, image},
    Element, Length, Command, Alignment, Color,
    alignment::Horizontal
};
use iced::widget::svg::Handle;

use crate::bluetooth::AirPodsBatteryStatus;
use crate::airpods::DetectedAirPods;
use crate::ui::Message;
use crate::ui::theme;
use crate::config::AppConfig;
use crate::ui::UiComponent;
use crate::ui::components::svg_icons::{settings_icon_svg_string, headset_icon_svg_string};
use crate::ui::theme::Theme;
use crate::ui::state::MergedBluetoothDevice;

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
        crate::debug_log!("ui", "MainWindow::view_content: merged_devices.len() = {}", self.merged_devices.len());
        
        // Check if we have AirPods devices and show compact popup
        if let Some(device) = self.merged_devices.first() {
            crate::debug_log!("ui", "Showing popup for device: {}", device.name);
            // Show our new graphical AirPods popup - inline implementation to avoid theme issues
            let header_row = row![
                text(device.name.clone())
                    .size(24.0)
                    .style(crate::ui::theme::TEXT),
                Space::with_width(Length::Fill),
                button(
                    Svg::new(Handle::from_memory(crate::assets::ui::SETTINGS_ICON))
                        .width(Length::Fixed(16.0))
                        .height(Length::Fixed(16.0))
                )
                .on_press(Message::OpenSettings)
                .style(crate::ui::theme::settings_button_style())
                .padding(6),
                button(
                    Svg::new(Handle::from_memory(crate::assets::ui::CLOSE_ICON))
                        .width(Length::Fixed(16.0))
                        .height(Length::Fixed(16.0))
                )
                .on_press(Message::Exit)
                .style(crate::ui::theme::close_button_style())
                .padding(6)
            ]
            .spacing(8)
            .align_items(Alignment::Center);

            let device_images_row = row![
                image("assets/icons/hw/airpodspro.png")
                    .height(Length::Fixed(128.0)),
                image("assets/icons/hw/airpodsprocase.png")
                    .height(Length::Fixed(128.0))
            ]
            .align_items(Alignment::Center)
            .spacing(40);

            let battery_indicators_row = row![
                // Earbuds group: Left and Right together
                row![
                    // Left battery indicator with colored icon
                    column![
                        crate::ui::components::battery_icon::battery_icon_display(
                            device.left_battery,
                            false, // TODO: Add charging status when available
                            64.0,  // Increased icon size for better visibility
                            self.animation_progress
                        ),
                        text("Left")
                            .size(12)
                            .style(crate::ui::theme::SUBTEXT1)
                            .horizontal_alignment(Horizontal::Center),
                        text(format!("{}%", device.left_battery.unwrap_or(0)))
                            .size(16)
                            .style(crate::ui::theme::TEXT)
                            .horizontal_alignment(Horizontal::Center)
                    ]
                    .align_items(Alignment::Center)
                    .spacing(4),
                    
                    // Right battery indicator with colored icon
                    column![
                        crate::ui::components::battery_icon::battery_icon_display(
                            device.right_battery,
                            false, // TODO: Add charging status when available
                            64.0,  // Increased icon size for better visibility
                            self.animation_progress
                        ),
                        text("Right")
                            .size(12)
                            .style(crate::ui::theme::SUBTEXT1)
                            .horizontal_alignment(Horizontal::Center),
                        text(format!("{}%", device.right_battery.unwrap_or(0)))
                            .size(16)
                            .style(crate::ui::theme::TEXT)
                            .horizontal_alignment(Horizontal::Center)
                    ]
                    .align_items(Alignment::Center)
                    .spacing(4)
                ]
                .spacing(20)
                .align_items(Alignment::Center),
                
                // Case indicator with colored icon
                column![
                    crate::ui::components::battery_icon::battery_icon_display(
                        device.case_battery,
                        false, // TODO: Add charging status when available
                        64.0,  // Increased icon size for better visibility
                        self.animation_progress
                    ),
                    text("Case")
                        .size(12)
                        .style(crate::ui::theme::SUBTEXT1)
                        .horizontal_alignment(Horizontal::Center),
                    text(format!("{}%", device.case_battery.unwrap_or(0)))
                        .size(16)
                        .style(crate::ui::theme::TEXT)
                        .horizontal_alignment(Horizontal::Center)
                ]
                .align_items(Alignment::Center)
                .spacing(4)
            ]
            .spacing(40)
            .align_items(Alignment::Center);

            // Wrap the popup in a polished container with modern styling
            container(
                column![
                    header_row,
                    device_images_row,
                    battery_indicators_row,
                ]
                .spacing(20)
                .align_items(Alignment::Center)
                .padding([20, 25, 25, 25])  // top, right, bottom, left
            )
            .width(Length::Fixed(420.0))
            .height(Length::Fixed(320.0))
            .style(iced::theme::Container::Box)
            .center_x()
            .center_y()
            .into()
        } else {
            crate::debug_log!("ui", "No AirPods found, showing search message");
            // Show a styled message in the new popup format
            let header_row = row![
                text("RustPods")
                    .size(24.0)
                    .style(crate::ui::theme::TEXT),
                Space::with_width(Length::Fill),
                button(
                    Svg::new(Handle::from_memory(crate::assets::ui::SETTINGS_ICON))
                        .width(Length::Fixed(16.0))
                        .height(Length::Fixed(16.0))
                )
                .on_press(Message::OpenSettings)
                .style(crate::ui::theme::settings_button_style())
                .padding(6),
                button(
                    Svg::new(Handle::from_memory(crate::assets::ui::CLOSE_ICON))
                        .width(Length::Fixed(16.0))
                        .height(Length::Fixed(16.0))
                )
                .on_press(Message::Exit)
                .style(crate::ui::theme::close_button_style())
                .padding(6)
            ]
            .spacing(8)
            .align_items(Alignment::Center);

            let search_message = column![
                text("🔍")
                    .size(48.0)
                    .horizontal_alignment(Horizontal::Center),
                text("Searching for AirPods...")
                    .size(18.0)
                    .style(crate::ui::theme::SUBTEXT1)
                    .horizontal_alignment(Horizontal::Center),
                text("Make sure your AirPods are:")
                    .size(14.0)
                    .style(crate::ui::theme::OVERLAY1)
                    .horizontal_alignment(Horizontal::Center),
                text("• Out of the case OR being used")
                    .size(12.0)
                    .style(crate::ui::theme::OVERLAY1)
                    .horizontal_alignment(Horizontal::Center),
                text("• Connected to this device")
                    .size(12.0)
                    .style(crate::ui::theme::OVERLAY1)
                    .horizontal_alignment(Horizontal::Center),
                text("• Broadcasting (not in deep sleep)")
                    .size(12.0)
                    .style(crate::ui::theme::OVERLAY1)
                    .horizontal_alignment(Horizontal::Center),
                text("")
                    .size(8.0),
                text("Scanning automatically every 15 seconds...")
                    .size(11.0)
                    .style(crate::ui::theme::YELLOW)
                    .horizontal_alignment(Horizontal::Center)
            ]
            .spacing(6)
            .align_items(Alignment::Center);

            // Wrap in the same styled container as the popup
            container(
                column![
                    header_row,
                    search_message,
                ]
                .spacing(20)
                .align_items(Alignment::Center)
                .padding([20, 25, 25, 25])  // top, right, bottom, left
            )
            .width(Length::Fixed(420.0))
            .height(Length::Fixed(320.0))
            .style(iced::theme::Container::Box)
            .center_x()
            .center_y()
            .into()
        }
    }

    
    /// Create a simple battery bar indicator
    fn create_simple_battery_bar(&self, battery_level: Option<u8>) -> Element<'_, Message, iced::Renderer<Theme>> {
        let level = battery_level.unwrap_or(0);
        
        // Simple battery indicator using text
        let battery_text = "█".repeat((level / 10) as usize);
        text(battery_text)
            .size(12)
            .style(if level > 20 { theme::GREEN } else if level > 10 { theme::YELLOW } else { theme::RED })
            .into()
    }
}

impl UiComponent for MainWindow {
    fn view(&self) -> Element<'_, Message, iced::Renderer<Theme>> {
        // Always show the new UI - no old fallback
        let content = self.view_content();
        
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(crate::ui::theme::device_row_style())
            .center_x()
            .center_y()
            .into()
    }
}

