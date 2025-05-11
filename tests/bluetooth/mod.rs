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
pub mod adapter;             // Tests for Bluetooth adapter management
pub mod battery_status;      // Tests for basic battery status functionality
pub mod battery_monitoring;  // Tests for battery monitoring and updates
pub mod config;              // Tests for scan configuration
pub mod detection;           // Tests for device detection
pub mod filtering;           // Tests for device filtering
pub mod scanning;            // Tests for device scanning

//! Bluetooth module tests
//!
//! Tests for Bluetooth scanning and connection functionality.

pub mod scanner_tests; 