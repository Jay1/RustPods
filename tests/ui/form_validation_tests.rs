#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::form_validation::{ValidationRule, ValidationError, FormValidator};

    #[test]
    fn test_validation_rule_creation() {
        let rule = ValidationRule::new("username")
            .required()
            .min_length(3)
            .max_length(20)
            .pattern(r"^[a-zA-Z0-9_]+$");
        assert_eq!(rule.field, "username");
        assert!(rule.required);
        assert_eq!(rule.min_length, Some(3));
        assert_eq!(rule.max_length, Some(20));
        assert!(rule.pattern.is_some());
    }

    #[test]
    fn test_validation_rule_validation() {
        let rule = ValidationRule::new("username")
            .required()
            .min_length(3);
        // Test required field
        assert!(rule.validate("").is_err());
        // Test min length
        assert!(rule.validate("ab").is_err());
        assert!(rule.validate("abc").is_ok());
        // Test with custom validator
        let rule = ValidationRule::new("password")
            .validator(|value| {
                if value.chars().any(|c| c.is_ascii_digit()) {
                    Ok(())
                } else {
                    Err(ValidationError::Custom("Password must contain at least one digit".to_string()))
                }
            });
        assert!(rule.validate("password").is_err());
        assert!(rule.validate("password123").is_ok());
    }

    #[test]
    fn test_form_validator() {
        let mut validator = FormValidator::new();
        // Add rules
        validator.add_rule(ValidationRule::new("username").required().min_length(3));
        validator.add_rule(ValidationRule::new("password").required().min_length(8));
        // Test initial state
        assert!(!validator.has_errors());
        // Test with valid data
        validator.set_field("username", "john");
        validator.set_field("password", "password123");
        assert!(!validator.has_errors());
        assert!(validator.validate_all());
        // Test with invalid data
        validator.set_field("username", "jo");
        assert!(validator.has_errors());
        assert!(validator.has_error("username"));
        assert!(!validator.has_error("password"));
        assert!(!validator.validate_all());
    }
} 