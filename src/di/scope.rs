//! Service scope definitions.

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

