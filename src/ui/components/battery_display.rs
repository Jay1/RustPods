use iced::widget::{column, container, progress_bar, row, text};use iced::Length;

use crate::ui::{Message, UiComponent};

/// Component for displaying battery levels
#[derive(Debug, Clone)]
pub struct BatteryDisplay {
    /// Left earbud battery level (0-100)
    left_level: Option<u8>,
    /// Right earbud battery level (0-100)
    right_level: Option<u8>,
    /// Case battery level (0-100)
    case_level: Option<u8>,
}

impl BatteryDisplay {
    /// Create a new battery display
    pub fn new(left_level: Option<u8>, right_level: Option<u8>, case_level: Option<u8>) -> Self {
        Self {
            left_level: left_level.map(|l| l.min(100)),
            right_level: right_level.map(|r| r.min(100)),
            case_level: case_level.map(|c| c.min(100)),
        }
    }
    
    /// Create an empty battery display
    pub fn empty() -> Self {
        Self {
            left_level: None,
            right_level: None,
            case_level: None,
        }
    }
}

impl UiComponent for BatteryDisplay {
    fn view(&self) -> iced::Element<'static, Message, iced::Renderer<crate::ui::theme::Theme>> {
        let mut content = column![]
            .spacing(20)
            .padding(20)
            .width(Length::Fill);
        
        // Title
        content = content.push(
            text("Battery Levels")
                .size(24)
                .width(Length::Fill),
        );
        
        // Left earbud
        content = content.push(
            row![
                text("Left").width(Length::FillPortion(1)),
                container(create_battery_indicator(self.left_level)).width(Length::FillPortion(4))
            ]
            .spacing(10)
            .width(Length::Fill),
        );
        
        // Right earbud
        content = content.push(
            row![
                text("Right").width(Length::FillPortion(1)),
                container(create_battery_indicator(self.right_level)).width(Length::FillPortion(4))
            ]
            .spacing(10)
            .width(Length::Fill),
        );
        
        // Case
        content = content.push(
            row![
                text("Case").width(Length::FillPortion(1)),
                container(create_battery_indicator(self.case_level)).width(Length::FillPortion(4))
            ]
            .spacing(10)
            .width(Length::Fill),
        );
        
        container(content).width(Length::Fill).into()
    }
}

/// Helper function to create a battery indicator
fn create_battery_indicator(level: Option<u8>) -> iced::Element<'static, Message, iced::Renderer<crate::ui::theme::Theme>> {
    match level {
        Some(level) => {
            let level_f32 = level as f32 / 100.0;
            
            row![
                progress_bar(0.0..=1.0, level_f32).width(Length::Fill),
                text(format!("{}%", level)).width(Length::Shrink),
            ]
            .spacing(10)
            .width(Length::Fill)
            .into()
        }
        None => {
            text("Not connected").into()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_battery_display_creation() {
        // Create with known values
        let display = BatteryDisplay::new(Some(75), Some(80), Some(90));
        
        // Verify fields
        assert_eq!(display.left_level, Some(75));
        assert_eq!(display.right_level, Some(80));
        assert_eq!(display.case_level, Some(90));
        
        // Test value capping at 100
        let display = BatteryDisplay::new(Some(120), Some(80), Some(150));
        assert_eq!(display.left_level, Some(100));
        assert_eq!(display.right_level, Some(80));
        assert_eq!(display.case_level, Some(100));
        
        // Test empty creation
        let display = BatteryDisplay::empty();
        assert_eq!(display.left_level, None);
        assert_eq!(display.right_level, None);
        assert_eq!(display.case_level, None);
    }
} 