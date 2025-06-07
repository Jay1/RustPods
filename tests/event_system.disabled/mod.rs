//! Event System test modules
//!
//! This module contains tests related to the event system functionality, including
//! event brokers, event filtering, and event handling.
//!
//! The modules test different aspects of the event system:
//! - core: Basic event system functionality and event filtering
//! - broker: Main event broker implementation
//! - simple_broker: Simplified event broker for basic use cases
//! - enhanced_broker: Advanced event broker with additional features

// Core event system tests
pub mod core;

// Event broker implementations tests
pub mod broker;
pub mod simple_broker;
pub mod enhanced_broker; 