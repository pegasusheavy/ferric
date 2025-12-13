//! Dependency Injection system for Ferric.
//!
//! A high-performance hierarchical injector for managing services and their dependencies.
//!
//! ## Features
//!
//! - **Hierarchical Injectors**: Parent/child relationships with inheritance
//! - **Injection Tokens**: Non-class dependencies with `InjectionToken<T>`
//! - **Multi-providers**: Multiple implementations for a single token
//! - **Resolution Modifiers**: Optional, Self, SkipSelf, Host
//! - **Provider Scopes**: Singleton, Transient, Scoped
//! - **Factory Providers**: Custom instance creation logic
//!
//! ## Performance Characteristics
//!
//! - O(1) service lookup using FxHashMap with TypeId keys
//! - Zero-cost singleton caching with Rc<T>
//! - Minimal allocations after initial registration
//! - Compile-time type safety with runtime flexibility
//!
//! ## Usage
//!
//! ### Basic Service Registration
//!
//! ```ignore
//! use ferric::di::*;
//!
//! // Define a service
//! struct MyService { value: i32 }
//!
//! impl Injectable for MyService {
//!     fn create(_: &Injector) -> Self {
//!         MyService { value: 42 }
//!     }
//! }
//!
//! // Create injector and register
//! let injector = Injector::root();
//! injector.register_singleton::<MyService>();
//!
//! // Resolve service
//! let service: Rc<MyService> = injector.resolve().unwrap();
//! ```
//!
//! ### Injection Tokens
//!
//! ```ignore
//! use ferric::di::*;
//!
//! // Define a token for a configuration value
//! const API_URL: InjectionToken<String> = InjectionToken::with_id("API_URL", 1);
//!
//! // Register the value
//! injector.register_token(&API_URL, "https://api.example.com".to_string());
//!
//! // Resolve
//! let url = injector.resolve_token(&API_URL).unwrap();
//! ```
//!
//! ### Multi-providers
//!
//! ```ignore
//! use ferric::di::*;
//!
//! // Define a multi-provider token
//! const VALIDATORS: MultiToken<Box<dyn Validator>> = MultiToken::with_id("VALIDATORS", 2);
//!
//! // Register multiple implementations
//! injector.add_multi(&VALIDATORS, Box::new(RequiredValidator));
//! injector.add_multi(&VALIDATORS, Box::new(EmailValidator));
//!
//! // Resolve all
//! let validators = injector.resolve_multi(&VALIDATORS);
//! ```
//!
//! ### Resolution Modifiers
//!
//! ```ignore
//! use ferric::di::*;
//!
//! // Optional dependency
//! let service = Optional::new(&injector).resolve::<MyService>();
//!
//! // Self-only (don't check parent)
//! let service = Self_::new(&injector).resolve::<MyService>();
//!
//! // Skip self (start from parent)
//! let service = SkipSelf::new(&injector).resolve::<MyService>();
//!
//! // Combined modifiers
//! let service = Resolve::new(&injector)
//!     .optional()
//!     .skip_self()
//!     .get::<MyService>();
//! ```

mod injector;
mod provider;
mod resolution;
mod scope;
mod storage;
mod token;

// Core types
pub use injector::{Injector, ResolutionError};
pub use provider::{Provider, ProviderBuilder, ServiceProvider};
pub use scope::{ProvidedIn, ProvideConfig, Scope, SelfProviding};
pub use token::{FactoryToken, InjectionToken, MultiToken, ProviderToken, TokenTypeId};

// Resolution modifiers
pub use resolution::{Host, Optional, Resolve, ResolutionOptions, Self_, SkipSelf};

// Re-export storage types for advanced use cases
#[doc(hidden)]
pub mod storage_types {
    pub use super::storage::{LazySlot, ServiceInstance};
}

// Re-export macros
pub use crate::{define_multi_tokens, define_tokens};

use std::any::Any;

/// Trait for services that can be injected.
///
/// Implement this trait to make a type injectable. The `create` method
/// receives the injector, allowing resolution of dependencies.
///
/// ## Example
///
/// ```ignore
/// struct UserService {
///     db: Rc<DatabaseService>,
/// }
///
/// impl Injectable for UserService {
///     fn create(injector: &Injector) -> Self {
///         Self {
///             db: injector.resolve_required::<DatabaseService>(),
///         }
///     }
/// }
/// ```
pub trait Injectable: Any + 'static {
    /// Create a new instance of the service.
    ///
    /// This method is called by the injector when a new instance is needed.
    /// Use the provided injector to resolve any dependencies.
    fn create(injector: &Injector) -> Self
    where
        Self: Sized;
}

/// Marker trait for services that should always create new instances.
pub trait Transient: Injectable {}

/// Prelude for convenient imports.
pub mod prelude {
    pub use super::{
        // Core
        Injectable, Injector, ResolutionError,

        // Providers
        Provider, ProviderBuilder, ServiceProvider,

        // Scopes
        ProvidedIn, ProvideConfig, Scope, SelfProviding,

        // Tokens
        InjectionToken, MultiToken, ProviderToken,

        // Resolution modifiers
        Host, Optional, Resolve, ResolutionOptions, Self_, SkipSelf,

        // Traits
        Transient,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    struct DatabaseService {
        connection_string: String,
    }

    impl Injectable for DatabaseService {
        fn create(_: &Injector) -> Self {
            Self {
                connection_string: "localhost".to_string(),
            }
        }
    }

    struct UserService {
        db: Rc<DatabaseService>,
    }

    impl Injectable for UserService {
        fn create(injector: &Injector) -> Self {
            Self {
                db: injector.resolve_required::<DatabaseService>(),
            }
        }
    }

    #[test]
    fn test_dependency_chain() {
        let injector = Injector::root();
        injector.register_singleton::<DatabaseService>();
        injector.register_singleton::<UserService>();

        let user_service = injector.resolve::<UserService>().unwrap();
        assert_eq!(user_service.db.connection_string, "localhost");
    }

    #[test]
    fn test_hierarchical_injector() {
        let parent = Rc::new(Injector::root());
        parent.register_singleton::<DatabaseService>();

        let child = Injector::child(Rc::clone(&parent));
        child.register_singleton::<UserService>();

        // Child can resolve from parent
        let user_service = child.resolve::<UserService>().unwrap();
        assert_eq!(user_service.db.connection_string, "localhost");
    }

    #[test]
    fn test_optional_resolution() {
        let injector = Injector::root();

        // Service not registered, should return None
        let result = Optional::new(&injector).resolve::<DatabaseService>();
        assert!(result.is_none());

        // With default
        let result = Optional::new(&injector).resolve_or(DatabaseService {
            connection_string: "default".to_string(),
        });
        assert_eq!(result.connection_string, "default");
    }

    #[test]
    fn test_injection_tokens() {
        const DB_URL: InjectionToken<String> = InjectionToken::with_id("DB_URL", 100);
        const MAX_CONNECTIONS: InjectionToken<u32> = InjectionToken::with_id("MAX_CONNECTIONS", 101);

        let injector = Injector::root();
        injector.register_token(&DB_URL, "postgres://localhost/db".to_string());
        injector.register_token(&MAX_CONNECTIONS, 10);

        let url = injector.resolve_token(&DB_URL).unwrap();
        let max_conn = injector.resolve_token(&MAX_CONNECTIONS).unwrap();

        assert_eq!(*url, "postgres://localhost/db");
        assert_eq!(*max_conn, 10);
    }

    #[test]
    fn test_factory_provider() {
        let injector = Injector::root();

        injector.register_factory(
            |_| DatabaseService {
                connection_string: "factory-created".to_string(),
            },
            Scope::Singleton,
        );

        let db = injector.resolve::<DatabaseService>().unwrap();
        assert_eq!(db.connection_string, "factory-created");
    }

    #[test]
    fn test_scoped_instances() {
        let parent = Rc::new(Injector::root());
        parent.register::<DatabaseService>(Scope::Scoped);

        let child1 = Injector::child(Rc::clone(&parent));
        let child2 = Injector::child(Rc::clone(&parent));

        // Each child gets its own scoped instance
        let db1 = child1.resolve::<DatabaseService>().unwrap();
        let db2 = child2.resolve::<DatabaseService>().unwrap();

        // Same child returns same instance
        let db1_again = child1.resolve::<DatabaseService>().unwrap();
        assert!(Rc::ptr_eq(&db1, &db1_again));
    }
}
