//! Context menu component for the application
//!
//! This component provides a popup menu for contextual actions
//! in response to right-clicks or other triggers.

use iced::widget::{button, column, container, row, text};
use iced::{alignment, Element, Length, Padding, Point, Rectangle};

use crate::ui::theme::Theme;
use crate::ui::{Message, UiComponent};

/// A context menu item with text and optional shortcut
#[derive(Debug, Clone)]
pub struct ContextMenuItem {
    /// The text label for the menu item
    pub text: String,
    /// Optional keyboard shortcut text
    pub shortcut: Option<String>,
    /// Whether the item is disabled
    pub disabled: bool,
    /// Whether the item is a separator
    pub is_separator: bool,
    /// Identifier for the item
    pub id: String,
}

impl ContextMenuItem {
    /// Create a new context menu item
    pub fn new<S: Into<String>, T: Into<String>>(id: S, text: T) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            shortcut: None,
            disabled: false,
            is_separator: false,
        }
    }

    /// Add a keyboard shortcut to the menu item
    pub fn with_shortcut<S: Into<String>>(mut self, shortcut: S) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Mark the item as disabled
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    /// Create a separator item
    pub fn separator<S: Into<String>>(id: S) -> Self {
        Self {
            id: id.into(),
            text: String::new(),
            shortcut: None,
            disabled: false,
            is_separator: true,
        }
    }
}

/// Context menu component
#[derive(Debug, Clone)]
pub struct ContextMenu {
    /// Items in the menu
    items: Vec<ContextMenuItem>,
    /// Position of the menu
    position: Point,
    /// Whether the menu is visible
    visible: bool,
    /// Whether to show right-aligned shortcuts
    show_shortcuts: bool,
    /// Width of the menu
    width: f32,
}

impl Default for ContextMenu {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            position: Point::new(0.0, 0.0),
            visible: false,
            show_shortcuts: true,
            width: 200.0,
        }
    }
}

impl ContextMenu {
    /// Create a new empty context menu
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an item to the menu
    pub fn add_item(&mut self, item: ContextMenuItem) {
        self.items.push(item);
    }

    /// Set the menu's position
    pub fn set_position(&mut self, position: Point) {
        self.position = position;
    }

    /// Show the menu
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide the menu
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Toggle the menu's visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Check if the menu is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Create a context menu with standard items for the application
    pub fn standard() -> Self {
        let mut menu = Self::new();

        menu.add_item(ContextMenuItem::new("refresh", "Refresh").with_shortcut("Ctrl+R"));
        menu.add_item(ContextMenuItem::new("settings", "Settings").with_shortcut("Ctrl+,"));
        menu.add_item(ContextMenuItem::separator("sep1"));
        menu.add_item(ContextMenuItem::new("connect", "Connect"));
        menu.add_item(ContextMenuItem::new("disconnect", "Disconnect"));
        menu.add_item(ContextMenuItem::separator("sep2"));
        menu.add_item(ContextMenuItem::new("exit", "Exit").with_shortcut("Ctrl+Q"));

        menu
    }

    /// Create a menu item view
    fn view_item(&self, item: &ContextMenuItem) -> Element<'_, Message, iced::Renderer<Theme>> {
        if item.is_separator {
            // Render a separator line
            return container(iced::widget::Rule::horizontal(1))
                .width(Length::Fill)
                .padding(Padding::from([4, 0]))
                .into();
        }

        // Create a button with the item text
        let text_element = text(&item.text)
            .size(14)
            .width(Length::Fill)
            .height(Length::Fill)
            .vertical_alignment(alignment::Vertical::Center);

        let mut row_content = row![text_element].spacing(10).padding(5);

        // Add shortcut if present and enabled
        if let Some(shortcut) = &item.shortcut {
            if self.show_shortcuts {
                let shortcut_element = text(shortcut)
                    .size(14)
                    .width(Length::Shrink)
                    .vertical_alignment(alignment::Vertical::Center);

                row_content = row_content.push(shortcut_element);
            }
        }

        // Create the button
        let btn = button(row_content)
            .width(Length::Fill)
            .padding(4)
            .style(iced::theme::Button::Secondary);

        btn.into()
    }

    /// Check if a point is inside the menu
    pub fn contains_point(&self, _bounds: &Rectangle, point: Point) -> bool {
        if !self.visible {
            return false;
        }

        let menu_bounds = Rectangle {
            x: self.position.x,
            y: self.position.y,
            width: self.width,
            // Estimate height based on number of items
            height: (self.items.len() as f32) * 30.0,
        };

        menu_bounds.contains(point)
    }
}

impl UiComponent for ContextMenu {
    fn view(&self) -> Element<'_, Message, iced::Renderer<Theme>> {
        if !self.visible {
            // Return an empty element when not visible
            return iced::widget::Space::new(Length::Fill, Length::Fill).into();
        }

        let mut content = column![].spacing(0).width(Length::Fill);

        for item in &self.items {
            content = content.push(self.view_item(item));
        }

        // Create the menu container
        let menu = container(content)
            .width(Length::Fixed(self.width))
            .padding(2)
            .style(iced::theme::Container::Box);

        // Position the menu as an overlay
        container(menu)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Left)
            .align_y(alignment::Vertical::Top)
            .style(iced::theme::Container::Box)
            .into()
    }
}
