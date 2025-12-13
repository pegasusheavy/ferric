//! Injection tokens for dependency injection.
//!
//! Tokens provide a way to inject values that don't have a specific type,
//! such as configuration values or abstract interfaces.

use std::any::TypeId;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};

/// Global counter for unique token IDs.
static TOKEN_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Generate a unique token ID.
fn next_token_id() -> u64 {
    TOKEN_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// A token used to identify a provider in the dependency injection system.
///
/// Injection tokens are useful when you want to inject a value that:
/// - Doesn't have a unique type (e.g., a primitive like `String`)
/// - Represents an abstract concept (e.g., "API_URL")
/// - Needs to be swapped out for testing
///
/// ## Example
///
/// ```ignore
/// // Define a token
/// const API_URL: InjectionToken<String> = InjectionToken::new("API_URL");
///
/// // Register with injector
/// injector.register_token(&API_URL, "https://api.example.com".to_string());
///
/// // Resolve
/// let url: Rc<String> = injector.resolve_token(&API_URL).unwrap();
/// ```
#[derive(Debug)]
pub struct InjectionToken<T> {
    /// Human-readable description for debugging.
    description: &'static str,
    /// Unique identifier for this token instance.
    unique_id: u64,
    /// Type marker.
    _marker: PhantomData<fn() -> T>,
}

impl<T: 'static> InjectionToken<T> {
    /// Create a new injection token with a description.
    ///
    /// Note: Each call creates a unique token. For global tokens, use `const`
    /// with `InjectionToken::with_id()` or lazy_static.
    #[inline]
    pub fn new(description: &'static str) -> Self {
        Self {
            description,
            unique_id: next_token_id(),
            _marker: PhantomData,
        }
    }

    /// Create a token with a specific ID (for const contexts).
    ///
    /// Use this when you need a compile-time constant token.
    /// Make sure each token has a unique ID.
    #[inline]
    pub const fn with_id(description: &'static str, id: u64) -> Self {
        Self {
            description,
            unique_id: id,
            _marker: PhantomData,
        }
    }

    /// Get the token's description.
    #[inline]
    pub const fn description(&self) -> &'static str {
        self.description
    }

    /// Get the token's unique identifier.
    #[inline]
    pub const fn unique_id(&self) -> u64 {
        self.unique_id
    }

    /// Get a stable type ID for this token.
    ///
    /// Combines the type T's TypeId with this token's unique ID.
    #[inline]
    pub fn token_type_id(&self) -> TokenTypeId {
        TokenTypeId {
            type_id: TypeId::of::<T>(),
            token_id: self.unique_id,
        }
    }
}

impl<T> Clone for InjectionToken<T> {
    fn clone(&self) -> Self {
        Self {
            description: self.description,
            unique_id: self.unique_id,
            _marker: PhantomData,
        }
    }
}

impl<T> Copy for InjectionToken<T> {}

impl<T> PartialEq for InjectionToken<T> {
    fn eq(&self, other: &Self) -> bool {
        self.unique_id == other.unique_id
    }
}

impl<T> Eq for InjectionToken<T> {}

impl<T> Hash for InjectionToken<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.unique_id.hash(state);
    }
}

/// Combined type ID and token ID for unique identification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TokenTypeId {
    type_id: TypeId,
    token_id: u64,
}

/// Trait for types that can be used as provider tokens.
pub trait ProviderToken {
    /// The type of value this token provides.
    type Value;

    /// Get the unique identifier for this token.
    fn token_id(&self) -> TokenTypeId;

    /// Get a description for debugging.
    fn description(&self) -> &'static str;
}

impl<T: 'static> ProviderToken for InjectionToken<T> {
    type Value = T;

    #[inline]
    fn token_id(&self) -> TokenTypeId {
        self.token_type_id()
    }

    #[inline]
    fn description(&self) -> &'static str {
        self.description
    }
}

/// Allow using PhantomData<T> as a type-based token.
impl<T: 'static> ProviderToken for PhantomData<T> {
    type Value = T;

    #[inline]
    fn token_id(&self) -> TokenTypeId {
        TokenTypeId {
            type_id: TypeId::of::<T>(),
            token_id: 0, // Type-based tokens all have ID 0
        }
    }

    #[inline]
    fn description(&self) -> &'static str {
        std::any::type_name::<T>()
    }
}

/// Multi-provider token for registering multiple implementations.
///
/// Use this when you want to inject a collection of services that
/// implement a common interface.
///
/// ## Example
///
/// ```ignore
/// const VALIDATORS: MultiToken<Box<dyn Validator>> = MultiToken::new("VALIDATORS");
///
/// injector.add_to_multi(&VALIDATORS, Box::new(RequiredValidator));
/// injector.add_to_multi(&VALIDATORS, Box::new(EmailValidator));
///
/// let validators: Vec<Rc<Box<dyn Validator>>> = injector.resolve_multi(&VALIDATORS);
/// ```
#[derive(Debug)]
pub struct MultiToken<T> {
    description: &'static str,
    unique_id: u64,
    _marker: PhantomData<fn() -> Vec<T>>,
}

impl<T: 'static> MultiToken<T> {
    /// Create a new multi-provider token.
    #[inline]
    pub fn new(description: &'static str) -> Self {
        Self {
            description,
            unique_id: next_token_id(),
            _marker: PhantomData,
        }
    }

    /// Create with a specific ID (for const contexts).
    #[inline]
    pub const fn with_id(description: &'static str, id: u64) -> Self {
        Self {
            description,
            unique_id: id,
            _marker: PhantomData,
        }
    }

    /// Get the token's description.
    #[inline]
    pub const fn description(&self) -> &'static str {
        self.description
    }

    /// Get the unique ID.
    #[inline]
    pub const fn unique_id(&self) -> u64 {
        self.unique_id
    }

    /// Get the token type ID.
    #[inline]
    pub fn token_type_id(&self) -> TokenTypeId {
        TokenTypeId {
            type_id: TypeId::of::<Vec<T>>(),
            token_id: self.unique_id,
        }
    }
}

impl<T> Clone for MultiToken<T> {
    fn clone(&self) -> Self {
        Self {
            description: self.description,
            unique_id: self.unique_id,
            _marker: PhantomData,
        }
    }
}

impl<T> Copy for MultiToken<T> {}

impl<T> PartialEq for MultiToken<T> {
    fn eq(&self, other: &Self) -> bool {
        self.unique_id == other.unique_id
    }
}

impl<T> Eq for MultiToken<T> {}

impl<T> Hash for MultiToken<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.unique_id.hash(state);
    }
}

/// A factory token for creating instances via a factory function.
///
/// Use when you need fine-grained control over instance creation.
#[derive(Debug)]
pub struct FactoryToken<T> {
    description: &'static str,
    unique_id: u64,
    _marker: PhantomData<fn() -> T>,
}

impl<T: 'static> FactoryToken<T> {
    /// Create a new factory token.
    pub fn new(description: &'static str) -> Self {
        Self {
            description,
            unique_id: next_token_id(),
            _marker: PhantomData,
        }
    }

    /// Get description.
    pub const fn description(&self) -> &'static str {
        self.description
    }

    /// Get unique ID.
    pub const fn unique_id(&self) -> u64 {
        self.unique_id
    }
}

impl<T> Clone for FactoryToken<T> {
    fn clone(&self) -> Self {
        Self {
            description: self.description,
            unique_id: self.unique_id,
            _marker: PhantomData,
        }
    }
}

impl<T> Copy for FactoryToken<T> {}

/// Helper macro to define injection tokens as constants.
///
/// # Example
///
/// ```ignore
/// define_tokens! {
///     /// The API base URL.
///     pub API_URL: String = 1;
///     /// Current user ID.
///     pub USER_ID: u32 = 2;
/// }
/// ```
#[macro_export]
macro_rules! define_tokens {
    ($($(#[$attr:meta])* $vis:vis $name:ident : $ty:ty = $id:expr;)*) => {
        $(
            $(#[$attr])*
            $vis const $name: $crate::di::InjectionToken<$ty> =
                $crate::di::InjectionToken::with_id(stringify!($name), $id);
        )*
    };
}

/// Helper macro to define multi-tokens as constants.
#[macro_export]
macro_rules! define_multi_tokens {
    ($($(#[$attr:meta])* $vis:vis $name:ident : $ty:ty = $id:expr;)*) => {
        $(
            $(#[$attr])*
            $vis const $name: $crate::di::MultiToken<$ty> =
                $crate::di::MultiToken::with_id(stringify!($name), $id);
        )*
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_injection_token_unique() {
        let token1 = InjectionToken::<String>::new("token1");
        let token2 = InjectionToken::<String>::new("token2");

        assert_ne!(token1.unique_id(), token2.unique_id());
    }

    #[test]
    fn test_injection_token_copy() {
        let token1 = InjectionToken::<i32>::new("test");
        let token2 = token1;

        assert_eq!(token1.unique_id(), token2.unique_id());
    }

    #[test]
    fn test_const_token() {
        const MY_TOKEN: InjectionToken<String> = InjectionToken::with_id("MY_TOKEN", 9999);
        assert_eq!(MY_TOKEN.unique_id(), 9999);
    }
}
