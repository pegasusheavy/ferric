//! Lifecycle manager for coordinating component lifecycle events.

use super::cleanup::{CleanupRegistry, with_cleanup_context};
use super::state::LifecycleState;
use super::{Changes, Lifecycle};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Error type for lifecycle operations.
#[derive(Debug, Clone)]
pub enum LifecycleError {
    /// Invalid state transition attempted.
    InvalidTransition {
        from: LifecycleState,
        to: LifecycleState,
    },
    /// Component not found.
    ComponentNotFound(ComponentId),
    /// Lifecycle hook panicked.
    HookPanicked(String),
    /// Component already destroyed.
    AlreadyDestroyed(ComponentId),
}

impl std::fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LifecycleError::InvalidTransition { from, to } => {
                write!(f, "Invalid lifecycle transition from {} to {}", from, to)
            }
            LifecycleError::ComponentNotFound(id) => {
                write!(f, "Component {:?} not found", id)
            }
            LifecycleError::HookPanicked(msg) => {
                write!(f, "Lifecycle hook panicked: {}", msg)
            }
            LifecycleError::AlreadyDestroyed(id) => {
                write!(f, "Component {:?} already destroyed", id)
            }
        }
    }
}

impl std::error::Error for LifecycleError {}

/// Result type for lifecycle operations.
pub type LifecycleResult<T> = Result<T, LifecycleError>;

/// Unique identifier for a component instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentId(u64);

impl ComponentId {
    /// Create a new component ID.
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the raw ID value.
    pub fn raw(&self) -> u64 {
        self.0
    }
}

/// Information about a managed component.
struct ManagedComponent {
    id: ComponentId,
    state: LifecycleState,
    cleanup_registry: Rc<CleanupRegistry>,
    parent: Option<ComponentId>,
    children: Vec<ComponentId>,
}

impl ManagedComponent {
    fn new(id: ComponentId, parent: Option<ComponentId>) -> Self {
        Self {
            id,
            state: LifecycleState::Created,
            cleanup_registry: Rc::new(CleanupRegistry::new()),
            parent,
            children: Vec::new(),
        }
    }
}

/// Manages the lifecycle of all components in an application.
pub struct LifecycleManager {
    components: RefCell<HashMap<ComponentId, ManagedComponent>>,
    next_id: RefCell<u64>,
    /// Components queued for initialization.
    init_queue: RefCell<Vec<ComponentId>>,
    /// Components queued for destruction.
    destroy_queue: RefCell<Vec<ComponentId>>,
}

impl LifecycleManager {
    /// Create a new lifecycle manager.
    pub fn new() -> Self {
        Self {
            components: RefCell::new(HashMap::new()),
            next_id: RefCell::new(1),
            init_queue: RefCell::new(Vec::new()),
            destroy_queue: RefCell::new(Vec::new()),
        }
    }

    /// Register a new component and return its ID.
    pub fn register(&self, parent: Option<ComponentId>) -> ComponentId {
        let mut next_id = self.next_id.borrow_mut();
        let id = ComponentId(*next_id);
        *next_id += 1;

        let component = ManagedComponent::new(id, parent);

        // Add as child to parent
        if let Some(parent_id) = parent
            && let Some(parent_comp) = self.components.borrow_mut().get_mut(&parent_id) {
                parent_comp.children.push(id);
            }

        self.components.borrow_mut().insert(id, component);
        self.init_queue.borrow_mut().push(id);

        id
    }

    /// Get the current state of a component.
    pub fn get_state(&self, id: ComponentId) -> Option<LifecycleState> {
        self.components.borrow().get(&id).map(|c| c.state)
    }

    /// Get the cleanup registry for a component.
    pub fn get_cleanup_registry(&self, id: ComponentId) -> Option<Rc<CleanupRegistry>> {
        self.components.borrow().get(&id).map(|c| c.cleanup_registry.clone())
    }

    /// Transition a component to a new state.
    fn transition(&self, id: ComponentId, new_state: LifecycleState) -> LifecycleResult<()> {
        let mut components = self.components.borrow_mut();
        let component = components
            .get_mut(&id)
            .ok_or(LifecycleError::ComponentNotFound(id))?;

        if !component.state.can_transition_to(new_state) {
            return Err(LifecycleError::InvalidTransition {
                from: component.state,
                to: new_state,
            });
        }

        component.state = new_state;
        Ok(())
    }

    /// Initialize a component through all init lifecycle hooks.
    pub fn initialize<T: Lifecycle>(&self, id: ComponentId, component: &mut T) -> LifecycleResult<()> {
        let cleanup_registry = self
            .get_cleanup_registry(id)
            .ok_or(LifecycleError::ComponentNotFound(id))?;

        // Run initialization within cleanup context
        with_cleanup_context(cleanup_registry, || {
            // OnInit
            self.transition(id, LifecycleState::Initializing)?;
            component.on_init();
            self.transition(id, LifecycleState::Initialized)?;

            // AfterViewInit
            self.transition(id, LifecycleState::ViewInitializing)?;
            component.after_view_init();
            self.transition(id, LifecycleState::ViewInitialized)?;

            // AfterContentInit
            self.transition(id, LifecycleState::ContentInitializing)?;
            component.after_content_init();
            self.transition(id, LifecycleState::ContentInitialized)?;

            // Active
            self.transition(id, LifecycleState::Active)?;

            Ok(())
        })
    }

    /// Run change detection on a component.
    pub fn check<T: Lifecycle>(&self, id: ComponentId, component: &mut T, changes: Option<&Changes>) -> LifecycleResult<()> {
        let state = self.get_state(id).ok_or(LifecycleError::ComponentNotFound(id))?;

        if !state.is_ready() {
            return Ok(()); // Not ready for checking yet
        }

        self.transition(id, LifecycleState::Checking)?;

        // OnChanges if there are changes
        if let Some(changes) = changes {
            component.on_changes(changes);
        }

        // DoCheck
        component.do_check();

        // AfterContentChecked
        component.after_content_checked();

        // AfterViewChecked
        component.after_view_checked();

        self.transition(id, LifecycleState::Active)?;

        Ok(())
    }

    /// Destroy a component and run cleanup.
    pub fn destroy<T: Lifecycle>(&self, id: ComponentId, component: &mut T) -> LifecycleResult<()> {
        let state = self.get_state(id).ok_or(LifecycleError::ComponentNotFound(id))?;

        if state == LifecycleState::Destroyed {
            return Err(LifecycleError::AlreadyDestroyed(id));
        }

        // First destroy all children
        let _children: Vec<ComponentId> = self
            .components
            .borrow()
            .get(&id)
            .map(|c| c.children.clone())
            .unwrap_or_default();

        // Note: Child destruction would need the actual child component references
        // This is a simplified version that just cleans up the tracking

        self.transition(id, LifecycleState::Destroying)?;

        // Run onDestroy hook
        component.on_destroy();

        // Run all registered cleanups
        if let Some(registry) = self.get_cleanup_registry(id) {
            registry.run_all();
        }

        self.transition(id, LifecycleState::Destroyed)?;

        // Remove from parent's children list
        if let Some(parent_id) = self.components.borrow().get(&id).and_then(|c| c.parent)
            && let Some(parent) = self.components.borrow_mut().get_mut(&parent_id) {
                parent.children.retain(|child_id| *child_id != id);
            }

        // Remove the component
        self.components.borrow_mut().remove(&id);

        Ok(())
    }

    /// Queue a component for destruction.
    pub fn queue_destroy(&self, id: ComponentId) {
        self.destroy_queue.borrow_mut().push(id);
    }

    /// Process all queued destructions.
    ///
    /// The callback receives a component ID and should call the appropriate
    /// destroy method on the component.
    pub fn process_destroy_queue<F>(&self, mut destroy_fn: F) -> Vec<LifecycleError>
    where
        F: FnMut(ComponentId, &Self) -> Option<LifecycleError>,
    {
        let mut errors = Vec::new();
        let queue: Vec<_> = self.destroy_queue.borrow_mut().drain(..).collect();

        for id in queue {
            if let Some(err) = destroy_fn(id, self) {
                errors.push(err);
            }
        }

        errors
    }

    /// Get all component IDs.
    pub fn component_ids(&self) -> Vec<ComponentId> {
        self.components.borrow().keys().copied().collect()
    }

    /// Get the number of managed components.
    pub fn component_count(&self) -> usize {
        self.components.borrow().len()
    }

    /// Check if a component exists.
    pub fn exists(&self, id: ComponentId) -> bool {
        self.components.borrow().contains_key(&id)
    }

    /// Get child components of a component.
    pub fn get_children(&self, id: ComponentId) -> Vec<ComponentId> {
        self.components
            .borrow()
            .get(&id)
            .map(|c| c.children.clone())
            .unwrap_or_default()
    }

    /// Get the parent of a component.
    pub fn get_parent(&self, id: ComponentId) -> Option<ComponentId> {
        self.components.borrow().get(&id).and_then(|c| c.parent)
    }
}

impl Default for LifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global lifecycle manager instance.
thread_local! {
    static LIFECYCLE_MANAGER: RefCell<LifecycleManager> = RefCell::new(LifecycleManager::new());
}

/// Get the global lifecycle manager.
pub fn lifecycle_manager() -> &'static std::thread::LocalKey<RefCell<LifecycleManager>> {
    &LIFECYCLE_MANAGER
}

/// Register a component with the global lifecycle manager.
pub fn register_component(parent: Option<ComponentId>) -> ComponentId {
    LIFECYCLE_MANAGER.with(|mgr| mgr.borrow().register(parent))
}

/// Get the state of a component from the global manager.
pub fn get_component_state(id: ComponentId) -> Option<LifecycleState> {
    LIFECYCLE_MANAGER.with(|mgr| mgr.borrow().get_state(id))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestComponent {
        init_called: bool,
        view_init_called: bool,
        content_init_called: bool,
        destroy_called: bool,
    }

    impl TestComponent {
        fn new() -> Self {
            Self {
                init_called: false,
                view_init_called: false,
                content_init_called: false,
                destroy_called: false,
            }
        }
    }

    impl Lifecycle for TestComponent {
        fn on_init(&mut self) {
            self.init_called = true;
        }

        fn after_view_init(&mut self) {
            self.view_init_called = true;
        }

        fn after_content_init(&mut self) {
            self.content_init_called = true;
        }

        fn on_destroy(&mut self) {
            self.destroy_called = true;
        }
    }

    #[test]
    fn test_lifecycle_initialization() {
        let manager = LifecycleManager::new();
        let mut component = TestComponent::new();

        let id = manager.register(None);
        assert_eq!(manager.get_state(id), Some(LifecycleState::Created));

        manager.initialize(id, &mut component).unwrap();

        assert!(component.init_called);
        assert!(component.view_init_called);
        assert!(component.content_init_called);
        assert_eq!(manager.get_state(id), Some(LifecycleState::Active));
    }

    #[test]
    fn test_lifecycle_destruction() {
        let manager = LifecycleManager::new();
        let mut component = TestComponent::new();

        let id = manager.register(None);
        manager.initialize(id, &mut component).unwrap();
        manager.destroy(id, &mut component).unwrap();

        assert!(component.destroy_called);
        assert!(!manager.exists(id));
    }

    #[test]
    fn test_parent_child_relationship() {
        let manager = LifecycleManager::new();

        let parent_id = manager.register(None);
        let child_id = manager.register(Some(parent_id));

        assert_eq!(manager.get_parent(child_id), Some(parent_id));
        assert_eq!(manager.get_children(parent_id), vec![child_id]);
    }
}

