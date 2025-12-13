//! Effects - side effects that run when dependencies change.

use super::SubscriptionId;
use std::cell::RefCell;
use std::rc::Rc;

/// An effect that runs when its dependencies change.
pub struct Effect {
    inner: Rc<RefCell<EffectInner>>,
}

struct EffectInner {
    effect_fn: Box<dyn Fn()>,
    cleanup_fn: Option<Box<dyn Fn()>>,
    active: bool,
    subscriptions: Vec<SubscriptionId>,
}

impl Effect {
    /// Create a new effect with the given function.
    pub fn new<F>(effect_fn: F) -> Self
    where
        F: Fn() + 'static,
    {
        let effect = Self {
            inner: Rc::new(RefCell::new(EffectInner {
                effect_fn: Box::new(effect_fn),
                cleanup_fn: None,
                active: true,
                subscriptions: Vec::new(),
            })),
        };

        // Run the effect immediately
        effect.run();

        effect
    }

    /// Create an effect with a cleanup function.
    pub fn with_cleanup<F, C>(effect_fn: F, cleanup_fn: C) -> Self
    where
        F: Fn() + 'static,
        C: Fn() + 'static,
    {
        let effect = Self {
            inner: Rc::new(RefCell::new(EffectInner {
                effect_fn: Box::new(effect_fn),
                cleanup_fn: Some(Box::new(cleanup_fn)),
                active: true,
                subscriptions: Vec::new(),
            })),
        };

        effect.run();

        effect
    }

    /// Run the effect.
    pub fn run(&self) {
        let inner = self.inner.borrow();
        if inner.active {
            (inner.effect_fn)();
        }
    }

    /// Stop the effect from running.
    pub fn stop(&self) {
        let mut inner = self.inner.borrow_mut();
        if inner.active {
            inner.active = false;
            if let Some(ref cleanup) = inner.cleanup_fn {
                cleanup();
            }
        }
    }

    /// Check if the effect is active.
    pub fn is_active(&self) -> bool {
        self.inner.borrow().active
    }
}

impl Drop for Effect {
    fn drop(&mut self) {
        self.stop();
    }
}

impl Clone for Effect {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

/// Create a new effect that runs immediately and when dependencies change.
pub fn effect<F>(f: F) -> Effect
where
    F: Fn() + 'static,
{
    Effect::new(f)
}

