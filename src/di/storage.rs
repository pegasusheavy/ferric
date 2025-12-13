//! Type-erased storage for dependency injection.
//!
//! Provides efficient storage that avoids double-boxing and minimizes allocations.

use std::any::{Any, TypeId};
use std::cell::UnsafeCell;
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

    /// Get a reference to the inner value.
    #[inline]
    pub fn get<T: Any + 'static>(&self) -> Option<Rc<T>> {
        if self.type_id == TypeId::of::<T>() {
            // SAFETY: We verified the type matches
            let ptr = Rc::as_ptr(&self.inner) as *const T;
            // Clone the Rc by incrementing ref count
            Some(unsafe { Rc::from_raw(ptr) }.clone())
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
/// Optimized for the common case where the service is initialized once
/// and then accessed many times.
pub struct LazySlot<T> {
    cell: UnsafeCell<Option<Rc<T>>>,
}

impl<T> LazySlot<T> {
    /// Create a new empty slot.
    #[inline]
    pub const fn new() -> Self {
        Self {
            cell: UnsafeCell::new(None),
        }
    }

    /// Get or initialize the value.
    ///
    /// # Safety
    ///
    /// This is safe in single-threaded WASM context. The closure
    /// must not recursively access this slot.
    #[inline]
    pub fn get_or_init<F>(&self, f: F) -> Rc<T>
    where
        F: FnOnce() -> T,
    {
        // SAFETY: WASM is single-threaded, no data races possible
        let slot = unsafe { &mut *self.cell.get() };

        if let Some(ref rc) = slot {
            Rc::clone(rc)
        } else {
            let rc = Rc::new(f());
            *slot = Some(Rc::clone(&rc));
            rc
        }
    }

    /// Check if the slot has a value.
    #[inline]
    pub fn is_initialized(&self) -> bool {
        // SAFETY: Just reading a boolean condition
        unsafe { (*self.cell.get()).is_some() }
    }

    /// Get the value if initialized.
    #[inline]
    pub fn get(&self) -> Option<Rc<T>> {
        // SAFETY: WASM is single-threaded
        unsafe { (*self.cell.get()).as_ref().map(Rc::clone) }
    }
}

impl<T> Default for LazySlot<T> {
    fn default() -> Self {
        Self::new()
    }
}

// SAFETY: WASM is single-threaded, these are safe
unsafe impl<T> Send for LazySlot<T> {}
unsafe impl<T> Sync for LazySlot<T> {}

