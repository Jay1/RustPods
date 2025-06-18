//! Waiting mode UI component for RustPods
//!
//! Displays a sophisticated waiting screen when no AirPods are detected,
//! replacing the arbitrary battery display with proper user feedback.

use iced::{
    alignment::Horizontal,
    widget::{column, container, text, Space},
    Alignment, Element, Length,
};
use std::time::Duration;

use crate::ui::state::DeviceDetectionState;
use crate::ui::{theme::Theme, Message, UiComponent};

/// Waiting mode component that displays when no AirPods are detected
#[derive(Debug, Clone)]
pub struct WaitingMode {
    /// Current device detection state
    pub detection_state: DeviceDetectionState,

    /// Animation progress for scanning indicator (0.0-1.0)
    pub animation_progress: f32,

    /// Time since last scan attempt
    pub time_since_last_scan: Option<Duration>,

    /// Next scan countdown
    pub next_scan_in: Option<Duration>,

    /// Whether manual scan is in progress
    pub manual_scan_in_progress: bool,
}

impl Default for WaitingMode {
    fn default() -> Self {
        Self::new()
    }
}

impl WaitingMode {
    /// Create a new waiting mode component
    pub fn new() -> Self {
        Self {
            detection_state: DeviceDetectionState::Scanning,
            animation_progress: 0.0,
            time_since_last_scan: None,
            next_scan_in: None,
            manual_scan_in_progress: false,
        }
    }

    /// Update the detection state
    pub fn update_detection_state(&mut self, state: DeviceDetectionState) {
        self.detection_state = state;
    }

    /// Update animation progress
    pub fn update_animation(&mut self, progress: f32) {
        self.animation_progress = progress.clamp(0.0, 1.0);
    }

    /// Set scan timing information
    pub fn update_scan_timing(&mut self, last_scan: Option<Duration>, next_scan: Option<Duration>) {
        self.time_since_last_scan = last_scan;
        self.next_scan_in = next_scan;
    }

    /// Set manual scan progress
    pub fn set_manual_scan_progress(&mut self, in_progress: bool) {
        self.manual_scan_in_progress = in_progress;
    }

    /// Create the scanning animation indicator
    fn scanning_indicator(&self) -> Element<'_, Message, iced::Renderer<Theme>> {
        // Create a simple pulsing circle animation
        let pulse_size = 40.0 + (self.animation_progress * 20.0);

        // Use a simple text-based animation for now (can be replaced with SVG later)
        container(
            text("⟲")
                .size(pulse_size)
                .style(crate::ui::theme::BLUE)
                .horizontal_alignment(Horizontal::Center),
        )
        .width(Length::Fixed(80.0))
        .height(Length::Fixed(80.0))
        .center_x()
        .center_y()
        .into()
    }

    /// Create the status message based on detection state
    fn status_message(&self) -> Element<'_, Message, iced::Renderer<Theme>> {
        let (primary_message, secondary_message) = match &self.detection_state {
            DeviceDetectionState::Scanning => (
                "Looking for your AirPods...".to_string(),
                "Make sure your AirPods are nearby and the case is open".to_string(),
            ),
            DeviceDetectionState::Idle => (
                "Ready to scan".to_string(),
                "Click scan to look for your AirPods".to_string(),
            ),
            DeviceDetectionState::DevicesFound => (
                "AirPods found".to_string(),
                "Loading battery information...".to_string(),
            ),
            DeviceDetectionState::DeviceFound { device_name, .. } => (
                format!("Found {}", device_name),
                "Connecting...".to_string(),
            ),
            DeviceDetectionState::NoDevicesFound => (
                "No AirPods found".to_string(),
                "Make sure your AirPods are nearby, paired, and the case is open".to_string(),
            ),
            DeviceDetectionState::Error {
                message,
                retry_count,
            } => {
                if *retry_count > 0 {
                    (
                        "Connection error - Retrying...".to_string(),
                        format!("Attempt {} - {}", retry_count + 1, message),
                    )
                } else {
                    ("Connection error".to_string(), message.clone())
                }
            }
            DeviceDetectionState::Connected { device_name, .. } => (
                format!("Connected to {}", device_name),
                "Loading battery information...".to_string(),
            ),
        };

        column![
            text(primary_message)
                .size(18.0)
                .style(crate::ui::theme::TEXT)
                .horizontal_alignment(Horizontal::Center),
            Space::with_height(Length::Fixed(8.0)),
            text(secondary_message)
                .size(14.0)
                .style(crate::ui::theme::SUBTEXT1)
                .horizontal_alignment(Horizontal::Center),
        ]
        .align_items(Alignment::Center)
        .into()
    }

    /// Create the scan timing display
    fn scan_timing_display(&self) -> Element<'_, Message, iced::Renderer<Theme>> {
        if let Some(next_scan) = self.next_scan_in {
            let seconds = next_scan.as_secs();
            if seconds > 0 {
                return text(format!("Next scan in {}s", seconds))
                    .size(12.0)
                    .style(crate::ui::theme::OVERLAY1)
                    .horizontal_alignment(Horizontal::Center)
                    .into();
            }
        }

        if let Some(last_scan) = self.time_since_last_scan {
            let seconds = last_scan.as_secs();
            return text(format!("Last scan {}s ago", seconds))
                .size(12.0)
                .style(crate::ui::theme::OVERLAY1)
                .horizontal_alignment(Horizontal::Center)
                .into();
        }

        Space::with_height(Length::Fixed(0.0)).into()
    }

    /// Create helpful tips section
    fn tips_section(&self) -> Element<'_, Message, iced::Renderer<Theme>> {
        column![
            text("Troubleshooting Tips:")
                .size(14.0)
                .style(crate::ui::theme::SUBTEXT1),
            Space::with_height(Length::Fixed(8.0)),
            text("• Make sure Bluetooth is enabled")
                .size(12.0)
                .style(crate::ui::theme::OVERLAY1),
            text("• Ensure AirPods are paired with this device")
                .size(12.0)
                .style(crate::ui::theme::OVERLAY1),
            text("• Keep the AirPods case open during scanning")
                .size(12.0)
                .style(crate::ui::theme::OVERLAY1),
            text("• Move closer to your AirPods")
                .size(12.0)
                .style(crate::ui::theme::OVERLAY1),
        ]
        .spacing(4.0)
        .align_items(Alignment::Start)
        .into()
    }
}

impl UiComponent for WaitingMode {
    fn view(&self) -> Element<'_, Message, iced::Renderer<Theme>> {
        container(
            column![
                // Scanning animation
                self.scanning_indicator(),
                Space::with_height(Length::Fixed(24.0)),
                // Status messages
                self.status_message(),
                Space::with_height(Length::Fixed(16.0)),
                // Scan timing
                self.scan_timing_display(),
                Space::with_height(Length::Fixed(32.0)),
                // Tips section
                self.tips_section(),
            ]
            .align_items(Alignment::Center)
            .spacing(0.0),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .padding(32.0)
        .into()
    }
}
