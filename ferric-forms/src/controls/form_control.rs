//! FormControl - single form input with value and validation.

use crate::state::{ControlStatus, FormState};
use crate::validators::{ValidationErrors, ValidationResult, Validator};
use super::AbstractControl;
use ferric_core::reactive::{Signal, signal};
use std::any::Any;
use std::cell::RefCell;

/// A single form control with value, validation, and state tracking.
pub struct FormControl<T: Clone + 'static> {
    /// The control's current value.
    value: Signal<T>,
    /// The initial value for reset.
    initial_value: T,
    /// Validators for this control.
    validators: Vec<Box<dyn Validator<T>>>,
    /// Current validation errors.
    errors: RefCell<Option<ValidationErrors>>,
    /// Form state (touched, dirty, etc.).
    state: FormState,
}

impl<T: Clone + 'static> FormControl<T> {
    /// Create a new form control with an initial value.
    pub fn new(initial_value: T) -> Self {
        Self {
            value: signal(initial_value.clone()),
            initial_value,
            validators: Vec::new(),
            errors: RefCell::new(None),
            state: FormState::new(),
        }
    }

    /// Create a form control with validators.
    pub fn with_validators(initial_value: T, validators: Vec<Box<dyn Validator<T>>>) -> Self {
        let control = Self {
            value: signal(initial_value.clone()),
            initial_value,
            validators,
            errors: RefCell::new(None),
            state: FormState::new(),
        };
        control.update_validity();
        control
    }

    /// Get the current value.
    #[inline]
    pub fn value(&self) -> T {
        self.value.get()
    }

    /// Set the value.
    pub fn set_value(&self, value: T) {
        self.value.set(value);
        self.state.mark_as_dirty();
        self.update_validity();
    }

    /// Patch the value (alias for set_value).
    pub fn patch_value(&self, value: T) {
        self.set_value(value);
    }

    /// Get the value signal for reactive binding.
    pub fn value_signal(&self) -> &Signal<T> {
        &self.value
    }

    /// Add a validator.
    pub fn add_validator(&mut self, validator: Box<dyn Validator<T>>) {
        self.validators.push(validator);
        self.update_validity();
    }

    /// Set validators, replacing existing ones.
    pub fn set_validators(&mut self, validators: Vec<Box<dyn Validator<T>>>) {
        self.validators = validators;
        self.update_validity();
    }

    /// Clear all validators.
    pub fn clear_validators(&mut self) {
        self.validators.clear();
        *self.errors.borrow_mut() = None;
        self.state.set_status(ControlStatus::Valid);
    }

    /// Run validation manually.
    pub fn validate(&self) -> ValidationResult {
        let value = self.value.get();
        let mut all_errors = ValidationErrors::new();

        for validator in &self.validators {
            if let Err(errors) = validator.validate(&value) {
                all_errors.extend(errors);
            }
        }

        if all_errors.is_empty() {
            Ok(())
        } else {
            Err(all_errors)
        }
    }
}

impl<T: Clone + Default + 'static> FormControl<T> {
    /// Create a form control with default value.
    pub fn default_value() -> Self {
        Self::new(T::default())
    }
}

impl<T: Clone + 'static> AbstractControl for FormControl<T> {
    fn errors(&self) -> Option<ValidationErrors> {
        self.errors.borrow().clone()
    }

    fn state(&self) -> &FormState {
        &self.state
    }

    fn reset(&self) {
        self.value.set(self.initial_value.clone());
        self.state.reset();
        self.update_validity();
    }

    fn update_validity(&self) {
        if self.state.disabled() {
            return;
        }

        match self.validate() {
            Ok(()) => {
                *self.errors.borrow_mut() = None;
                self.state.set_status(ControlStatus::Valid);
            }
            Err(errors) => {
                *self.errors.borrow_mut() = Some(errors);
                self.state.set_status(ControlStatus::Invalid);
            }
        }
    }

    fn value_as_any(&self) -> &dyn Any {
        self
    }
}

// Convenience constructors for common types

impl FormControl<String> {
    /// Create a string form control.
    pub fn text(initial: impl Into<String>) -> Self {
        Self::new(initial.into())
    }

    /// Create a required string form control.
    pub fn text_required(initial: impl Into<String>) -> Self {
        use crate::validators::required;
        Self::with_validators(initial.into(), vec![Box::new(required::<String>())])
    }
}

impl FormControl<i32> {
    /// Create an integer form control.
    pub fn number(initial: i32) -> Self {
        Self::new(initial)
    }
}

impl FormControl<f64> {
    /// Create a float form control.
    pub fn decimal(initial: f64) -> Self {
        Self::new(initial)
    }
}

impl FormControl<bool> {
    /// Create a boolean form control.
    pub fn checkbox(initial: bool) -> Self {
        Self::new(initial)
    }
}

impl<T: Clone + 'static> Clone for FormControl<T> {
    fn clone(&self) -> Self {
        Self {
            value: signal(self.value.get()),
            initial_value: self.initial_value.clone(),
            validators: Vec::new(), // Validators not cloned
            errors: RefCell::new(self.errors.borrow().clone()),
            state: self.state.clone(),
        }
    }
}

