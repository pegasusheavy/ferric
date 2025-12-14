//! Client-side hydration utilities for server-rendered content.
//!
//! This module provides the client-side (WASM) support for hydrating
//! server-rendered HTML with interactivity.

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, Window};
use std::collections::HashMap;

/// Hydration context for the client-side.
pub struct HydrationContext {
    /// The root element where hydration starts.
    pub root: Element,

    /// Serialized state from the server.
    pub state: HashMap<String, serde_json::Value>,

    /// Document reference.
    pub document: Document,

    /// Window reference.
    pub window: Window,
}

impl HydrationContext {
    /// Create a new hydration context from the root element ID.
    pub fn from_root_id(root_id: &str) -> Result<Self, JsValue> {
        let window = web_sys::window().ok_or("no global window")?;
        let document = window.document().ok_or("no document")?;
        let root = document
            .get_element_by_id(root_id)
            .ok_or_else(|| format!("root element not found: {}", root_id))?;

        Ok(Self {
            root,
            state: HashMap::new(),
            document,
            window,
        })
    }

    /// Create a hydration context with state loaded from a script tag.
    pub fn from_root_with_state(root_id: &str, state_script_id: &str) -> Result<Self, JsValue> {
        let mut ctx = Self::from_root_id(root_id)?;
        ctx.load_state(state_script_id)?;
        Ok(ctx)
    }

    /// Load serialized state from a script tag.
    pub fn load_state(&mut self, state_script_id: &str) -> Result<(), JsValue> {
        if let Some(script) = self.document.get_element_by_id(state_script_id) {
            let state_json = script.text_content().unwrap_or_default();
            match serde_json::from_str::<HashMap<String, serde_json::Value>>(&state_json) {
                Ok(state) => {
                    self.state = state;
                    Ok(())
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to parse state: {:?}", e).into());
                    Err(format!("Failed to parse state: {:?}", e).into())
                }
            }
        } else {
            Ok(()) // No state script is okay
        }
    }

    /// Get state for a specific component.
    pub fn get_component_state(&self, component_id: &str) -> Option<&serde_json::Value> {
        self.state.get(component_id)
    }

    /// Find all elements marked for hydration.
    pub fn find_hydration_targets(&self) -> Result<Vec<Element>, JsValue> {
        let elements = self.root.query_selector_all("[data-ferric-hydrate='true']")?;
        let mut targets = Vec::new();

        for i in 0..elements.length() {
            if let Some(el) = elements.item(i)
                && let Some(element) = el.dyn_ref::<Element>() {
                    targets.push(element.clone());
                }
        }

        Ok(targets)
    }

    /// Find elements for a specific component ID.
    pub fn find_component_elements(&self, component_id: &str) -> Result<Vec<Element>, JsValue> {
        let selector = format!("[data-ferric-id='{}']", component_id);
        let elements = self.root.query_selector_all(&selector)?;
        let mut targets = Vec::new();

        for i in 0..elements.length() {
            if let Some(el) = elements.item(i)
                && let Some(element) = el.dyn_ref::<Element>() {
                    targets.push(element.clone());
                }
        }

        Ok(targets)
    }
}

/// Trait for types that can be hydrated on the client-side.
pub trait Hydratable {
    /// Hydrate the component from a server-rendered element.
    fn hydrate(&mut self, element: &Element, state: Option<&serde_json::Value>) -> Result<(), JsValue>;

    /// Get the component ID for hydration.
    fn component_id(&self) -> &str;
}

/// Hydration manager for coordinating client-side hydration.
#[derive(Debug)]
pub struct HydrationManager {
    hydrated: HashMap<String, bool>,
}

impl HydrationManager {
    /// Create a new hydration manager.
    pub fn new() -> Self {
        Self {
            hydrated: HashMap::new(),
        }
    }

    /// Check if a component has been hydrated.
    pub fn is_hydrated(&self, component_id: &str) -> bool {
        self.hydrated.get(component_id).copied().unwrap_or(false)
    }

    /// Mark a component as hydrated.
    pub fn mark_hydrated(&mut self, component_id: &str) {
        self.hydrated.insert(component_id.to_string(), true);
    }

    /// Hydrate all components in the given context.
    pub fn hydrate_all<T: Hydratable>(
        &mut self,
        ctx: &HydrationContext,
        mut component_factory: impl FnMut(&str, &Element) -> Option<T>,
    ) -> Result<Vec<T>, JsValue> {
        let targets = ctx.find_hydration_targets()?;
        let mut components = Vec::new();

        for element in targets {
            if let Some(component_id) = element.get_attribute("data-ferric-id") {
                if self.is_hydrated(&component_id) {
                    continue;
                }

                if let Some(mut component) = component_factory(&component_id, &element) {
                    let state = ctx.get_component_state(&component_id);
                    component.hydrate(&element, state)?;
                    self.mark_hydrated(&component_id);

                    // Mark element as hydrated
                    element.set_attribute("data-ferric-hydrated", "true")?;

                    components.push(component);
                }
            }
        }

        // Dispatch global hydration event
        self.dispatch_hydration_complete(&ctx.window, components.len())?;

        Ok(components)
    }

    /// Dispatch hydration complete event.
    fn dispatch_hydration_complete(&self, window: &Window, count: usize) -> Result<(), JsValue> {
        let event = web_sys::CustomEvent::new("ferric:hydrated")?;
        let detail = js_sys::Object::new();
        js_sys::Reflect::set(&detail, &"componentCount".into(), &JsValue::from(count))?;

        window.dispatch_event(&event)?;
        Ok(())
    }
}

impl Default for HydrationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global hydration API exposed to JavaScript.
#[wasm_bindgen]
pub struct FerricHydration {
    manager: HydrationManager,
}

#[wasm_bindgen]
impl FerricHydration {
    /// Create a new Ferric hydration instance.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            manager: HydrationManager::new(),
        }
    }

    /// Initialize hydration (called from generated hydration script).
    #[wasm_bindgen(js_name = hydrateComponent)]
    pub fn hydrate_component(
        &mut self,
        element: &Element,
        component_id: &str,
        state: JsValue,
        _props: JsValue,
    ) -> Result<(), JsValue> {
        if self.manager.is_hydrated(component_id) {
            return Ok(());
        }

        // Log hydration
        web_sys::console::log_2(
            &"[Ferric] Hydrating component:".into(),
            &component_id.into(),
        );

        // Parse state if provided
        let _state_data: Option<HashMap<String, serde_json::Value>> = if !state.is_undefined() && !state.is_null() {
            serde_wasm_bindgen::from_value(state).ok()
        } else {
            None
        };

        // In a real implementation, you would:
        // 1. Look up the component type registry
        // 2. Create the component instance
        // 3. Attach event listeners
        // 4. Restore state
        // 5. Connect to reactive system

        self.manager.mark_hydrated(component_id);
        element.set_attribute("data-ferric-hydrated", "true")?;

        Ok(())
    }

    /// Check if a component is hydrated.
    #[wasm_bindgen(js_name = isHydrated)]
    pub fn is_hydrated(&self, component_id: &str) -> bool {
        self.manager.is_hydrated(component_id)
    }
}

/// Initialize the global Ferric hydration API.
pub fn init_hydration_api() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("no global window")?;
    let ferric = FerricHydration::new();

    // Expose to window.__FERRIC__
    js_sys::Reflect::set(
        &window,
        &"__FERRIC__".into(),
        &JsValue::from(ferric),
    )?;

    // Dispatch ready event
    let event = web_sys::CustomEvent::new("ferric:ready")?;
    window.dispatch_event(&event)?;

    web_sys::console::log_1(&"[Ferric] Hydration API initialized".into());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a browser environment or wasm-bindgen-test
    // For now, they're just structural tests

    #[test]
    fn test_hydration_manager() {
        let mut manager = HydrationManager::new();

        assert!(!manager.is_hydrated("test-component"));
        manager.mark_hydrated("test-component");
        assert!(manager.is_hydrated("test-component"));
    }
}

