//! Reactive runtime - tracks dependency relationships and manages updates.
//!
//! This module provides the core infrastructure for automatic dependency tracking
//! between signals, computed values, and effects.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::{Rc, Weak};

/// Unique identifier for a reactive node (signal, computed, or effect).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReactiveId(u64);

impl ReactiveId {
    /// Generate a new unique reactive ID.
    pub fn new() -> Self {
        RUNTIME.with(|rt| {
            let mut rt = rt.borrow_mut();
            let id = rt.next_id;
            rt.next_id += 1;
            ReactiveId(id)
        })
    }
}

impl Default for ReactiveId {
    fn default() -> Self {
        Self::new()
    }
}

/// A subscriber that can be notified of changes.
pub trait Subscriber: 'static {
    /// Called when a dependency changes.
    fn notify(&self);

    /// Get the reactive ID of this subscriber.
    fn id(&self) -> ReactiveId;
}

/// Weak reference to a subscriber.
pub type WeakSubscriber = Weak<dyn Subscriber>;

/// The reactive runtime that manages dependency tracking.
pub struct Runtime {
    /// Current tracking context (the effect/computed currently being evaluated).
    tracking_context: Option<Rc<dyn Subscriber>>,
    /// Whether we're currently in a batch.
    batching: bool,
    /// Pending notifications during batch.
    pending_notifications: HashSet<ReactiveId>,
    /// All registered subscribers (weak refs to avoid cycles).
    subscribers: Vec<(ReactiveId, WeakSubscriber)>,
    /// Next ID to assign.
    next_id: u64,
}

impl Runtime {
    fn new() -> Self {
        Self {
            tracking_context: None,
            batching: false,
            pending_notifications: HashSet::new(),
            subscribers: Vec::new(),
            next_id: 0,
        }
    }
}

// Thread-local runtime instance.
thread_local! {
    static RUNTIME: RefCell<Runtime> = RefCell::new(Runtime::new());
}

/// Start tracking dependencies for the given subscriber.
pub fn start_tracking(subscriber: Rc<dyn Subscriber>) {
    RUNTIME.with(|rt| {
        rt.borrow_mut().tracking_context = Some(subscriber);
    });
}

/// Stop tracking dependencies and return the previous context.
pub fn stop_tracking() -> Option<Rc<dyn Subscriber>> {
    RUNTIME.with(|rt| rt.borrow_mut().tracking_context.take())
}

/// Get the current tracking context, if any.
pub fn current_tracking_context() -> Option<Rc<dyn Subscriber>> {
    RUNTIME.with(|rt| rt.borrow().tracking_context.clone())
}

/// Register a subscriber for cleanup.
pub fn register_subscriber(id: ReactiveId, subscriber: WeakSubscriber) {
    RUNTIME.with(|rt| {
        rt.borrow_mut().subscribers.push((id, subscriber));
    });
}

/// Unregister a subscriber.
pub fn unregister_subscriber(id: ReactiveId) {
    RUNTIME.with(|rt| {
        rt.borrow_mut().subscribers.retain(|(sub_id, _)| *sub_id != id);
    });
}

/// Check if we're currently batching updates.
pub fn is_batching() -> bool {
    RUNTIME.with(|rt| rt.borrow().batching)
}

/// Start a batch of updates.
pub fn start_batch() {
    RUNTIME.with(|rt| {
        rt.borrow_mut().batching = true;
    });
}

/// End a batch and flush pending notifications.
pub fn end_batch() {
    // Get pending notifications and subscribers while holding borrow
    let (pending, subscribers_to_notify): (Vec<ReactiveId>, Vec<Rc<dyn Subscriber>>) = RUNTIME.with(|rt| {
        let mut rt = rt.borrow_mut();
        rt.batching = false;

        // Get pending notifications
        let pending: Vec<ReactiveId> = rt.pending_notifications.drain().collect();

        // Collect subscribers that need notification
        let mut to_notify = Vec::new();
        for id in &pending {
            for (sub_id, weak_sub) in &rt.subscribers {
                if *sub_id == *id {
                    if let Some(subscriber) = weak_sub.upgrade() {
                        to_notify.push(subscriber);
                        break;
                    }
                }
            }
        }

        // Cleanup dead weak refs
        rt.subscribers.retain(|(_, weak)| weak.strong_count() > 0);

        (pending, to_notify)
    });

    // Now notify outside the borrow
    for subscriber in subscribers_to_notify {
        subscriber.notify();
    }
}

/// Queue a notification (batched if in batch mode, immediate otherwise).
pub fn queue_notification(id: ReactiveId) {
    RUNTIME.with(|rt| {
        let rt_ref = rt.borrow();
        if rt_ref.batching {
            drop(rt_ref);
            rt.borrow_mut().pending_notifications.insert(id);
        } else {
            // Find and notify immediately
            for (sub_id, weak_sub) in &rt_ref.subscribers {
                if *sub_id == id {
                    if let Some(subscriber) = weak_sub.upgrade() {
                        drop(rt_ref);
                        subscriber.notify();
                        return;
                    }
                }
            }
        }
    });
}

/// Execute a function within a batch context.
pub fn batch<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let was_batching = is_batching();
    if !was_batching {
        start_batch();
    }

    let result = f();

    if !was_batching {
        end_batch();
    }

    result
}

/// Run a function without tracking dependencies.
pub fn untracked<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let prev_context = stop_tracking();
    let result = f();
    if let Some(ctx) = prev_context {
        start_tracking(ctx);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reactive_id() {
        let id1 = ReactiveId::new();
        let id2 = ReactiveId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_batch() {
        assert!(!is_batching());

        batch(|| {
            assert!(is_batching());
        });

        assert!(!is_batching());
    }
}

