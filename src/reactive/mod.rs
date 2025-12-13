//! Reactive state management for Ferric.
//!
//! Provides signals, computed values, and effects for fine-grained reactivity.

mod computed;
mod effect;
mod signal;

pub use computed::*;
pub use effect::*;
pub use signal::*;

use std::cell::RefCell;

/// Trait for reactive values that can notify subscribers of changes.
pub trait Reactive<T> {
    /// Get the current value.
    fn get(&self) -> T;

    /// Subscribe to value changes.
    fn subscribe(&self, callback: Box<dyn Fn(&T)>) -> SubscriptionId;

    /// Unsubscribe from value changes.
    fn unsubscribe(&self, id: SubscriptionId);
}

/// Unique identifier for a subscription.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(pub(crate) u64);

// Global subscription counter.
thread_local! {
    static NEXT_SUBSCRIPTION_ID: RefCell<u64> = RefCell::new(0);
}

// Generate a new unique subscription ID.
pub(crate) fn next_subscription_id() -> SubscriptionId {
    NEXT_SUBSCRIPTION_ID.with(|id| {
        let current = *id.borrow();
        *id.borrow_mut() = current + 1;
        SubscriptionId(current)
    })
}

/// Batch multiple reactive updates into a single change detection cycle.
pub fn batch<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    // TODO: Implement batching logic
    f()
}

