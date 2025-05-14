//! UI test modules
//!
//! This module contains tests related to the user interface functionality, including
//! components, rendering, state management, and message handling.
//!
//! The modules test different aspects of the UI:
//! - components: Individual UI components and their behavior
//! - device_display: Device-specific UI elements and their rendering
//! - integration: Integration between UI and application state
//! - messages: Message passing and handling within the UI
//! - rendering: Rendering of UI components
//! - state: Application state management

// UI component tests
pub mod components;
pub mod device_display;

// UI integration and functionality tests
pub mod integration;
pub mod messages;
pub mod rendering;
pub mod state;

// UI module tests
//
// Tests for user interface components and theme functionality.

pub mod theme_tests;
pub mod state_tests;
pub mod battery_display_tests;
pub mod user_interaction_tests;

// Re-export all test modules
pub use state::*;
pub use messages::*;
pub use rendering::*;
pub use components::*;
pub use battery_display_tests::*;
pub use user_interaction_tests::*; 