//! Component lifecycle hooks for Ferric.
//!
//! Lifecycle hooks allow components to respond to key events during their existence.
//! The hooks are called in a specific order during initialization, change detection,
//! and destruction.
//!
//! ## Lifecycle Order
//!
//! ### Initialization (called once)
//! 1. `on_init()` - Component created, inputs set
//! 2. `after_view_init()` - View has been initialized
//! 3. `after_content_init()` - Projected content is available
//!
//! ### Change Detection (called on each cycle)
//! 1. `on_changes()` - Input properties changed
//! 2. `do_check()` - Custom change detection
//! 3. `after_content_checked()` - Content children checked
//! 4. `after_view_checked()` - View children checked
//!
//! ### Destruction
//! 1. `on_destroy()` - Cleanup before removal
//!
//! ## Example
//!
//! ```ignore
//! use ferric_core::lifecycle::{Lifecycle, Changes};
//!
//! struct MyComponent {
//!     data: Vec<String>,
//!     subscription_id: Option<u64>,
//! }
//!
//! impl Lifecycle for MyComponent {
//!     fn on_init(&mut self) {
//!         // Fetch initial data, set up subscriptions
//!         self.subscription_id = Some(subscribe_to_updates());
//!     }
//!
//!     fn on_changes(&mut self, changes: &Changes) {
//!         if changes.has_change("filter") {
//!             self.refresh_data();
//!         }
//!     }
//!
//!     fn on_destroy(&mut self) {
//!         // Clean up subscriptions
//!         if let Some(id) = self.subscription_id.take() {
//!             unsubscribe(id);
//!         }
//!     }
//! }
//! ```
//!
//! ## Cleanup Registration
//!
//! Components can register cleanup functions that run automatically on destruction:
//!
//! ```ignore
//! use ferric_core::lifecycle::on_cleanup;
//!
//! fn on_init(&mut self) {
//!     let interval = set_interval(|| update(), 1000);
//!
//!     // This will be called automatically on destroy
//!     on_cleanup(move || {
//!         clear_interval(interval);
//!     });
//! }
//! ```

mod cleanup;
mod hooks;
mod manager;
mod state;

pub use cleanup::{
    CleanupFn, CleanupGuard, CleanupHandle, CleanupRegistry, OnCleanup,
    clear_cleanup_context, on_cleanup, set_cleanup_context, with_cleanup_context,
};
pub use hooks::{
    AsyncLifecycle, DocumentLifecycle, ErrorBoundary, FocusLifecycle,
    HookError, HookResult, LifecycleHookBuilder, NavigationLifecycle,
    ResizeLifecycle, VisibilityLifecycle,
};
pub use manager::{
    ComponentId, LifecycleError, LifecycleManager, LifecycleResult,
    get_component_state, lifecycle_manager, register_component,
};
pub use state::LifecycleState;

use std::collections::HashMap;

/// Trait providing lifecycle hooks for components.
pub trait Lifecycle {
    /// Called once when the component is initialized.
    /// Use this for one-time setup that doesn't require DOM access.
    fn on_init(&mut self) {}

    /// Called after the component's view has been initialized.
    /// The component's element is now available.
    fn after_view_init(&mut self) {}

    /// Called after the component's content has been initialized.
    /// Projected content is now available.
    fn after_content_init(&mut self) {}

    /// Called when input properties change.
    /// Receives information about which properties changed.
    fn on_changes(&mut self, _changes: &Changes) {}

    /// Called during every change detection cycle.
    /// Use sparingly as it can impact performance.
    fn do_check(&mut self) {}

    /// Called after the component's view is checked.
    fn after_view_checked(&mut self) {}

    /// Called after the component's content is checked.
    fn after_content_checked(&mut self) {}

    /// Called just before the component is destroyed.
    /// Use this for cleanup like unsubscribing from observables.
    fn on_destroy(&mut self) {}
}

/// Information about changes to component inputs.
#[derive(Debug, Default)]
pub struct Changes {
    /// Map of property names to their change information.
    pub changes: HashMap<String, Change>,
}

impl Changes {
    /// Create a new empty Changes instance.
    pub fn new() -> Self {
        Self {
            changes: HashMap::new(),
        }
    }

    /// Check if a specific property changed.
    pub fn has_change(&self, property: &str) -> bool {
        self.changes.contains_key(property)
    }

    /// Get the change for a specific property.
    pub fn get(&self, property: &str) -> Option<&Change> {
        self.changes.get(property)
    }

    /// Add a change.
    pub fn add(&mut self, property: &str, change: Change) {
        self.changes.insert(property.to_string(), change);
    }

    /// Check if this is the first change (initialization).
    pub fn is_first_change(&self) -> bool {
        self.changes.values().any(|c| c.first_change)
    }

    /// Check if any changes occurred.
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Get all changed property names.
    pub fn changed_properties(&self) -> Vec<&str> {
        self.changes.keys().map(|s| s.as_str()).collect()
    }

    /// Iterate over all changes.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Change)> {
        self.changes.iter().map(|(k, v)| (k.as_str(), v))
    }
}

/// Information about a single property change.
#[derive(Debug, Clone)]
pub struct Change {
    /// The previous value (as a string representation).
    pub previous_value: Option<String>,
    /// The current value (as a string representation).
    pub current_value: String,
    /// Whether this is the first change (initialization).
    pub first_change: bool,
}

impl Change {
    /// Create a new change.
    pub fn new(previous: Option<String>, current: String, first: bool) -> Self {
        Self {
            previous_value: previous,
            current_value: current,
            first_change: first,
        }
    }

    /// Create a first change (no previous value).
    pub fn first(current: String) -> Self {
        Self::new(None, current, true)
    }

    /// Create a subsequent change.
    pub fn subsequent(previous: String, current: String) -> Self {
        Self::new(Some(previous), current, false)
    }

    /// Check if the value actually changed (not just the first assignment).
    pub fn value_changed(&self) -> bool {
        match &self.previous_value {
            Some(prev) => prev != &self.current_value,
            None => true,
        }
    }
}

/// Typed change information with generic values.
#[derive(Debug, Clone)]
pub struct TypedChange<T> {
    /// The previous value.
    pub previous_value: Option<T>,
    /// The current value.
    pub current_value: T,
    /// Whether this is the first change.
    pub first_change: bool,
}

impl<T: Clone> TypedChange<T> {
    /// Create a new typed change.
    pub fn new(previous: Option<T>, current: T, first: bool) -> Self {
        Self {
            previous_value: previous,
            current_value: current,
            first_change: first,
        }
    }

    /// Create a first change.
    pub fn first(current: T) -> Self {
        Self::new(None, current, true)
    }

    /// Create a subsequent change.
    pub fn subsequent(previous: T, current: T) -> Self {
        Self::new(Some(previous), current, false)
    }
}

impl<T: Clone + PartialEq> TypedChange<T> {
    /// Check if the value actually changed.
    pub fn value_changed(&self) -> bool {
        match &self.previous_value {
            Some(prev) => prev != &self.current_value,
            None => true,
        }
    }
}

/// Marker trait for components that implement OnInit.
pub trait OnInit: Lifecycle {
    /// Custom initialization logic.
    fn fe_on_init(&mut self);
}

/// Marker trait for components that implement OnDestroy.
pub trait OnDestroy: Lifecycle {
    /// Custom cleanup logic.
    fn fe_on_destroy(&mut self);
}

/// Marker trait for components that implement OnChanges.
pub trait OnChanges: Lifecycle {
    /// Custom change handling logic.
    fn fe_on_changes(&mut self, changes: &Changes);
}

/// Marker trait for components that implement DoCheck.
pub trait DoCheck: Lifecycle {
    /// Custom change detection logic.
    fn fe_do_check(&mut self);
}

/// Marker trait for components that implement AfterViewInit.
pub trait AfterViewInit: Lifecycle {
    /// Custom logic after view initialization.
    fn fe_after_view_init(&mut self);
}

/// Marker trait for components that implement AfterContentInit.
pub trait AfterContentInit: Lifecycle {
    /// Custom logic after content initialization.
    fn fe_after_content_init(&mut self);
}

/// Marker trait for components that implement AfterViewChecked.
pub trait AfterViewChecked: Lifecycle {
    /// Custom logic after view checking.
    fn fe_after_view_checked(&mut self);
}

/// Marker trait for components that implement AfterContentChecked.
pub trait AfterContentChecked: Lifecycle {
    /// Custom logic after content checking.
    fn fe_after_content_checked(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_changes() {
        let mut changes = Changes::new();
        changes.add("name", Change::first("John".to_string()));
        changes.add("age", Change::subsequent("25".to_string(), "26".to_string()));

        assert!(changes.has_change("name"));
        assert!(changes.has_change("age"));
        assert!(!changes.has_change("email"));

        assert!(changes.is_first_change());

        let name_change = changes.get("name").unwrap();
        assert!(name_change.first_change);
        assert_eq!(name_change.current_value, "John");
    }

    #[test]
    fn test_typed_change() {
        let change: TypedChange<i32> = TypedChange::subsequent(5, 10);
        assert!(change.value_changed());
        assert!(!change.first_change);
        assert_eq!(change.previous_value, Some(5));
        assert_eq!(change.current_value, 10);
    }
}
