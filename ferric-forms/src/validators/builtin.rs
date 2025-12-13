//! Built-in validators.

use super::{validation_error, validation_error_with_params, ValidationResult, Validator};
use std::collections::HashMap;

// ============================================================================
// Required Validators
// ============================================================================

/// Validator that requires a non-empty value.
pub struct Required;

impl Validator<String> for Required {
    fn validate(&self, value: &String) -> ValidationResult {
        if value.trim().is_empty() {
            Err(validation_error("required", "This field is required"))
        } else {
            Ok(())
        }
    }

    fn name(&self) -> &'static str {
        "required"
    }
}

impl<T> Validator<Option<T>> for Required {
    fn validate(&self, value: &Option<T>) -> ValidationResult {
        if value.is_none() {
            Err(validation_error("required", "This field is required"))
        } else {
            Ok(())
        }
    }

    fn name(&self) -> &'static str {
        "required"
    }
}

/// Create a required validator.
pub fn required<T>() -> Required {
    Required
}

/// Validator that requires a true boolean value.
pub struct RequiredTrue;

impl Validator<bool> for RequiredTrue {
    fn validate(&self, value: &bool) -> ValidationResult {
        if *value {
            Ok(())
        } else {
            Err(validation_error("requiredTrue", "This field must be checked"))
        }
    }

    fn name(&self) -> &'static str {
        "requiredTrue"
    }
}

/// Create a required true validator.
pub fn required_true() -> RequiredTrue {
    RequiredTrue
}

// ============================================================================
// Length Validators
// ============================================================================

/// Validator for minimum string length.
pub struct MinLength {
    min: usize,
}

impl Validator<String> for MinLength {
    fn validate(&self, value: &String) -> ValidationResult {
        if value.len() >= self.min {
            Ok(())
        } else {
            let mut params = HashMap::new();
            params.insert("requiredLength".to_string(), self.min.to_string());
            params.insert("actualLength".to_string(), value.len().to_string());
            Err(validation_error_with_params(
                "minLength",
                &format!("Minimum length is {}", self.min),
                params,
            ))
        }
    }

    fn name(&self) -> &'static str {
        "minLength"
    }
}

/// Create a minimum length validator.
pub fn min_length(min: usize) -> MinLength {
    MinLength { min }
}

/// Validator for maximum string length.
pub struct MaxLength {
    max: usize,
}

impl Validator<String> for MaxLength {
    fn validate(&self, value: &String) -> ValidationResult {
        if value.len() <= self.max {
            Ok(())
        } else {
            let mut params = HashMap::new();
            params.insert("requiredLength".to_string(), self.max.to_string());
            params.insert("actualLength".to_string(), value.len().to_string());
            Err(validation_error_with_params(
                "maxLength",
                &format!("Maximum length is {}", self.max),
                params,
            ))
        }
    }

    fn name(&self) -> &'static str {
        "maxLength"
    }
}

/// Create a maximum length validator.
pub fn max_length(max: usize) -> MaxLength {
    MaxLength { max }
}

// ============================================================================
// Numeric Validators
// ============================================================================

/// Validator for minimum numeric value.
pub struct Min<T> {
    min: T,
}

impl<T: PartialOrd + std::fmt::Display + Copy> Validator<T> for Min<T> {
    fn validate(&self, value: &T) -> ValidationResult {
        if *value >= self.min {
            Ok(())
        } else {
            let mut params = HashMap::new();
            params.insert("min".to_string(), self.min.to_string());
            params.insert("actual".to_string(), value.to_string());
            Err(validation_error_with_params(
                "min",
                &format!("Value must be at least {}", self.min),
                params,
            ))
        }
    }

    fn name(&self) -> &'static str {
        "min"
    }
}

/// Create a minimum value validator.
pub fn min<T>(min: T) -> Min<T> {
    Min { min }
}

/// Validator for maximum numeric value.
pub struct Max<T> {
    max: T,
}

impl<T: PartialOrd + std::fmt::Display + Copy> Validator<T> for Max<T> {
    fn validate(&self, value: &T) -> ValidationResult {
        if *value <= self.max {
            Ok(())
        } else {
            let mut params = HashMap::new();
            params.insert("max".to_string(), self.max.to_string());
            params.insert("actual".to_string(), value.to_string());
            Err(validation_error_with_params(
                "max",
                &format!("Value must be at most {}", self.max),
                params,
            ))
        }
    }

    fn name(&self) -> &'static str {
        "max"
    }
}

/// Create a maximum value validator.
pub fn max<T>(max: T) -> Max<T> {
    Max { max }
}

// ============================================================================
// Pattern Validators
// ============================================================================

/// Validator for regex pattern matching.
pub struct Pattern {
    pattern: regex::Regex,
    pattern_str: String,
}

impl Validator<String> for Pattern {
    fn validate(&self, value: &String) -> ValidationResult {
        if value.is_empty() || self.pattern.is_match(value) {
            Ok(())
        } else {
            let mut params = HashMap::new();
            params.insert("pattern".to_string(), self.pattern_str.clone());
            Err(validation_error_with_params(
                "pattern",
                "Value does not match the required pattern",
                params,
            ))
        }
    }

    fn name(&self) -> &'static str {
        "pattern"
    }
}

/// Create a pattern validator.
pub fn pattern(pattern: &str) -> Pattern {
    Pattern {
        pattern: regex::Regex::new(pattern).expect("Invalid regex pattern"),
        pattern_str: pattern.to_string(),
    }
}

// ============================================================================
// Email Validator
// ============================================================================

/// Validator for email format.
pub struct Email {
    pattern: regex::Regex,
}

impl Default for Email {
    fn default() -> Self {
        Self {
            // RFC 5322 compliant email regex (simplified)
            pattern: regex::Regex::new(
                r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
            ).unwrap(),
        }
    }
}

impl Validator<String> for Email {
    fn validate(&self, value: &String) -> ValidationResult {
        if value.is_empty() || self.pattern.is_match(value) {
            Ok(())
        } else {
            Err(validation_error("email", "Please enter a valid email address"))
        }
    }

    fn name(&self) -> &'static str {
        "email"
    }
}

/// Create an email validator.
pub fn email() -> Email {
    Email::default()
}

// ============================================================================
// Custom Validator Helper
// ============================================================================

/// Create a custom validator from a closure.
pub struct CustomValidator<T, F>
where
    F: Fn(&T) -> ValidationResult,
{
    name: &'static str,
    validate_fn: F,
    _marker: std::marker::PhantomData<T>,
}

impl<T, F> Validator<T> for CustomValidator<T, F>
where
    F: Fn(&T) -> ValidationResult,
{
    fn validate(&self, value: &T) -> ValidationResult {
        (self.validate_fn)(value)
    }

    fn name(&self) -> &'static str {
        self.name
    }
}

/// Create a custom validator.
pub fn custom<T, F>(name: &'static str, validate_fn: F) -> CustomValidator<T, F>
where
    F: Fn(&T) -> ValidationResult,
{
    CustomValidator {
        name,
        validate_fn,
        _marker: std::marker::PhantomData,
    }
}

