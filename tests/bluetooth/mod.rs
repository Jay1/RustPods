//! Bluetooth module tests
//!
//! This module contains tests for Bluetooth functionality including adapters,
//! scanning, device detection, and filtering.

// Core module functionality  
pub mod common_utils;           // Common utilities for Bluetooth tests
pub mod bluetooth_tests;        // Basic Bluetooth functionality tests
pub mod mock_tests;             // Bluetooth mocking tests

// Adapter and scanning
pub mod adapter;                // Tests for Bluetooth adapter management  
pub mod scanner_tests;          // Scanner-specific tests
pub mod scanning;               // Tests for scanning lifecycle and management

// Device detection and filtering
pub mod detection;              // Tests for device detection
pub mod filtering;              // Tests for device filtering operations

// Battery functionality
pub mod battery_status;         // Tests for basic battery status functionality

// Configuration
pub mod config;                 // Tests for scan configuration

// Temporarily disabled due to performance issues:
// pub mod battery_monitoring;  // battery_monitoring.rs.disabled - Tests for battery monitoring and updates 