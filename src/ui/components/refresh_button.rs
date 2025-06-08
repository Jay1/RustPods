use iced::{
    alignment, theme,
    widget::{button, container, text},
    Element, Length,
};

/// A button for refreshing data that can display a loading spinner
#[derive(Debug, Clone)]
pub struct RefreshButton {
    /// Whether the button is currently showing a loading state
    is_loading: bool,
    /// Animation progress for the refresh icon (0.0 - 1.0)
    animation_progress: f32,
    /// Optional text label
    label: Option<String>,
}

impl RefreshButton {
    /// Create a new refresh button
    pub fn new() -> Self {
        Self {
            is_loading: false,
            animation_progress: 0.0,
            label: None,
        }
    }

    /// Set whether the button is in a loading state
    pub fn loading(mut self, is_loading: bool) -> Self {
        self.is_loading = is_loading;
        self
    }

    /// Set the animation progress (0.0 - 1.0)
    pub fn with_animation_progress(mut self, progress: f32) -> Self {
        self.animation_progress = progress;
        self
    }

    /// Add a text label to the button
    pub fn with_text<S: Into<String>>(mut self, text: S) -> Self {
        self.label = Some(text.into());
        self
    }

    /// Create a view of this component
    pub fn view<'a, Message>(&self, on_press: Message) -> Element<'a, Message>
    where
        Message: 'a + Clone,
    {
        // Create a simple button with text
        let btn_text = if let Some(label) = &self.label {
            if self.is_loading {
                format!("⟳ {}", label)
            } else {
                format!("↻ {}", label)
            }
        } else if self.is_loading {
            "⟳".to_string()
        } else {
            "↻".to_string()
        };

        let btn = button(
            container(text(btn_text).width(Length::Shrink).height(Length::Shrink))
                .width(Length::Shrink)
                .align_x(alignment::Horizontal::Center)
                .padding(8),
        )
        .style(if self.is_loading {
            theme::Button::Secondary
        } else {
            theme::Button::Primary
        });

        // Add on press handler if not loading
        if self.is_loading {
            btn.into()
        } else {
            btn.on_press(on_press).into()
        }
    }
}

impl Default for RefreshButton {
    fn default() -> Self {
        Self::new()
    }
}
