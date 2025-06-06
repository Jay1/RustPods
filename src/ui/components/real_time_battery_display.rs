//! Real-time battery information display with animations and enhanced visuals
//!
//! This component displays battery information with animations, color coding, and charging indicators.

use iced::alignment;
use iced::widget::{column, container, progress_bar, row, text};
use iced::{Color, Element, Length};
use std::time::Instant;

use crate::ui::{Message, UiComponent};
use crate::ui::theme::Theme;
use crate::bluetooth::AirPodsBatteryStatus;

// Constants for animation
const ANIMATION_DURATION_MS: u64 = 1000;
const CHARGING_PULSE_SPEED: f32 = 0.5;
const MIN_PULSE_OPACITY: f32 = 0.7;

/// Component for displaying real-time battery information with animations
#[derive(Debug, Clone)]
pub struct RealTimeBatteryDisplay {
    /// Battery status information
    pub battery_status: Option<AirPodsBatteryStatus>,
    /// Current animation progress (0.0-1.0)
    pub animation_progress: f32,
    /// Time when the component was last updated
    last_update: Option<Instant>,
    /// Show time since last update
    pub show_time_since_update: bool,
    /// Show detailed info (connection time, estimates)
    pub show_detailed_info: bool,
    /// Battery levels from previous update for animation
    pub previous_levels: Option<(Option<u8>, Option<u8>, Option<u8>)>,
    /// Show compact view
    pub compact_view: bool,
}

impl Default for RealTimeBatteryDisplay {
    fn default() -> Self {
        Self {
            battery_status: None,
            animation_progress: 0.0,
            last_update: None,
            show_time_since_update: true,
            show_detailed_info: true,
            previous_levels: None,
            compact_view: false,
        }
    }
}

impl RealTimeBatteryDisplay {
    /// Create a new real-time battery display
    pub fn new(battery_status: Option<AirPodsBatteryStatus>) -> Self {
        Self {
            battery_status,
            animation_progress: 0.0,
            last_update: Some(Instant::now()),
            show_time_since_update: true,
            show_detailed_info: false,
            previous_levels: None,
            compact_view: false,
        }
    }
    
    /// Set the animation progress
    pub fn with_animation_progress(mut self, progress: f32) -> Self {
        self.animation_progress = progress;
        self
    }
    
    /// Update the battery status
    pub fn update(&mut self, battery_status: Option<AirPodsBatteryStatus>) {
        // Store current levels for animation transition
        if let Some(current_status) = &self.battery_status {
            let battery = &current_status.battery;
            self.previous_levels = Some((
                battery.left,
                battery.right,
                battery.case,
            ));
        }

        self.battery_status = battery_status;
        self.last_update = Some(Instant::now());
        self.animation_progress = 0.0; // Reset animation when updating
    }
    
    /// Set compact view mode
    pub fn with_compact_view(mut self, compact: bool) -> Self {
        self.compact_view = compact;
        self
    }
    
    /// Set whether to show time since last update
    pub fn with_time_since_update(mut self, show: bool) -> Self {
        self.show_time_since_update = show;
        self
    }
    
    /// Set whether to show detailed information
    pub fn with_detailed_info(mut self, show: bool) -> Self {
        self.show_detailed_info = show;
        self
    }
    
    /// Set the last update time - primarily used for testing
    pub fn set_last_update(&mut self, time: Option<Instant>) {
        self.last_update = time;
    }
    
    /// Get interpolated battery level for animations
    pub fn get_animated_level(&self, current: Option<u8>, previous: Option<u8>) -> Option<u8> {
        match (current, previous) {
            (Some(current), Some(previous)) => {
                let diff = current as f32 - previous as f32;
                let animated = previous as f32 + (diff * self.animation_progress);
                Some(animated as u8)
            },
            (Some(current), None) => {
                // Animate from 0 to current
                let animated = current as f32 * self.animation_progress;
                Some(animated as u8)
            },
            _ => current,
        }
    }
    
    /// Calculate estimated time remaining in minutes based on battery levels
    /// 
    /// This is exposed as public for testing purposes.
    pub fn calculate_time_remaining(&self) -> Option<u32> {
        // This is a simplified estimation model
        // In a real app, you'd use historical battery drain rates
        
        if let Some(status) = &self.battery_status {
            // Get the minimum non-zero battery level of earbuds
            let min_level = match (status.battery.left, status.battery.right) {
                (Some(left), Some(right)) => Some(left.min(right)),
                (Some(left), None) => Some(left),
                (None, Some(right)) => Some(right),
                _ => None,
            };
            
            // Simple estimation: 5 hours for 100% battery
            // Adjust based on actual device specifications
            if let Some(level) = min_level {
                if level == 0 { return Some(0); }
                
                // Average battery life in minutes (300 = 5 hours)
                let max_battery_life_minutes = 300;
                let remaining_minutes = (level as u32 * max_battery_life_minutes) / 100;
                
                return Some(remaining_minutes);
            }
        }
        
        None
    }
    
    /// Format time remaining in a human-readable format
    /// 
    /// This is exposed as public for testing purposes.
    pub fn format_time_remaining(&self, minutes: u32) -> String {
        let hours = minutes / 60;
        let mins = minutes % 60;
        
        if hours > 0 {
            format!("{}h {}m", hours, mins)
        } else {
            format!("{}m", mins)
        }
    }
    
    /// Get the time since last update as a formatted string
    fn get_time_since_update(&self) -> Option<String> {
        self.last_update.map(|time| {
            let elapsed = time.elapsed();
            let seconds = elapsed.as_secs();
            
            if seconds < 60 {
                format!("{}s ago", seconds)
            } else if seconds < 3600 {
                format!("{}m {}s ago", seconds / 60, seconds % 60)
            } else {
                format!(
                    "{}h {}m ago", 
                    seconds / 3600, 
                    (seconds % 3600) / 60
                )
            }
        })
    }
    
    /// Calculate the pulsing effect for charging animation
    fn calculate_pulse_effect(&self) -> f32 {
        let pulse = (1.0 + (self.animation_progress * CHARGING_PULSE_SPEED * std::f32::consts::PI * 2.0).sin()) * 0.5;
        MIN_PULSE_OPACITY + ((1.0 - MIN_PULSE_OPACITY) * pulse)
    }
    
    /// Get color based on battery level
    fn get_color_for_level(&self, level: Option<u8>, is_charging: bool) -> Color {
        match level {
            Some(_level) if is_charging => Color::from_rgb(0.2, 0.6, 0.8), // Blue for charging
            Some(level) if level <= 20 => Color::from_rgb(0.8, 0.2, 0.2), // Red for low
            Some(level) if level <= 50 => Color::from_rgb(0.9, 0.6, 0.1), // Orange for medium
            Some(_) => Color::from_rgb(0.2, 0.7, 0.2),                   // Green for good
            None => Color::from_rgb(0.5, 0.5, 0.5),                      // Gray for unknown
        }
    }
    
    /// Create a stylized battery bar for an AirPods component
    fn create_battery_bar(
        &self,
        label: &str,
        current_level: Option<u8>,
        previous_level: Option<u8>,
        is_charging: bool,
    ) -> Element<'static, Message, iced::Renderer<Theme>> {
        // Get interpolated level for animation
        let animated_level = self.get_animated_level(current_level, previous_level);
        
        // Determine color based on level and charging status
        let _color = self.get_color_for_level(current_level, is_charging);
        
        // Calculate the pulse effect for charging animation
        let _opacity = if is_charging { self.calculate_pulse_effect() } else { 1.0 };
        
        let level_text = match current_level {
            Some(level) => format!("{}%", level),
            None => "N/A".to_string(),
        };
        
        let level_f32 = animated_level.unwrap_or(0) as f32 / 100.0;
        
        // Create a custom progress bar with dynamic colors based on the battery level
        let progress = progress_bar(0.0..=1.0, level_f32)
            .height(18.0);
            
        let charging_icon = if is_charging {
            text("⚡")
        } else {
            text("")
        };
        
        row![
            text(label)
                .width(Length::Fixed(50.0)),
            progress.width(Length::Fill),
            text(level_text)
                .width(Length::Fixed(50.0))
                .horizontal_alignment(alignment::Horizontal::Right),
            charging_icon.width(Length::Fixed(20.0)),
        ]
        .spacing(10)
        .align_items(alignment::Alignment::Center)
        .width(Length::Fill)
        .padding(2)
        .into()
    }
    
    /// Create a status summary display
    fn create_status_summary(&self) -> Element<'static, Message, iced::Renderer<Theme>> {
        if let Some(status) = &self.battery_status {
            let is_charging = status.battery.charging.as_ref().is_some_and(|c| c.is_any_charging());
                               
            let is_low_battery = status.battery.left.is_some_and(|l| l <= 20) ||
                                status.battery.right.is_some_and(|r| r <= 20) ||
                                status.battery.case.is_some_and(|c| c <= 20);
            
            let (status_text, _color) = if is_charging {
                ("Charging", Color::from_rgb(0.2, 0.6, 0.8))
            } else if is_low_battery {
                ("Low Battery", Color::from_rgb(0.8, 0.2, 0.2))
            } else {
                ("Connected", Color::from_rgb(0.2, 0.7, 0.2))
            };
            
            // Add time remaining estimate
            let time_text = if let Some(minutes) = self.calculate_time_remaining() {
                format!(" • Approx. {} remaining", self.format_time_remaining(minutes))
            } else {
                "".to_string()
            };
            
            // Last update time
            let update_text = if self.show_time_since_update {
                self.get_time_since_update()
                    .map(|time| format!(" • Updated {}", time))
                    .unwrap_or_default()
            } else {
                String::new()
            };
            
            let combined_text = format!("{}{}{}", status_text, time_text, update_text);
            
            text(combined_text)
                .size(14)
                .horizontal_alignment(alignment::Horizontal::Center)
                .width(Length::Fill)
                .into()
        } else {
            text("Not Connected")
                .size(14)
                .horizontal_alignment(alignment::Horizontal::Center)
                .width(Length::Fill)
                .into()
        }
    }
    
    /// Create a static view with the given battery status
    pub fn create_static_view(battery_status: AirPodsBatteryStatus) -> Element<'static, Message, iced::Renderer<Theme>> {
        let display = Self::new(Some(battery_status));
        
        let mut content = column![]
            .spacing(15)
            .width(Length::Fill);
            
        // Add title
        content = content.push(
            text("Battery Status")
                .size(24)
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Center)
        );
        
        if let Some(status) = &display.battery_status {
            // Get previous levels for animation
            let (prev_left, prev_right, prev_case) = display.previous_levels
                .unwrap_or((None, None, None));
            
            // Add battery bars
            content = content.push(display.create_battery_bar(
                "Left",
                status.battery.left,
                prev_left,
                status.battery.charging.as_ref().is_some_and(|c| c.is_left_charging()),
            ));
            
            content = content.push(display.create_battery_bar(
                "Right",
                status.battery.right,
                prev_right,
                status.battery.charging.as_ref().is_some_and(|c| c.is_right_charging()),
            ));
            
            content = content.push(display.create_battery_bar(
                "Case",
                status.battery.case,
                prev_case,
                status.battery.charging.as_ref().is_some_and(|c| c.is_case_charging()),
            ));
            
            // Add status summary
            let summary = display.create_status_summary();
            content = content.push(summary);
        } else {
            // No battery status available
            content = content.push(
                container(
                    text("No battery information available")
                        .size(16)
                        .horizontal_alignment(alignment::Horizontal::Center)
                        .width(Length::Fill)
                )
                .padding(20)
                .width(Length::Fill)
            );
        }
        
        container(content)
            .padding(20)
            .width(Length::Fill)
            .into()
    }
    
    /// Create an empty view when no battery status is available
    pub fn create_empty_view() -> Element<'static, Message, iced::Renderer<Theme>> {
        container(
            column![
                text("Battery Status")
                    .size(24)
                    .width(Length::Fill)
                    .horizontal_alignment(alignment::Horizontal::Center),
                container(
                    text("No battery information available")
                        .size(16)
                        .horizontal_alignment(alignment::Horizontal::Center)
                        .width(Length::Fill)
                )
                .padding(20)
                .width(Length::Fill)
            ]
            .spacing(15)
            .width(Length::Fill)
        )
        .padding(20)
        .width(Length::Fill)
        .into()
    }
}

impl UiComponent for RealTimeBatteryDisplay {
    fn view(&self) -> Element<'static, Message, iced::Renderer<Theme>> {
        let mut content = column![]
            .spacing(15)
            .width(Length::Fill);
            
        // Add title
        content = content.push(
            text("Battery Status")
                .size(24)
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Center)
        );
        
        if let Some(status) = &self.battery_status {
            // Get previous levels for animation
            let (prev_left, prev_right, prev_case) = self.previous_levels
                .unwrap_or((None, None, None));
            
            // Add battery bars
            content = content.push(self.create_battery_bar(
                "Left",
                status.battery.left,
                prev_left,
                status.battery.charging.as_ref().is_some_and(|c| c.is_left_charging()),
            ));
            
            content = content.push(self.create_battery_bar(
                "Right",
                status.battery.right,
                prev_right,
                status.battery.charging.as_ref().is_some_and(|c| c.is_right_charging()),
            ));
            
            content = content.push(self.create_battery_bar(
                "Case",
                status.battery.case,
                prev_case,
                status.battery.charging.as_ref().is_some_and(|c| c.is_case_charging()),
            ));
            
            // Add status summary
            content = content.push(self.create_status_summary());
        } else {
            // No battery status available
            content = content.push(
                container(
                    text("No battery information available")
                        .size(16)
                        .horizontal_alignment(alignment::Horizontal::Center)
                        .width(Length::Fill)
                )
                .padding(20)
                .width(Length::Fill)
            );
        }
        
        if self.compact_view {
            container(content)
                .padding(10)
                .width(Length::Fill)
                .into()
        } else {
            container(content)
                .padding(20)
                .width(Length::Fill)
                .into()
        }
    }
} 