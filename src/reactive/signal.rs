//! Signals - reactive primitive for mutable state.

use super::{next_subscription_id, Reactive, SubscriptionId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// A reactive signal that holds a value and notifies subscribers when it changes.
pub struct Signal<T> {
    inner: Rc<RefCell<SignalInner<T>>>,
}

struct SignalInner<T> {
    value: T,
    subscribers: HashMap<SubscriptionId, Box<dyn Fn(&T)>>,
}

impl<T: Clone> Signal<T> {
    /// Create a new signal with an initial value.
    pub fn new(value: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(SignalInner {
                value,
                subscribers: HashMap::new(),
            })),
        }
    }

    /// Get the current value.
    pub fn get(&self) -> T {
        self.inner.borrow().value.clone()
    }

    /// Set a new value and notify subscribers.
    pub fn set(&self, value: T) {
        let mut inner = self.inner.borrow_mut();
        inner.value = value;
        let value_ref = &inner.value;
        for callback in inner.subscribers.values() {
            callback(value_ref);
        }
    }

    /// Update the value using a function.
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&T) -> T,
    {
        let new_value = {
            let inner = self.inner.borrow();
            f(&inner.value)
        };
        self.set(new_value);
    }

    /// Mutate the value in place and notify subscribers.
    pub fn mutate<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        let mut inner = self.inner.borrow_mut();
        f(&mut inner.value);
        let value_ref = &inner.value;
        for callback in inner.subscribers.values() {
            callback(value_ref);
        }
    }
}

impl<T: Clone> Reactive<T> for Signal<T> {
    fn get(&self) -> T {
        Signal::get(self)
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

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

/// Create a new signal with the given initial value.
pub fn signal<T: Clone>(value: T) -> Signal<T> {
    Signal::new(value)
}

