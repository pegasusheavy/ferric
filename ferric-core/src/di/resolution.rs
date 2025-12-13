//! Resolution modifiers for dependency injection.
//!
//! These modifiers control how dependencies are resolved, similar to
//! Angular's `@Optional()`, `@Self()`, `@SkipSelf()`, and `@Host()` decorators.

use super::{Injector, ResolutionError};
use std::rc::Rc;

/// Options for controlling dependency resolution behavior.
#[derive(Debug, Clone, Default)]
pub struct ResolutionOptions {
    /// If true, return None instead of error when service not found.
    pub optional: bool,
    /// If true, only search the current injector (not parents).
    pub self_only: bool,
    /// If true, skip the current injector and start from parent.
    pub skip_self: bool,
    /// If true, stop at the host injector (for component boundaries).
    pub host: bool,
}

impl ResolutionOptions {
    /// Create default resolution options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark as optional (won't error if not found).
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// Only search current injector.
    pub fn self_only(mut self) -> Self {
        self.self_only = true;
        self
    }

    /// Skip current injector, start from parent.
    pub fn skip_self(mut self) -> Self {
        self.skip_self = true;
        self
    }

    /// Stop at host boundary.
    pub fn host(mut self) -> Self {
        self.host = true;
        self
    }
}

/// A builder for optional dependency resolution.
///
/// # Example
///
/// ```ignore
/// // Try to resolve, return None if not found
/// let service: Option<Rc<MyService>> = Optional::new(&injector).resolve();
/// ```
pub struct Optional<'a> {
    injector: &'a Injector,
}

impl<'a> Optional<'a> {
    /// Create a new optional resolver.
    pub fn new(injector: &'a Injector) -> Self {
        Self { injector }
    }

    /// Try to resolve, returning None if not found.
    pub fn resolve<T: 'static>(&self) -> Option<Rc<T>> {
        self.injector.resolve::<T>()
    }

    /// Try to resolve with a fallback value.
    pub fn resolve_or<T: 'static>(&self, default: T) -> Rc<T> {
        self.resolve::<T>().unwrap_or_else(|| Rc::new(default))
    }

    /// Try to resolve with a fallback factory.
    pub fn resolve_or_else<T: 'static, F: FnOnce() -> T>(&self, f: F) -> Rc<T> {
        self.resolve::<T>().unwrap_or_else(|| Rc::new(f()))
    }
}

/// A builder for self-only dependency resolution.
///
/// # Example
///
/// ```ignore
/// // Only check current injector, not parents
/// let service: Option<Rc<MyService>> = Self_::new(&injector).resolve();
/// ```
pub struct Self_<'a> {
    injector: &'a Injector,
}

impl<'a> Self_<'a> {
    /// Create a new self-only resolver.
    pub fn new(injector: &'a Injector) -> Self {
        Self { injector }
    }

    /// Resolve only from current injector.
    pub fn resolve<T: 'static>(&self) -> Option<Rc<T>> {
        self.injector.resolve_self::<T>()
    }

    /// Resolve required, only from current injector.
    pub fn resolve_required<T: 'static>(&self) -> Result<Rc<T>, ResolutionError> {
        self.resolve::<T>()
            .ok_or_else(|| ResolutionError::NotFound(std::any::type_name::<T>()))
    }
}

/// A builder for skip-self dependency resolution.
///
/// # Example
///
/// ```ignore
/// // Skip current injector, resolve from parent
/// let service: Option<Rc<MyService>> = SkipSelf::new(&injector).resolve();
/// ```
pub struct SkipSelf<'a> {
    injector: &'a Injector,
}

impl<'a> SkipSelf<'a> {
    /// Create a new skip-self resolver.
    pub fn new(injector: &'a Injector) -> Self {
        Self { injector }
    }

    /// Resolve starting from parent injector.
    pub fn resolve<T: 'static>(&self) -> Option<Rc<T>> {
        self.injector.resolve_skip_self::<T>()
    }

    /// Resolve required, starting from parent.
    pub fn resolve_required<T: 'static>(&self) -> Result<Rc<T>, ResolutionError> {
        self.resolve::<T>()
            .ok_or_else(|| ResolutionError::NotFound(std::any::type_name::<T>()))
    }
}

/// A builder for host-boundary dependency resolution.
///
/// Stops resolution at the component host boundary.
pub struct Host<'a> {
    injector: &'a Injector,
}

impl<'a> Host<'a> {
    /// Create a new host resolver.
    pub fn new(injector: &'a Injector) -> Self {
        Self { injector }
    }

    /// Resolve up to host boundary.
    pub fn resolve<T: 'static>(&self) -> Option<Rc<T>> {
        self.injector.resolve_to_host::<T>()
    }
}

/// Combine resolution modifiers.
///
/// # Example
///
/// ```ignore
/// // Optional + SkipSelf
/// let service = Resolve::new(&injector)
///     .optional()
///     .skip_self()
///     .get::<MyService>();
/// ```
pub struct Resolve<'a> {
    injector: &'a Injector,
    options: ResolutionOptions,
}

impl<'a> Resolve<'a> {
    /// Create a new resolution builder.
    pub fn new(injector: &'a Injector) -> Self {
        Self {
            injector,
            options: ResolutionOptions::default(),
        }
    }

    /// Mark as optional.
    pub fn optional(mut self) -> Self {
        self.options.optional = true;
        self
    }

    /// Only search current injector.
    pub fn self_only(mut self) -> Self {
        self.options.self_only = true;
        self
    }

    /// Skip current injector.
    pub fn skip_self(mut self) -> Self {
        self.options.skip_self = true;
        self
    }

    /// Stop at host boundary.
    pub fn host(mut self) -> Self {
        self.options.host = true;
        self
    }

    /// Execute the resolution.
    pub fn get<T: 'static>(self) -> Option<Rc<T>> {
        self.injector.resolve_with_options::<T>(&self.options)
    }

    /// Execute required resolution.
    pub fn get_required<T: 'static>(self) -> Result<Rc<T>, ResolutionError> {
        self.get::<T>()
            .ok_or_else(|| ResolutionError::NotFound(std::any::type_name::<T>()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::di::Injectable;

    struct TestService;
    impl Injectable for TestService {
        fn create(_: &Injector) -> Self { TestService }
    }

    #[test]
    fn test_optional_not_found() {
        let injector = Injector::root();
        let result = Optional::new(&injector).resolve::<TestService>();
        assert!(result.is_none());
    }

    #[test]
    fn test_optional_with_default() {
        let injector = Injector::root();
        let result = Optional::new(&injector).resolve_or(TestService);
        assert!(Rc::strong_count(&result) == 1);
    }
}

