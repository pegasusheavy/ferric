//! Component registry for storing and retrieving component definitions.

use super::ComponentMetadata;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

/// Global component registry.
static REGISTRY: Lazy<RwLock<ComponentRegistry>> =
    Lazy::new(|| RwLock::new(ComponentRegistry::new()));

/// Registry for component definitions.
pub struct ComponentRegistry {
    components: HashMap<String, ComponentMetadata>,
}

impl ComponentRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    /// Register a component with its metadata.
    pub fn register(&mut self, selector: &str, metadata: ComponentMetadata) {
        self.components.insert(selector.to_string(), metadata);
    }

    /// Get a component's metadata by its selector.
    pub fn get(&self, selector: &str) -> Option<&ComponentMetadata> {
        self.components.get(selector)
    }

    /// Check if a component is registered.
    pub fn contains(&self, selector: &str) -> bool {
        self.components.contains_key(selector)
    }

    /// Get all registered selectors.
    pub fn selectors(&self) -> Vec<&String> {
        self.components.keys().collect()
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Register a component globally.
pub fn register_component(selector: &str, metadata: ComponentMetadata) {
    let mut registry = REGISTRY.write().unwrap();
    registry.register(selector, metadata);
}

/// Get a component's metadata from the global registry.
pub fn get_component(selector: &str) -> Option<ComponentMetadata> {
    let registry = REGISTRY.read().unwrap();
    registry.get(selector).cloned()
}

/// Check if a component is registered globally.
pub fn is_component_registered(selector: &str) -> bool {
    let registry = REGISTRY.read().unwrap();
    registry.contains(selector)
}

