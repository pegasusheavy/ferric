//! Service scope definitions.
//!
//! Defines how service instances are created and shared.

/// Defines the lifetime/scope of a service instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scope {
    /// A single instance is created and shared across all requests.
    /// The instance lives as long as the injector.
    Singleton,

    /// A new instance is created for each resolution request.
    /// No caching is performed.
    Transient,

    /// Instance is scoped to a specific injector level.
    /// Child injectors get their own instance, not inherited from parent.
    Scoped,
}

impl Default for Scope {
    #[inline]
    fn default() -> Self {
        Scope::Singleton
    }
}

/// Specifies where a service should be provided.
///
/// This is similar to Angular's `providedIn` option.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Default)]
pub enum ProvidedIn {
    /// Provide in the root injector (application-wide singleton).
    Root,
    /// Provide in any injector that imports the module.
    Any,
    /// Don't provide automatically (must be explicitly provided).
    #[default]
    None,
    /// Provide in a specific module (not yet implemented).
    Module,
    /// Provide in the platform injector (above root, for multi-app scenarios).
    Platform,
}


/// Configuration for automatic service provision.
#[derive(Debug, Clone, Default)]
pub struct ProvideConfig {
    /// Where the service should be provided.
    pub provided_in: ProvidedIn,
    /// The scope of the service.
    pub scope: Scope,
}

impl ProvideConfig {
    /// Create a new provide config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Provide in root injector.
    pub fn root() -> Self {
        Self {
            provided_in: ProvidedIn::Root,
            scope: Scope::Singleton,
        }
    }

    /// Provide in any injector.
    pub fn any() -> Self {
        Self {
            provided_in: ProvidedIn::Any,
            scope: Scope::Singleton,
        }
    }

    /// Don't provide automatically.
    pub fn none() -> Self {
        Self {
            provided_in: ProvidedIn::None,
            scope: Scope::Singleton,
        }
    }

    /// Set the scope.
    pub fn with_scope(mut self, scope: Scope) -> Self {
        self.scope = scope;
        self
    }

    /// Set as transient.
    pub fn transient(mut self) -> Self {
        self.scope = Scope::Transient;
        self
    }

    /// Set as scoped.
    pub fn scoped(mut self) -> Self {
        self.scope = Scope::Scoped;
        self
    }
}

/// Trait for services that declare their own provision configuration.
///
/// Implement this trait to have a service automatically registered
/// when accessed for the first time.
///
/// # Example
///
/// ```ignore
/// struct MyService;
///
/// impl SelfProviding for MyService {
///     fn provide_config() -> ProvideConfig {
///         ProvideConfig::root()
///     }
/// }
/// ```
pub trait SelfProviding {
    /// Get the provision configuration for this service.
    fn provide_config() -> ProvideConfig;
}
