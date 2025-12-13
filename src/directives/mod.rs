//! Built-in directives for Ferric.
//!
//! Directives extend the behavior of elements in templates.

pub mod structural;
pub mod attribute;

pub use attribute::{ClassDirective, StyleDirective, DisabledDirective, HiddenDirective};
pub use structural::{IfDirective, ForDirective, SwitchDirective};

use std::any::Any;

/// Trait for structural directives that modify the DOM structure.
pub trait StructuralDirective {
    /// The selector used to identify this directive (e.g., "*if", "*for").
    fn selector(&self) -> &'static str;

    /// Create view(s) based on the directive's expression.
    fn create_view(&self, context: &dyn Any) -> Result<(), String>;

    /// Update the view when the expression changes.
    fn update_view(&self, context: &dyn Any) -> Result<(), String>;

    /// Destroy the view(s) created by this directive.
    fn destroy_view(&self) -> Result<(), String>;
}

/// Trait for attribute directives that modify element behavior.
pub trait AttributeDirective {
    /// The selector used to identify this directive.
    fn selector(&self) -> &'static str;

    /// Called when the directive is initialized on an element.
    fn on_init(&mut self, element: &web_sys::Element);

    /// Called when the directive's input changes.
    fn on_changes(&mut self, element: &web_sys::Element);

    /// Called when the directive is destroyed.
    fn on_destroy(&mut self, element: &web_sys::Element);
}

/// Host binding for accessing the host element from a directive.
pub struct HostBinding {
    /// The host element.
    element: web_sys::Element,
}

impl HostBinding {
    /// Create a new host binding.
    pub fn new(element: web_sys::Element) -> Self {
        Self { element }
    }

    /// Get the host element.
    pub fn element(&self) -> &web_sys::Element {
        &self.element
    }

    /// Set a property on the host element.
    pub fn set_property(&self, name: &str, value: &str) -> Result<(), wasm_bindgen::JsValue> {
        self.element.set_attribute(name, value)
    }

    /// Add a class to the host element.
    pub fn add_class(&self, class: &str) {
        let _ = self.element.class_list().add_1(class);
    }

    /// Remove a class from the host element.
    pub fn remove_class(&self, class: &str) {
        let _ = self.element.class_list().remove_1(class);
    }
}

