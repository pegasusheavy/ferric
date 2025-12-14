//! Dependency injection container.
//!
//! The injector manages service instances and their dependencies.
//!
//! # Examples
//!
//! ## Basic Service Registration
//!
//! ```
//! use ferric_core::di::{Injector, Injectable, ProviderBuilder, Scope};
//!
//! #[derive(Clone)]
//! struct MyService {
//!     value: i32,
//! }
//!
//! impl Injectable for MyService {
//!     fn create(_injector: &Injector) -> Self {
//!         Self { value: 42 }
//!     }
//! }
//!
//! let injector = Injector::root();
//! injector.provide(ProviderBuilder::<MyService>::new()
//!     .scope(Scope::Singleton)
//!     .build());
//!
//! let service = injector.resolve_required::<MyService>();
//! assert_eq!(service.value, 42);
//! ```
//!
//! ## Hierarchical Injectors
//!
//! ```
//! use ferric_core::di::{Injector, Injectable, ProviderBuilder};
//!
//! #[derive(Clone)]
//! struct Config {
//!     env: String,
//! }
//!
//! impl Injectable for Config {
//!     fn create(_: &Injector) -> Self {
//!         Self { env: "production".to_string() }
//!     }
//! }
//!
//! let root = Injector::root();
//! root.provide(ProviderBuilder::<Config>::new().build());
//!
//! let child = root.create_child();
//! child.provide(ProviderBuilder::<Config>::new()
//!     .factory(|_| Config { env: "development".to_string() })
//!     .build());
//!
//! let root_config = root.resolve_required::<Config>();
//! let child_config = child.resolve_required::<Config>();
//!
//! assert_eq!(root_config.env, "production");
//! assert_eq!(child_config.env, "development");
//! ```
//!
//! ## Injection Tokens
//!
//! ```
//! use ferric_core::di::{Injector, InjectionToken};
//!
//! const API_URL: InjectionToken<String> = InjectionToken::with_id("API_URL", 1);
//! const MAX_RETRIES: InjectionToken<u32> = InjectionToken::with_id("MAX_RETRIES", 2);
//!
//! let injector = Injector::root();
//! injector.register_token(&API_URL, "https://api.example.com".to_string());
//! injector.register_token(&MAX_RETRIES, 3);
//!
//! assert_eq!(injector.resolve_token(&API_URL).unwrap(), "https://api.example.com");
//! assert_eq!(*injector.resolve_token(&MAX_RETRIES).unwrap(), 3);
//! ```

use super::{Injectable, Provider, InjectionToken, Scope};
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Dependency injection container.
///
/// # Examples
///
/// ```
/// use ferric_core::di::{Injector, Injectable};
///
/// #[derive(Clone)]
/// struct Logger;
///
/// impl Injectable for Logger {
///     fn create(_: &Injector) -> Self { Logger }
/// }
///
/// let injector = Injector::root();
/// // Use injector to resolve services
/// ```
pub struct Injector {
    parent: Option<Rc<Injector>>,
    providers: RefCell<HashMap<TypeId, Provider>>,
    singletons: RefCell<HashMap<TypeId, Rc<dyn Any>>>,
    tokens: RefCell<HashMap<u64, Rc<dyn Any>>>,
}

impl Injector {
    /// Creates a root injector with no parent.
    ///
    /// # Examples
    ///
    /// ```
    /// use ferric_core::di::Injector;
    ///
    /// let injector = Injector::root();
    /// // Register providers and resolve dependencies
    /// ```
    pub fn root() -> Rc<Self> {
        Rc::new(Self {
            parent: None,
            providers: RefCell::new(HashMap::new()),
            singletons: RefCell::new(HashMap::new()),
            tokens: RefCell::new(HashMap::new()),
        })
    }

    /// Creates a child injector.
    ///
    /// Child injectors inherit providers from their parent but maintain
    /// separate singleton instances.
    ///
    /// # Examples
    ///
    /// ```
    /// use ferric_core::di::{Injector, Injectable, ProviderBuilder};
    ///
    /// #[derive(Clone)]
    /// struct Service { level: String }
    ///
    /// impl Injectable for Service {
    ///     fn create(_: &Injector) -> Self {
    ///         Self { level: "root".to_string() }
    ///     }
    /// }
    ///
    /// let root = Injector::root();
    /// root.provide(ProviderBuilder::<Service>::new().build());
    ///
    /// let child = root.create_child();
    /// child.provide(ProviderBuilder::<Service>::new()
    ///     .factory(|_| Service { level: "child".to_string() })
    ///     .build());
    /// ```
    pub fn create_child(self: &Rc<Self>) -> Rc<Self> {
        Rc::new(Self {
            parent: Some(self.clone()),
            providers: RefCell::new(HashMap::new()),
            singletons: RefCell::new(HashMap::new()),
            tokens: RefCell::new(HashMap::new()),
        })
    }

    /// Registers a provider.
    ///
    /// # Examples
    ///
    /// ```
    /// use ferric_core::di::{Injector, Injectable, ProviderBuilder, Scope};
    ///
    /// #[derive(Clone)]
    /// struct MyService;
    ///
    /// impl Injectable for MyService {
    ///     fn create(_: &Injector) -> Self { MyService }
    /// }
    ///
    /// let injector = Injector::root();
    /// injector.provide(ProviderBuilder::<MyService>::new()
    ///     .scope(Scope::Singleton)
    ///     .build());
    /// ```
    pub fn provide(&self, provider: Provider) {
        self.providers.borrow_mut().insert(provider.type_id, provider);
    }

    /// Resolves a service, returning `None` if not found.
    ///
    /// # Examples
    ///
    /// ```
    /// use ferric_core::di::{Injector, Injectable, ProviderBuilder};
    ///
    /// #[derive(Clone)]
    /// struct Optional;
    ///
    /// impl Injectable for Optional {
    ///     fn create(_: &Injector) -> Self { Optional }
    /// }
    ///
    /// let injector = Injector::root();
    ///
    /// // Not registered yet
    /// assert!(injector.resolve::<Optional>().is_none());
    ///
    /// // After registration
    /// injector.provide(ProviderBuilder::<Optional>::new().build());
    /// assert!(injector.resolve::<Optional>().is_some());
    /// ```
    pub fn resolve<T: Injectable + Clone + 'static>(&self) -> Option<T> {
        let type_id = TypeId::of::<T>();

        // Check if singleton exists
        if let Some(instance) = self.singletons.borrow().get(&type_id) {
            if let Some(t) = instance.downcast_ref::<T>() {
                return Some(t.clone());
            }
        }

        // Check for provider
        if let Some(provider) = self.providers.borrow().get(&type_id) {
            let instance = T::create(self);

            if matches!(provider.scope, Scope::Singleton) {
                self.singletons.borrow_mut().insert(type_id, Rc::new(instance.clone()));
            }

            return Some(instance);
        }

        // Check parent
        if let Some(parent) = &self.parent {
            return parent.resolve::<T>();
        }

        None
    }

    /// Resolves a service, panicking if not found.
    ///
    /// # Examples
    ///
    /// ```
    /// use ferric_core::di::{Injector, Injectable, ProviderBuilder};
    ///
    /// #[derive(Clone)]
    /// struct Required { value: i32 }
    ///
    /// impl Injectable for Required {
    ///     fn create(_: &Injector) -> Self {
    ///         Self { value: 123 }
    ///     }
    /// }
    ///
    /// let injector = Injector::root();
    /// injector.provide(ProviderBuilder::<Required>::new().build());
    ///
    /// let service = injector.resolve_required::<Required>();
    /// assert_eq!(service.value, 123);
    /// ```
    pub fn resolve_required<T: Injectable + Clone + 'static>(&self) -> T {
        self.resolve::<T>()
            .expect(&format!("Failed to resolve required service: {}", std::any::type_name::<T>()))
    }

    /// Registers a value for an injection token.
    ///
    /// # Examples
    ///
    /// ```
    /// use ferric_core::di::{Injector, InjectionToken};
    ///
    /// const CONFIG_PATH: InjectionToken<String> = InjectionToken::with_id("CONFIG_PATH", 1);
    ///
    /// let injector = Injector::root();
    /// injector.register_token(&CONFIG_PATH, "/etc/app/config.toml".to_string());
    ///
    /// let path = injector.resolve_token(&CONFIG_PATH).unwrap();
    /// assert_eq!(path.as_str(), "/etc/app/config.toml");
    /// ```
    pub fn register_token<T: Clone + 'static>(&self, token: &InjectionToken<T>, value: T) {
        self.tokens.borrow_mut().insert(token.id, Rc::new(value));
    }

    /// Resolves a value for an injection token.
    ///
    /// # Examples
    ///
    /// ```
    /// use ferric_core::di::{Injector, InjectionToken};
    ///
    /// const PORT: InjectionToken<u16> = InjectionToken::with_id("PORT", 1);
    ///
    /// let injector = Injector::root();
    /// injector.register_token(&PORT, 8080);
    ///
    /// assert_eq!(*injector.resolve_token(&PORT).unwrap(), 8080);
    /// ```
    pub fn resolve_token<T: Clone + 'static>(&self, token: &InjectionToken<T>) -> Option<Rc<T>> {
        if let Some(value) = self.tokens.borrow().get(&token.id) {
            if let Some(t) = value.downcast_ref::<T>() {
                return Some(Rc::new(t.clone()));
            }
        }

        // Check parent
        if let Some(parent) = &self.parent {
            return parent.resolve_token(token);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::di::ProviderBuilder;

    #[derive(Clone)]
    struct TestService {
        value: i32,
    }

    impl Injectable for TestService {
        fn create(_: &Injector) -> Self {
            Self { value: 42 }
        }
    }

    #[test]
    fn test_basic_resolution() {
        let injector = Injector::root();
        injector.provide(ProviderBuilder::<TestService>::new().build());

        let service = injector.resolve::<TestService>().unwrap();
        assert_eq!(service.value, 42);
    }

    #[test]
    fn test_singleton_scope() {
        let injector = Injector::root();
        injector.provide(ProviderBuilder::<TestService>::new()
            .scope(Scope::Singleton)
            .build());

        let service1 = injector.resolve::<TestService>().unwrap();
        let service2 = injector.resolve::<TestService>().unwrap();

        // Should be the same instance (both have value 42)
        assert_eq!(service1.value, 42);
        assert_eq!(service2.value, 42);
    }

    #[test]
    fn test_hierarchical_injectors() {
        let root = Injector::root();
        root.provide(ProviderBuilder::<TestService>::new().build());

        let child = root.create_child();

        // Child should inherit from parent
        let service = child.resolve::<TestService>().unwrap();
        assert_eq!(service.value, 42);
    }

    #[test]
    fn test_injection_tokens() {
        const TEST_TOKEN: InjectionToken<String> = InjectionToken::with_id("TEST", 999);

        let injector = Injector::root();
        injector.register_token(&TEST_TOKEN, "test value".to_string());

        let value = injector.resolve_token(&TEST_TOKEN).unwrap();
        assert_eq!(value.as_str(), "test value");
    }
}
