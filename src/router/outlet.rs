//! Router outlet for rendering routed components.

use wasm_bindgen::prelude::*;

/// A router outlet that renders the currently active route's component.
pub struct RouterOutlet {
    element: web_sys::Element,
    name: String,
}

impl RouterOutlet {
    /// Create a new router outlet attached to an element.
    pub fn new(element: web_sys::Element) -> Self {
        Self {
            element,
            name: String::from("primary"),
        }
    }

    /// Create a named router outlet (for auxiliary routes).
    pub fn named(element: web_sys::Element, name: &str) -> Self {
        Self {
            element,
            name: name.to_string(),
        }
    }

    /// Get the outlet's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the outlet's element.
    pub fn element(&self) -> &web_sys::Element {
        &self.element
    }

    /// Render a component into this outlet.
    pub fn render(&self, component: &web_sys::Element) -> Result<(), JsValue> {
        // Clear existing content
        self.element.set_inner_html("");

        // Append the new component
        self.element.append_child(component)?;

        Ok(())
    }

    /// Clear the outlet.
    pub fn clear(&self) {
        self.element.set_inner_html("");
    }
}

/// Events emitted by the router outlet.
#[derive(Debug, Clone)]
pub enum RouterOutletEvent {
    /// A component is about to be attached.
    Attach,
    /// A component has been attached.
    Attached,
    /// A component is about to be detached.
    Detach,
    /// A component has been detached.
    Detached,
}

