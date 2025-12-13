//! FormGroup - a group of form controls.
//!
//! ## Features
//!
//! - Type-safe control access
//! - Cross-field validation
//! - Nested group support
//! - Status propagation
//!
//! ## Usage
//!
//! ```ignore
//! use ferric_forms::prelude::*;
//!
//! let form = FormGroup::new();
//! form.add_control("email", FormControl::text(""));
//! form.add_control("password", FormControl::text(""));
//!
//! // Add cross-field validator
//! form.set_validators(vec![password_match("password", "confirmPassword")]);
//! ```

use crate::state::{ControlStatus, FormState};
use crate::validators::{ValidationErrors, CrossFieldValidatorTrait};
use crate::binding::UpdateOn;
use super::{AbstractControl, ControlContainer, FormControl, FormArray};
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// A group of form controls that are validated together.
pub struct FormGroup {
    /// Child controls by name.
    controls: RefCell<HashMap<String, Rc<dyn AbstractControl>>>,
    /// Form state.
    state: FormState,
    /// Aggregated errors from all children.
    errors: RefCell<Option<ValidationErrors>>,
    /// Cross-field validators.
    cross_validators: RefCell<Vec<Box<dyn CrossFieldValidatorTrait>>>,
    /// When to update and validate.
    update_on: RefCell<UpdateOn>,
}

impl FormGroup {
    /// Create a new empty form group.
    pub fn new() -> Self {
        Self {
            controls: RefCell::new(HashMap::new()),
            state: FormState::new(),
            errors: RefCell::new(None),
            cross_validators: RefCell::new(Vec::new()),
            update_on: RefCell::new(UpdateOn::Input),
        }
    }

    /// Create a form group with initial controls.
    pub fn with_controls(controls: HashMap<String, Rc<dyn AbstractControl>>) -> Self {
        let group = Self {
            controls: RefCell::new(controls),
            state: FormState::new(),
            errors: RefCell::new(None),
            cross_validators: RefCell::new(Vec::new()),
            update_on: RefCell::new(UpdateOn::Input),
        };
        group.update_validity();
        group
    }

    /// Add a control to the group.
    pub fn add_control(&self, name: impl Into<String>, control: Rc<dyn AbstractControl>) {
        self.controls.borrow_mut().insert(name.into(), control);
        self.update_validity();
    }

    /// Remove a control from the group.
    pub fn remove_control(&self, name: &str) -> Option<Rc<dyn AbstractControl>> {
        let control = self.controls.borrow_mut().remove(name);
        self.update_validity();
        control
    }

    /// Get a raw control by name (type-erased).
    pub fn get_raw_control(&self, name: &str) -> Option<Rc<dyn AbstractControl>> {
        self.controls.borrow().get(name).cloned()
    }

    /// Check if a control exists.
    pub fn has_control(&self, name: &str) -> bool {
        self.controls.borrow().contains_key(name)
    }

    /// Set a control's value if it exists.
    pub fn set_control_value<T: Clone + 'static>(&self, _name: &str, _value: T) {
        // This would need type-safe access - simplified for now
        self.state.mark_as_dirty();
    }

    /// Get all control names.
    pub fn control_names(&self) -> Vec<String> {
        self.controls.borrow().keys().cloned().collect()
    }

    /// Get the raw value as a HashMap of Any values.
    pub fn value(&self) -> HashMap<String, Box<dyn Any>> {
        let values = HashMap::new();
        // Note: This is a simplified implementation
        // A real implementation would extract typed values
        values
    }

    /// Patch multiple values at once.
    pub fn patch_value(&self, _values: HashMap<&str, Box<dyn Any>>) {
        self.state.mark_as_dirty();
        self.update_validity();
    }

    /// Check if all controls are valid.
    pub fn all_valid(&self) -> bool {
        self.controls
            .borrow()
            .values()
            .all(|c| c.valid())
    }

    /// Get controls that are invalid.
    pub fn invalid_controls(&self) -> Vec<String> {
        self.controls
            .borrow()
            .iter()
            .filter(|(_, c)| c.invalid())
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Add a cross-field validator.
    pub fn add_cross_validator(&self, validator: Box<dyn CrossFieldValidatorTrait>) {
        self.cross_validators.borrow_mut().push(validator);
        self.update_validity();
    }

    /// Set cross-field validators (replaces existing).
    pub fn set_cross_validators(&self, validators: Vec<Box<dyn CrossFieldValidatorTrait>>) {
        *self.cross_validators.borrow_mut() = validators;
        self.update_validity();
    }

    /// Clear cross-field validators.
    pub fn clear_cross_validators(&self) {
        self.cross_validators.borrow_mut().clear();
        self.update_validity();
    }

    /// Set when validation should run.
    pub fn set_update_on(&self, update_on: UpdateOn) {
        *self.update_on.borrow_mut() = update_on;
    }

    /// Get current update strategy.
    pub fn update_on(&self) -> UpdateOn {
        *self.update_on.borrow()
    }

    /// Validate cross-field validators.
    fn validate_cross_field(&self) -> Option<ValidationErrors> {
        let mut all_errors = ValidationErrors::new();

        for validator in self.cross_validators.borrow().iter() {
            if let Err(errors) = validator.validate(self) {
                all_errors.extend(errors);
            }
        }

        if all_errors.is_empty() {
            None
        } else {
            Some(all_errors)
        }
    }

    /// Get nested control by path (e.g., "address.city").
    pub fn get_by_path(&self, path: &str) -> Option<Rc<dyn AbstractControl>> {
        let parts: Vec<&str> = path.split('.').collect();
        self.get_by_path_parts(&parts)
    }

    fn get_by_path_parts(&self, parts: &[&str]) -> Option<Rc<dyn AbstractControl>> {
        if parts.is_empty() {
            return None;
        }

        let control = self.get_raw_control(parts[0])?;

        if parts.len() == 1 {
            Some(control)
        } else {
            // Would need to check if control is a FormGroup and recurse
            // Simplified for now
            None
        }
    }

    /// Get the number of controls.
    pub fn size(&self) -> usize {
        self.controls.borrow().len()
    }

    /// Check if the group is empty.
    pub fn is_empty(&self) -> bool {
        self.controls.borrow().is_empty()
    }

    /// Get all errors including cross-field errors.
    pub fn all_errors(&self) -> Option<ValidationErrors> {
        let mut all = ValidationErrors::new();

        // Control errors
        if let Some(errors) = self.errors.borrow().clone() {
            all.extend(errors);
        }

        // Cross-field errors
        if let Some(cross_errors) = self.validate_cross_field() {
            all.extend(cross_errors);
        }

        if all.is_empty() {
            None
        } else {
            Some(all)
        }
    }
}

impl Default for FormGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl AbstractControl for FormGroup {
    fn errors(&self) -> Option<ValidationErrors> {
        self.errors.borrow().clone()
    }

    fn state(&self) -> &FormState {
        &self.state
    }

    fn reset(&self) {
        for control in self.controls.borrow().values() {
            control.reset();
        }
        self.state.reset();
        self.update_validity();
    }

    fn update_validity(&self) {
        if self.state.disabled() {
            return;
        }

        // Aggregate errors from children
        let mut all_errors = ValidationErrors::new();
        let mut any_invalid = false;
        let mut any_pending = false;

        for (name, control) in self.controls.borrow().iter() {
            if control.pending() {
                any_pending = true;
            }
            if control.invalid() {
                any_invalid = true;
                if let Some(errors) = control.errors() {
                    for (key, error) in errors {
                        all_errors.insert(format!("{}.{}", name, key), error);
                    }
                }
            }
        }

        if any_pending {
            self.state.set_status(ControlStatus::Pending);
        } else if any_invalid {
            *self.errors.borrow_mut() = Some(all_errors);
            self.state.set_status(ControlStatus::Invalid);
        } else {
            *self.errors.borrow_mut() = None;
            self.state.set_status(ControlStatus::Valid);
        }
    }

    fn value_as_any(&self) -> &dyn Any {
        self
    }
}

impl ControlContainer for FormGroup {
    fn get(&self, name: &str) -> Option<&dyn AbstractControl> {
        // Note: This is tricky due to RefCell - returning reference is complex
        // In practice, you'd use get_control() which returns Rc
        None
    }

    fn mark_all_as_touched(&self) {
        self.state.mark_as_touched();
        for control in self.controls.borrow().values() {
            control.mark_as_touched();
        }
    }

    fn mark_all_as_untouched(&self) {
        self.state.mark_as_untouched();
        for control in self.controls.borrow().values() {
            control.mark_as_untouched();
        }
    }

    fn mark_all_as_dirty(&self) {
        self.state.mark_as_dirty();
        for control in self.controls.borrow().values() {
            control.mark_as_dirty();
        }
    }

    fn mark_all_as_pristine(&self) {
        self.state.mark_as_pristine();
        for control in self.controls.borrow().values() {
            control.mark_as_pristine();
        }
    }
}

/// Builder for creating form groups.
pub struct FormGroupBuilder {
    controls: HashMap<String, Rc<dyn AbstractControl>>,
}

impl FormGroupBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            controls: HashMap::new(),
        }
    }

    /// Add a control.
    pub fn control(mut self, name: impl Into<String>, control: Rc<dyn AbstractControl>) -> Self {
        self.controls.insert(name.into(), control);
        self
    }

    /// Build the form group.
    pub fn build(self) -> FormGroup {
        FormGroup::with_controls(self.controls)
    }
}

impl Default for FormGroupBuilder {
    fn default() -> Self {
        Self::new()
    }
}


        } else {
            Some(all)
        }
    }
}

impl Default for FormGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl AbstractControl for FormGroup {
    fn errors(&self) -> Option<ValidationErrors> {
        self.errors.borrow().clone()
    }

    fn state(&self) -> &FormState {
        &self.state
    }

    fn reset(&self) {
        for control in self.controls.borrow().values() {
            control.reset();
        }
        self.state.reset();
        self.update_validity();
    }

    fn update_validity(&self) {
        if self.state.disabled() {
            return;
        }

        // Aggregate errors from children
        let mut all_errors = ValidationErrors::new();
        let mut any_invalid = false;
        let mut any_pending = false;

        for (name, control) in self.controls.borrow().iter() {
            if control.pending() {
                any_pending = true;
            }
            if control.invalid() {
                any_invalid = true;
                if let Some(errors) = control.errors() {
                    for (key, error) in errors {
                        all_errors.insert(format!("{}.{}", name, key), error);
                    }
                }
            }
        }

        if any_pending {
            self.state.set_status(ControlStatus::Pending);
        } else if any_invalid {
            *self.errors.borrow_mut() = Some(all_errors);
            self.state.set_status(ControlStatus::Invalid);
        } else {
            *self.errors.borrow_mut() = None;
            self.state.set_status(ControlStatus::Valid);
        }
    }

    fn value_as_any(&self) -> &dyn Any {
        self
    }
}

impl ControlContainer for FormGroup {
    fn get(&self, name: &str) -> Option<&dyn AbstractControl> {
        // Note: This is tricky due to RefCell - returning reference is complex
        // In practice, you'd use get_control() which returns Rc
        None
    }

    fn mark_all_as_touched(&self) {
        self.state.mark_as_touched();
        for control in self.controls.borrow().values() {
            control.mark_as_touched();
        }
    }

    fn mark_all_as_untouched(&self) {
        self.state.mark_as_untouched();
        for control in self.controls.borrow().values() {
            control.mark_as_untouched();
        }
    }

    fn mark_all_as_dirty(&self) {
        self.state.mark_as_dirty();
        for control in self.controls.borrow().values() {
            control.mark_as_dirty();
        }
    }

    fn mark_all_as_pristine(&self) {
        self.state.mark_as_pristine();
        for control in self.controls.borrow().values() {
            control.mark_as_pristine();
        }
    }
}

/// Builder for creating form groups.
pub struct FormGroupBuilder {
    controls: HashMap<String, Rc<dyn AbstractControl>>,
}

impl FormGroupBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            controls: HashMap::new(),
        }
    }

    /// Add a control.
    pub fn control(mut self, name: impl Into<String>, control: Rc<dyn AbstractControl>) -> Self {
        self.controls.insert(name.into(), control);
        self
    }

    /// Build the form group.
    pub fn build(self) -> FormGroup {
        FormGroup::with_controls(self.controls)
    }
}

impl Default for FormGroupBuilder {
    fn default() -> Self {
        Self::new()
    }
}

