//! Template context for expression evaluation.
//!
//! The context holds values that can be referenced in template expressions.

use rustc_hash::FxHashMap;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

/// A value that can be stored in the template context.
pub trait ContextValue: Any {
    /// Convert to Any for downcasting.
    fn as_any(&self) -> &dyn Any;

    /// Clone the value into a Box.
    fn clone_box(&self) -> Box<dyn ContextValue>;

    /// Convert to string for display.
    fn to_display_string(&self) -> String;
}

impl<T: Clone + Any + std::fmt::Display + 'static> ContextValue for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn ContextValue> {
        Box::new(self.clone())
    }

    fn to_display_string(&self) -> String {
        self.to_string()
    }
}

/// A reference to a reactive value in the context.
#[derive(Clone)]
pub struct ContextRef {
    inner: Rc<RefCell<Box<dyn ContextValue>>>,
    subscribers: Rc<RefCell<Vec<Box<dyn Fn()>>>>,
}

impl ContextRef {
    /// Create a new context reference.
    pub fn new<T: ContextValue + 'static>(value: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(Box::new(value))),
            subscribers: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Get the value as a specific type.
    pub fn get<T: Clone + 'static>(&self) -> Option<T> {
        let inner = self.inner.borrow();
        inner.as_any().downcast_ref::<T>().cloned()
    }

    /// Get the display string.
    pub fn to_string(&self) -> String {
        self.inner.borrow().to_display_string()
    }

    /// Set a new value and notify subscribers.
    pub fn set<T: ContextValue + 'static>(&self, value: T) {
        *self.inner.borrow_mut() = Box::new(value);
        self.notify();
    }

    /// Subscribe to changes.
    pub fn subscribe<F: Fn() + 'static>(&self, callback: F) {
        self.subscribers.borrow_mut().push(Box::new(callback));
    }

    /// Notify all subscribers.
    fn notify(&self) {
        for callback in self.subscribers.borrow().iter() {
            callback();
        }
    }
}

/// Template context that holds values for expression evaluation.
#[derive(Clone, Default)]
pub struct TemplateContext {
    values: Rc<RefCell<FxHashMap<String, ContextRef>>>,
    parent: Option<Box<TemplateContext>>,
}

impl TemplateContext {
    /// Create a new empty context.
    pub fn new() -> Self {
        Self {
            values: Rc::new(RefCell::new(FxHashMap::default())),
            parent: None,
        }
    }

    /// Create a child context that inherits from this one.
    pub fn child(&self) -> Self {
        Self {
            values: Rc::new(RefCell::new(FxHashMap::default())),
            parent: Some(Box::new(self.clone())),
        }
    }

    /// Set a value in the context.
    pub fn set<T: ContextValue + 'static>(&self, key: &str, value: T) {
        let ref_val = ContextRef::new(value);
        self.values.borrow_mut().insert(key.to_string(), ref_val);
    }

    /// Set a reactive reference.
    pub fn set_ref(&self, key: &str, ref_val: ContextRef) {
        self.values.borrow_mut().insert(key.to_string(), ref_val);
    }

    /// Get a value from the context.
    pub fn get(&self, key: &str) -> Option<ContextRef> {
        // Check local values first
        if let Some(val) = self.values.borrow().get(key) {
            return Some(val.clone());
        }

        // Check parent context
        if let Some(ref parent) = self.parent {
            return parent.get(key);
        }

        None
    }

    /// Get a typed value from the context.
    pub fn get_value<T: Clone + 'static>(&self, key: &str) -> Option<T> {
        self.get(key).and_then(|r| r.get::<T>())
    }

    /// Check if a key exists.
    pub fn contains(&self, key: &str) -> bool {
        if self.values.borrow().contains_key(key) {
            return true;
        }
        if let Some(ref parent) = self.parent {
            return parent.contains(key);
        }
        false
    }

    /// Subscribe to changes for a specific key.
    pub fn subscribe<F: Fn() + 'static>(&self, key: &str, callback: F) {
        if let Some(ref_val) = self.get(key) {
            ref_val.subscribe(callback);
        }
    }

    /// Update a value.
    pub fn update<T: ContextValue + 'static>(&self, key: &str, value: T) {
        if let Some(ref_val) = self.get(key) {
            ref_val.set(value);
        } else {
            self.set(key, value);
        }
    }
}

/// Builder for creating template contexts.
pub struct ContextBuilder {
    context: TemplateContext,
}

impl ContextBuilder {
    /// Create a new context builder.
    pub fn new() -> Self {
        Self {
            context: TemplateContext::new(),
        }
    }

    /// Add a value to the context.
    pub fn with<T: ContextValue + 'static>(self, key: &str, value: T) -> Self {
        self.context.set(key, value);
        self
    }

    /// Build the context.
    pub fn build(self) -> TemplateContext {
        self.context
    }
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_basic() {
        let ctx = TemplateContext::new();
        ctx.set("name", "Ferric".to_string());
        ctx.set("count", 42i32);

        assert_eq!(ctx.get_value::<String>("name"), Some("Ferric".to_string()));
        assert_eq!(ctx.get_value::<i32>("count"), Some(42));
    }

    #[test]
    fn test_context_child() {
        let parent = TemplateContext::new();
        parent.set("name", "Parent".to_string());

        let child = parent.child();
        child.set("local", "Child".to_string());

        // Child can access parent values
        assert_eq!(child.get_value::<String>("name"), Some("Parent".to_string()));
        // Child has its own values
        assert_eq!(child.get_value::<String>("local"), Some("Child".to_string()));
        // Parent doesn't have child values
        assert_eq!(parent.get_value::<String>("local"), None);
    }
}

