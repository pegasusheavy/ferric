//! Component context and runtime state.

use crate::di::Injector;
use std::cell::RefCell;
use std::rc::Rc;

/// Runtime context for a component instance.
pub struct ComponentContext {
    /// The dependency injector for this component.
    injector: Rc<Injector>,
    /// Whether the component has been initialized.
    initialized: RefCell<bool>,
    /// Whether the component is currently being rendered.
    rendering: RefCell<bool>,
}

impl ComponentContext {
    /// Create a new component context.
    pub fn new(injector: Rc<Injector>) -> Self {
        Self {
            injector,
            initialized: RefCell::new(false),
            rendering: RefCell::new(false),
        }
    }

    /// Get the dependency injector.
    pub fn injector(&self) -> &Injector {
        &self.injector
    }

    /// Check if the component has been initialized.
    pub fn is_initialized(&self) -> bool {
        *self.initialized.borrow()
    }

    /// Mark the component as initialized.
    pub fn set_initialized(&self) {
        *self.initialized.borrow_mut() = true;
    }

    /// Check if the component is currently rendering.
    pub fn is_rendering(&self) -> bool {
        *self.rendering.borrow()
    }

    /// Set the rendering state.
    pub fn set_rendering(&self, rendering: bool) {
        *self.rendering.borrow_mut() = rendering;
    }
}

