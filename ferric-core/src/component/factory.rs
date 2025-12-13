//! Component factory for dynamic component creation.
//!
//! This module provides functionality for creating components dynamically at runtime,
//! useful for scenarios like:
//! - Loading components conditionally
//! - Creating components from user input or configuration
//! - Building complex UIs with dynamic layouts
//!
//! ## Usage
//!
//! ```ignore
//! use ferric_core::component::{ComponentFactory, ComponentRef};
//!
//! // Create a factory
//! let factory = ComponentFactory::new();
//!
//! // Create a component dynamically
//! let comp_ref = factory.create("app-button", Some(container_element))?;
//!
//! // Access the component
//! let element = comp_ref.element();
//! ```

use super::{Component, ComponentMetadata, get_component};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::JsValue;

/// A reference to a dynamically created component instance.
#[derive(Clone)]
pub struct ComponentRef {
    /// The component's selector.
    selector: String,
    /// The rendered DOM element.
    element: web_sys::Element,
    /// The component's root element (may be different from element).
    root: Option<web_sys::Element>,
    /// Component instance ID for tracking.
    instance_id: usize,
}

impl ComponentRef {
    /// Create a new component reference.
    pub fn new(selector: &str, element: web_sys::Element, instance_id: usize) -> Self {
        Self {
            selector: selector.to_string(),
            element,
            root: None,
            instance_id,
        }
    }

    /// Get the component's selector.
    pub fn selector(&self) -> &str {
        &self.selector
    }

    /// Get the rendered element.
    pub fn element(&self) -> &web_sys::Element {
        &self.element
    }

    /// Get the component instance ID.
    pub fn instance_id(&self) -> usize {
        self.instance_id
    }

    /// Set the root element (useful for components with wrappers).
    pub fn set_root(&mut self, root: web_sys::Element) {
        self.root = Some(root);
    }

    /// Get the root element if set.
    pub fn root(&self) -> Option<&web_sys::Element> {
        self.root.as_ref()
    }

    /// Destroy this component instance (remove from DOM).
    pub fn destroy(&self) -> Result<(), JsValue> {
        if let Some(parent) = self.element.parent_node() {
            parent.remove_child(&self.element)?;
        }
        Ok(())
    }
}

/// Factory for creating component instances dynamically.
pub struct ComponentFactory {
    /// Counter for generating unique instance IDs.
    next_instance_id: RefCell<usize>,
    /// Active component instances.
    instances: RefCell<HashMap<usize, ComponentRef>>,
}

impl ComponentFactory {
    /// Create a new component factory.
    pub fn new() -> Self {
        Self {
            next_instance_id: RefCell::new(1),
            instances: RefCell::new(HashMap::new()),
        }
    }

    /// Create a component instance by selector.
    ///
    /// The component will be rendered and optionally attached to a container.
    ///
    /// # Arguments
    ///
    /// * `selector` - The component's selector (e.g., "app-button")
    /// * `container` - Optional container element to append the component to
    ///
    /// # Returns
    ///
    /// A `ComponentRef` that can be used to interact with the component.
    pub fn create(
        &self,
        selector: &str,
        container: Option<&web_sys::Element>,
    ) -> Result<ComponentRef, String> {
        // Get component metadata from registry
        let metadata = get_component(selector)
            .ok_or_else(|| format!("Component '{}' not found in registry", selector))?;

        // Generate instance ID
        let instance_id = {
            let mut next_id = self.next_instance_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        // Render the component
        let element = self.render_component(&metadata)?;

        // Attach to container if provided
        if let Some(container) = container {
            container
                .append_child(&element)
                .map_err(|e| format!("Failed to append component: {:?}", e))?;
        }

        // Create component reference
        let comp_ref = ComponentRef::new(selector, element, instance_id);

        // Track instance
        self.instances.borrow_mut().insert(instance_id, comp_ref.clone());

        Ok(comp_ref)
    }

    /// Create multiple components from a list of selectors.
    pub fn create_many(
        &self,
        selectors: &[&str],
        container: Option<&web_sys::Element>,
    ) -> Result<Vec<ComponentRef>, String> {
        selectors
            .iter()
            .map(|sel| self.create(sel, container))
            .collect()
    }

    /// Render a component from its metadata.
    fn render_component(&self, metadata: &ComponentMetadata) -> Result<web_sys::Element, String> {
        let document = web_sys::window()
            .ok_or("No window")?
            .document()
            .ok_or("No document")?;

        // Create element with the component's selector as tag name
        let element = document
            .create_element(&metadata.selector)
            .map_err(|e| format!("Failed to create element: {:?}", e))?;

        // Set component content (simplified - in a real implementation this would
        // compile the template and set up bindings)
        if let Some(template) = &metadata.template {
            element.set_inner_html(template);
        }

        // Apply styles if present
        if !metadata.styles.is_empty() {
            let styles_combined = metadata.styles.join("\n");
            let style_element = document
                .create_element("style")
                .map_err(|e| format!("Failed to create style element: {:?}", e))?;
            style_element.set_inner_html(&styles_combined);
            element
                .append_child(&style_element)
                .map_err(|e| format!("Failed to append styles: {:?}", e))?;
        }

        Ok(element)
    }

    /// Get a component instance by ID.
    pub fn get_instance(&self, instance_id: usize) -> Option<ComponentRef> {
        self.instances.borrow().get(&instance_id).cloned()
    }

    /// Destroy a component instance by ID.
    pub fn destroy(&self, instance_id: usize) -> Result<(), String> {
        if let Some(comp_ref) = self.instances.borrow_mut().remove(&instance_id) {
            comp_ref.destroy().map_err(|e| format!("{:?}", e))?;
        }
        Ok(())
    }

    /// Destroy all component instances.
    pub fn destroy_all(&self) -> Result<(), String> {
        let instance_ids: Vec<usize> = self.instances.borrow().keys().copied().collect();
        for id in instance_ids {
            self.destroy(id)?;
        }
        Ok(())
    }

    /// Get the count of active component instances.
    pub fn instance_count(&self) -> usize {
        self.instances.borrow().len()
    }

    /// Check if a component instance exists.
    pub fn has_instance(&self, instance_id: usize) -> bool {
        self.instances.borrow().contains_key(&instance_id)
    }
}

impl Default for ComponentFactory {
    fn default() -> Self {
        Self::new()
    }
}

use std::sync::OnceLock;

/// Global component factory instance (thread-local for WASM).
thread_local! {
    static GLOBAL_FACTORY: Rc<ComponentFactory> = Rc::new(ComponentFactory::new());
}

/// Get a clone of the global component factory.
pub fn global_factory() -> Rc<ComponentFactory> {
    GLOBAL_FACTORY.with(|f| f.clone())
}

/// Create a component using the global factory.
pub fn create_component(
    selector: &str,
    container: Option<&web_sys::Element>,
) -> Result<ComponentRef, String> {
    let factory = global_factory();
    factory.create(selector, container)
}

/// Create multiple components using the global factory.
pub fn create_components(
    selectors: &[&str],
    container: Option<&web_sys::Element>,
) -> Result<Vec<ComponentRef>, String> {
    let factory = global_factory();
    factory.create_many(selectors, container)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_factory_creation() {
        let factory = ComponentFactory::new();
        assert_eq!(factory.instance_count(), 0);
    }

    #[test]
    fn test_component_ref() {
        // This would need a real component registered to test fully
        // For now, just test the structure
        let factory = ComponentFactory::new();
        assert_eq!(factory.instance_count(), 0);
    }
}

