//! Form state management.

use std::cell::RefCell;
use std::rc::Rc;

/// Status of a form control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlStatus {
    /// The control is valid.
    Valid,
    /// The control is invalid.
    Invalid,
    /// The control is pending async validation.
    Pending,
    /// The control is disabled.
    Disabled,
}

impl Default for ControlStatus {
    fn default() -> Self {
        Self::Valid
    }
}

/// State tracking for form controls.
#[derive(Debug, Clone)]
pub struct FormState {
    /// Current validation status.
    status: Rc<RefCell<ControlStatus>>,
    /// Whether the control has been interacted with.
    touched: Rc<RefCell<bool>>,
    /// Whether the control value has changed.
    dirty: Rc<RefCell<bool>>,
    /// Whether the control is currently focused.
    focused: Rc<RefCell<bool>>,
}

impl Default for FormState {
    fn default() -> Self {
        Self::new()
    }
}

impl FormState {
    /// Create a new form state.
    pub fn new() -> Self {
        Self {
            status: Rc::new(RefCell::new(ControlStatus::Valid)),
            touched: Rc::new(RefCell::new(false)),
            dirty: Rc::new(RefCell::new(false)),
            focused: Rc::new(RefCell::new(false)),
        }
    }

    // Status getters

    /// Get the current status.
    #[inline]
    pub fn status(&self) -> ControlStatus {
        *self.status.borrow()
    }

    /// Check if the control is valid.
    #[inline]
    pub fn valid(&self) -> bool {
        matches!(self.status(), ControlStatus::Valid)
    }

    /// Check if the control is invalid.
    #[inline]
    pub fn invalid(&self) -> bool {
        matches!(self.status(), ControlStatus::Invalid)
    }

    /// Check if the control is pending validation.
    #[inline]
    pub fn pending(&self) -> bool {
        matches!(self.status(), ControlStatus::Pending)
    }

    /// Check if the control is disabled.
    #[inline]
    pub fn disabled(&self) -> bool {
        matches!(self.status(), ControlStatus::Disabled)
    }

    /// Check if the control is enabled.
    #[inline]
    pub fn enabled(&self) -> bool {
        !self.disabled()
    }

    // Interaction state getters

    /// Check if the control has been touched (blurred).
    #[inline]
    pub fn touched(&self) -> bool {
        *self.touched.borrow()
    }

    /// Check if the control has not been touched.
    #[inline]
    pub fn untouched(&self) -> bool {
        !self.touched()
    }

    /// Check if the control value has been modified.
    #[inline]
    pub fn dirty(&self) -> bool {
        *self.dirty.borrow()
    }

    /// Check if the control value has not been modified.
    #[inline]
    pub fn pristine(&self) -> bool {
        !self.dirty()
    }

    /// Check if the control is currently focused.
    #[inline]
    pub fn focused(&self) -> bool {
        *self.focused.borrow()
    }

    // State setters

    /// Set the control status.
    pub fn set_status(&self, status: ControlStatus) {
        *self.status.borrow_mut() = status;
    }

    /// Mark the control as touched.
    pub fn mark_as_touched(&self) {
        *self.touched.borrow_mut() = true;
    }

    /// Mark the control as untouched.
    pub fn mark_as_untouched(&self) {
        *self.touched.borrow_mut() = false;
    }

    /// Mark the control as dirty.
    pub fn mark_as_dirty(&self) {
        *self.dirty.borrow_mut() = true;
    }

    /// Mark the control as pristine.
    pub fn mark_as_pristine(&self) {
        *self.dirty.borrow_mut() = false;
    }

    /// Set the focused state.
    pub fn set_focused(&self, focused: bool) {
        *self.focused.borrow_mut() = focused;
    }

    /// Reset the state to initial values.
    pub fn reset(&self) {
        *self.status.borrow_mut() = ControlStatus::Valid;
        *self.touched.borrow_mut() = false;
        *self.dirty.borrow_mut() = false;
        *self.focused.borrow_mut() = false;
    }

    // Utility methods for CSS classes

    /// Get CSS classes based on current state.
    pub fn css_classes(&self) -> Vec<&'static str> {
        let mut classes = Vec::with_capacity(6);

        // Validity classes
        match self.status() {
            ControlStatus::Valid => classes.push("fc-valid"),
            ControlStatus::Invalid => classes.push("fc-invalid"),
            ControlStatus::Pending => classes.push("fc-pending"),
            ControlStatus::Disabled => classes.push("fc-disabled"),
        }

        // Interaction classes
        if self.touched() {
            classes.push("fc-touched");
        } else {
            classes.push("fc-untouched");
        }

        if self.dirty() {
            classes.push("fc-dirty");
        } else {
            classes.push("fc-pristine");
        }

        if self.focused() {
            classes.push("fc-focused");
        }

        classes
    }

    /// Get CSS class string.
    pub fn css_class_string(&self) -> String {
        self.css_classes().join(" ")
    }
}

