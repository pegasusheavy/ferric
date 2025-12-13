//! Computed values - derived reactive state with automatic dependency tracking.
//!
//! Computed values automatically track which signals they depend on and
//! re-compute when those signals change. They are memoized, meaning they
//! only recompute when necessary.
//!
//! # Example
//!
//! ```ignore
//! use ferric::reactive::*;
//!
//! let first_name = signal("John".to_string());
//! let last_name = signal("Doe".to_string());
//!
//! // Computed automatically tracks first_name and last_name as dependencies
//! let full_name = computed(move || {
//!     format!("{} {}", first_name.get(), last_name.get())
//! });
//!
//! assert_eq!(full_name.get(), "John Doe");
//!
//! // When a dependency changes, the computed value updates
//! first_name.set("Jane".to_string());
//! assert_eq!(full_name.get(), "Jane Doe");
//! ```

use super::runtime::{
    current_tracking_context, register_subscriber, start_tracking, stop_tracking,
    unregister_subscriber, ReactiveId, Subscriber, WeakSubscriber,
};
use super::{next_subscription_id, Reactive, SubscriptionId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// A computed value that derives from other reactive sources.
///
/// Computed values are lazy and memoized. They only recompute when:
/// 1. A dependency changes
/// 2. The value is accessed
pub struct Computed<T> {
    inner: Rc<ComputedInner<T>>,
}

struct ComputedInner<T> {
    /// The computation function.
    compute: Box<dyn Fn() -> T>,
    /// Unique identifier.
    id: ReactiveId,
    /// Mutable state.
    state: RefCell<ComputedState<T>>,
}

struct ComputedState<T> {
    /// Cached value.
    cached: Option<T>,
    /// Whether the value needs recomputation.
    dirty: bool,
    /// Legacy callback subscribers.
    subscribers: HashMap<SubscriptionId, Box<dyn Fn(&T)>>,
    /// Reactive subscribers.
    reactive_subscribers: Vec<WeakSubscriber>,
}

impl<T: Clone + 'static> Computed<T> {
    /// Create a new computed value from a computation function.
    ///
    /// The computation function will be called lazily when the value is first
    /// accessed, and will automatically track any signals read during computation.
    pub fn new<F>(compute: F) -> Self
    where
        F: Fn() -> T + 'static,
    {
        let inner = Rc::new(ComputedInner {
            compute: Box::new(compute),
            id: ReactiveId::new(),
            state: RefCell::new(ComputedState {
                cached: None,
                dirty: true,
                subscribers: HashMap::new(),
                reactive_subscribers: Vec::new(),
            }),
        });

        // Register this computed as a subscriber so it can be notified
        register_subscriber(inner.id, Rc::downgrade(&inner) as WeakSubscriber);

        Computed { inner }
    }

    /// Get the current computed value.
    ///
    /// If the value is dirty (a dependency changed) or hasn't been computed yet,
    /// this will trigger a recomputation. Otherwise, returns the cached value.
    pub fn get(&self) -> T {
        // Track this computed as a dependency if we're in a tracking context
        if let Some(subscriber) = current_tracking_context() {
            let mut state = self.inner.state.borrow_mut();
            let already_subscribed = state.reactive_subscribers.iter().any(|weak| {
                weak.upgrade().map(|s| s.id() == subscriber.id()).unwrap_or(false)
            });
            if !already_subscribed {
                state.reactive_subscribers.push(Rc::downgrade(&subscriber) as WeakSubscriber);
            }
        }

        self.compute_if_needed()
    }

    /// Get the current value without tracking dependencies.
    pub fn get_untracked(&self) -> T {
        self.compute_if_needed()
    }

    /// Compute the value if needed.
    fn compute_if_needed(&self) -> T {
        let needs_compute = {
            let state = self.inner.state.borrow();
            state.dirty || state.cached.is_none()
        };

        if needs_compute {
            // Start tracking dependencies for this computed
            let self_as_subscriber: Rc<dyn Subscriber> = Rc::clone(&self.inner) as Rc<dyn Subscriber>;
            let prev_context = stop_tracking();
            start_tracking(self_as_subscriber);

            // Compute the new value
            let value = (self.inner.compute)();

            // Restore previous tracking context
            stop_tracking();
            if let Some(ctx) = prev_context {
                start_tracking(ctx);
            }

            // Cache the value
            let mut state = self.inner.state.borrow_mut();
            state.cached = Some(value.clone());
            state.dirty = false;

            value
        } else {
            self.inner.state.borrow().cached.clone().unwrap()
        }
    }

    /// Mark the computed value as dirty, requiring recomputation.
    pub fn invalidate(&self) {
        let mut state = self.inner.state.borrow_mut();
        state.dirty = true;

        // Collect subscribers before notifying
        let subscribers: Vec<Rc<dyn Subscriber>> = state
            .reactive_subscribers
            .iter()
            .filter_map(|weak| weak.upgrade())
            .collect();

        // Also notify legacy subscribers with cached value
        if let Some(ref value) = state.cached {
            let value = value.clone();
            for callback in state.subscribers.values() {
                callback(&value);
            }
        }

        drop(state); // Release borrow before notifying

        // Notify reactive subscribers
        for subscriber in subscribers {
            subscriber.notify();
        }

        // Cleanup dead weak refs
        self.inner.state.borrow_mut().reactive_subscribers.retain(|weak| weak.strong_count() > 0);
    }

    /// Get the reactive ID.
    pub fn id(&self) -> ReactiveId {
        self.inner.id
    }
}

impl<T: Clone + 'static> Subscriber for ComputedInner<T> {
    fn notify(&self) {
        // Mark as dirty and propagate to our subscribers
        let mut state = self.state.borrow_mut();
        state.dirty = true;

        // Collect subscribers
        let subscribers: Vec<Rc<dyn Subscriber>> = state
            .reactive_subscribers
            .iter()
            .filter_map(|weak| weak.upgrade())
            .collect();

        drop(state);

        for subscriber in subscribers {
            subscriber.notify();
        }
    }

    fn id(&self) -> ReactiveId {
        self.id
    }
}

impl<T: Clone + 'static> Reactive<T> for Computed<T> {
    fn get(&self) -> T {
        Computed::get(self)
    }

    fn subscribe(&self, callback: Box<dyn Fn(&T)>) -> SubscriptionId {
        let id = next_subscription_id();
        self.inner.state.borrow_mut().subscribers.insert(id, callback);
        id
    }

    fn unsubscribe(&self, id: SubscriptionId) {
        self.inner.state.borrow_mut().subscribers.remove(&id);
    }
}

impl<T> Clone for Computed<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<T> Drop for Computed<T> {
    fn drop(&mut self) {
        // Only unregister if we're the last strong reference
        if Rc::strong_count(&self.inner) == 1 {
            unregister_subscriber(self.inner.id);
        }
    }
}

impl<T: std::fmt::Debug + Clone + 'static> std::fmt::Debug for Computed<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Computed")
            .field("value", &self.inner.state.borrow().cached)
            .field("dirty", &self.inner.state.borrow().dirty)
            .finish()
    }
}

/// Create a new computed value from a computation function.
///
/// This is the primary way to create derived reactive state.
///
/// # Example
///
/// ```ignore
/// let count = signal(1);
/// let doubled = computed(move || count.get() * 2);
///
/// assert_eq!(doubled.get(), 2);
/// count.set(5);
/// assert_eq!(doubled.get(), 10);
/// ```
pub fn computed<T: Clone + 'static, F>(compute: F) -> Computed<T>
where
    F: Fn() -> T + 'static,
{
    Computed::new(compute)
}

/// Create a computed value with explicit dependencies (memo pattern).
///
/// This is useful when you want to control exactly when recomputation happens.
///
/// # Example
///
/// ```ignore
/// let items = signal(vec![1, 2, 3, 4, 5]);
/// let expensive_sum = memo(
///     move || items.get(),
///     |items| items.iter().sum::<i32>()
/// );
/// ```
pub fn memo<T, D, F>(deps: D, compute: F) -> Computed<T>
where
    T: Clone + 'static,
    D: Fn() -> T + 'static,
    F: Fn(&T) -> T + 'static,
{
    computed(move || {
        let deps_value = deps();
        compute(&deps_value)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reactive::signal;

    #[test]
    fn test_computed_basic() {
        let c = computed(|| 42);
        assert_eq!(c.get(), 42);
    }

    #[test]
    fn test_computed_memoization() {
        use std::cell::Cell;
        use std::rc::Rc;

        let call_count = Rc::new(Cell::new(0));
        let call_count_clone = Rc::clone(&call_count);

        let c = computed(move || {
            call_count_clone.set(call_count_clone.get() + 1);
            42
        });

        // First call computes
        assert_eq!(c.get(), 42);
        assert_eq!(call_count.get(), 1);

        // Second call uses cache
        assert_eq!(c.get(), 42);
        assert_eq!(call_count.get(), 1);
    }

    #[test]
    fn test_computed_invalidation() {
        use std::cell::Cell;
        use std::rc::Rc;

        let call_count = Rc::new(Cell::new(0));
        let call_count_clone = Rc::clone(&call_count);

        let c = computed(move || {
            call_count_clone.set(call_count_clone.get() + 1);
            42
        });

        assert_eq!(c.get(), 42);
        assert_eq!(call_count.get(), 1);

        c.invalidate();

        assert_eq!(c.get(), 42);
        assert_eq!(call_count.get(), 2);
    }
}
