//! Tests for form validation
//! These tests verify the functionality of form validation components

use rustpods::ui::form_validation::ValidationError;

// Create a test module locally instead of importing it
mod test_helpers {
    use rustpods::ui::form_validation::{Result, ValidationError, ValidationRule};
    use std::collections::HashMap;

    pub struct TestForm {
        pub values: HashMap<String, String>,
        pub errors: HashMap<String, String>,
        validators: HashMap<String, ValidationRule>,
    }

    impl TestForm {
        pub fn new() -> Self {
            let mut form = Self {
                values: HashMap::new(),
                errors: HashMap::new(),
                validators: HashMap::new(),
            };

            // Add some default validators
            form.validators.insert(
                "required_field".to_string(),
                ValidationRule::new("required_field")
                    .required()
                    .validator(|val| {
                        if val.is_empty() {
                            Err(ValidationError::Custom("Field cannot be empty".to_string()))
                        } else {
                            Ok(())
                        }
                    }),
            );

            form.validators.insert(
                "number_field".to_string(),
                ValidationRule::new("number_field").validator(|val| match val.parse::<i32>() {
                    Ok(num) if num >= 0 && num <= 100 => Ok(()),
                    Ok(_) => Err(ValidationError::Custom("Number out of range".to_string())),
                    Err(_) => Err(ValidationError::Custom("Not a valid number".to_string())),
                }),
            );

            form
        }

        pub fn set_field(&mut self, name: &str, value: &str) {
            self.values.insert(name.to_string(), value.to_string());
            self.validate_field(name);
        }

        fn validate_field(&mut self, name: &str) {
            if let Some(rule) = self.validators.get(name) {
                if let Some(validator) = &rule.validator {
                    if let Some(value) = self.values.get(name) {
                        match validator(value) {
                            Ok(_) => {
                                self.errors.remove(name);
                            }
                            Err(err) => {
                                self.errors.insert(name.to_string(), err.to_string());
                            }
                        }
                    }
                }
            }
        }

        pub fn is_valid(&self) -> bool {
            self.errors.is_empty()
        }
    }

    pub fn test_validator(value: &str) -> Result<()> {
        if value.is_empty() {
            Err(ValidationError::Custom("Field cannot be empty".to_string()))
        } else {
            Ok(())
        }
    }
}

use test_helpers::{test_validator, TestForm};

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
        Err(err) => {
            if let ValidationError::Custom(msg) = err {
                assert_eq!(msg, "Field cannot be empty");
            } else {
                panic!("Expected Custom error, got {:?}", err);
            }
        }
    }
}
