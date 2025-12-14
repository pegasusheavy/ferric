//! Throttling utilities for async operations.

use std::cell::RefCell;
use std::time::{Duration, Instant};

/// Throttled function wrapper.
pub struct Throttled<F> {
    func: F,
    delay: Duration,
    last_call: RefCell<Option<Instant>>,
    pending: RefCell<bool>,
}

impl<F> Throttled<F> {
    /// Create a new throttled function.
    pub fn new(func: F, delay: Duration) -> Self {
        Self {
            func,
            delay,
            last_call: RefCell::new(None),
            pending: RefCell::new(false),
        }
    }

    /// Call the throttled function.
    pub fn call(&self)
    where
        F: Fn(),
    {
        let now = Instant::now();
        let should_call = match *self.last_call.borrow() {
            None => true,
            Some(last) => now.duration_since(last) >= self.delay,
        };

        if should_call && !*self.pending.borrow() {
            (self.func)();
            *self.last_call.borrow_mut() = Some(now);
            *self.pending.borrow_mut() = false;
        } else if !*self.pending.borrow() {
            *self.pending.borrow_mut() = true;

            // Schedule a delayed call
            // Note: This would need proper async scheduling in a real implementation
        }
    }

    /// Reset the throttle state.
    pub fn reset(&self) {
        *self.last_call.borrow_mut() = None;
        *self.pending.borrow_mut() = false;
    }
}

/// Create a throttled function.
pub fn throttle<F>(func: F, delay: Duration) -> Throttled<F>
where
    F: Fn(),
{
    Throttled::new(func, delay)
}

