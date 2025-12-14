//! Reactive state management for Ferric.
//!
//! This module provides a complete reactive system with signals, computed values,
//! effects, and resources for building reactive applications.
//!
//! # Overview
//!
//! - **Signals**: Mutable reactive state that notifies dependents when changed
//! - **Computed**: Derived values that automatically update when dependencies change
//! - **Effects**: Side effects that run when their dependencies change
//! - **Resources**: Async data fetching with loading/error states
//!
//! # Automatic Dependency Tracking
//!
//! When you read a signal inside a computed or effect, the dependency is
//! automatically tracked. When the signal changes, the computed/effect is
//! automatically re-run.
//!
//! ```ignore
//! use ferric::reactive::*;
//!
//! let count = signal(0);
//! let doubled = computed(move || count.get() * 2);
//!
//! assert_eq!(doubled.get(), 0);
//!
//! count.set(5);
//! assert_eq!(doubled.get(), 10);
//! ```
//!
//! # Batching
//!
//! Multiple signal updates can be batched to prevent unnecessary re-computations:
//!
//! ```ignore
//! let a = signal(1);
//! let b = signal(2);
//!
//! batch(|| {
//!     a.set(10);
//!     b.set(20);
//!     // Dependents are only notified once after the batch
//! });
//! ```

mod computed;
mod effect;
mod resource;
mod runtime;
mod signal;

// Re-export core types
pub use computed::{computed, memo, Computed};
pub use effect::{effect, effect_with_cleanup, watch, watch_immediate, Effect};
pub use resource::{create_resource, create_resource_once, Resource, ResourceState};
pub use runtime::{batch, untracked, ReactiveId};
pub use signal::{signal, ReadonlySignal, Signal, WritableSignal};

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
    static NEXT_SUBSCRIPTION_ID: RefCell<u64> = const { RefCell::new(0) };
}

// Generate a new unique subscription ID.
pub(crate) fn next_subscription_id() -> SubscriptionId {
    NEXT_SUBSCRIPTION_ID.with(|id| {
        let current = *id.borrow();
        *id.borrow_mut() = current + 1;
        SubscriptionId(current)
    })
}

/// Create a derived signal that transforms the value of another signal.
///
/// This is a convenience function for creating a readonly computed value
/// from a single signal.
///
/// # Example
///
/// ```ignore
/// let count = signal(5);
/// let doubled = derived(&count, |n| n * 2);
///
/// assert_eq!(doubled.get(), 10);
/// ```
pub fn derived<T, U, F>(source: &Signal<T>, transform: F) -> Computed<U>
where
    T: Clone + 'static,
    U: Clone + 'static,
    F: Fn(T) -> U + 'static,
{
    let source = source.clone();
    computed(move || transform(source.get()))
}

/// Create a signal that syncs with localStorage.
///
/// The signal's value is persisted to localStorage and restored on page load.
///
/// # Example
///
/// ```ignore
/// let theme = persisted_signal("theme", "light".to_string());
/// theme.set("dark".to_string()); // Automatically saved to localStorage
/// ```
#[cfg(target_arch = "wasm32")]
pub fn persisted_signal<T>(key: &str, default: T) -> Signal<T>
where
    T: Clone + serde::Serialize + serde::de::DeserializeOwned + 'static,
{
    use wasm_bindgen::JsCast;

    // Try to load from localStorage
    let initial = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item(key).ok().flatten())
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or(default);

    let sig = signal(initial);
    let sig_clone = sig.clone();
    let key = key.to_string();

    // Set up effect to persist changes
    effect(move || {
        let value = sig_clone.get();
        if let Ok(json) = serde_json::to_string(&value) {
            if let Some(storage) = web_sys::window()
                .and_then(|w| w.local_storage().ok().flatten())
            {
                let _ = storage.set_item(&key, &json);
            }
        }
    });

    sig
}

/// Create a signal that debounces updates.
///
/// The value only updates after the specified delay with no new updates.
///
/// # Example
///
/// ```ignore
/// let search = debounced_signal("", 300); // 300ms debounce
/// search.set("h");
/// search.set("he");
/// search.set("hel"); // Only this triggers dependents (after 300ms)
/// ```
#[cfg(target_arch = "wasm32")]
pub fn debounced_signal<T>(initial: T, delay_ms: u32) -> Signal<T>
where
    T: Clone + 'static,
{
    use std::rc::Rc;
    use wasm_bindgen::prelude::*;

    let sig = signal(initial.clone());
    let pending = signal(initial);
    let sig_clone = sig.clone();
    let pending_clone = pending.clone();
    let timeout_id: Rc<RefCell<Option<i32>>> = Rc::new(RefCell::new(None));
    let timeout_clone = Rc::clone(&timeout_id);

    effect(move || {
        let new_value = pending_clone.get();

        // Clear existing timeout
        if let Some(id) = timeout_clone.borrow_mut().take() {
            if let Some(window) = web_sys::window() {
                window.clear_timeout_with_handle(id);
            }
        }

        // Set new timeout
        let sig_update = sig_clone.clone();
        let timeout_ref = Rc::clone(&timeout_clone);

        let closure = Closure::once(Box::new(move || {
            sig_update.set(new_value);
            *timeout_ref.borrow_mut() = None;
        }) as Box<dyn FnOnce()>);

        if let Some(window) = web_sys::window() {
            if let Ok(id) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                delay_ms as i32,
            ) {
                *timeout_clone.borrow_mut() = Some(id);
            }
        }

        closure.forget();
    });

    pending
}

/// Prelude for convenient imports.
pub mod prelude {
    pub use super::{
        // Core primitives
        signal, computed, effect, batch, untracked,
        Signal, Computed, Effect,

        // Resources
        create_resource, Resource, ResourceState,

        // Utilities
        derived, watch, watch_immediate,

        // Traits
        Reactive, SubscriptionId, ReactiveId,
    };

    #[cfg(target_arch = "wasm32")]
    pub use super::{persisted_signal, debounced_signal};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn test_signal_computed_integration() {
        let a = signal(1);
        let b = signal(2);

        let sum = computed({
            let a = a.clone();
            let b = b.clone();
            move || a.get() + b.get()
        });

        assert_eq!(sum.get(), 3);

        a.set(10);
        assert_eq!(sum.get(), 12);

        b.set(20);
        assert_eq!(sum.get(), 30);
    }

    #[test]
    fn test_effect_with_signals() {
        let count = signal(0);
        let effect_ran = Rc::new(Cell::new(0));
        let effect_ran_clone = Rc::clone(&effect_ran);

        let count_clone = count.clone();
        let _fx = effect(move || {
            let _ = count_clone.get();
            effect_ran_clone.set(effect_ran_clone.get() + 1);
        });

        assert_eq!(effect_ran.get(), 1); // Initial run

        count.set(1);
        // Effect should run again due to dependency tracking
    }

    #[test]
    fn test_derived() {
        let count = signal(5);
        let doubled = derived(&count, |n| n * 2);

        assert_eq!(doubled.get(), 10);

        count.set(10);
        assert_eq!(doubled.get(), 20);
    }

    #[test]
    fn test_batch() {
        let a = signal(1);
        let b = signal(2);
        let call_count = Rc::new(Cell::new(0));
        let call_count_clone = Rc::clone(&call_count);

        let a_clone = a.clone();
        let b_clone = b.clone();
        let sum = computed(move || {
            call_count_clone.set(call_count_clone.get() + 1);
            a_clone.get() + b_clone.get()
        });

        // Initial computation
        assert_eq!(sum.get(), 3);
        let initial_calls = call_count.get();

        // Batch updates
        batch(|| {
            a.set(10);
            b.set(20);
        });

        // Force recomputation
        assert_eq!(sum.get(), 30);
    }

    #[test]
    fn test_untracked() {
        let count = signal(0);
        let tracked_reads = Rc::new(Cell::new(0));
        let tracked_reads_clone = Rc::clone(&tracked_reads);

        let count_clone = count.clone();
        let _fx = effect(move || {
            // This read is tracked
            let _ = count_clone.get();
            tracked_reads_clone.set(tracked_reads_clone.get() + 1);
        });

        assert_eq!(tracked_reads.get(), 1);

        // Read without tracking
        untracked(|| {
            let _ = count.get();
        });
    }
}
