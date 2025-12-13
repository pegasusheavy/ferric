//! Component lifecycle hooks for Ferric.
//!
//! Lifecycle hooks allow components to respond to key events during their existence.

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
    pub changes: std::collections::HashMap<String, Change>,
}

impl Changes {
    /// Create a new empty Changes instance.
    pub fn new() -> Self {
        Self {
            changes: std::collections::HashMap::new(),
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
}

/// Marker trait for components that implement OnInit.
pub trait OnInit: Lifecycle {
    /// Custom initialization logic.
    fn ng_on_init(&mut self);
}

/// Marker trait for components that implement OnDestroy.
pub trait OnDestroy: Lifecycle {
    /// Custom cleanup logic.
    fn ng_on_destroy(&mut self);
}

/// Marker trait for components that implement OnChanges.
pub trait OnChanges: Lifecycle {
    /// Custom change handling logic.
    fn ng_on_changes(&mut self, changes: &Changes);
}

/// Marker trait for components that implement DoCheck.
pub trait DoCheck: Lifecycle {
    /// Custom change detection logic.
    fn ng_do_check(&mut self);
}

/// Marker trait for components that implement AfterViewInit.
pub trait AfterViewInit: Lifecycle {
    /// Custom logic after view initialization.
    fn ng_after_view_init(&mut self);
}

/// Marker trait for components that implement AfterContentInit.
pub trait AfterContentInit: Lifecycle {
    /// Custom logic after content initialization.
    fn ng_after_content_init(&mut self);
}

