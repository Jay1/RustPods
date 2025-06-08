//! Form validation for RustPods user interface
//!
//! This module provides validation utilities for handling user input in forms.

use std::collections::HashMap;
use std::fmt;
use thiserror::Error;

use crate::error::{ErrorContext, RustPodsError};

/// Form validation error
#[derive(Debug, Error)]
pub enum ValidationError {
    /// Field is required
    #[error("Field '{0}' is required")]
    Required(String),

    /// Field is too short
    #[error("Field '{0}' must be at least {1} characters")]
    TooShort(String, usize),

    /// Field is too long
    #[error("Field '{0}' must be at most {1} characters")]
    TooLong(String, usize),

    /// Field doesn't match pattern
    #[error("Field '{0}' has invalid format: {1}")]
    InvalidFormat(String, String),

    /// Custom validation error
    #[error("{0}")]
    Custom(String),
}

impl From<ValidationError> for RustPodsError {
    fn from(err: ValidationError) -> Self {
        RustPodsError::Validation(err.to_string())
    }
}

/// Validation result type
pub type Result<T> = std::result::Result<T, ValidationError>;

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
    pub validator: Option<Box<dyn Fn(&str) -> Result<()> + Send + Sync>>,
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
        F: Fn(&str) -> Result<()> + Send + Sync + 'static,
    {
        self.validator = Some(Box::new(validator));
        self
    }

    /// Validate a field value against this rule
    pub fn validate(&self, value: &str) -> Result<()> {
        // Create error context for logging
        let _ctx = ErrorContext::new("FormValidation", "validate")
            .with_metadata("field", self.field.clone())
            .with_metadata("value_length", value.len().to_string());

        // Check if the field is required
        if self.required && value.trim().is_empty() {
            log::debug!("Required field '{}' is empty", self.field);
            return Err(ValidationError::Required(self.field.clone()));
        }

        // Check minimum length
        if let Some(min_length) = self.min_length {
            if value.len() < min_length {
                log::debug!(
                    "Field '{}' is too short ({}), minimum {}",
                    self.field,
                    value.len(),
                    min_length
                );
                return Err(ValidationError::TooShort(self.field.clone(), min_length));
            }
        }

        // Check maximum length
        if let Some(max_length) = self.max_length {
            if value.len() > max_length {
                log::debug!(
                    "Field '{}' is too long ({}), maximum {}",
                    self.field,
                    value.len(),
                    max_length
                );
                return Err(ValidationError::TooLong(self.field.clone(), max_length));
            }
        }

        // Check pattern
        if let Some(pattern) = &self.pattern {
            let valid = match pattern.as_str() {
                "email" => value.contains('@') && value.contains('.'),
                "number" => value
                    .chars()
                    .all(|c| c.is_ascii_digit() || c == '-' || c == '.'),
                "integer" => value.chars().all(|c| c.is_ascii_digit() || c == '-'),
                "alphanumeric" => value
                    .chars()
                    .all(|c| c.is_alphanumeric() || c.is_whitespace()),
                "url" => value.starts_with("http://") || value.starts_with("https://"),
                "phone" => value.chars().all(|c| {
                    c.is_ascii_digit() || c == '+' || c == '-' || c == '(' || c == ')' || c == ' '
                }),
                _ => {
                    if let Ok(regex) = regex::Regex::new(pattern) {
                        regex.is_match(value)
                    } else {
                        log::warn!("Invalid regex pattern: {}", pattern);
                        true
                    }
                }
            };

            if !valid {
                log::debug!(
                    "Field '{}' has invalid format, should match {}",
                    self.field,
                    pattern
                );
                return Err(ValidationError::InvalidFormat(
                    self.field.clone(),
                    format!("Must match format: {}", pattern),
                ));
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

impl Default for FormValidator {
    fn default() -> Self {
        Self::new()
    }
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
        let field_name = field.clone().into();
        self.values.insert(field_name.clone(), value.into());
        self.validate_field(&field_name);
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
        // Find all rules for this field
        let rules = self
            .rules
            .iter()
            .filter(|r| r.field == field)
            .collect::<Vec<_>>();

        // If no rules, field is valid
        if rules.is_empty() {
            self.errors.remove(field);
            return true;
        }

        // Get the field value
        let value = match self.values.get(field) {
            Some(v) => v,
            None => {
                // Field is not set, check if it's required
                if rules.iter().any(|r| r.required) {
                    self.errors
                        .insert(field.to_string(), format!("Field {} is required", field));
                    return false;
                }
                return true;
            }
        };

        // Apply rules
        for rule in rules {
            match rule.validate(value) {
                Ok(_) => {}
                Err(err) => {
                    self.errors.insert(field.to_string(), err.to_string());
                    log::debug!("Validation failed for field {}: {}", field, err);
                    return false;
                }
            }
        }

        // Field is valid
        self.errors.remove(field);
        true
    }

    /// Validate all fields
    pub fn validate_all(&mut self) -> bool {
        let mut valid = true;

        // Get all unique fields from rules
        let fields = self
            .rules
            .iter()
            .map(|r| r.field.clone())
            .collect::<std::collections::HashSet<_>>();

        // Validate each field
        for field in fields {
            if !self.validate_field(&field) {
                valid = false;
            }
        }

        valid
    }

    /// Get a map of all fields and their validation status
    pub fn get_validation_status(&self) -> HashMap<String, ValidationResult> {
        self.values
            .keys()
            .map(|field| {
                let error = self.errors.get(field).cloned();
                (
                    field.clone(),
                    ValidationResult {
                        valid: error.is_none(),
                        error_message: error,
                    },
                )
            })
            .collect()
    }
}

/// Validation result for a field
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the validation was successful
    pub valid: bool,
    /// Optional error message for validation failures
    pub error_message: Option<String>,
}

/// Helper struct for field validation
pub struct FieldValidator {
    /// Error message to display when validation fails
    pub error_message: String,
    /// Optional validator function
    #[allow(clippy::type_complexity)]
    pub validator: Option<Box<dyn Fn(&str) -> Result<()> + Send + Sync>>,
}

impl FieldValidator {
    /// Create a required field validator
    pub fn required(error_message: &str) -> Self {
        let error_message = error_message.to_string(); // Clone the string to avoid lifetime issues
        Self {
            error_message: error_message.clone(),
            validator: Some(Box::new(move |value: &str| {
                if value.trim().is_empty() {
                    Err(ValidationError::Custom(error_message.clone()))
                } else {
                    Ok(())
                }
            })),
        }
    }

    /// Create a number range validator
    pub fn number_range(min: i32, max: i32, error_message: &str) -> Self {
        let error_message = error_message.to_string(); // Clone the string to avoid lifetime issues
        Self {
            error_message: error_message.clone(),
            validator: Some(Box::new(move |value: &str| match value.parse::<i32>() {
                Ok(num) if num >= min && num <= max => Ok(()),
                Ok(_) => Err(ValidationError::Custom(error_message.clone())),
                Err(_) => Err(ValidationError::Custom(format!(
                    "Invalid number format: {}",
                    value
                ))),
            })),
        }
    }

    /// Chain multiple validators into one
    pub fn chain(validators: Vec<Self>) -> Self {
        if validators.is_empty() {
            return Self {
                error_message: String::new(),
                validator: None,
            };
        }

        let first_error_message = validators[0].error_message.clone();

        Self {
            error_message: first_error_message,
            validator: Some(Box::new(move |value: &str| {
                for validator in &validators {
                    if let Some(ref validator_fn) = validator.validator {
                        validator_fn(value)?;
                    }
                }
                Ok(())
            })),
        }
    }

    /// Validate a value against this validator
    pub fn validate(&self, value: &str) -> ValidationResult {
        if let Some(ref validator) = self.validator {
            if let Err(e) = validator(value) {
                ValidationResult {
                    valid: false,
                    error_message: Some(e.to_string()),
                }
            } else {
                ValidationResult {
                    valid: true,
                    error_message: None,
                }
            }
        } else {
            ValidationResult {
                valid: true,
                error_message: None,
            }
        }
    }
}

impl ValidationResult {
    /// Check if the validation was successful
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    /// Get the error message if validation failed
    pub fn error(&self) -> Option<&str> {
        self.error_message.as_deref()
    }
}

impl fmt::Debug for FieldValidator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FieldValidator")
            .field("error_message", &self.error_message)
            .field("validator", &format_args!("<function>"))
            .finish()
    }
}

impl Clone for FieldValidator {
    fn clone(&self) -> Self {
        Self {
            error_message: self.error_message.clone(),
            validator: None, // Cannot clone the function, so we set it to None in the clone
        }
    }
}
