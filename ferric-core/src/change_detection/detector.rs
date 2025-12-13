//! Change detector reference for individual components.
//!
//! Each component gets a `ChangeDetectorRef` that allows it to:
//! - Mark itself for check (OnPush components)
//! - Trigger immediate change detection
//! - Detach from the change detection tree
//! - Reattach to the change detection tree

use super::{ChangeDetectionResult, DetectorId};
use crate::component::ChangeDetectionStrategy;
use crate::reactive::{signal, Effect, Signal};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// Status of a change detector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeDetectorStatus {
    /// Detector is active and will be checked.
    CheckAlways,
    /// Detector is marked for check (will be checked once).
    CheckOnce,
    /// Detector is detached and won't be checked.
    Detached,
    /// Detector has errored and is in a bad state.
    Errored,
}

/// Reference to a component's change detector.
///
/// This is the primary API for components to interact with change detection.
#[derive(Clone)]
pub struct ChangeDetectorRef {
    inner: Rc<RefCell<ChangeDetectorInner>>,
}

struct ChangeDetectorInner {
    /// Unique identifier.
    id: DetectorId,
    /// Current status.
    status: ChangeDetectorStatus,
    /// Change detection strategy.
    strategy: ChangeDetectionStrategy,
    /// Whether the detector is dirty (needs checking).
    dirty: bool,
    /// Parent detector (weak ref to avoid cycles).
    parent: Option<Weak<RefCell<ChangeDetectorInner>>>,
    /// Child detectors.
    children: Vec<Weak<RefCell<ChangeDetectorInner>>>,
    /// Callback to trigger actual view update.
    update_fn: Option<Box<dyn Fn() -> ChangeDetectionResult>>,
    /// Signal that triggers when marked dirty.
    dirty_signal: Signal<bool>,
}

impl ChangeDetectorRef {
    /// Create a new change detector with the given strategy.
    pub fn new(strategy: ChangeDetectionStrategy) -> Self {
        Self {
            inner: Rc::new(RefCell::new(ChangeDetectorInner {
                id: DetectorId::new(),
                status: if strategy == ChangeDetectionStrategy::OnPush {
                    ChangeDetectorStatus::CheckOnce
                } else {
                    ChangeDetectorStatus::CheckAlways
                },
                strategy,
                dirty: true, // Initially dirty for first render
                parent: None,
                children: Vec::new(),
                update_fn: None,
                dirty_signal: signal(true),
            })),
        }
    }

    /// Create a default (CheckAlways) change detector.
    pub fn default_strategy() -> Self {
        Self::new(ChangeDetectionStrategy::Default)
    }

    /// Create an OnPush change detector.
    pub fn on_push() -> Self {
        Self::new(ChangeDetectionStrategy::OnPush)
    }

    /// Get the detector's unique ID.
    pub fn id(&self) -> DetectorId {
        self.inner.borrow().id
    }

    /// Get the current status.
    pub fn status(&self) -> ChangeDetectorStatus {
        self.inner.borrow().status
    }

    /// Get the change detection strategy.
    pub fn strategy(&self) -> ChangeDetectionStrategy {
        self.inner.borrow().strategy
    }

    /// Check if the detector is dirty.
    pub fn is_dirty(&self) -> bool {
        self.inner.borrow().dirty
    }

    /// Mark this component and its ancestors for check.
    ///
    /// This is the primary API for OnPush components to trigger change detection.
    /// After calling this, the component will be checked in the next CD cycle.
    ///
    /// # Example
    ///
    /// ```ignore
    /// fn handle_async_result(&self, result: Data) {
    ///     self.data.set(result);
    ///     self.cd.mark_for_check();
    /// }
    /// ```
    pub fn mark_for_check(&self) {
        let mut inner = self.inner.borrow_mut();
        inner.dirty = true;
        inner.dirty_signal.set(true);

        if inner.status == ChangeDetectorStatus::Detached {
            return;
        }

        inner.status = ChangeDetectorStatus::CheckOnce;

        // Mark ancestors
        if let Some(ref parent_weak) = inner.parent {
            if let Some(parent) = parent_weak.upgrade() {
                drop(inner); // Release borrow before recursing
                let parent_ref = ChangeDetectorRef {
                    inner: parent,
                };
                parent_ref.mark_for_check();
            }
        }
    }

    /// Immediately run change detection on this component and its children.
    ///
    /// This bypasses the normal batching and runs synchronously.
    /// Use sparingly as it can cause performance issues.
    ///
    /// # Example
    ///
    /// ```ignore
    /// fn force_update(&self) {
    ///     self.cd.detect_changes();
    /// }
    /// ```
    pub fn detect_changes(&self) -> ChangeDetectionResult {
        let mut inner = self.inner.borrow_mut();

        if inner.status == ChangeDetectorStatus::Detached {
            return ChangeDetectionResult::Skipped;
        }

        // Run update function if set
        let result = if let Some(ref update_fn) = inner.update_fn {
            update_fn()
        } else {
            ChangeDetectionResult::NoChanges
        };

        // Clear dirty flag
        inner.dirty = false;
        inner.dirty_signal.set(false);

        // Reset status for OnPush
        if inner.strategy == ChangeDetectionStrategy::OnPush {
            inner.status = ChangeDetectorStatus::CheckOnce;
        }

        // Detect changes on children
        let children: Vec<_> = inner.children.iter()
            .filter_map(|w| w.upgrade())
            .collect();

        drop(inner); // Release borrow before processing children

        for child in children {
            let child_ref = ChangeDetectorRef { inner: child };
            child_ref.detect_changes();
        }

        result
    }

    /// Check without recursing to children.
    pub fn check_no_changes(&self) -> ChangeDetectionResult {
        let mut inner = self.inner.borrow_mut();

        if inner.status == ChangeDetectorStatus::Detached {
            return ChangeDetectionResult::Skipped;
        }

        if !inner.dirty {
            return ChangeDetectionResult::NoChanges;
        }

        let result = if let Some(ref update_fn) = inner.update_fn {
            update_fn()
        } else {
            ChangeDetectionResult::NoChanges
        };

        inner.dirty = false;
        inner.dirty_signal.set(false);

        result
    }

    /// Detach this detector from the change detection tree.
    ///
    /// A detached detector won't be checked during normal change detection cycles.
    /// Useful for components that manage their own updates or are temporarily hidden.
    ///
    /// # Example
    ///
    /// ```ignore
    /// fn on_hide(&self) {
    ///     self.cd.detach();
    /// }
    /// ```
    pub fn detach(&self) {
        let mut inner = self.inner.borrow_mut();
        inner.status = ChangeDetectorStatus::Detached;
    }

    /// Reattach a detached detector to the change detection tree.
    ///
    /// # Example
    ///
    /// ```ignore
    /// fn on_show(&self) {
    ///     self.cd.reattach();
    ///     self.cd.mark_for_check();
    /// }
    /// ```
    pub fn reattach(&self) {
        let mut inner = self.inner.borrow_mut();
        if inner.status == ChangeDetectorStatus::Detached {
            inner.status = if inner.strategy == ChangeDetectionStrategy::OnPush {
                ChangeDetectorStatus::CheckOnce
            } else {
                ChangeDetectorStatus::CheckAlways
            };
        }
    }

    /// Set the update function that runs during change detection.
    pub fn set_update_fn<F>(&self, f: F)
    where
        F: Fn() -> ChangeDetectionResult + 'static,
    {
        self.inner.borrow_mut().update_fn = Some(Box::new(f));
    }

    /// Add a child detector.
    pub fn add_child(&self, child: &ChangeDetectorRef) {
        let mut inner = self.inner.borrow_mut();
        inner.children.push(Rc::downgrade(&child.inner));

        // Set parent on child
        child.inner.borrow_mut().parent = Some(Rc::downgrade(&self.inner));
    }

    /// Remove a child detector.
    pub fn remove_child(&self, child: &ChangeDetectorRef) {
        let child_id = child.id();
        self.inner.borrow_mut().children.retain(|w| {
            w.upgrade()
                .map(|c| ChangeDetectorRef { inner: c }.id() != child_id)
                .unwrap_or(false)
        });

        // Clear parent on child
        child.inner.borrow_mut().parent = None;
    }

    /// Get the dirty signal for reactive subscriptions.
    pub fn dirty_signal(&self) -> Signal<bool> {
        self.inner.borrow().dirty_signal.clone()
    }

    /// Create an effect that triggers change detection when any tracked signal changes.
    ///
    /// This is useful for automatically marking OnPush components dirty when
    /// their state changes.
    pub fn watch_signals<F>(&self, track: F) -> Effect
    where
        F: Fn() + 'static,
    {
        let cd = self.clone();
        crate::reactive::effect(move || {
            track();
            cd.mark_for_check();
        })
    }
}

impl Default for ChangeDetectorRef {
    fn default() -> Self {
        Self::default_strategy()
    }
}

impl std::fmt::Debug for ChangeDetectorRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = self.inner.borrow();
        f.debug_struct("ChangeDetectorRef")
            .field("id", &inner.id)
            .field("status", &inner.status)
            .field("strategy", &inner.strategy)
            .field("dirty", &inner.dirty)
            .finish()
    }
}

/// Tree structure for managing change detectors.
pub struct DetectorTree {
    /// Root detector.
    root: Option<ChangeDetectorRef>,
    /// All detectors by ID.
    detectors: std::collections::HashMap<DetectorId, ChangeDetectorRef>,
}

impl DetectorTree {
    /// Create a new detector tree.
    pub fn new() -> Self {
        Self {
            root: None,
            detectors: std::collections::HashMap::new(),
        }
    }

    /// Set the root detector.
    pub fn set_root(&mut self, detector: ChangeDetectorRef) {
        let id = detector.id();
        self.root = Some(detector.clone());
        self.detectors.insert(id, detector);
    }

    /// Get the root detector.
    pub fn root(&self) -> Option<&ChangeDetectorRef> {
        self.root.as_ref()
    }

    /// Add a detector to the tree.
    pub fn add(&mut self, parent_id: DetectorId, detector: ChangeDetectorRef) {
        let id = detector.id();

        if let Some(parent) = self.detectors.get(&parent_id) {
            parent.add_child(&detector);
        }

        self.detectors.insert(id, detector);
    }

    /// Remove a detector from the tree.
    pub fn remove(&mut self, id: DetectorId) {
        if let Some(detector) = self.detectors.remove(&id) {
            // Remove from parent's children
            if let Some(parent_weak) = detector.inner.borrow().parent.as_ref() {
                if let Some(parent) = parent_weak.upgrade() {
                    let parent_ref = ChangeDetectorRef { inner: parent };
                    parent_ref.remove_child(&detector);
                }
            }
        }
    }

    /// Get a detector by ID.
    pub fn get(&self, id: DetectorId) -> Option<&ChangeDetectorRef> {
        self.detectors.get(&id)
    }

    /// Run change detection on the entire tree.
    pub fn tick(&self) -> ChangeDetectionResult {
        if let Some(ref root) = self.root {
            root.detect_changes()
        } else {
            ChangeDetectionResult::NoChanges
        }
    }
}

impl Default for DetectorTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_detector_default() {
        let cd = ChangeDetectorRef::default_strategy();
        assert_eq!(cd.status(), ChangeDetectorStatus::CheckAlways);
        assert!(cd.is_dirty());
    }

    #[test]
    fn test_change_detector_on_push() {
        let cd = ChangeDetectorRef::on_push();
        assert_eq!(cd.status(), ChangeDetectorStatus::CheckOnce);
        assert_eq!(cd.strategy(), ChangeDetectionStrategy::OnPush);
    }

    #[test]
    fn test_detach_reattach() {
        let cd = ChangeDetectorRef::default_strategy();

        cd.detach();
        assert_eq!(cd.status(), ChangeDetectorStatus::Detached);

        cd.reattach();
        assert_eq!(cd.status(), ChangeDetectorStatus::CheckAlways);
    }

    #[test]
    fn test_parent_child() {
        let parent = ChangeDetectorRef::default_strategy();
        let child = ChangeDetectorRef::on_push();

        parent.add_child(&child);

        // Child should have parent set
        assert!(child.inner.borrow().parent.is_some());
    }
}

