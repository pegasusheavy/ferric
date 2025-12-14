//! Computed values that derive from signals.
//!
//! Computed values automatically update when their dependencies change.
//!
//! # Examples
//!
//! ```
//! use ferric_core::reactive::{signal, computed};
//!
//! let count = signal(10);
//! let doubled = computed(move || count.get() * 2);
//!
//! assert_eq!(doubled.get(), 20);
//!
//! count.set(15);
//! assert_eq!(doubled.get(), 30);
//! ```
//!
//! # Complex Dependencies
//!
//! ```
//! use ferric_core::reactive::{signal, computed};
//!
//! let first = signal("Hello");
//! let last = signal("World");
//! let full_name = computed(move || {
//!     format!("{} {}", first.get(), last.get())
//! });
//!
//! assert_eq!(full_name.get(), "Hello World");
//!
//! first.set("Hi");
//! assert_eq!(full_name.get(), "Hi World");
//! ```

use super::runtime::{Runtime, SubscriberId};
use std::cell::RefCell;
use std::rc::Rc;

/// A computed value that derives from other reactive values.
///
/// # Examples
///
/// ```
/// use ferric_core::reactive::{signal, Computed};
///
/// let a = signal(5);
/// let b = signal(10);
/// let sum = Computed::new(move || a.get() + b.get());
///
/// assert_eq!(sum.get(), 15);
/// ```
#[derive(Clone)]
pub struct Computed<T> {
    inner: Rc<RefCell<ComputedInner<T>>>,
}

struct ComputedInner<T> {
    compute_fn: Box<dyn Fn() -> T>,
    cached_value: Option<T>,
    subscriber_id: Option<SubscriberId>,
    is_dirty: bool,
}

impl<T: Clone + 'static> Computed<T> {
    /// Creates a new computed value.
    ///
    /// # Examples
    ///
    /// ```
    /// use ferric_core::reactive::{signal, Computed};
    ///
    /// let count = signal(0);
    /// let is_even = Computed::new(move || count.get() % 2 == 0);
    ///
    /// assert_eq!(is_even.get(), true);
    /// count.set(1);
    /// assert_eq!(is_even.get(), false);
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: Fn() -> T + 'static,
    {
        Self {
            inner: Rc::new(RefCell::new(ComputedInner {
                compute_fn: Box::new(f),
                cached_value: None,
                subscriber_id: None,
                is_dirty: true,
            })),
        }
    }

    /// Gets the current computed value.
    ///
    /// # Examples
    ///
    /// ```
    /// use ferric_core::reactive::{signal, computed};
    ///
    /// let x = signal(3);
    /// let squared = computed(move || x.get() * x.get());
    ///
    /// assert_eq!(squared.get(), 9);
    /// ```
    pub fn get(&self) -> T {
        let mut inner = self.inner.borrow_mut();

        if inner.is_dirty || inner.cached_value.is_none() {
            // Re-compute the value
            let value = (inner.compute_fn)();
            inner.cached_value = Some(value.clone());
            inner.is_dirty = false;
            value
        } else {
            inner.cached_value.clone().unwrap()
        }
    }

    /// Marks this computed as dirty, forcing recomputation on next access.
    pub(crate) fn mark_dirty(&self) {
        self.inner.borrow_mut().is_dirty = true;
    }
}

/// Creates a new computed value.
///
/// # Examples
///
/// ```
/// use ferric_core::reactive::{signal, computed};
///
/// let radius = signal(5.0);
/// let area = computed(move || std::f64::consts::PI * radius.get() * radius.get());
///
/// assert!((area.get() - 78.54).abs() < 0.01);
/// ```
pub fn computed<T, F>(f: F) -> Computed<T>
where
    T: Clone + 'static,
    F: Fn() -> T + 'static,
{
    Computed::new(f)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reactive::signal;

    #[test]
    fn test_computed_creation() {
        let s = signal(10);
        let c = computed(move || s.get() * 2);
        assert_eq!(c.get(), 20);
    }

    #[test]
    fn test_computed_updates() {
        let s = signal(5);
        let c = computed(move || s.get() * 2);

        assert_eq!(c.get(), 10);
        s.set(10);
        assert_eq!(c.get(), 20);
    }

    #[test]
    fn test_computed_caching() {
        let s = signal(1);
        let call_count = Rc::new(RefCell::new(0));
        let call_count_clone = call_count.clone();

        let c = computed(move || {
            *call_count_clone.borrow_mut() += 1;
            s.get() * 2
        });

        // First access should compute
        let _ = c.get();
        assert_eq!(*call_count.borrow(), 1);

        // Second access should use cache
        let _ = c.get();
        assert_eq!(*call_count.borrow(), 1);
    }
}
