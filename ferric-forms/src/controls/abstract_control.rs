//! Abstract control trait for all form controls.

use crate::state::{ControlStatus, FormState};
use crate::validators::ValidationErrors;
use std::any::Any;

/// Trait implemented by all form controls.
pub trait AbstractControl {
    /// Get the control's validation errors.
    fn errors(&self) -> Option<ValidationErrors>;

    /// Get the control's state.
    fn state(&self) -> &FormState;

    /// Check if the control is valid.
    fn valid(&self) -> bool {
        self.state().valid()
    }

    /// Check if the control is invalid.
    fn invalid(&self) -> bool {
        self.state().invalid()
    }

    /// Check if the control is pending validation.
    fn pending(&self) -> bool {
        self.state().pending()
    }

    /// Check if the control is disabled.
    fn disabled(&self) -> bool {
        self.state().disabled()
    }

    /// Check if the control is enabled.
    fn enabled(&self) -> bool {
        self.state().enabled()
    }

    /// Check if the control has been touched.
    fn touched(&self) -> bool {
        self.state().touched()
    }

    /// Check if the control is untouched.
    fn untouched(&self) -> bool {
        self.state().untouched()
    }

    /// Check if the control is dirty.
    fn dirty(&self) -> bool {
        self.state().dirty()
    }

    /// Check if the control is pristine.
    fn pristine(&self) -> bool {
        self.state().pristine()
    }

    /// Mark the control as touched.
    fn mark_as_touched(&self) {
        self.state().mark_as_touched();
    }

    /// Mark the control as untouched.
    fn mark_as_untouched(&self) {
        self.state().mark_as_untouched();
    }

    /// Mark the control as dirty.
    fn mark_as_dirty(&self) {
        self.state().mark_as_dirty();
    }

    /// Mark the control as pristine.
    fn mark_as_pristine(&self) {
        self.state().mark_as_pristine();
    }

    /// Reset the control to its initial state.
    fn reset(&self);

    /// Disable the control.
    fn disable(&self) {
        self.state().set_status(ControlStatus::Disabled);
    }

    /// Enable the control.
    fn enable(&self) {
        self.update_validity();
    }

    /// Update the control's validity.
    fn update_validity(&self);

    /// Get the raw value as Any for type erasure.
    fn value_as_any(&self) -> &dyn Any;

    /// Check if the control has a specific error.
    fn has_error(&self, error_code: &str) -> bool {
        self.errors()
            .map(|e| e.contains_key(error_code))
            .unwrap_or(false)
    }

    /// Get a specific error message.
    fn get_error(&self, error_code: &str) -> Option<String> {
        self.errors()
            .and_then(|e| e.get(error_code).map(|err| err.message.clone()))
    }
}

/// Trait for controls that contain other controls.
pub trait ControlContainer: AbstractControl {
    /// Get a child control by name.
    fn get(&self, name: &str) -> Option<&dyn AbstractControl>;

    /// Check if a control exists.
    fn contains(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    /// Mark all child controls as touched.
    fn mark_all_as_touched(&self);

    /// Mark all child controls as untouched.
    fn mark_all_as_untouched(&self);

    /// Mark all child controls as dirty.
    fn mark_all_as_dirty(&self);

    /// Mark all child controls as pristine.
    fn mark_all_as_pristine(&self);
}

