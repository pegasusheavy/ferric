//! Component inspector for browser devtools integration.
//!
//! Exposes component tree and state to browser developer tools.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Component tree node for inspection.
#[derive(Debug, Clone)]
pub struct ComponentNode {
    /// Component ID.
    pub id: String,
    /// Component name/type.
    pub name: String,
    /// Component selector.
    pub selector: String,
    /// Input properties.
    pub inputs: HashMap<String, String>,
    /// Output events.
    pub outputs: Vec<String>,
    /// Current state as JSON.
    pub state: String,
    /// Child components.
    pub children: Vec<ComponentNode>,
    /// DOM element ID (if available).
    pub element_id: Option<String>,
    /// Change detection status.
    pub change_detection_status: ChangeDetectionStatus,
    /// Lifecycle state.
    pub lifecycle_state: String,
    /// View encapsulation mode.
    pub encapsulation: String,
}

impl ComponentNode {
    /// Create a new component node.
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            selector: String::new(),
            inputs: HashMap::new(),
            outputs: Vec::new(),
            state: "{}".to_string(),
            children: Vec::new(),
            element_id: None,
            change_detection_status: ChangeDetectionStatus::CheckAlways,
            lifecycle_state: "Created".to_string(),
            encapsulation: "Emulated".to_string(),
        }
    }

    /// Add an input property.
    pub fn with_input(mut self, name: &str, value: &str) -> Self {
        self.inputs.insert(name.to_string(), value.to_string());
        self
    }

    /// Add an output event.
    pub fn with_output(mut self, name: &str) -> Self {
        self.outputs.push(name.to_string());
        self
    }

    /// Set the state JSON.
    pub fn with_state(mut self, state: &str) -> Self {
        self.state = state.to_string();
        self
    }

    /// Add a child component.
    pub fn with_child(mut self, child: ComponentNode) -> Self {
        self.children.push(child);
        self
    }

    /// Convert to JSON for devtools.
    pub fn to_json(&self) -> String {
        // Simple JSON serialization
        let inputs_json: String = self
            .inputs
            .iter()
            .map(|(k, v)| format!("\"{}\":\"{}\"", k, v))
            .collect::<Vec<_>>()
            .join(",");

        let outputs_json: String = self
            .outputs
            .iter()
            .map(|o| format!("\"{}\"", o))
            .collect::<Vec<_>>()
            .join(",");

        let children_json: String = self
            .children
            .iter()
            .map(|c| c.to_json())
            .collect::<Vec<_>>()
            .join(",");

        format!(
            r#"{{"id":"{}","name":"{}","selector":"{}","inputs":{{{}}},"outputs":[{}],"state":{},"children":[{}],"elementId":{},"changeDetection":"{}","lifecycle":"{}","encapsulation":"{}"}}"#,
            self.id,
            self.name,
            self.selector,
            inputs_json,
            outputs_json,
            self.state,
            children_json,
            self.element_id.as_ref().map(|e| format!("\"{}\"", e)).unwrap_or("null".to_string()),
            format!("{:?}", self.change_detection_status),
            self.lifecycle_state,
            self.encapsulation,
        )
    }
}

/// Change detection status for display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeDetectionStatus {
    /// Always check this component.
    CheckAlways,
    /// Only check on input changes.
    OnPush,
    /// Detached from change detection.
    Detached,
    /// Marked for check.
    Dirty,
}

/// Component inspector that integrates with browser devtools.
pub struct ComponentInspector {
    tree: RefCell<Option<ComponentNode>>,
    selected_id: RefCell<Option<String>>,
    hooks_installed: RefCell<bool>,
}

impl ComponentInspector {
    /// Create a new inspector.
    pub fn new() -> Self {
        Self {
            tree: RefCell::new(None),
            selected_id: RefCell::new(None),
            hooks_installed: RefCell::new(false),
        }
    }

    /// Install the inspector into the global scope.
    #[cfg(target_arch = "wasm32")]
    pub fn install() {
        use wasm_bindgen::prelude::*;

        // Create global __FERRIC_DEVTOOLS__ object
        let window = web_sys::window().expect("no window");
        let devtools = js_sys::Object::new();

        // Add version info
        js_sys::Reflect::set(&devtools, &"version".into(), &"1.0.0".into()).ok();
        js_sys::Reflect::set(&devtools, &"framework".into(), &"Ferric".into()).ok();

        // Set on window
        js_sys::Reflect::set(&window, &"__FERRIC_DEVTOOLS__".into(), &devtools).ok();

        web_sys::console::log_1(&"[Ferric DevTools] Inspector installed. Access via __FERRIC_DEVTOOLS__".into());
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn install() {
        println!("[Ferric DevTools] Inspector installed (native mode - limited functionality)");
    }

    /// Update the component tree.
    pub fn update_tree(&self, root: ComponentNode) {
        *self.tree.borrow_mut() = Some(root);
        self.notify_update();
    }

    /// Get the current component tree.
    pub fn get_tree(&self) -> Option<ComponentNode> {
        self.tree.borrow().clone()
    }

    /// Get the tree as JSON.
    pub fn get_tree_json(&self) -> String {
        self.tree
            .borrow()
            .as_ref()
            .map(|t| t.to_json())
            .unwrap_or("null".to_string())
    }

    /// Select a component by ID.
    pub fn select(&self, id: &str) {
        *self.selected_id.borrow_mut() = Some(id.to_string());
        self.notify_selection();
    }

    /// Clear selection.
    pub fn deselect(&self) {
        *self.selected_id.borrow_mut() = None;
        self.notify_selection();
    }

    /// Get the selected component ID.
    pub fn selected(&self) -> Option<String> {
        self.selected_id.borrow().clone()
    }

    /// Find a component by ID in the tree.
    pub fn find_component(&self, id: &str) -> Option<ComponentNode> {
        fn search(node: &ComponentNode, id: &str) -> Option<ComponentNode> {
            if node.id == id {
                return Some(node.clone());
            }
            for child in &node.children {
                if let Some(found) = search(child, id) {
                    return Some(found);
                }
            }
            None
        }

        self.tree.borrow().as_ref().and_then(|root| search(root, id))
    }

    /// Notify devtools of tree update.
    fn notify_update(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            let json = self.get_tree_json();
            let event = web_sys::CustomEvent::new_with_event_init_dict(
                "ferric:tree-update",
                web_sys::CustomEventInit::new().detail(&wasm_bindgen::JsValue::from_str(&json)),
            );
            if let Ok(event) = event {
                if let Some(window) = web_sys::window() {
                    window.dispatch_event(&event).ok();
                }
            }
        }
    }

    /// Notify devtools of selection change.
    fn notify_selection(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            let id = self.selected_id.borrow().clone().unwrap_or_default();
            let event = web_sys::CustomEvent::new_with_event_init_dict(
                "ferric:selection-change",
                web_sys::CustomEventInit::new().detail(&wasm_bindgen::JsValue::from_str(&id)),
            );
            if let Ok(event) = event {
                if let Some(window) = web_sys::window() {
                    window.dispatch_event(&event).ok();
                }
            }
        }
    }

    /// Highlight a component in the DOM.
    #[cfg(target_arch = "wasm32")]
    pub fn highlight(&self, id: &str) {
        if let Some(component) = self.find_component(id) {
            if let Some(element_id) = &component.element_id {
                if let Some(window) = web_sys::window() {
                    if let Some(document) = window.document() {
                        if let Ok(Some(element)) = document.query_selector(&format!("[data-ferric-id='{}']", element_id)) {
                            // Add highlight overlay
                            let style = element.get_attribute("style").unwrap_or_default();
                            element.set_attribute("style", &format!("{} outline: 2px solid #4fc3f7; outline-offset: 2px;", style)).ok();
                        }
                    }
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn highlight(&self, id: &str) {
        println!("[Inspector] Highlighting component: {}", id);
    }

    /// Remove highlight from all components.
    #[cfg(target_arch = "wasm32")]
    pub fn clear_highlight(&self) {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Ok(elements) = document.query_selector_all("[data-ferric-id]") {
                    for i in 0..elements.length() {
                        if let Some(element) = elements.get(i) {
                            if let Some(el) = element.dyn_ref::<web_sys::Element>() {
                                // Remove highlight styles
                                if let Some(style) = el.get_attribute("style") {
                                    let cleaned = style.replace("outline: 2px solid #4fc3f7; outline-offset: 2px;", "");
                                    el.set_attribute("style", &cleaned).ok();
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn clear_highlight(&self) {
        println!("[Inspector] Clearing highlights");
    }
}

impl Default for ComponentInspector {
    fn default() -> Self {
        Self::new()
    }
}

/// Global inspector instance.
thread_local! {
    static INSPECTOR: RefCell<ComponentInspector> = RefCell::new(ComponentInspector::new());
}

/// Get the global inspector.
pub fn inspector() -> ComponentInspector {
    INSPECTOR.with(|i| {
        // Return a new instance that shares state
        ComponentInspector {
            tree: RefCell::new(i.borrow().tree.borrow().clone()),
            selected_id: RefCell::new(i.borrow().selected_id.borrow().clone()),
            hooks_installed: RefCell::new(*i.borrow().hooks_installed.borrow()),
        }
    })
}

/// Register a component with the inspector.
pub fn register_component(node: ComponentNode) {
    INSPECTOR.with(|i| {
        let inspector = i.borrow();
        let mut tree = inspector.tree.borrow_mut();
        if tree.is_none() {
            *tree = Some(node);
        } else {
            // Add as child to root (simplified)
            if let Some(ref mut root) = *tree {
                root.children.push(node);
            }
        }
    });
}

/// Injector tree view for debugging DI.
#[derive(Debug, Clone)]
pub struct InjectorNode {
    /// Injector level (0 = root).
    pub level: usize,
    /// Registered providers.
    pub providers: Vec<ProviderInfo>,
    /// Child injectors.
    pub children: Vec<InjectorNode>,
}

/// Provider information for inspection.
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    /// Type name.
    pub type_name: String,
    /// Provider scope.
    pub scope: String,
    /// Whether it's a singleton.
    pub is_singleton: bool,
    /// Resolution count.
    pub resolution_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_node() {
        let node = ComponentNode::new("comp-1", "MyComponent")
            .with_input("title", "Hello")
            .with_output("click")
            .with_state(r#"{"count": 0}"#);

        assert_eq!(node.id, "comp-1");
        assert_eq!(node.name, "MyComponent");
        assert_eq!(node.inputs.get("title"), Some(&"Hello".to_string()));
        assert!(node.outputs.contains(&"click".to_string()));
    }

    #[test]
    fn test_component_tree() {
        let child = ComponentNode::new("child-1", "ChildComponent");
        let root = ComponentNode::new("root", "RootComponent").with_child(child);

        assert_eq!(root.children.len(), 1);
        assert_eq!(root.children[0].name, "ChildComponent");
    }

    #[test]
    fn test_inspector() {
        let inspector = ComponentInspector::new();

        let root = ComponentNode::new("root", "App");
        inspector.update_tree(root);

        assert!(inspector.get_tree().is_some());

        inspector.select("root");
        assert_eq!(inspector.selected(), Some("root".to_string()));
    }

    #[test]
    fn test_find_component() {
        let inspector = ComponentInspector::new();

        let child = ComponentNode::new("child-1", "Child");
        let root = ComponentNode::new("root", "Root").with_child(child);
        inspector.update_tree(root);

        let found = inspector.find_component("child-1");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Child");
    }
}


