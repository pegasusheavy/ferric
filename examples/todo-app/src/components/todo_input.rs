//! Todo input component - now just a reference, main input is in the app template.

use web_sys::Element;

/// Input component for adding new todos.
/// Note: The actual input is now part of the main app template.
/// This struct just holds a reference to the input element.
pub struct TodoInput {
    element: Element,
}

impl TodoInput {
    /// Create a new reference to an input element.
    pub fn new(element: Element) -> Self {
        Self { element }
    }

    /// Get the DOM element.
    pub fn element(&self) -> &Element {
        &self.element
    }
}
