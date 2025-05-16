use iced::widget::{text, container, row, progress_bar, Column, Row};
use iced::{Element, Length, Color, alignment};

use crate::ui::{Message, UiComponent};
use crate::airpods::AirPodsBattery;
use crate::ui::theme::Theme;

/// Component for displaying enhanced battery levels with charging indicators
#[derive(Debug, Clone)]
pub struct EnhancedBatteryDisplay {
    /// The battery information to display
    battery: Option<AirPodsBattery>,
}

impl EnhancedBatteryDisplay {
    /// Create a new enhanced battery display with battery information
    pub fn new(battery: Option<AirPodsBattery>) -> Self {
        Self { battery }
    }
    
    /// Create an empty enhanced battery display
    pub fn empty() -> Self {
        Self { battery: None }
    }
    
    /// Create a static view with the given battery information
    pub fn create_static_view(battery: AirPodsBattery) -> Element<'static, Message, iced::Renderer<Theme>> {
        // Clone the battery before moving it to display
        let battery_clone = battery.clone();
        let display = Self::new(Some(battery));
        
        // Create a title for the section
        let title = text("Battery Status")
            .size(22)
            .width(Length::Fill)
            .horizontal_alignment(alignment::Horizontal::Center);
            
        // Create rows for each component
        let left_row = display.create_battery_row("Left", battery_clone.left, battery_clone.charging.as_ref().is_some_and(|c| c.is_left_charging()));
        let right_row = display.create_battery_row("Right", battery_clone.right, battery_clone.charging.as_ref().is_some_and(|c| c.is_right_charging()));
        let case_row = display.create_battery_row("Case", battery_clone.case, battery_clone.charging.as_ref().is_some_and(|c| c.is_case_charging()));
        
        // Create a status row
        let status = text(display.get_battery_status())
            .size(16)
            .width(Length::Fill)
            .horizontal_alignment(alignment::Horizontal::Center);
            
        // Combine rows into a column
        let content = Column::new()
            .push(title)
            .push(left_row)
            .push(right_row)
            .push(case_row)
            .push(status)
            .spacing(10)
            .padding(10)
            .width(Length::Fill);
            
        container(content)
            .width(Length::Fill)
            .into()
    }
}

impl UiComponent for EnhancedBatteryDisplay {
    fn view(&self) -> Element<'static, Message, iced::Renderer<Theme>> {
        if let Some(battery) = &self.battery {
            // Create a title for the section
            let title = text("Battery Status")
                .size(22)
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Center);
                
            // Create rows for each component
            let left_row = self.create_battery_row("Left", battery.left, battery.charging.as_ref().is_some_and(|c| c.is_left_charging()));
            let right_row = self.create_battery_row("Right", battery.right, battery.charging.as_ref().is_some_and(|c| c.is_right_charging()));
            let case_row = self.create_battery_row("Case", battery.case, battery.charging.as_ref().is_some_and(|c| c.is_case_charging()));
            
            // Create a status row
            let status = text(self.get_battery_status())
                .size(16)
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Center);
                
            // Combine rows into a column
            let content = Column::new()
                .push(title)
                .push(left_row)
                .push(right_row)
                .push(case_row)
                .push(status)
                .spacing(10)
                .padding(10)
                .width(Length::Fill);
                
            container(content)
                .width(Length::Fill)
                .into()
        } else {
            // No battery information available
            container(
                text("Battery information not available")
                    .size(16)
                    .width(Length::Fill)
                    .horizontal_alignment(alignment::Horizontal::Center)
            )
            .width(Length::Fill)
            .into()
        }
    }
}

impl EnhancedBatteryDisplay {
    /// Create a battery row with label, progress bar, and charging indicator
    fn create_battery_row(
        &self,
        label: &str,
        level: Option<u8>,
        is_charging: bool,
    ) -> Row<'static, Message, iced::Renderer<Theme>> {
        // Create label
        let label_text = text(label)
            .size(16)
            .width(Length::Fixed(50.0));
            
        // Create progress bar or status
        let battery_display: Element<'static, Message, iced::Renderer<Theme>> = if let Some(level) = level {
            // Create progress bar with appropriate color
            let pb: iced::widget::ProgressBar<iced::Renderer<Theme>> = progress_bar(0.0..=100.0, level as f32)
                .height(20.0)
                .width(Length::Fill);
                
            // Convert progress bar to element
            pb.into()
        } else {
            // No battery level available
            text("N/A")
                .size(16)
                .horizontal_alignment(alignment::Horizontal::Center)
                .width(Length::Fill)
                .into()
        };
        
        // Create percentage text
        let percentage_text = if let Some(level) = level {
            format!("{}%", level)
        } else {
            "".to_string()
        };
        
        let percentage = text(percentage_text)
            .size(16)
            .width(Length::Fixed(50.0))
            .horizontal_alignment(alignment::Horizontal::Right);
            
        // Create charging indicator
        let charging_icon = if is_charging {
            text("⚡")
                .size(16)
                .width(Length::Fixed(20.0))
        } else {
            text("")
                .size(16)
                .width(Length::Fixed(20.0))
        };
        
        // Combine elements into a row
        row![
            label_text,
            battery_display,
            percentage,
            charging_icon,
        ]
        .spacing(10)
        .align_items(alignment::Alignment::Center)
        .width(Length::Fill)
    }
    
    /// Get the battery status (charging, low battery, etc.)
    fn get_battery_status(&self) -> &'static str {
        if let Some(battery) = &self.battery {
            // Check if any component is charging
            let is_charging = battery.charging.as_ref().is_some_and(|c| c.is_any_charging());
            
            if is_charging {
                return "Charging";
            }
            
            // Check if any component has low battery
            let has_low_battery = battery.case.is_some_and(|level| level < 20) ||
                                 battery.left.is_some_and(|level| level < 20) ||
                                 battery.right.is_some_and(|level| level < 20);
            
            if has_low_battery {
                return "Low Battery";
            }
        }
        
        // Default status
        ""
    }
}

/// Define text color for status text
#[allow(dead_code)]
fn status_text_color(_level: &str) -> iced::theme::Text {
    match _level {
        "Charging" => iced::theme::Text::Color(Color::from_rgb(0.2, 0.6, 0.8)),
        "Low Battery" => iced::theme::Text::Color(Color::from_rgb(0.9, 0.3, 0.3)),
        _ => iced::theme::Text::Default,
    }
}

/// Helper function to check if any component has a low battery
#[allow(dead_code)]
fn battery_level_low(battery: &AirPodsBattery) -> bool {
    let is_left_low = battery.left.is_some_and(|level| level <= 20);
    let is_right_low = battery.right.is_some_and(|level| level <= 20);
    let is_case_low = battery.case.is_some_and(|level| level <= 20);
    
    is_left_low || is_right_low || is_case_low
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::airpods::AirPodsChargingState;
    
    #[test]
    fn test_enhanced_battery_display_creation() {
        // Create with battery info
        let battery = AirPodsBattery {
            left: Some(80),
            right: Some(75),
            case: Some(90),
            charging: Some(AirPodsChargingState::CaseCharging),
        };
        
        let display = EnhancedBatteryDisplay::new(Some(battery.clone()));
        assert!(display.battery.is_some());
        
        if let Some(b) = display.battery {
            assert_eq!(b.left, Some(80));
            assert_eq!(b.right, Some(75));
            assert_eq!(b.case, Some(90));
            assert!(b.charging.is_some());
            assert_eq!(*b.charging.as_ref().unwrap(), AirPodsChargingState::CaseCharging);
        }
        
        // Test empty creation
        let empty_display = EnhancedBatteryDisplay::empty();
        assert!(empty_display.battery.is_none());
    }
    
    #[test]
    fn test_battery_level_low_detection() {
        // Test with low battery
        let low_battery = AirPodsBattery {
            left: Some(15),
            right: Some(50),
            case: Some(75),
            charging: Some(AirPodsChargingState::NotCharging),
        };
        
        assert!(battery_level_low(&low_battery));
        
        // Test with no low battery
        let good_battery = AirPodsBattery {
            left: Some(60),
            right: Some(50),
            case: Some(75),
            charging: Some(AirPodsChargingState::NotCharging),
        };
        
        assert!(!battery_level_low(&good_battery));
    }
} 