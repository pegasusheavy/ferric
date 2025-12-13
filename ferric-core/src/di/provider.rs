//! Service providers for dependency injection.
//!
//! Providers define how service instances are created.

use super::{Injectable, Injector, Scope};
use std::any::TypeId;
use std::rc::Rc;

/// Type-erased factory function.
type FactoryFn = Box<dyn Fn(&Injector) -> Rc<dyn std::any::Any>>;

/// A provider that knows how to create instances of a service.
pub struct ServiceProvider {
    /// The factory function that creates instances.
    factory: FactoryFn,
    /// The scope of instances created by this provider.
    scope: Scope,
    /// The type ID of the service this provider creates.
    type_id: TypeId,
}

impl ServiceProvider {
    /// Create a provider for an Injectable type.
    #[inline]
    pub fn for_type<T: Injectable>() -> Self {
        Self {
            factory: Box::new(|injector| Rc::new(T::create(injector)) as Rc<dyn std::any::Any>),
            scope: Scope::Singleton,
            type_id: TypeId::of::<T>(),
        }
    }

    /// Create a provider with a custom factory.
    #[inline]
    pub fn with_factory<T, F>(factory: F) -> Self
    where
        T: 'static,
        F: Fn(&Injector) -> T + 'static,
    {
        Self {
            factory: Box::new(move |injector| Rc::new(factory(injector)) as Rc<dyn std::any::Any>),
            scope: Scope::Singleton,
            type_id: TypeId::of::<T>(),
        }
    }

    /// Create a provider that returns a pre-existing value.
    #[inline]
    pub fn with_value<T: 'static>(value: T) -> Self {
        let rc = Rc::new(value);
        Self {
            factory: Box::new(move |_| Rc::clone(&rc) as Rc<dyn std::any::Any>),
            scope: Scope::Singleton,
            type_id: TypeId::of::<T>(),
        }
    }

    /// Create a provider from an existing Rc.
    #[inline]
    pub fn with_rc<T: 'static>(rc: Rc<T>) -> Self {
        Self {
            factory: Box::new(move |_| Rc::clone(&rc) as Rc<dyn std::any::Any>),
            scope: Scope::Singleton,
            type_id: TypeId::of::<T>(),
        }
    }

    /// Set the scope for this provider.
    #[inline]
    pub fn scope(mut self, scope: Scope) -> Self {
        self.scope = scope;
        self
    }

    /// Mark this provider as transient (new instance each time).
    #[inline]
    pub fn transient(mut self) -> Self {
        self.scope = Scope::Transient;
        self
    }

    /// Mark this provider as singleton (shared instance).
    #[inline]
    pub fn singleton(mut self) -> Self {
        self.scope = Scope::Singleton;
        self
    }

    /// Mark this provider as scoped (per-injector instance).
    #[inline]
    pub fn scoped(mut self) -> Self {
        self.scope = Scope::Scoped;
        self
    }

    /// Get the scope of this provider.
    #[inline]
    pub fn get_scope(&self) -> Scope {
        self.scope
    }

    /// Get the type ID this provider creates.
    #[inline]
    pub fn get_type_id(&self) -> TypeId {
        self.type_id
    }

    /// Create an instance using this provider.
    #[inline]
    pub(crate) fn create(&self, injector: &Injector) -> Rc<dyn std::any::Any> {
        (self.factory)(injector)
    }
}

/// Builder for creating service providers with a fluent API.
pub struct ProviderBuilder<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T: Injectable> ProviderBuilder<T> {
    /// Create a new provider builder.
    #[inline]
    pub const fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }

    /// Build a provider using the type's Injectable implementation.
    #[inline]
    pub fn build(self) -> ServiceProvider {
        ServiceProvider::for_type::<T>()
    }

    /// Build a provider with a custom factory.
    #[inline]
    pub fn factory<F>(self, f: F) -> ServiceProvider
    where
        F: Fn(&Injector) -> T + 'static,
    {
        ServiceProvider::with_factory(f)
    }

    /// Build a provider with a specific value.
    #[inline]
    pub fn value(self, value: T) -> ServiceProvider {
        ServiceProvider::with_value(value)
    }
}

impl<T: Injectable> Default for ProviderBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

// Keep the old Provider type for backwards compatibility
/// Legacy provider type (prefer ServiceProvider for new code).
pub struct Provider<T> {
    factory: Box<dyn Fn(&Injector) -> T>,
}

impl<T: Injectable> Provider<T> {
    /// Create a provider that uses the service's default creation method.
    #[inline]
    pub fn new() -> Self
    where
        T: Sized,
    {
        Self {
            factory: Box::new(|injector| T::create(injector)),
        }
    }

    /// Create a provider with a custom factory function.
    #[inline]
    pub fn with_factory<F>(factory: F) -> Self
    where
        F: Fn(&Injector) -> T + 'static,
    {
        Self {
            factory: Box::new(factory),
        }
    }

    /// Create a provider that always returns a specific value.
    #[inline]
    pub fn with_value(value: T) -> Self
    where
        T: Clone + 'static,
    {
        Self {
            factory: Box::new(move |_| value.clone()),
        }
    }

    /// Create an instance using this provider.
    #[inline]
    pub fn create(&self, injector: &Injector) -> T {
        (self.factory)(injector)
    }
}

impl<T: Injectable> Default for Provider<T>
where
    T: Sized,
{
    fn default() -> Self {
        Self::new()
    }
}
