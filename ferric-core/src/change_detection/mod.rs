//! Zone-less change detection for Ferric.
//!
//! This module provides a signal-driven change detection system that doesn't
//! rely on Zone.js-style monkey-patching of async APIs. Instead, change detection
//! is triggered explicitly through:
//!
//! - Signal changes (automatic via reactive system)
//! - Manual `markForCheck()` or `detectChanges()` calls
//! - Event handlers (via scheduler integration)
//!
//! ## Change Detection Strategies
//!
//! - **Default**: Component is checked whenever any signal in the app changes
//! - **OnPush**: Component only checked when its inputs change or it's explicitly marked
//!
//! ## Usage
//!
//! ```ignore
//! use ferric::change_detection::*;
//!
//! // In a component
//! impl MyComponent {
//!     fn handle_click(&self) {
//!         // Update state
//!         self.count.update(|n| n + 1);
//!
//!         // For OnPush components, mark for check
//!         self.cd.mark_for_check();
//!     }
//! }
//! ```

mod application;
mod detector;
mod scheduler;

pub use application::{ApplicationRef, ApplicationConfig};
pub use detector::{ChangeDetectorRef, ChangeDetectorStatus, DetectorTree};
pub use scheduler::{Scheduler, SchedulerConfig, schedule, schedule_on_stable};

use crate::component::ChangeDetectionStrategy;
use crate::reactive::{Signal, signal};
use std::cell::RefCell;

/// A unique identifier for a change detector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DetectorId(u64);

impl DetectorId {
    /// Generate a new unique detector ID.
    pub fn new() -> Self {
        NEXT_DETECTOR_ID.with(|id| {
            let current = *id.borrow();
            *id.borrow_mut() = current + 1;
            DetectorId(current)
        })
    }
}

impl Default for DetectorId {
    fn default() -> Self {
        Self::new()
    }
}

thread_local! {
    static NEXT_DETECTOR_ID: RefCell<u64> = const { RefCell::new(0) };
}

/// Represents the result of a change detection cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeDetectionResult {
    /// No changes were detected.
    NoChanges,
    /// Changes were detected and the view was updated.
    Changed,
    /// The detector was detached and skipped.
    Skipped,
}

/// Trait for types that can participate in change detection.
pub trait ChangeDetectable {
    /// Get the change detector reference.
    fn change_detector(&self) -> &ChangeDetectorRef;

    /// Get the change detection strategy.
    fn strategy(&self) -> ChangeDetectionStrategy {
        ChangeDetectionStrategy::Default
    }

    /// Check for changes and update the view if needed.
    fn detect_changes(&mut self) -> ChangeDetectionResult;

    /// Called when inputs change (for OnPush optimization).
    fn on_input_change(&mut self) {
        if self.strategy() == ChangeDetectionStrategy::OnPush {
            self.change_detector().mark_for_check();
        }
    }
}

/// Context for change detection operations.
#[derive(Clone)]
pub struct ChangeDetectionContext {
    /// Whether we're in a change detection cycle.
    pub in_cycle: Signal<bool>,
    /// The current cycle count (for debugging).
    pub cycle_count: Signal<u64>,
    /// Whether change detection is currently disabled.
    pub disabled: Signal<bool>,
}

impl ChangeDetectionContext {
    /// Create a new change detection context.
    pub fn new() -> Self {
        Self {
            in_cycle: signal(false),
            cycle_count: signal(0),
            disabled: signal(false),
        }
    }

    /// Begin a change detection cycle.
    pub fn begin_cycle(&self) {
        self.in_cycle.set(true);
        self.cycle_count.update(|n| n + 1);
    }

    /// End a change detection cycle.
    pub fn end_cycle(&self) {
        self.in_cycle.set(false);
    }

    /// Check if we're currently in a cycle.
    pub fn is_in_cycle(&self) -> bool {
        self.in_cycle.get()
    }

    /// Disable change detection temporarily.
    pub fn disable(&self) {
        self.disabled.set(true);
    }

    /// Re-enable change detection.
    pub fn enable(&self) {
        self.disabled.set(false);
    }

    /// Run a function with change detection disabled.
    pub fn run_outside_change_detection<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let was_disabled = self.disabled.get();
        self.disabled.set(true);
        let result = f();
        self.disabled.set(was_disabled);
        result
    }
}

impl Default for ChangeDetectionContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Prelude for convenient imports.
pub mod prelude {
    pub use super::{
        ApplicationRef, ApplicationConfig,
        ChangeDetectorRef, ChangeDetectorStatus, DetectorTree,
        Scheduler, SchedulerConfig, schedule, schedule_on_stable,
        ChangeDetectable, ChangeDetectionContext, ChangeDetectionResult,
        DetectorId,
    };
}

