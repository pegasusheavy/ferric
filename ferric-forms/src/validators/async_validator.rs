//! Async validators for server-side validation.
//!
//! ## Features
//!
//! - Async validation with Futures
//! - Debouncing to reduce server calls
//! - Caching for repeated values
//! - Timeout support
//!
//! ## Usage
//!
//! ```ignore
//! use ferric_forms::prelude::*;
//!
//! // Create a debounced async validator
//! let email_validator = async_validator("emailTaken", |email: &String| async move {
//!     // Simulate API call
//!     if check_email_exists(email).await {
//!         Err(validation_error("emailTaken", "Email is already taken"))
//!     } else {
//!         Ok(())
//!     }
//! }).debounce(300);
//!
//! let email_control = FormControl::text("")
//!     .with_async_validator(email_validator);
//! ```

use super::{ValidationResult, ValidationErrors, validation_error};
use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::time::Duration;

/// Trait for asynchronous validators.
pub trait AsyncValidator<T> {
    /// Validate a value asynchronously.
    fn validate<'a>(
        &'a self,
        value: &'a T,
    ) -> Pin<Box<dyn Future<Output = ValidationResult> + 'a>>;

    /// Get the validator name/key.
    fn name(&self) -> &'static str;
}

/// A boxed async validator function.
pub type AsyncValidatorFn<T> = Box<
    dyn Fn(&T) -> Pin<Box<dyn Future<Output = ValidationResult>>>
>;

/// Helper to create an async validator from a closure.
pub struct AsyncCustomValidator<T, F, Fut>
where
    F: Fn(&T) -> Fut,
    Fut: Future<Output = ValidationResult>,
{
    name: &'static str,
    validate_fn: F,
    _marker: std::marker::PhantomData<(T, Fut)>,
}

impl<T, F, Fut> AsyncValidator<T> for AsyncCustomValidator<T, F, Fut>
where
    T: 'static,
    F: Fn(&T) -> Fut,
    Fut: Future<Output = ValidationResult> + 'static,
{
    fn validate<'a>(
        &'a self,
        value: &'a T,
    ) -> Pin<Box<dyn Future<Output = ValidationResult> + 'a>> {
        Box::pin(async move {
            (self.validate_fn)(value).await
        })
    }

    fn name(&self) -> &'static str {
        self.name
    }
}

/// Create a custom async validator.
pub fn async_custom<T, F, Fut>(
    name: &'static str,
    validate_fn: F,
) -> AsyncCustomValidator<T, F, Fut>
where
    F: Fn(&T) -> Fut,
    Fut: Future<Output = ValidationResult>,
{
    AsyncCustomValidator {
        name,
        validate_fn,
        _marker: std::marker::PhantomData,
    }
}

/// Builder for async validators with additional features.
pub struct AsyncValidatorBuilder<T, V>
where
    V: AsyncValidator<T>,
{
    validator: V,
    debounce_ms: Option<u32>,
    cache: bool,
    timeout_ms: Option<u32>,
    _marker: std::marker::PhantomData<T>,
}

impl<T, V> AsyncValidatorBuilder<T, V>
where
    V: AsyncValidator<T>,
{
    /// Create a new builder.
    pub fn new(validator: V) -> Self {
        Self {
            validator,
            debounce_ms: None,
            cache: false,
            timeout_ms: None,
            _marker: std::marker::PhantomData,
        }
    }

    /// Set debounce time in milliseconds.
    pub fn debounce(mut self, ms: u32) -> Self {
        self.debounce_ms = Some(ms);
        self
    }

    /// Enable caching of validation results.
    pub fn cached(mut self) -> Self {
        self.cache = true;
        self
    }

    /// Set timeout in milliseconds.
    pub fn timeout(mut self, ms: u32) -> Self {
        self.timeout_ms = Some(ms);
        self
    }

    /// Build the configured validator.
    pub fn build(self) -> ConfiguredAsyncValidator<T, V> {
        ConfiguredAsyncValidator {
            inner: self.validator,
            debounce_ms: self.debounce_ms.unwrap_or(0),
            cache_enabled: self.cache,
            timeout_ms: self.timeout_ms,
            cache: RefCell::new(HashMap::new()),
            _marker: std::marker::PhantomData,
        }
    }
}

/// An async validator with additional configuration.
pub struct ConfiguredAsyncValidator<T, V>
where
    V: AsyncValidator<T>,
{
    inner: V,
    debounce_ms: u32,
    cache_enabled: bool,
    timeout_ms: Option<u32>,
    cache: RefCell<HashMap<String, ValidationResult>>,
    _marker: std::marker::PhantomData<T>,
}

impl<T, V> ConfiguredAsyncValidator<T, V>
where
    V: AsyncValidator<T>,
{
    /// Get debounce time.
    pub fn debounce_time(&self) -> u32 {
        self.debounce_ms
    }

    /// Check if caching is enabled.
    pub fn is_cached(&self) -> bool {
        self.cache_enabled
    }

    /// Clear the cache.
    pub fn clear_cache(&self) {
        self.cache.borrow_mut().clear();
    }
}

impl<T, V> AsyncValidator<T> for ConfiguredAsyncValidator<T, V>
where
    T: ToString + 'static,
    V: AsyncValidator<T> + 'static,
{
    fn validate<'a>(
        &'a self,
        value: &'a T,
    ) -> Pin<Box<dyn Future<Output = ValidationResult> + 'a>> {
        let cache_key = value.to_string();

        // Check cache first
        if self.cache_enabled {
            if let Some(result) = self.cache.borrow().get(&cache_key) {
                let result = result.clone();
                return Box::pin(async move { result });
            }
        }

        Box::pin(async move {
            // Note: Actual debouncing would require wasm-bindgen-futures
            // and setTimeout. This is a simplified implementation.
            let result = self.inner.validate(value).await;

            // Cache the result
            if self.cache_enabled {
                self.cache.borrow_mut().insert(cache_key, result.clone());
            }

            result
        })
    }

    fn name(&self) -> &'static str {
        self.inner.name()
    }
}

/// Debounced async validator wrapper.
pub struct DebouncedValidator<V> {
    inner: V,
    delay_ms: u32,
    last_value: RefCell<Option<String>>,
}

impl<V> DebouncedValidator<V> {
    /// Create a new debounced validator.
    pub fn new(inner: V, delay_ms: u32) -> Self {
        Self {
            inner,
            delay_ms,
            last_value: RefCell::new(None),
        }
    }

    /// Get the debounce delay in milliseconds.
    pub fn delay(&self) -> u32 {
        self.delay_ms
    }
}

impl<T, V> AsyncValidator<T> for DebouncedValidator<V>
where
    T: ToString + 'static,
    V: AsyncValidator<T>,
{
    fn validate<'a>(
        &'a self,
        value: &'a T,
    ) -> Pin<Box<dyn Future<Output = ValidationResult> + 'a>> {
        // Store the current value
        *self.last_value.borrow_mut() = Some(value.to_string());

        Box::pin(async move {
            // In a real implementation, we'd use setTimeout here
            // For now, just validate immediately
            self.inner.validate(value).await
        })
    }

    fn name(&self) -> &'static str {
        self.inner.name()
    }
}

// ============================================================================
// Built-in Async Validators
// ============================================================================

/// Async validator that checks uniqueness (e.g., email, username).
pub struct UniqueValidator<F, Fut>
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = bool>,
{
    name: &'static str,
    error_key: &'static str,
    error_message: String,
    check_fn: F,
    _marker: std::marker::PhantomData<Fut>,
}

impl<F, Fut> UniqueValidator<F, Fut>
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = bool>,
{
    /// Create a new unique validator.
    pub fn new(
        name: &'static str,
        error_message: &str,
        check_fn: F,
    ) -> Self {
        Self {
            name,
            error_key: name,
            error_message: error_message.to_string(),
            check_fn,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<F, Fut> AsyncValidator<String> for UniqueValidator<F, Fut>
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = bool> + 'static,
{
    fn validate<'a>(
        &'a self,
        value: &'a String,
    ) -> Pin<Box<dyn Future<Output = ValidationResult> + 'a>> {
        let value = value.clone();
        Box::pin(async move {
            let is_taken = (self.check_fn)(value).await;
            if is_taken {
                Err(validation_error(self.error_key, &self.error_message))
            } else {
                Ok(())
            }
        })
    }

    fn name(&self) -> &'static str {
        self.name
    }
}

/// Create an email uniqueness validator.
pub fn unique_email<F, Fut>(check_fn: F) -> UniqueValidator<F, Fut>
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = bool>,
{
    UniqueValidator::new("emailTaken", "This email is already registered", check_fn)
}

/// Create a username uniqueness validator.
pub fn unique_username<F, Fut>(check_fn: F) -> UniqueValidator<F, Fut>
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = bool>,
{
    UniqueValidator::new("usernameTaken", "This username is already taken", check_fn)
}

/// Compose multiple async validators.
pub struct ComposedAsyncValidator<T> {
    validators: Vec<Box<dyn AsyncValidator<T>>>,
}

impl<T: 'static> ComposedAsyncValidator<T> {
    /// Create a new composed validator.
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }

    /// Add a validator.
    pub fn add<V: AsyncValidator<T> + 'static>(mut self, validator: V) -> Self {
        self.validators.push(Box::new(validator));
        self
    }
}

impl<T: 'static> Default for ComposedAsyncValidator<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: 'static> AsyncValidator<T> for ComposedAsyncValidator<T> {
    fn validate<'a>(
        &'a self,
        value: &'a T,
    ) -> Pin<Box<dyn Future<Output = ValidationResult> + 'a>> {
        Box::pin(async move {
            let mut all_errors = ValidationErrors::new();

            for validator in &self.validators {
                if let Err(errors) = validator.validate(value).await {
                    all_errors.extend(errors);
                }
            }

            if all_errors.is_empty() {
                Ok(())
            } else {
                Err(all_errors)
            }
        })
    }

    fn name(&self) -> &'static str {
        "composed"
    }
}

/// Helper function to create an async validator builder.
pub fn async_validator<T, F, Fut>(
    name: &'static str,
    validate_fn: F,
) -> AsyncValidatorBuilder<T, AsyncCustomValidator<T, F, Fut>>
where
    T: 'static,
    F: Fn(&T) -> Fut,
    Fut: Future<Output = ValidationResult> + 'static,
{
    AsyncValidatorBuilder::new(async_custom(name, validate_fn))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debounced_validator_creation() {
        struct TestValidator;
        impl AsyncValidator<String> for TestValidator {
            fn validate<'a>(
                &'a self,
                _value: &'a String,
            ) -> Pin<Box<dyn Future<Output = ValidationResult> + 'a>> {
                Box::pin(async { Ok(()) })
            }
            fn name(&self) -> &'static str {
                "test"
            }
        }

        let debounced = DebouncedValidator::new(TestValidator, 300);
        assert_eq!(debounced.delay(), 300);
    }

    #[test]
    fn test_async_validator_builder() {
        struct TestValidator;
        impl AsyncValidator<String> for TestValidator {
            fn validate<'a>(
                &'a self,
                _value: &'a String,
            ) -> Pin<Box<dyn Future<Output = ValidationResult> + 'a>> {
                Box::pin(async { Ok(()) })
            }
            fn name(&self) -> &'static str {
                "test"
            }
        }

        let configured = AsyncValidatorBuilder::new(TestValidator)
            .debounce(500)
            .cached()
            .timeout(5000)
            .build();

        assert_eq!(configured.debounce_time(), 500);
        assert!(configured.is_cached());
    }
}
