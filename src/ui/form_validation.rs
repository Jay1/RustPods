//! Form validation for RustPods user interface
//!
//! This module provides validation utilities for handling user input in forms.

use std::collections::HashMap;
use std::fmt;

use crate::ui::Message;

/// Validation rule for a field
pub struct ValidationRule {
    /// The field name to validate
    pub field: String,
    /// Whether the field is required
    pub required: bool,
    /// Minimum length requirement
    pub min_length: Option<usize>,
    /// Maximum length requirement
    pub max_length: Option<usize>,
    /// Regular expression pattern to match
    pub pattern: Option<String>,
    /// Custom validation function
    #[allow(clippy::type_complexity)]
    pub validator: Option<Box<dyn Fn(&str) -> Result<(), String> + Send + Sync>>,
}

impl ValidationRule {
    /// Create a new validation rule for a field
    pub fn new<S: Into<String>>(field: S) -> Self {
        Self {
            field: field.into(),
            required: false,
            min_length: None,
            max_length: None,
            pattern: None,
            validator: None,
        }
    }
    
    /// Mark the field as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
    
    /// Set a minimum length requirement
    pub fn min_length(mut self, length: usize) -> Self {
        self.min_length = Some(length);
        self
    }
    
    /// Set a maximum length requirement
    pub fn max_length(mut self, length: usize) -> Self {
        self.max_length = Some(length);
        self
    }
    
    /// Set a regular expression pattern for validation
    pub fn pattern<S: Into<String>>(mut self, pattern: S) -> Self {
        self.pattern = Some(pattern.into());
        self
    }
    
    /// Add a custom validation function
    pub fn validator<F>(mut self, validator: F) -> Self
    where
        F: Fn(&str) -> Result<(), String> + Send + Sync + 'static,
    {
        self.validator = Some(Box::new(validator));
        self
    }
    
    /// Validate a field value against this rule
    pub fn validate(&self, value: &str) -> Result<(), String> {
        // Check if the field is required
        if self.required && value.trim().is_empty() {
            return Err(format!("{} is required", self.field));
        }
        
        // Check minimum length
        if let Some(min_length) = self.min_length {
            if value.len() < min_length {
                return Err(format!("{} must be at least {} characters", self.field, min_length));
            }
        }
        
        // Check maximum length
        if let Some(max_length) = self.max_length {
            if value.len() > max_length {
                return Err(format!("{} must be at most {} characters", self.field, max_length));
            }
        }
        
        // Check pattern
        // Note: In a real implementation, you'd use a proper regex crate
        if let Some(pattern) = &self.pattern {
            if pattern == "email" && !value.contains('@') {
                return Err(format!("{} must be a valid email address", self.field));
            }
        }
        
        // Apply custom validator
        if let Some(validator) = &self.validator {
            return validator(value);
        }
        
        Ok(())
    }
}

// Manual implementation of Debug for ValidationRule
impl fmt::Debug for ValidationRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValidationRule")
            .field("field", &self.field)
            .field("required", &self.required)
            .field("min_length", &self.min_length)
            .field("max_length", &self.max_length)
            .field("pattern", &self.pattern)
            .field("validator", &format_args!("<function>"))
            .finish()
    }
}

// Manual implementation of Clone for ValidationRule
impl Clone for ValidationRule {
    fn clone(&self) -> Self {
        Self {
            field: self.field.clone(),
            required: self.required,
            min_length: self.min_length,
            max_length: self.max_length,
            pattern: self.pattern.clone(),
            validator: None, // Cannot clone the validator function
        }
    }
}

/// A form validator that can validate multiple fields
#[derive(Debug, Clone)]
pub struct FormValidator {
    /// The rules for each field
    rules: Vec<ValidationRule>,
    /// The current state of the form fields
    values: HashMap<String, String>,
    /// The current validation errors
    errors: HashMap<String, String>,
}

impl FormValidator {
    /// Create a new form validator
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            values: HashMap::new(),
            errors: HashMap::new(),
        }
    }
    
    /// Add a validation rule
    pub fn add_rule(&mut self, rule: ValidationRule) {
        self.rules.push(rule);
    }
    
    /// Set a field value
    pub fn set_field<S: Into<String> + Clone, T: Into<String>>(&mut self, field: S, value: T) {
        self.values.insert(field.clone().into(), value.into());
        self.validate_field(&field.into());
    }
    
    /// Get a field value
    pub fn get_field(&self, field: &str) -> Option<&String> {
        self.values.get(field)
    }
    
    /// Get a validation error for a field
    pub fn get_error(&self, field: &str) -> Option<&String> {
        self.errors.get(field)
    }
    
    /// Check if a field has an error
    pub fn has_error(&self, field: &str) -> bool {
        self.errors.contains_key(field)
    }
    
    /// Check if the form has any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    /// Get all validation errors
    pub fn get_errors(&self) -> &HashMap<String, String> {
        &self.errors
    }
    
    /// Validate a specific field
    pub fn validate_field(&mut self, field: &str) -> bool {
        let value = match self.values.get(field) {
            Some(value) => value,
            None => return false,
        };
        
        // Find the rule for this field
        let rule = self.rules.iter().find(|r| r.field == field);
        
        if let Some(rule) = rule {
            match rule.validate(value) {
                Ok(()) => {
                    // Remove any existing error
                    self.errors.remove(field);
                    true
                }
                Err(message) => {
                    // Add the error
                    self.errors.insert(field.to_string(), message);
                    false
                }
            }
        } else {
            // No rule for this field, so it's valid
            true
        }
    }
    
    /// Validate all fields
    pub fn validate_all(&mut self) -> bool {
        let mut valid = true;
        
        // Clear all errors first
        self.errors.clear();
        
        // Validate each field
        for rule in &self.rules {
            let field = &rule.field;
            let value = self.values.get(field).cloned().unwrap_or_default();
            
            if let Err(message) = rule.validate(&value) {
                self.errors.insert(field.clone(), message);
                valid = false;
            }
        }
        
        valid
    }
    
    /// Convert validation errors to UI messages
    pub fn to_messages(&self) -> Vec<Message> {
        self.errors
            .iter()
            .map(|(field, message)| {
                Message::FormValidationError {
                    field: field.clone(),
                    message: message.clone(),
                }
            })
            .collect()
    }
}

/// Represents a validation result for a form field
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the validation was successful
    pub valid: bool,
    /// Optional error message for validation failures
    pub error_message: Option<String>,
}

/// A field validator for form inputs
// Remove default derives and manually implement Debug and Clone later
pub struct FieldValidator {
    /// Error message to display when validation fails
    pub error_message: String,
    /// Optional validator function
    pub validator: Option<Box<dyn Fn(&str) -> Result<(), String> + Send + Sync>>,
}

// Manual implementation of Debug for FieldValidator
impl fmt::Debug for FieldValidator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FieldValidator")
            .field("error_message", &self.error_message)
            .field("validator", &format_args!("<function>"))
            .finish()
    }
}

// Manual implementation of Clone for FieldValidator
impl Clone for FieldValidator {
    fn clone(&self) -> Self {
        Self {
            error_message: self.error_message.clone(),
            validator: None, // Cannot clone the validator function
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validation_rule_creation() {
        let rule = ValidationRule::new("username")
            .required()
            .min_length(3)
            .max_length(20);
        
        assert_eq!(rule.field, "username");
        assert!(rule.required);
        assert_eq!(rule.min_length, Some(3));
        assert_eq!(rule.max_length, Some(20));
    }
    
    #[test]
    fn test_validation_rule_validation() {
        let rule = ValidationRule::new("username")
            .required()
            .min_length(3)
            .max_length(20);
        
        // Test required validation
        assert!(rule.validate("").is_err());
        
        // Test min length validation
        assert!(rule.validate("ab").is_err());
        
        // Test max length validation
        assert!(rule.validate("abcdefghijklmnopqrstuvwxyz").is_err());
        
        // Test valid input
        assert!(rule.validate("username").is_ok());
    }
    
    #[test]
    fn test_form_validator() {
        let mut validator = FormValidator::new();
        
        // Add rules
        validator.add_rule(
            ValidationRule::new("username")
                .required()
                .min_length(3)
                .max_length(20),
        );
        
        validator.add_rule(
            ValidationRule::new("email")
                .required()
                .pattern("email"),
        );
        
        // Set values
        validator.set_field("username", "ab");
        validator.set_field("email", "not-an-email");
        
        // Check errors
        assert!(validator.has_error("username"));
        assert!(validator.has_error("email"));
        assert!(validator.has_errors());
        
        // Fix the errors
        validator.set_field("username", "validusername");
        validator.set_field("email", "valid@example.com");
        
        // Check errors again
        assert!(!validator.has_error("username"));
        assert!(!validator.has_error("email"));
        assert!(!validator.has_errors());
    }
} 