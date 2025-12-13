//! Type-erased storage for dependency injection.
//!
//! Provides efficient storage using safe Rust abstractions.

use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::rc::Rc;

/// Type-erased container for a service instance.
///
/// Uses Rc internally to allow sharing without cloning the actual value.
pub struct ServiceInstance {
    inner: Rc<dyn Any>,
    type_id: TypeId,
}

impl ServiceInstance {
    /// Create a new service instance wrapper.
    #[inline]
    pub fn new<T: Any + 'static>(value: T) -> Self {
        Self {
            inner: Rc::new(value),
            type_id: TypeId::of::<T>(),
        }
    }

    /// Create from an existing Rc.
    #[inline]
    pub fn from_rc<T: Any + 'static>(rc: Rc<T>) -> Self {
        Self {
            inner: rc,
            type_id: TypeId::of::<T>(),
        }
    }

    /// Get a clone of the inner Rc if types match.
    #[inline]
    pub fn get<T: Any + 'static>(&self) -> Option<Rc<T>> {
        if self.type_id == TypeId::of::<T>() {
            // Downcast the Rc<dyn Any> to Rc<T>
            self.inner.clone().downcast::<T>().ok()
        } else {
            None
        }
    }

    /// Get the type ID of the stored value.
    #[inline]
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }
}

impl Clone for ServiceInstance {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
            type_id: self.type_id,
        }
    }
}

/// A slot that can hold a lazily-initialized service instance.
///
/// Uses RefCell for safe interior mutability.
pub struct LazySlot<T> {
    cell: RefCell<Option<Rc<T>>>,
}

impl<T> LazySlot<T> {
    /// Create a new empty slot.
    #[inline]
    pub fn new() -> Self {
        Self {
            cell: RefCell::new(None),
        }
    }

    /// Get or initialize the value.
    ///
    /// If the slot is empty, calls the provided closure to create the value.
    /// Panics if called recursively (would cause RefCell borrow error).
    #[inline]
    pub fn get_or_init<F>(&self, f: F) -> Rc<T>
    where
        F: FnOnce() -> T,
    {
        // First check if we already have a value (shared borrow)
        if let Some(rc) = self.cell.borrow().as_ref() {
            return Rc::clone(rc);
        }

        // Need to initialize - get exclusive borrow
        let mut slot = self.cell.borrow_mut();

        // Double-check after acquiring exclusive borrow
        if let Some(rc) = slot.as_ref() {
            return Rc::clone(rc);
        }

        // Initialize
        let rc = Rc::new(f());
        *slot = Some(Rc::clone(&rc));
        rc
    }

    /// Check if the slot has a value.
    #[inline]
    pub fn is_initialized(&self) -> bool {
        self.cell.borrow().is_some()
    }

    /// Get the value if initialized.
    #[inline]
    pub fn get(&self) -> Option<Rc<T>> {
        self.cell.borrow().as_ref().map(Rc::clone)
    }

    /// Clear the slot.
    #[inline]
    pub fn clear(&self) {
        *self.cell.borrow_mut() = None;
    }
}

impl<T> Default for LazySlot<T> {
    fn default() -> Self {
        Self::new()
    }
}
