//! Effects - side effects that run when dependencies change.
//!
//! Effects automatically track which signals they depend on and re-run
//! when those signals change. They're ideal for syncing reactive state
//! with external systems (DOM, localStorage, network, etc.).
//!
//! # Example
//!
//! ```ignore
//! use ferric::reactive::*;
//!
//! let count = signal(0);
//!
//! // Effect runs immediately and when count changes
//! let fx = effect(move || {
//!     web_sys::console::log_1(&format!("Count is: {}", count.get()).into());
//! });
//!
//! count.set(1); // Logs "Count is: 1"
//! count.set(2); // Logs "Count is: 2"
//!
//! fx.stop(); // No longer runs when count changes
//! ```

use super::runtime::{
    register_subscriber, start_tracking, stop_tracking, unregister_subscriber,
    ReactiveId, Subscriber, WeakSubscriber,
};
use super::SubscriptionId;
use std::cell::RefCell;
use std::rc::Rc;

/// An effect that runs when its dependencies change.
///
/// Effects are used for side effects like logging, DOM manipulation,
/// or syncing with external state. They automatically track dependencies
/// when signals are read inside the effect function.
pub struct Effect {
    inner: Rc<EffectInner>,
}

struct EffectInner {
    /// Unique identifier.
    id: ReactiveId,
    /// Mutable state.
    state: RefCell<EffectState>,
}

struct EffectState {
    /// The effect function.
    effect_fn: Box<dyn Fn()>,
    /// Optional cleanup function called before re-running.
    cleanup_fn: Option<Box<dyn Fn()>>,
    /// Whether the effect is active.
    active: bool,
    /// Legacy subscriptions.
    subscriptions: Vec<SubscriptionId>,
    /// Is currently running (to prevent infinite loops).
    running: bool,
}

impl Effect {
    /// Create a new effect with the given function.
    ///
    /// The effect runs immediately and then re-runs whenever its
    /// dependencies change.
    pub fn new<F>(effect_fn: F) -> Self
    where
        F: Fn() + 'static,
    {
        let inner = Rc::new(EffectInner {
            id: ReactiveId::new(),
            state: RefCell::new(EffectState {
                effect_fn: Box::new(effect_fn),
                cleanup_fn: None,
                active: true,
                subscriptions: Vec::new(),
                running: false,
            }),
        });

        // Register this effect as a subscriber
        register_subscriber(inner.id, Rc::downgrade(&inner) as WeakSubscriber);

        let effect = Effect { inner };

        // Run the effect immediately
        effect.run();

        effect
    }

    /// Create an effect with a cleanup function.
    ///
    /// The cleanup function is called before the effect re-runs and when
    /// the effect is stopped. Use it to clean up resources like event
    /// listeners or timers.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let fx = Effect::with_cleanup(
    ///     move || {
    ///         // Setup
    ///         let value = count.get();
    ///         start_animation(value);
    ///     },
    ///     || {
    ///         // Cleanup
    ///         stop_animation();
    ///     }
    /// );
    /// ```
    pub fn with_cleanup<F, C>(effect_fn: F, cleanup_fn: C) -> Self
    where
        F: Fn() + 'static,
        C: Fn() + 'static,
    {
        let inner = Rc::new(EffectInner {
            id: ReactiveId::new(),
            state: RefCell::new(EffectState {
                effect_fn: Box::new(effect_fn),
                cleanup_fn: Some(Box::new(cleanup_fn)),
                active: true,
                subscriptions: Vec::new(),
                running: false,
            }),
        });

        register_subscriber(inner.id, Rc::downgrade(&inner) as WeakSubscriber);

        let effect = Effect { inner };
        effect.run();
        effect
    }

    /// Run the effect, tracking dependencies.
    pub fn run(&self) {
        let should_run = {
            let state = self.inner.state.borrow();
            state.active && !state.running
        };

        if !should_run {
            return;
        }

        // Mark as running to prevent infinite loops
        self.inner.state.borrow_mut().running = true;

        // Run cleanup if we have one
        {
            let state = self.inner.state.borrow();
            if let Some(ref cleanup) = state.cleanup_fn {
                cleanup();
            }
        }

        // Start tracking dependencies
        let self_as_subscriber: Rc<dyn Subscriber> = Rc::clone(&self.inner) as Rc<dyn Subscriber>;
        let prev_context = stop_tracking();
        start_tracking(self_as_subscriber);

        // Run the effect
        {
            let state = self.inner.state.borrow();
            (state.effect_fn)();
        }

        // Restore previous tracking context
        stop_tracking();
        if let Some(ctx) = prev_context {
            start_tracking(ctx);
        }

        // Mark as not running
        self.inner.state.borrow_mut().running = false;
    }

    /// Stop the effect from running.
    ///
    /// This calls the cleanup function if one was provided and prevents
    /// the effect from running again.
    pub fn stop(&self) {
        let mut state = self.inner.state.borrow_mut();
        if state.active {
            state.active = false;
            if let Some(ref cleanup) = state.cleanup_fn {
                cleanup();
            }
        }
    }

    /// Resume a stopped effect.
    pub fn resume(&self) {
        {
            let mut state = self.inner.state.borrow_mut();
            if !state.active {
                state.active = true;
            }
        }
        self.run();
    }

    /// Check if the effect is active.
    pub fn is_active(&self) -> bool {
        self.inner.state.borrow().active
    }

    /// Get the reactive ID.
    pub fn id(&self) -> ReactiveId {
        self.inner.id
    }
}

impl Subscriber for EffectInner {
    fn notify(&self) {
        let should_run = {
            let state = self.state.borrow();
            state.active && !state.running
        };

        if !should_run {
            return;
        }

        // Mark as running
        self.state.borrow_mut().running = true;

        // Run cleanup
        {
            let state = self.state.borrow();
            if let Some(ref cleanup) = state.cleanup_fn {
                cleanup();
            }
        }

        // Run effect
        {
            let state = self.state.borrow();
            (state.effect_fn)();
        }

        // Mark as not running
        self.state.borrow_mut().running = false;
    }

    fn id(&self) -> ReactiveId {
        self.id
    }
}

impl Drop for Effect {
    fn drop(&mut self) {
        self.stop();
        // Only unregister if we're the last strong reference
        if Rc::strong_count(&self.inner) == 1 {
            unregister_subscriber(self.inner.id);
        }
    }
}

impl Clone for Effect {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl std::fmt::Debug for Effect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Effect")
            .field("id", &self.inner.id)
            .field("active", &self.inner.state.borrow().active)
            .finish()
    }
}

/// Create a new effect that runs immediately and when dependencies change.
///
/// This is the primary way to create side effects that react to signal changes.
///
/// # Example
///
/// ```ignore
/// let count = signal(0);
///
/// effect(move || {
///     println!("Count changed to: {}", count.get());
/// });
///
/// count.set(1); // Prints "Count changed to: 1"
/// ```
pub fn effect<F>(f: F) -> Effect
where
    F: Fn() + 'static,
{
    Effect::new(f)
}

/// Create an effect with a cleanup function.
///
/// The cleanup runs before each re-execution and when the effect is stopped.
///
/// # Example
///
/// ```ignore
/// let visible = signal(true);
///
/// effect_with_cleanup(
///     move || {
///         if visible.get() {
///             show_modal();
///         }
///     },
///     || hide_modal()
/// );
/// ```
pub fn effect_with_cleanup<F, C>(effect_fn: F, cleanup_fn: C) -> Effect
where
    F: Fn() + 'static,
    C: Fn() + 'static,
{
    Effect::with_cleanup(effect_fn, cleanup_fn)
}

/// Watch specific values and run a callback when they change.
///
/// Unlike `effect`, `watch` doesn't run immediately. It only runs when
/// the watched values change.
///
/// # Example
///
/// ```ignore
/// let count = signal(0);
///
/// watch(
///     move || count.get(),
///     |new_value, old_value| {
///         println!("Count changed from {:?} to {}", old_value, new_value);
///     }
/// );
/// ```
pub fn watch<T, W, F>(watched: W, callback: F) -> Effect
where
    T: Clone + 'static,
    W: Fn() -> T + 'static,
    F: Fn(T, Option<T>) + 'static,
{
    let prev_value: Rc<RefCell<Option<T>>> = Rc::new(RefCell::new(None));
    let prev_value_clone = Rc::clone(&prev_value);
    let first_run = Rc::new(RefCell::new(true));
    let first_run_clone = Rc::clone(&first_run);

    effect(move || {
        let new_value = watched();

        if *first_run_clone.borrow() {
            *first_run_clone.borrow_mut() = false;
            *prev_value_clone.borrow_mut() = Some(new_value);
            return;
        }

        let old_value = prev_value_clone.borrow().clone();
        callback(new_value.clone(), old_value);
        *prev_value_clone.borrow_mut() = Some(new_value);
    })
}

/// Watch with immediate execution.
///
/// Like `watch`, but also runs the callback immediately with the initial value.
pub fn watch_immediate<T, W, F>(watched: W, callback: F) -> Effect
where
    T: Clone + 'static,
    W: Fn() -> T + 'static,
    F: Fn(T, Option<T>) + 'static,
{
    let prev_value: Rc<RefCell<Option<T>>> = Rc::new(RefCell::new(None));
    let prev_value_clone = Rc::clone(&prev_value);

    effect(move || {
        let new_value = watched();
        let old_value = prev_value_clone.borrow().clone();
        callback(new_value.clone(), old_value);
        *prev_value_clone.borrow_mut() = Some(new_value);
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reactive::signal;
    use std::cell::Cell;

    #[test]
    fn test_effect_runs_immediately() {
        let ran = Rc::new(Cell::new(false));
        let ran_clone = Rc::clone(&ran);

        let _fx = effect(move || {
            ran_clone.set(true);
        });

        assert!(ran.get());
    }

    #[test]
    fn test_effect_stop() {
        let count = Rc::new(Cell::new(0));
        let count_clone = Rc::clone(&count);

        let fx = effect(move || {
            count_clone.set(count_clone.get() + 1);
        });

        assert_eq!(count.get(), 1); // Initial run

        fx.stop();
        assert!(!fx.is_active());
    }

    #[test]
    fn test_effect_cleanup() {
        let cleanup_ran = Rc::new(Cell::new(false));
        let cleanup_ran_clone = Rc::clone(&cleanup_ran);

        let fx = Effect::with_cleanup(
            || {},
            move || cleanup_ran_clone.set(true)
        );

        fx.stop();
        assert!(cleanup_ran.get());
    }
}
