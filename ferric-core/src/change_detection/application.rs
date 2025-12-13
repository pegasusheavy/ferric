//! Application reference for root-level change detection.
//!
//! The `ApplicationRef` is the entry point for the zone-less change detection
//! system. It manages the detector tree, scheduler, and provides the main
//! API for triggering change detection.

use super::detector::{ChangeDetectorRef, DetectorTree};
use super::scheduler::{Scheduler, SchedulerConfig};
use super::{ChangeDetectionContext, ChangeDetectionResult};
use crate::reactive::{signal, Effect, Signal};
use std::cell::RefCell;
use std::rc::Rc;

/// Configuration for the application.
#[derive(Debug, Clone, Default)]
pub struct ApplicationConfig {
    /// Scheduler configuration.
    pub scheduler: SchedulerConfig,
    /// Whether to enable development mode checks.
    pub dev_mode: bool,
    /// Whether to enable automatic change detection on signal changes.
    pub auto_detect: bool,
}

impl ApplicationConfig {
    /// Create a production configuration.
    pub fn production() -> Self {
        Self {
            scheduler: SchedulerConfig::default(),
            dev_mode: false,
            auto_detect: true,
        }
    }

    /// Create a development configuration.
    pub fn development() -> Self {
        Self {
            scheduler: SchedulerConfig::default(),
            dev_mode: true,
            auto_detect: true,
        }
    }
}

/// State of the application.
struct ApplicationState {
    /// The detector tree.
    tree: DetectorTree,
    /// Context for change detection.
    context: ChangeDetectionContext,
    /// Whether a tick is scheduled.
    tick_scheduled: bool,
    /// Callbacks to run after tick.
    after_tick: Vec<Box<dyn FnOnce()>>,
    /// Number of ticks since creation.
    tick_count: u64,
    /// Configuration.
    config: ApplicationConfig,
}

/// Reference to the running application.
///
/// This is the main entry point for zone-less change detection.
#[derive(Clone)]
pub struct ApplicationRef {
    state: Rc<RefCell<ApplicationState>>,
    scheduler: Scheduler,
    /// Signal that emits when the app is stable.
    is_stable: Signal<bool>,
    /// Effect for automatic change detection (if enabled).
    _auto_effect: Option<Rc<Effect>>,
}

impl ApplicationRef {
    /// Create a new application reference.
    pub fn new() -> Self {
        Self::with_config(ApplicationConfig::default())
    }

    /// Create with custom configuration.
    pub fn with_config(config: ApplicationConfig) -> Self {
        let scheduler = Scheduler::with_config(config.scheduler.clone());
        let auto_detect = config.auto_detect;

        let app = Self {
            state: Rc::new(RefCell::new(ApplicationState {
                tree: DetectorTree::new(),
                context: ChangeDetectionContext::new(),
                tick_scheduled: false,
                after_tick: Vec::new(),
                tick_count: 0,
                config,
            })),
            scheduler,
            is_stable: signal(true),
            _auto_effect: None,
        };

        // Set up automatic change detection if enabled
        if auto_detect {
            // The auto effect will be set up when a root component is registered
        }

        app
    }

    /// Get the scheduler.
    pub fn scheduler(&self) -> &Scheduler {
        &self.scheduler
    }

    /// Get the change detection context.
    pub fn context(&self) -> ChangeDetectionContext {
        self.state.borrow().context.clone()
    }

    /// Check if the app is stable.
    pub fn is_stable(&self) -> bool {
        self.is_stable.get()
    }

    /// Get the tick count.
    pub fn tick_count(&self) -> u64 {
        self.state.borrow().tick_count
    }

    /// Register the root component's change detector.
    pub fn register_root(&self, detector: ChangeDetectorRef) {
        self.state.borrow_mut().tree.set_root(detector);
    }

    /// Register a component's change detector as a child of another.
    pub fn register_component(&self, parent_id: super::DetectorId, detector: ChangeDetectorRef) {
        self.state.borrow_mut().tree.add(parent_id, detector);
    }

    /// Unregister a component's change detector.
    pub fn unregister_component(&self, id: super::DetectorId) {
        self.state.borrow_mut().tree.remove(id);
    }

    /// Run change detection on the entire component tree.
    ///
    /// This is the main API for triggering change detection.
    /// In most cases, you should use `scheduleTick()` instead for batching.
    pub fn tick(&self) -> ChangeDetectionResult {
        self.is_stable.set(false);

        let (context, dev_mode) = {
            let mut state = self.state.borrow_mut();
            state.tick_scheduled = false;
            state.tick_count += 1;
            (state.context.clone(), state.config.dev_mode)
        };

        // Begin the cycle
        context.begin_cycle();

        // Run change detection
        let result = self.state.borrow().tree.tick();

        // End the cycle
        context.end_cycle();

        // Run after_tick callbacks
        let callbacks: Vec<Box<dyn FnOnce()>> = {
            std::mem::take(&mut self.state.borrow_mut().after_tick)
        };

        for callback in callbacks {
            callback();
        }

        // Check for infinite loop in dev mode
        if dev_mode {
            let tick_count = self.state.borrow().tick_count;
            if tick_count > 100 {
                web_sys::console::warn_1(
                    &"Warning: High tick count detected. Possible infinite change detection loop.".into()
                );
            }
        }

        self.is_stable.set(true);
        result
    }

    /// Schedule a tick to run in the next microtask.
    ///
    /// Multiple calls to `scheduleTick()` will result in a single tick.
    pub fn schedule_tick(&self) {
        let should_schedule = {
            let mut state = self.state.borrow_mut();
            if state.tick_scheduled {
                false
            } else {
                state.tick_scheduled = true;
                true
            }
        };

        if should_schedule {
            let app = self.clone();
            self.scheduler.schedule(move || {
                app.tick();
            });
        }
    }

    /// Run a callback after the next tick completes.
    pub fn after_tick<F>(&self, callback: F)
    where
        F: FnOnce() + 'static,
    {
        self.state.borrow_mut().after_tick.push(Box::new(callback));
    }

    /// Run a callback when the app becomes stable.
    pub fn when_stable<F>(&self, callback: F)
    where
        F: FnOnce() + 'static,
    {
        if self.is_stable() {
            callback();
        } else {
            self.scheduler.on_stable(callback);
        }
    }

    /// Run a function outside of change detection.
    ///
    /// Updates made inside this function won't trigger automatic change detection.
    pub fn run_outside_change_detection<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.state.borrow().context.run_outside_change_detection(f)
    }

    /// Attach a callback to run on every tick (for debugging).
    pub fn on_tick<F>(&self, callback: F) -> Effect
    where
        F: Fn(u64) + 'static,
    {
        let state = Rc::clone(&self.state);
        crate::reactive::effect(move || {
            let tick_count = state.borrow().tick_count;
            callback(tick_count);
        })
    }

    /// Get a detector by ID.
    pub fn get_detector(&self, id: super::DetectorId) -> Option<ChangeDetectorRef> {
        self.state.borrow().tree.get(id).cloned()
    }

    /// Bootstrap a component as the root.
    ///
    /// This creates a change detector for the component and registers it.
    pub fn bootstrap<F>(&self, create_component: F) -> ChangeDetectorRef
    where
        F: FnOnce(&ChangeDetectorRef),
    {
        let cd = ChangeDetectorRef::default_strategy();
        create_component(&cd);
        self.register_root(cd.clone());

        // Initial tick
        self.tick();

        cd
    }
}

impl Default for ApplicationRef {
    fn default() -> Self {
        Self::new()
    }
}

// Global application reference
thread_local! {
    static GLOBAL_APP: RefCell<Option<ApplicationRef>> = RefCell::new(None);
}

/// Get or create the global application reference.
pub fn global_app() -> ApplicationRef {
    GLOBAL_APP.with(|app| {
        let mut app_ref = app.borrow_mut();
        if app_ref.is_none() {
            *app_ref = Some(ApplicationRef::new());
        }
        app_ref.clone().unwrap()
    })
}

/// Set a custom global application reference.
pub fn set_global_app(app: ApplicationRef) {
    GLOBAL_APP.with(|a| {
        *a.borrow_mut() = Some(app);
    });
}

/// Trigger change detection on the global app.
pub fn tick() -> ChangeDetectionResult {
    global_app().tick()
}

/// Schedule a tick on the global app.
pub fn schedule_tick() {
    global_app().schedule_tick();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_application_creation() {
        let app = ApplicationRef::new();
        assert!(app.is_stable());
        assert_eq!(app.tick_count(), 0);
    }

    #[test]
    fn test_tick() {
        let app = ApplicationRef::new();
        let root_cd = ChangeDetectorRef::default_strategy();
        app.register_root(root_cd);

        let result = app.tick();
        assert_eq!(app.tick_count(), 1);
    }
}

