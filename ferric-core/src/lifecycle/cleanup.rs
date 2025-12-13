//! Cleanup registration for component destruction.
//!
//! Components can register cleanup callbacks that will be executed
//! when the component is destroyed.

use std::cell::RefCell;
use std::rc::Rc;

/// A cleanup function that will be called when a component is destroyed.
pub type CleanupFn = Box<dyn FnOnce()>;

/// A handle to a registered cleanup function.
/// Dropping the handle does NOT cancel the cleanup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CleanupHandle(u64);

/// Registry for cleanup functions associated with a component.
#[derive(Default)]
pub struct CleanupRegistry {
    cleanups: RefCell<Vec<(CleanupHandle, Option<CleanupFn>)>>,
    next_id: RefCell<u64>,
}

impl CleanupRegistry {
    /// Create a new cleanup registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a cleanup function to be called on destruction.
    ///
    /// Returns a handle that can be used to cancel the cleanup.
    pub fn register<F>(&self, cleanup: F) -> CleanupHandle
    where
        F: FnOnce() + 'static,
    {
        let mut next_id = self.next_id.borrow_mut();
        let handle = CleanupHandle(*next_id);
        *next_id += 1;

        self.cleanups.borrow_mut().push((handle, Some(Box::new(cleanup))));
        handle
    }

    /// Cancel a previously registered cleanup.
    ///
    /// Returns true if the cleanup was found and cancelled.
    pub fn cancel(&self, handle: CleanupHandle) -> bool {
        let mut cleanups = self.cleanups.borrow_mut();
        if let Some(pos) = cleanups.iter().position(|(h, _)| *h == handle) {
            cleanups.remove(pos);
            true
        } else {
            false
        }
    }

    /// Execute all registered cleanups and clear the registry.
    pub fn run_all(&self) {
        let cleanups: Vec<_> = self.cleanups.borrow_mut().drain(..).collect();
        for (_, cleanup) in cleanups {
            if let Some(f) = cleanup {
                f();
            }
        }
    }

    /// Get the number of registered cleanups.
    pub fn len(&self) -> usize {
        self.cleanups.borrow().len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.cleanups.borrow().is_empty()
    }
}

impl std::fmt::Debug for CleanupRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CleanupRegistry")
            .field("count", &self.len())
            .finish()
    }
}

/// A scoped cleanup guard that runs cleanup when dropped.
///
/// Useful for RAII-style resource management.
pub struct CleanupGuard {
    cleanup: Option<CleanupFn>,
}

impl CleanupGuard {
    /// Create a new cleanup guard.
    pub fn new<F>(cleanup: F) -> Self
    where
        F: FnOnce() + 'static,
    {
        Self {
            cleanup: Some(Box::new(cleanup)),
        }
    }

    /// Prevent the cleanup from running.
    pub fn defuse(mut self) {
        self.cleanup = None;
    }
}

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            cleanup();
        }
    }
}

/// Extension trait for registering cleanups on reactive primitives.
pub trait OnCleanup {
    /// Register a cleanup function to run when this is disposed.
    fn on_cleanup<F>(&self, cleanup: F)
    where
        F: FnOnce() + 'static;
}

/// Thread-local cleanup context for the current component.
thread_local! {
    static CLEANUP_CONTEXT: RefCell<Option<Rc<CleanupRegistry>>> = RefCell::new(None);
}

/// Set the cleanup context for the current component.
pub fn set_cleanup_context(registry: Rc<CleanupRegistry>) {
    CLEANUP_CONTEXT.with(|ctx| {
        *ctx.borrow_mut() = Some(registry);
    });
}

/// Clear the cleanup context.
pub fn clear_cleanup_context() {
    CLEANUP_CONTEXT.with(|ctx| {
        *ctx.borrow_mut() = None;
    });
}

/// Register a cleanup function with the current context.
///
/// This is typically called from within a component or effect.
pub fn on_cleanup<F>(cleanup: F)
where
    F: FnOnce() + 'static,
{
    CLEANUP_CONTEXT.with(|ctx| {
        if let Some(registry) = ctx.borrow().as_ref() {
            registry.register(cleanup);
        }
    });
}

/// Run cleanup within a specific context.
pub fn with_cleanup_context<F, R>(registry: Rc<CleanupRegistry>, f: F) -> R
where
    F: FnOnce() -> R,
{
    let previous = CLEANUP_CONTEXT.with(|ctx| ctx.borrow_mut().replace(registry.clone()));

    let result = f();

    CLEANUP_CONTEXT.with(|ctx| {
        *ctx.borrow_mut() = previous;
    });

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn test_cleanup_registry() {
        let registry = CleanupRegistry::new();
        let counter = Rc::new(Cell::new(0));

        let counter1 = counter.clone();
        registry.register(move || {
            counter1.set(counter1.get() + 1);
        });

        let counter2 = counter.clone();
        registry.register(move || {
            counter2.set(counter2.get() + 10);
        });

        assert_eq!(registry.len(), 2);
        assert_eq!(counter.get(), 0);

        registry.run_all();

        assert_eq!(counter.get(), 11);
        assert!(registry.is_empty());
    }

    #[test]
    fn test_cleanup_cancel() {
        let registry = CleanupRegistry::new();
        let counter = Rc::new(Cell::new(0));

        let counter1 = counter.clone();
        let handle = registry.register(move || {
            counter1.set(counter1.get() + 1);
        });

        assert!(registry.cancel(handle));
        assert!(registry.is_empty());

        registry.run_all();
        assert_eq!(counter.get(), 0);
    }

    #[test]
    fn test_cleanup_guard() {
        let counter = Rc::new(Cell::new(0));

        {
            let counter1 = counter.clone();
            let _guard = CleanupGuard::new(move || {
                counter1.set(counter1.get() + 1);
            });
            assert_eq!(counter.get(), 0);
        }

        assert_eq!(counter.get(), 1);
    }

    #[test]
    fn test_cleanup_guard_defuse() {
        let counter = Rc::new(Cell::new(0));

        {
            let counter1 = counter.clone();
            let guard = CleanupGuard::new(move || {
                counter1.set(counter1.get() + 1);
            });
            guard.defuse();
        }

        assert_eq!(counter.get(), 0);
    }
}

