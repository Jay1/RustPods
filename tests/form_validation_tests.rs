//! Tests for form validation
//! These tests verify the functionality of form validation components

use crate::test_helpers::{TestForm, test_validator};

/// Test form validation basic functionality
#[test]
fn test_form_validation_basic() {
    // Create a test form
    let mut form = TestForm::new();
    
    // A form with no values should be valid
    assert!(form.is_valid());
    
    // Set a required field with a valid value
    form.set_field("required_field", "value");
    assert!(form.is_valid());
    
    // Set a required field with an invalid value
    form.set_field("required_field", "");
    assert!(!form.is_valid());
    assert!(form.errors.contains_key("required_field"));
    
    // Set it back to valid
    form.set_field("required_field", "value");
    assert!(form.is_valid());
}

/// Test number field validation
#[test]
fn test_form_validation_number_field() {
    // Create a test form
    let mut form = TestForm::new();
    
    // Set a number field with a valid value
    form.set_field("number_field", "50");
    assert!(form.is_valid());
    
    // Set a number field with a non-numeric value
    form.set_field("number_field", "not a number");
    assert!(!form.is_valid());
    assert!(form.errors.contains_key("number_field"));
    
    // Set a number field with a out-of-range value
    form.set_field("number_field", "150");
    assert!(!form.is_valid());
    assert!(form.errors.contains_key("number_field"));
    
    // Set it back to valid
    form.set_field("number_field", "75");
    assert!(form.is_valid());
}

/// Test validator function
#[test]
fn test_validator_function() {
    // Test the validator with a valid input
    let result = test_validator("valid");
    assert!(result.is_ok());
    
    // Test the validator with an invalid input
    let result = test_validator("");
    assert!(result.is_err());
    
    // Check error message
    match result {
        Ok(_) => panic!("Expected an error, but got Ok"),
        Err(msg) => assert_eq!(msg, "Field cannot be empty"),
    }
} 