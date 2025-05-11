//! Bluetooth test modules
//!
//! This module contains tests related to Bluetooth functionality, including
//! adapter management, device scanning, AirPods detection, etc.
//!
//! The common_utils module provides shared utilities for Bluetooth tests,
//! while the other modules test specific aspects of Bluetooth functionality.

// Common utilities for Bluetooth tests
pub mod common_utils;

// Specific test categories
pub mod adapter;
pub mod battery_status;
pub mod config;
pub mod detection;
pub mod filtering;
pub mod scanning; 