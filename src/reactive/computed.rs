//! Computed values - derived reactive state.

use super::{next_subscription_id, Reactive, SubscriptionId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// A computed value that derives from other reactive sources.
pub struct Computed<T> {
    inner: Rc<RefCell<ComputedInner<T>>>,
}

struct ComputedInner<T> {
    compute: Box<dyn Fn() -> T>,
    cached: Option<T>,
    dirty: bool,
    subscribers: HashMap<SubscriptionId, Box<dyn Fn(&T)>>,
}

impl<T: Clone> Computed<T> {
    /// Create a new computed value from a computation function.
    pub fn new<F>(compute: F) -> Self
    where
        F: Fn() -> T + 'static,
    {
        Self {
            inner: Rc::new(RefCell::new(ComputedInner {
                compute: Box::new(compute),
                cached: None,
                dirty: true,
                subscribers: HashMap::new(),
            })),
        }
    }

    /// Get the current computed value.
    pub fn get(&self) -> T {
        let mut inner = self.inner.borrow_mut();

        if inner.dirty || inner.cached.is_none() {
            let value = (inner.compute)();
            inner.cached = Some(value);
            inner.dirty = false;
        }

        inner.cached.clone().unwrap()
    }

    /// Mark the computed value as dirty, requiring recomputation.
    pub fn invalidate(&self) {
        let mut inner = self.inner.borrow_mut();
        inner.dirty = true;

        // Notify subscribers
        if let Some(ref value) = inner.cached {
            let value = value.clone();
            for callback in inner.subscribers.values() {
                callback(&value);
            }
        }
    }
}

impl<T: Clone> Reactive<T> for Computed<T> {
    fn get(&self) -> T {
        Computed::get(self)
    }

    fn subscribe(&self, callback: Box<dyn Fn(&T)>) -> SubscriptionId {
        let id = next_subscription_id();
        self.inner.borrow_mut().subscribers.insert(id, callback);
        id
    }

    fn unsubscribe(&self, id: SubscriptionId) {
        self.inner.borrow_mut().subscribers.remove(&id);
    }
}

impl<T> Clone for Computed<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

/// Create a new computed value from a computation function.
pub fn computed<T: Clone, F>(compute: F) -> Computed<T>
where
    F: Fn() -> T + 'static,
{
    Computed::new(compute)
}

