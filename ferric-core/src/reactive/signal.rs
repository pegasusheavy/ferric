//! Signals - reactive primitive for mutable state.
//!
//! Signals are the core reactive primitive in Ferric. They hold a value and
//! automatically notify subscribers when the value changes.
//!
//! # Example
//!
//! ```ignore
//! use ferric::reactive::*;
//!
//! let count = signal(0);
//!
//! // Read the value
//! assert_eq!(count.get(), 0);
//!
//! // Update the value
//! count.set(1);
//! assert_eq!(count.get(), 1);
//!
//! // Update using a function
//! count.update(|n| n + 1);
//! assert_eq!(count.get(), 2);
//! ```

use super::runtime::{
    current_tracking_context, is_batching, queue_notification,
    ReactiveId, Subscriber, WeakSubscriber,
};
use super::{next_subscription_id, Reactive, SubscriptionId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

/// A reactive signal that holds a value and notifies subscribers when it changes.
///
/// Signals are the fundamental building blocks of reactivity. When you read a signal
/// inside an effect or computed, it automatically tracks the dependency.
pub struct Signal<T> {
    inner: Rc<RefCell<SignalInner<T>>>,
}

struct SignalInner<T> {
    /// The current value.
    value: T,
    /// Unique identifier for this signal.
    id: ReactiveId,
    /// Legacy callback subscribers.
    subscribers: HashMap<SubscriptionId, Box<dyn Fn(&T)>>,
    /// Weak references to reactive subscribers (effects/computed).
    reactive_subscribers: Vec<WeakSubscriber>,
}

impl<T: Clone> Signal<T> {
    /// Create a new signal with an initial value.
    pub fn new(value: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(SignalInner {
                value,
                id: ReactiveId::new(),
                subscribers: HashMap::new(),
                reactive_subscribers: Vec::new(),
            })),
        }
    }

    /// Get the reactive ID of this signal.
    pub fn id(&self) -> ReactiveId {
        self.inner.borrow().id
    }

    /// Get the current value.
    ///
    /// If called within an effect or computed context, this will automatically
    /// register the signal as a dependency.
    pub fn get(&self) -> T {
        // Track this signal as a dependency if we're in a tracking context
        if let Some(subscriber) = current_tracking_context() {
            let mut inner = self.inner.borrow_mut();
            // Check if already subscribed
            let already_subscribed = inner.reactive_subscribers.iter().any(|weak| {
                weak.upgrade().map(|s| s.id() == subscriber.id()).unwrap_or(false)
            });
            if !already_subscribed {
                inner.reactive_subscribers.push(Rc::downgrade(&subscriber) as WeakSubscriber);
            }
        }

        self.inner.borrow().value.clone()
    }

    /// Get the current value without tracking dependencies.
    ///
    /// Use this when you need to read the value but don't want to create a dependency.
    pub fn get_untracked(&self) -> T {
        self.inner.borrow().value.clone()
    }

    /// Set a new value and notify subscribers.
    ///
    /// If called within a `batch()`, notifications are deferred until the batch ends.
    pub fn set(&self, value: T) {
        {
            let mut inner = self.inner.borrow_mut();
            inner.value = value;
        }
        self.notify_subscribers();
    }

    /// Set a new value only if it's different from the current value.
    ///
    /// Returns `true` if the value was changed.
    pub fn set_if_changed(&self, value: T) -> bool
    where
        T: PartialEq,
    {
        let should_update = {
            let inner = self.inner.borrow();
            inner.value != value
        };

        if should_update {
            self.set(value);
            true
        } else {
            false
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
        {
            let mut inner = self.inner.borrow_mut();
            f(&mut inner.value);
        }
        self.notify_subscribers();
    }

    /// Get a readonly view of this signal.
    pub fn readonly(&self) -> ReadonlySignal<T> {
        ReadonlySignal {
            inner: Rc::downgrade(&self.inner),
        }
    }

    /// Notify all subscribers of a change.
    fn notify_subscribers(&self) {
        let inner = self.inner.borrow();

        // Notify legacy callback subscribers
        if !is_batching() {
            let value_ref = &inner.value;
            for callback in inner.subscribers.values() {
                callback(value_ref);
            }
        }

        // Notify reactive subscribers
        let live_subscribers: Vec<Rc<dyn Subscriber>> = inner
            .reactive_subscribers
            .iter()
            .filter_map(|weak| weak.upgrade())
            .collect();

        drop(inner); // Release borrow before calling notify

        for subscriber in live_subscribers {
            if is_batching() {
                queue_notification(subscriber.id());
            } else {
                subscriber.notify();
            }
        }

        // Cleanup dead weak refs
        self.inner.borrow_mut().reactive_subscribers.retain(|weak| weak.strong_count() > 0);
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

impl<T: Default + Clone> Default for Signal<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: std::fmt::Debug + Clone> std::fmt::Debug for Signal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Signal")
            .field("value", &self.inner.borrow().value)
            .finish()
    }
}

/// A readonly view of a signal.
///
/// This can be shared without allowing modification of the underlying value.
pub struct ReadonlySignal<T> {
    inner: Weak<RefCell<SignalInner<T>>>,
}

impl<T: Clone> ReadonlySignal<T> {
    /// Get the current value.
    pub fn get(&self) -> Option<T> {
        self.inner.upgrade().map(|inner| {
            // Track dependency if in tracking context
            if let Some(subscriber) = current_tracking_context() {
                let mut inner_ref = inner.borrow_mut();
                let already_subscribed = inner_ref.reactive_subscribers.iter().any(|weak| {
                    weak.upgrade().map(|s| s.id() == subscriber.id()).unwrap_or(false)
                });
                if !already_subscribed {
                    inner_ref.reactive_subscribers.push(Rc::downgrade(&subscriber) as WeakSubscriber);
                }
            }
            inner.borrow().value.clone()
        })
    }
}

impl<T> Clone for ReadonlySignal<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Weak::clone(&self.inner),
        }
    }
}

/// A writable signal type alias for clarity.
pub type WritableSignal<T> = Signal<T>;

/// Create a new signal with the given initial value.
///
/// This is the primary way to create reactive state in Ferric.
///
/// # Example
///
/// ```ignore
/// let count = signal(0);
/// let name = signal("Ferric".to_string());
/// let items = signal(vec![1, 2, 3]);
/// ```
pub fn signal<T: Clone>(value: T) -> Signal<T> {
    Signal::new(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_basic() {
        let s = signal(0);
        assert_eq!(s.get(), 0);

        s.set(42);
        assert_eq!(s.get(), 42);
    }

    #[test]
    fn test_signal_update() {
        let s = signal(10);
        s.update(|n| n * 2);
        assert_eq!(s.get(), 20);
    }

    #[test]
    fn test_signal_mutate() {
        let s = signal(vec![1, 2, 3]);
        s.mutate(|v| v.push(4));
        assert_eq!(s.get(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_signal_set_if_changed() {
        let s = signal(10);

        assert!(!s.set_if_changed(10)); // Same value
        assert!(s.set_if_changed(20));  // Different value
        assert_eq!(s.get(), 20);
    }

    #[test]
    fn test_signal_subscribe() {
        use std::cell::Cell;
        use std::rc::Rc;

        let s = signal(0);
        let called = Rc::new(Cell::new(0));
        let called_clone = Rc::clone(&called);

        let _sub = s.subscribe(Box::new(move |_| {
            called_clone.set(called_clone.get() + 1);
        }));

        s.set(1);
        assert_eq!(called.get(), 1);

        s.set(2);
        assert_eq!(called.get(), 2);
    }
}
