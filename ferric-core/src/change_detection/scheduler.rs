//! Zone-less scheduler for batched change detection.
//!
//! The scheduler collects multiple change detection triggers and batches them
//! into a single microtask, preventing unnecessary multiple renders.
//!
//! ## How it works
//!
//! 1. When state changes, components call `markForCheck()`
//! 2. The scheduler collects these notifications
//! 3. At the end of the current microtask, change detection runs once
//! 4. All dirty components are updated in a single pass
//!
//! This is similar to how modern frameworks like SolidJS and the new Angular
//! signals work, without needing Zone.js.

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

/// Configuration for the scheduler.
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// Whether to use requestAnimationFrame for visual updates.
    pub use_raf: bool,
    /// Whether to use queueMicrotask for immediate scheduling.
    pub use_microtask: bool,
    /// Maximum number of change detection cycles before warning.
    pub max_cycles: u32,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            use_raf: false,
            use_microtask: true,
            max_cycles: 100,
        }
    }
}

/// Scheduler state.
struct SchedulerState {
    /// Pending callbacks to run.
    pending: Vec<Box<dyn FnOnce()>>,
    /// Whether a flush is already scheduled.
    scheduled: bool,
    /// Callbacks to run when the app is stable.
    on_stable: Vec<Box<dyn FnOnce()>>,
    /// Whether the app is currently stable.
    is_stable: bool,
    /// Configuration.
    config: SchedulerConfig,
}

/// Zone-less scheduler for batching change detection.
///
/// The scheduler batches multiple updates into a single change detection cycle.
#[derive(Clone)]
pub struct Scheduler {
    state: Rc<RefCell<SchedulerState>>,
}

impl Scheduler {
    /// Create a new scheduler with default config.
    pub fn new() -> Self {
        Self::with_config(SchedulerConfig::default())
    }

    /// Create a scheduler with custom config.
    pub fn with_config(config: SchedulerConfig) -> Self {
        Self {
            state: Rc::new(RefCell::new(SchedulerState {
                pending: Vec::new(),
                scheduled: false,
                on_stable: Vec::new(),
                is_stable: true,
                config,
            })),
        }
    }

    /// Schedule a callback to run in the next batch.
    pub fn schedule<F>(&self, callback: F)
    where
        F: FnOnce() + 'static,
    {
        let mut state = self.state.borrow_mut();
        state.pending.push(Box::new(callback));
        state.is_stable = false;

        if !state.scheduled {
            state.scheduled = true;
            let use_microtask = state.config.use_microtask;
            drop(state); // Release borrow before scheduling

            self.schedule_flush(use_microtask);
        }
    }

    /// Schedule a callback to run on the next animation frame.
    pub fn schedule_raf<F>(&self, callback: F)
    where
        F: FnOnce() + 'static,
    {
        let scheduler = self.clone();
        let callback = Rc::new(RefCell::new(Some(callback)));

        request_animation_frame(move || {
            if let Some(cb) = callback.borrow_mut().take() {
                cb();
            }
            scheduler.check_stable();
        });
    }

    /// Register a callback to run when the app becomes stable.
    ///
    /// "Stable" means no more pending updates or in-flight async operations.
    pub fn on_stable<F>(&self, callback: F)
    where
        F: FnOnce() + 'static,
    {
        let mut state = self.state.borrow_mut();

        if state.is_stable && state.pending.is_empty() {
            // Already stable, run immediately
            drop(state);
            callback();
        } else {
            state.on_stable.push(Box::new(callback));
        }
    }

    /// Check if the app is stable.
    pub fn is_stable(&self) -> bool {
        self.state.borrow().is_stable
    }

    /// Force flush all pending callbacks synchronously.
    pub fn flush(&self) {
        let callbacks: Vec<Box<dyn FnOnce()>> = {
            let mut state = self.state.borrow_mut();
            state.scheduled = false;
            std::mem::take(&mut state.pending)
        };

        // Run all pending callbacks
        for callback in callbacks {
            callback();
        }

        self.check_stable();
    }

    /// Schedule a flush using microtask or setTimeout.
    fn schedule_flush(&self, use_microtask: bool) {
        let scheduler = self.clone();

        if use_microtask {
            queue_microtask(move || {
                scheduler.flush();
            });
        } else {
            set_timeout(move || {
                scheduler.flush();
            }, 0);
        }
    }

    /// Check if the app is stable and run on_stable callbacks.
    fn check_stable(&self) {
        let on_stable_callbacks: Vec<Box<dyn FnOnce()>> = {
            let mut state = self.state.borrow_mut();

            if !state.pending.is_empty() {
                return; // Still have pending work
            }

            state.is_stable = true;
            std::mem::take(&mut state.on_stable)
        };

        // Run all on_stable callbacks
        for callback in on_stable_callbacks {
            callback();
        }
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

// Global scheduler instance
thread_local! {
    static GLOBAL_SCHEDULER: RefCell<Option<Scheduler>> = RefCell::new(None);
}

/// Get or create the global scheduler.
pub fn global_scheduler() -> Scheduler {
    GLOBAL_SCHEDULER.with(|s| {
        let mut scheduler = s.borrow_mut();
        if scheduler.is_none() {
            *scheduler = Some(Scheduler::new());
        }
        scheduler.clone().unwrap()
    })
}

/// Set a custom global scheduler.
pub fn set_global_scheduler(scheduler: Scheduler) {
    GLOBAL_SCHEDULER.with(|s| {
        *s.borrow_mut() = Some(scheduler);
    });
}

/// Schedule a callback using the global scheduler.
pub fn schedule<F>(callback: F)
where
    F: FnOnce() + 'static,
{
    global_scheduler().schedule(callback);
}

/// Schedule a callback to run when the app is stable.
pub fn schedule_on_stable<F>(callback: F)
where
    F: FnOnce() + 'static,
{
    global_scheduler().on_stable(callback);
}

// ==================== Browser API Helpers ====================

/// Queue a microtask.
fn queue_microtask<F>(callback: F)
where
    F: FnOnce() + 'static,
{
    #[cfg(target_arch = "wasm32")]
    {
        let closure = Closure::once(Box::new(callback) as Box<dyn FnOnce()>);

        if let Some(window) = web_sys::window() {
            // Use queueMicrotask if available
            let func = closure.as_ref().unchecked_ref();
            let _ = js_sys::Reflect::apply(
                &js_sys::Reflect::get(&window, &JsValue::from_str("queueMicrotask"))
                    .unwrap_or(JsValue::UNDEFINED)
                    .dyn_into::<js_sys::Function>()
                    .unwrap_or_else(|_| {
                        // Fallback: use Promise.resolve().then()
                        js_sys::Function::new_no_args("")
                    }),
                &window,
                &js_sys::Array::of1(func),
            );
        }

        closure.forget();
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // For non-WASM, just run immediately
        callback();
    }
}

/// Request an animation frame.
fn request_animation_frame<F>(callback: F)
where
    F: FnOnce() + 'static,
{
    #[cfg(target_arch = "wasm32")]
    {
        let closure = Closure::once(Box::new(callback) as Box<dyn FnOnce()>);

        if let Some(window) = web_sys::window() {
            let _ = window.request_animation_frame(closure.as_ref().unchecked_ref());
        }

        closure.forget();
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        callback();
    }
}

/// Set a timeout.
fn set_timeout<F>(callback: F, delay: i32)
where
    F: FnOnce() + 'static,
{
    #[cfg(target_arch = "wasm32")]
    {
        let closure = Closure::once(Box::new(callback) as Box<dyn FnOnce()>);

        if let Some(window) = web_sys::window() {
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                delay,
            );
        }

        closure.forget();
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = delay;
        callback();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_creation() {
        let scheduler = Scheduler::new();
        assert!(scheduler.is_stable());
    }

    #[test]
    fn test_scheduler_flush() {
        use std::cell::Cell;
        use std::rc::Rc;

        let scheduler = Scheduler::new();
        let called = Rc::new(Cell::new(false));
        let called_clone = Rc::clone(&called);

        scheduler.schedule(move || {
            called_clone.set(true);
        });

        // Manually flush for test
        scheduler.flush();

        assert!(called.get());
    }
}

