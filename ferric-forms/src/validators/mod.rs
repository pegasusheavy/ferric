//! Form validators.
//!
//! Provides built-in validators and support for custom validation functions.

mod builtin;
mod async_validator;
mod cross_field;

pub use builtin::*;
pub use async_validator::*;
pub use cross_field::*;

use std::collections::HashMap;
use std::fmt;

/// Validation errors returned by validators.
pub type ValidationErrors = HashMap<String, ValidationError>;

/// Result of a validation check.
pub type ValidationResult = Result<(), ValidationErrors>;

/// A single validation error.
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// The error message.
    pub message: String,
    /// Additional context/parameters.
    pub params: HashMap<String, String>,
}

impl ValidationError {
    /// Create a new validation error.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            params: HashMap::new(),
        }
    }

    /// Create a validation error with parameters.
    pub fn with_params(message: impl Into<String>, params: HashMap<String, String>) -> Self {
        Self {
            message: message.into(),
            params,
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Trait for synchronous validators.
pub trait Validator<T> {
    /// Validate a value and return errors if invalid.
    fn validate(&self, value: &T) -> ValidationResult;

    /// Get the validator name/key.
    fn name(&self) -> &'static str;
}

/// A boxed validator function.
pub type ValidatorFn<T> = Box<dyn Fn(&T) -> ValidationResult>;

/// Compose multiple validators into one.
pub fn compose<T>(validators: Vec<Box<dyn Validator<T>>>) -> impl Validator<T>
where
    T: 'static,
{
    ComposedValidator { validators }
}

struct ComposedValidator<T> {
    validators: Vec<Box<dyn Validator<T>>>,
}

impl<T> Validator<T> for ComposedValidator<T>
where
    T: 'static,
{
    fn validate(&self, value: &T) -> ValidationResult {
        let mut all_errors = HashMap::new();

        for validator in &self.validators {
            if let Err(errors) = validator.validate(value) {
                all_errors.extend(errors);
            }
        }

        if all_errors.is_empty() {
            Ok(())
        } else {
            Err(all_errors)
        }
    }

    fn name(&self) -> &'static str {
        "composed"
    }
}

/// Helper to create validation errors.
pub fn validation_error(key: &str, message: &str) -> ValidationErrors {
    let mut errors = HashMap::new();
    errors.insert(key.to_string(), ValidationError::new(message));
    errors
}

/// Helper to create validation errors with params.
pub fn validation_error_with_params(
    key: &str,
    message: &str,
    params: HashMap<String, String>,
) -> ValidationErrors {
    let mut errors = HashMap::new();
    errors.insert(key.to_string(), ValidationError::with_params(message, params));
    errors
}



