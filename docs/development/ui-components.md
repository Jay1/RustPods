# UI Component Patterns

This document outlines the UI component patterns used in RustPods, including best practices for working with Iced, component composition, and handling reference lifetimes.

## Core Architecture

RustPods uses the [Iced](https://github.com/iced-rs/iced) framework for its user interface. Iced follows an Elm-like architecture with:

- **State**: A central struct (`AppState`) that holds all application data
- **Messages**: Enums that represent events that can update the state
- **Views**: Functions that convert state into UI elements

## Component Structure

### UiComponent Trait

All UI components in RustPods implement the `UiComponent` trait, which provides a consistent interface:

```rust
pub trait UiComponent {
    fn view(&self) -> Element<'_, Message, iced::Renderer<Theme>>;
}
```

This trait ensures all components have a `view` method that returns an Iced `Element`.

### Component Composition

Components are composed hierarchically:

1. `AppState` renders the main application view
2. The main view delegates to either `MainWindow` or `SettingsWindow`
3. These windows further delegate to smaller components like `DeviceList`, `BatteryDisplay`, etc.

## Reference and Ownership Management

When working with Iced components, careful attention must be paid to lifetimes and ownership. A common issue is the "cannot return value referencing temporary value" error, which occurs when a component returns a reference to a local variable.

### The Problem

```rust
// ❌ Problematic pattern that causes "cannot return value referencing temporary value"
fn view(&self) -> Element<'_, Message, iced::Renderer<Theme>> {
    // Create a temporary component
    let status = ConnectionStatus::new(
        self.is_connected
    );
    
    // Problem: This returns a reference to the temporary 'status' variable
    status.view()
}
```

### The Solution: Wrapper Component Pattern

To solve this issue, RustPods uses a wrapper component pattern:

1. Create a wrapper component that owns its data
2. Implement a `render()` method that returns an owned `Element<'static, ...>`
3. Implement `From<Component>` for `Element` to enable direct use in macros

```rust
// ✅ Correct pattern using a wrapper component
pub struct ConnectionStatusWrapper {
    pub is_connected: bool,
    pub animation_progress: f32,
}

impl ConnectionStatusWrapper {
    // Create an owned Element without borrowing self
    pub fn render(&self) -> Element<'static, Message, iced::Renderer<Theme>> {
        // Implementation that creates an owned Element
    }
}

impl UiComponent for ConnectionStatusWrapper {
    fn view(&self) -> Element<'_, Message, iced::Renderer<Theme>> {
        self.render()
    }
}

// Allow direct use in column! and row! macros
impl<'a> From<ConnectionStatusWrapper> for Element<'a, Message, iced::Renderer<Theme>> {
    fn from(wrapper: ConnectionStatusWrapper) -> Self {
        wrapper.render()
    }
}
```

### Helper Method Pattern

For complex components, break the UI into smaller helper methods:

```rust
impl MainWindow {
    fn create_header(&self) -> Element<'_, Message, iced::Renderer<Theme>> {
        // Header implementation
    }
    
    fn view_content(&self) -> Element<'_, Message, iced::Renderer<Theme>> {
        // Content implementation
    }
    
    fn render_device_list(&self) -> Element<'_, Message, iced::Renderer<Theme>> {
        // Device list implementation
    }
}
```

## Best Practices

- **Avoid Temporary Value References**: Never return references to local variables in `view()` methods
- **Use Owned Data**: Create components that own their data rather than borrowing
- **Implement From trait**: Allow components to be directly used in macros
- **Break Down Complex Views**: Use helper methods to decompose complex UI rendering
- **Component Composition**: Prefer composing smaller components over complex rendering logic

## Example Components

RustPods includes several components that demonstrate these patterns:

- `MainWindow`: Uses helper methods and component composition
- `ConnectionStatusWrapper`: Demonstrates the wrapper component pattern
- `DeviceList`: Shows composing elements from a collection
- `BatteryDisplay`: Illustrates rendering dynamic content

## Testing UI Components

When testing UI components:

1. Focus on testing the component logic, not just the rendering
2. Avoid complex chaining for test implementations
3. Use simplistic views in test code
4. Ensure components can be rendered without borrowing issues

For more detailed information on specific components, see the code documentation in the `src/ui/components` directory. 