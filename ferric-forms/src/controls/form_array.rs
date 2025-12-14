//! FormArray - a dynamic array of form controls.

use crate::state::{ControlStatus, FormState};
use crate::validators::ValidationErrors;
use super::{AbstractControl, ControlContainer};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

/// A dynamic array of form controls.
pub struct FormArray {
    /// Child controls.
    controls: RefCell<Vec<Rc<dyn AbstractControl>>>,
    /// Form state.
    state: FormState,
    /// Aggregated errors.
    errors: RefCell<Option<ValidationErrors>>,
}

impl FormArray {
    /// Create a new empty form array.
    pub fn new() -> Self {
        Self {
            controls: RefCell::new(Vec::new()),
            state: FormState::new(),
            errors: RefCell::new(None),
        }
    }

    /// Create a form array with initial controls.
    pub fn with_controls(controls: Vec<Rc<dyn AbstractControl>>) -> Self {
        let array = Self {
            controls: RefCell::new(controls),
            state: FormState::new(),
            errors: RefCell::new(None),
        };
        array.update_validity();
        array
    }

    /// Get the number of controls.
    pub fn length(&self) -> usize {
        self.controls.borrow().len()
    }

    /// Check if the array is empty.
    pub fn is_empty(&self) -> bool {
        self.controls.borrow().is_empty()
    }

    /// Push a control to the end.
    pub fn push(&self, control: Rc<dyn AbstractControl>) {
        self.controls.borrow_mut().push(control);
        self.update_validity();
    }

    /// Insert a control at a specific index.
    pub fn insert(&self, index: usize, control: Rc<dyn AbstractControl>) {
        let mut controls = self.controls.borrow_mut();
        if index <= controls.len() {
            controls.insert(index, control);
        }
        drop(controls);
        self.update_validity();
    }

    /// Remove a control at a specific index.
    pub fn remove_at(&self, index: usize) -> Option<Rc<dyn AbstractControl>> {
        let mut controls = self.controls.borrow_mut();
        if index < controls.len() {
            let control = controls.remove(index);
            drop(controls);
            self.update_validity();
            Some(control)
        } else {
            None
        }
    }

    /// Get a control at a specific index.
    pub fn at(&self, index: usize) -> Option<Rc<dyn AbstractControl>> {
        self.controls.borrow().get(index).cloned()
    }

    /// Clear all controls.
    pub fn clear(&self) {
        self.controls.borrow_mut().clear();
        self.update_validity();
    }

    /// Get all controls.
    pub fn controls(&self) -> Vec<Rc<dyn AbstractControl>> {
        self.controls.borrow().clone()
    }

    /// Iterate over controls with indices.
    pub fn enumerate(&self) -> Vec<(usize, Rc<dyn AbstractControl>)> {
        self.controls
            .borrow()
            .iter()
            .enumerate()
            .map(|(i, c)| (i, c.clone()))
            .collect()
    }

    /// Move a control from one index to another.
    pub fn move_control(&self, from: usize, to: usize) {
        let mut controls = self.controls.borrow_mut();
        if from < controls.len() && to < controls.len() {
            let control = controls.remove(from);
            controls.insert(to, control);
        }
    }
}

impl Default for FormArray {
    fn default() -> Self {
        Self::new()
    }
}

impl AbstractControl for FormArray {
    fn errors(&self) -> Option<ValidationErrors> {
        self.errors.borrow().clone()
    }

    fn state(&self) -> &FormState {
        &self.state
    }

    fn reset(&self) {
        for control in self.controls.borrow().iter() {
            control.reset();
        }
        self.state.reset();
        self.update_validity();
    }

    fn update_validity(&self) {
        if self.state.disabled() {
            return;
        }

        let mut all_errors = ValidationErrors::new();
        let mut any_invalid = false;
        let mut any_pending = false;

        for (index, control) in self.controls.borrow().iter().enumerate() {
            if control.pending() {
                any_pending = true;
            }
            if control.invalid() {
                any_invalid = true;
                if let Some(errors) = control.errors() {
                    for (key, error) in errors {
                        all_errors.insert(format!("[{}].{}", index, key), error);
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

impl ControlContainer for FormArray {
    fn get(&self, _name: &str) -> Option<&dyn AbstractControl> {
        // Parse index from name
        None
    }

    fn mark_all_as_touched(&self) {
        self.state.mark_as_touched();
        for control in self.controls.borrow().iter() {
            control.mark_as_touched();
        }
    }

    fn mark_all_as_untouched(&self) {
        self.state.mark_as_untouched();
        for control in self.controls.borrow().iter() {
            control.mark_as_untouched();
        }
    }

    fn mark_all_as_dirty(&self) {
        self.state.mark_as_dirty();
        for control in self.controls.borrow().iter() {
            control.mark_as_dirty();
        }
    }

    fn mark_all_as_pristine(&self) {
        self.state.mark_as_pristine();
        for control in self.controls.borrow().iter() {
            control.mark_as_pristine();
        }
    }
}

