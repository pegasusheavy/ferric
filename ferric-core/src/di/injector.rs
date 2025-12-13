//! High-performance hierarchical dependency injector.
//!
//! Uses safe Rust with RefCell for interior mutability.
//! Optimized for single-threaded WASM environment with:
//! - FxHashMap for O(1) lookups with fast hashing
//! - Rc-based instance sharing (no cloning overhead)
//! - Minimal runtime type checking

use super::resolution::ResolutionOptions;
use super::token::{InjectionToken, MultiToken, TokenTypeId};
use super::{Injectable, Provider, Scope, ServiceProvider};
use rustc_hash::FxHashMap;
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// Resolution state to detect circular dependencies.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ResolutionState {
    /// Currently being resolved (circular dependency if seen again).
    Resolving,
}

/// Internal storage for providers and instances.
struct InjectorStorage {
    /// Registered service providers (wrapped in Rc for borrow-safe cloning).
    providers: FxHashMap<TypeId, Rc<ServiceProvider>>,
    /// Cached singleton instances.
    singletons: FxHashMap<TypeId, Rc<dyn Any>>,
    /// Scoped instances (per-injector).
    scoped: FxHashMap<TypeId, Rc<dyn Any>>,
    /// Resolution state for circular dependency detection.
    resolving: FxHashMap<TypeId, ResolutionState>,
    /// Token-based providers.
    token_providers: FxHashMap<TokenTypeId, Rc<dyn Any>>,
    /// Multi-provider values.
    multi_providers: FxHashMap<TokenTypeId, Vec<Rc<dyn Any>>>,
}

impl InjectorStorage {
    #[inline]
    fn new() -> Self {
        Self {
            providers: FxHashMap::default(),
            singletons: FxHashMap::default(),
            scoped: FxHashMap::default(),
            resolving: FxHashMap::default(),
            token_providers: FxHashMap::default(),
            multi_providers: FxHashMap::default(),
        }
    }

    #[inline]
    fn with_capacity(cap: usize) -> Self {
        Self {
            providers: FxHashMap::with_capacity_and_hasher(cap, Default::default()),
            singletons: FxHashMap::with_capacity_and_hasher(cap, Default::default()),
            scoped: FxHashMap::default(),
            resolving: FxHashMap::default(),
            token_providers: FxHashMap::default(),
            multi_providers: FxHashMap::default(),
        }
    }
}

/// Flags for special injector properties.
#[derive(Default, Clone, Copy)]
struct InjectorFlags {
    /// Whether this is a host boundary (for component isolation).
    is_host: bool,
}

/// A high-performance hierarchical dependency injector.
///
/// The injector manages service registration and resolution with support for:
/// - Singleton scope (shared instance)
/// - Transient scope (new instance per request)
/// - Scoped instances (per-injector level)
/// - Hierarchical resolution (child inherits from parent)
/// - Injection tokens for non-class dependencies
/// - Multi-providers for collections of services
/// - Resolution modifiers (Optional, Self, SkipSelf, Host)
///
/// ## Performance
///
/// - Uses FxHashMap for fast TypeId-based lookups
/// - Rc-based sharing avoids Clone overhead
/// - RefCell provides safe interior mutability
/// - Circular dependency detection
///
/// ## Example
///
/// ```ignore
/// let injector = Injector::root();
/// injector.register_singleton::<MyService>();
///
/// let service: Rc<MyService> = injector.resolve().unwrap();
/// ```
pub struct Injector {
    /// Parent injector for hierarchical resolution.
    parent: Option<Weak<Injector>>,
    /// Internal storage behind RefCell for interior mutability.
    storage: RefCell<InjectorStorage>,
    /// Depth in the injector hierarchy (for debugging).
    depth: u8,
    /// Special flags.
    flags: InjectorFlags,
}

impl Injector {
    /// Create a root injector with no parent.
    #[inline]
    pub fn root() -> Self {
        Self {
            parent: None,
            storage: RefCell::new(InjectorStorage::new()),
            depth: 0,
            flags: InjectorFlags::default(),
        }
    }

    /// Create a root injector with pre-allocated capacity.
    #[inline]
    pub fn root_with_capacity(capacity: usize) -> Self {
        Self {
            parent: None,
            storage: RefCell::new(InjectorStorage::with_capacity(capacity)),
            depth: 0,
            flags: InjectorFlags::default(),
        }
    }

    /// Create a child injector with this injector as its parent.
    #[inline]
    pub fn child(parent: Rc<Injector>) -> Self {
        let depth = parent.depth.saturating_add(1);
        Self {
            parent: Some(Rc::downgrade(&parent)),
            storage: RefCell::new(InjectorStorage::new()),
            depth,
            flags: InjectorFlags::default(),
        }
    }

    /// Create a child injector marked as a host boundary.
    ///
    /// Host boundaries stop resolution for `@Host()` decorated dependencies.
    #[inline]
    pub fn host_child(parent: Rc<Injector>) -> Self {
        let depth = parent.depth.saturating_add(1);
        Self {
            parent: Some(Rc::downgrade(&parent)),
            storage: RefCell::new(InjectorStorage::new()),
            depth,
            flags: InjectorFlags { is_host: true },
        }
    }

    // ==================== Type-based Registration ====================

    /// Register a service with its default Injectable implementation.
    #[inline]
    pub fn register<T: Injectable>(&self, scope: Scope) {
        let provider = Rc::new(ServiceProvider::for_type::<T>().scope(scope));
        self.storage.borrow_mut().providers.insert(TypeId::of::<T>(), provider);
    }

    /// Register a service as a singleton.
    #[inline]
    pub fn register_singleton<T: Injectable>(&self) {
        self.register::<T>(Scope::Singleton);
    }

    /// Register a service as transient.
    #[inline]
    pub fn register_transient<T: Injectable>(&self) {
        self.register::<T>(Scope::Transient);
    }

    /// Register a service provider.
    #[inline]
    pub fn register_provider(&self, provider: ServiceProvider) {
        let type_id = provider.get_type_id();
        self.storage.borrow_mut().providers.insert(type_id, Rc::new(provider));
    }

    /// Register a pre-existing value as a singleton.
    #[inline]
    pub fn register_value<T: 'static>(&self, value: T) {
        self.storage.borrow_mut().singletons.insert(TypeId::of::<T>(), Rc::new(value));
    }

    /// Register an existing Rc as a singleton.
    #[inline]
    pub fn register_rc<T: 'static>(&self, rc: Rc<T>) {
        self.storage.borrow_mut().singletons.insert(TypeId::of::<T>(), rc);
    }

    /// Register a factory function for a type.
    #[inline]
    pub fn register_factory<T, F>(&self, factory: F, scope: Scope)
    where
        T: 'static,
        F: Fn(&Injector) -> T + 'static,
    {
        let provider = Rc::new(ServiceProvider::with_factory(factory).scope(scope));
        self.storage.borrow_mut().providers.insert(TypeId::of::<T>(), provider);
    }

    // ==================== Token-based Registration ====================

    /// Register a value for an injection token.
    #[inline]
    pub fn register_token<T: 'static>(&self, token: &InjectionToken<T>, value: T) {
        let token_id = token.token_type_id();
        self.storage.borrow_mut().token_providers.insert(token_id, Rc::new(value));
    }

    /// Register an Rc for an injection token.
    #[inline]
    pub fn register_token_rc<T: 'static>(&self, token: &InjectionToken<T>, rc: Rc<T>) {
        let token_id = token.token_type_id();
        self.storage.borrow_mut().token_providers.insert(token_id, rc);
    }

    /// Register a factory for an injection token.
    pub fn register_token_factory<T, F>(&self, token: &InjectionToken<T>, factory: F)
    where
        T: 'static,
        F: FnOnce(&Injector) -> T,
    {
        let value = factory(self);
        self.register_token(token, value);
    }

    // ==================== Multi-provider Registration ====================

    /// Add a value to a multi-provider token.
    #[inline]
    pub fn add_multi<T: 'static>(&self, token: &MultiToken<T>, value: T) {
        let token_id = token.token_type_id();
        let rc: Rc<dyn Any> = Rc::new(value);
        self.storage.borrow_mut()
            .multi_providers
            .entry(token_id)
            .or_default()
            .push(rc);
    }

    /// Add an Rc to a multi-provider token.
    #[inline]
    pub fn add_multi_rc<T: 'static>(&self, token: &MultiToken<T>, rc: Rc<T>) {
        let token_id = token.token_type_id();
        self.storage.borrow_mut()
            .multi_providers
            .entry(token_id)
            .or_default()
            .push(rc);
    }

    // ==================== Type-based Resolution ====================

    /// Resolve a service, returning a shared reference.
    ///
    /// Returns `None` if the service is not registered.
    /// Panics if a circular dependency is detected.
    #[inline]
    pub fn resolve<T: 'static>(&self) -> Option<Rc<T>> {
        let type_id = TypeId::of::<T>();
        self.resolve_by_type_id(type_id)
            .and_then(|rc| rc.downcast::<T>().ok())
    }

    /// Resolve a service or panic with a descriptive error.
    #[inline]
    pub fn resolve_required<T: 'static>(&self) -> Rc<T> {
        self.resolve::<T>().unwrap_or_else(|| {
            panic!(
                "Required service not found: {}",
                std::any::type_name::<T>()
            )
        })
    }

    /// Try to resolve a service, returning an error if not found.
    #[inline]
    pub fn try_resolve<T: 'static>(&self) -> Result<Rc<T>, ResolutionError> {
        self.resolve::<T>()
            .ok_or_else(|| ResolutionError::NotFound(std::any::type_name::<T>()))
    }

    /// Resolve only from this injector (Self-only).
    pub fn resolve_self<T: 'static>(&self) -> Option<Rc<T>> {
        let type_id = TypeId::of::<T>();
        self.resolve_self_by_type_id(type_id)
            .and_then(|rc| rc.downcast::<T>().ok())
    }

    /// Resolve starting from parent (SkipSelf).
    pub fn resolve_skip_self<T: 'static>(&self) -> Option<Rc<T>> {
        if let Some(ref parent_weak) = self.parent {
            if let Some(parent) = parent_weak.upgrade() {
                return parent.resolve::<T>();
            }
        }
        None
    }

    /// Resolve up to host boundary.
    pub fn resolve_to_host<T: 'static>(&self) -> Option<Rc<T>> {
        let type_id = TypeId::of::<T>();
        self.resolve_to_host_by_type_id(type_id)
            .and_then(|rc| rc.downcast::<T>().ok())
    }

    /// Resolve with custom options.
    pub fn resolve_with_options<T: 'static>(&self, options: &ResolutionOptions) -> Option<Rc<T>> {
        let type_id = TypeId::of::<T>();

        // Handle skip_self
        let start_injector: &Injector = if options.skip_self {
            if let Some(parent) = self.parent() {
                // We need to resolve from parent, but we can't return a reference
                // So we handle this case directly
                return parent.resolve_with_options_inner::<T>(type_id, options);
            } else {
                return None;
            }
        } else {
            self
        };

        start_injector.resolve_with_options_inner(type_id, options)
    }

    fn resolve_with_options_inner<T: 'static>(&self, type_id: TypeId, options: &ResolutionOptions) -> Option<Rc<T>> {
        // Try to resolve at this level
        if let Some(rc) = self.resolve_self_by_type_id(type_id) {
            return rc.downcast::<T>().ok();
        }

        // If self_only, don't check parent
        if options.self_only {
            return None;
        }

        // If at host boundary and host flag is set, stop
        if options.host && self.flags.is_host {
            return None;
        }

        // Check parent
        if let Some(parent) = self.parent() {
            return parent.resolve_with_options_inner(type_id, options);
        }

        None
    }

    // ==================== Token-based Resolution ====================

    /// Resolve a value by injection token.
    pub fn resolve_token<T: 'static>(&self, token: &InjectionToken<T>) -> Option<Rc<T>> {
        let token_id = token.token_type_id();

        // Check this injector
        {
            let storage = self.storage.borrow();
            if let Some(rc) = storage.token_providers.get(&token_id) {
                return rc.clone().downcast::<T>().ok();
            }
        }

        // Check parent
        if let Some(parent) = self.parent() {
            return parent.resolve_token(token);
        }

        None
    }

    /// Resolve a token or return a default value.
    pub fn resolve_token_or<T: 'static>(&self, token: &InjectionToken<T>, default: T) -> Rc<T> {
        self.resolve_token(token).unwrap_or_else(|| Rc::new(default))
    }

    /// Resolve a required token.
    pub fn resolve_token_required<T: 'static>(&self, token: &InjectionToken<T>) -> Rc<T> {
        self.resolve_token(token).unwrap_or_else(|| {
            panic!("Required token not found: {}", token.description())
        })
    }

    // ==================== Multi-provider Resolution ====================

    /// Resolve all values for a multi-provider token.
    pub fn resolve_multi<T: 'static>(&self, token: &MultiToken<T>) -> Vec<Rc<T>> {
        let token_id = token.token_type_id();
        let mut results = Vec::new();

        // Collect from this injector
        {
            let storage = self.storage.borrow();
            if let Some(values) = storage.multi_providers.get(&token_id) {
                for rc in values {
                    if let Ok(typed) = rc.clone().downcast::<T>() {
                        results.push(typed);
                    }
                }
            }
        }

        // Collect from parent (prepend parent values)
        if let Some(parent) = self.parent() {
            let mut parent_results = parent.resolve_multi(token);
            parent_results.extend(results);
            return parent_results;
        }

        results
    }

    // ==================== Internal Resolution ====================

    /// Internal resolution by TypeId.
    fn resolve_by_type_id(&self, type_id: TypeId) -> Option<Rc<dyn Any>> {
        // Check for circular dependency
        {
            let storage = self.storage.borrow();
            if storage.resolving.get(&type_id) == Some(&ResolutionState::Resolving) {
                panic!("Circular dependency detected during resolution");
            }
        }

        // Fast path: check singletons first
        {
            let storage = self.storage.borrow();
            if let Some(instance) = storage.singletons.get(&type_id) {
                return Some(Rc::clone(instance));
            }

            // Check scoped instances
            if let Some(instance) = storage.scoped.get(&type_id) {
                return Some(Rc::clone(instance));
            }
        }

        // Check if we have a provider and get it (clone the Rc to avoid holding borrow)
        let provider_rc = {
            let storage = self.storage.borrow();
            storage.providers.get(&type_id).cloned()
        };

        if let Some(provider) = provider_rc {
            let scope = provider.get_scope();

            // Mark as resolving (circular dependency detection)
            self.storage.borrow_mut().resolving.insert(type_id, ResolutionState::Resolving);

            // Create the instance - borrow is now released so create() can re-enter
            let instance = provider.create(self);

            // Clear resolution state and cache based on scope
            {
                let mut storage = self.storage.borrow_mut();
                storage.resolving.remove(&type_id);

                match scope {
                    Scope::Singleton => {
                        storage.singletons.insert(type_id, Rc::clone(&instance));
                    }
                    Scope::Scoped => {
                        storage.scoped.insert(type_id, Rc::clone(&instance));
                    }
                    Scope::Transient => {
                        // Don't cache transient instances
                    }
                }
            }

            return Some(instance);
        }

        // Check parent injector
        if let Some(ref parent_weak) = self.parent {
            if let Some(parent) = parent_weak.upgrade() {
                return parent.resolve_by_type_id(type_id);
            }
        }

        None
    }

    /// Resolve only at this level (no parent lookup).
    fn resolve_self_by_type_id(&self, type_id: TypeId) -> Option<Rc<dyn Any>> {
        // Check singletons
        {
            let storage = self.storage.borrow();
            if let Some(instance) = storage.singletons.get(&type_id) {
                return Some(Rc::clone(instance));
            }
            if let Some(instance) = storage.scoped.get(&type_id) {
                return Some(Rc::clone(instance));
            }
        }

        // Check providers at this level
        let (has_provider, scope) = {
            let storage = self.storage.borrow();
            match storage.providers.get(&type_id) {
                Some(provider) => (true, provider.get_scope()),
                None => return None,
            }
        };

        if has_provider {
            self.storage.borrow_mut().resolving.insert(type_id, ResolutionState::Resolving);

            let instance = {
                let storage = self.storage.borrow();
                let provider = storage.providers.get(&type_id).unwrap();
                provider.create(self)
            };

            {
                let mut storage = self.storage.borrow_mut();
                storage.resolving.remove(&type_id);

                match scope {
                    Scope::Singleton => {
                        storage.singletons.insert(type_id, Rc::clone(&instance));
                    }
                    Scope::Scoped => {
                        storage.scoped.insert(type_id, Rc::clone(&instance));
                    }
                    Scope::Transient => {}
                }
            }

            return Some(instance);
        }

        None
    }

    /// Resolve up to host boundary.
    fn resolve_to_host_by_type_id(&self, type_id: TypeId) -> Option<Rc<dyn Any>> {
        // Try at this level
        if let Some(rc) = self.resolve_self_by_type_id(type_id) {
            return Some(rc);
        }

        // If this is a host boundary, stop
        if self.flags.is_host {
            return None;
        }

        // Try parent
        if let Some(parent) = self.parent() {
            return parent.resolve_to_host_by_type_id(type_id);
        }

        None
    }

    // ==================== Utility Methods ====================

    /// Check if a service is registered (at this level or any parent).
    #[inline]
    pub fn has<T: 'static>(&self) -> bool {
        self.has_by_type_id(TypeId::of::<T>())
    }

    /// Check if a service is registered by TypeId.
    fn has_by_type_id(&self, type_id: TypeId) -> bool {
        {
            let storage = self.storage.borrow();
            if storage.providers.contains_key(&type_id)
                || storage.singletons.contains_key(&type_id)
                || storage.scoped.contains_key(&type_id)
            {
                return true;
            }
        }

        if let Some(ref parent_weak) = self.parent {
            if let Some(parent) = parent_weak.upgrade() {
                return parent.has_by_type_id(type_id);
            }
        }

        false
    }

    /// Check if a token is registered.
    pub fn has_token<T: 'static>(&self, token: &InjectionToken<T>) -> bool {
        let token_id = token.token_type_id();

        if self.storage.borrow().token_providers.contains_key(&token_id) {
            return true;
        }

        if let Some(parent) = self.parent() {
            return parent.has_token(token);
        }

        false
    }

    /// Get the depth of this injector in the hierarchy.
    #[inline]
    pub fn depth(&self) -> u8 {
        self.depth
    }

    /// Check if this is a root injector.
    #[inline]
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    /// Check if this is a host boundary.
    #[inline]
    pub fn is_host(&self) -> bool {
        self.flags.is_host
    }

    /// Get the parent injector if one exists.
    #[inline]
    pub fn parent(&self) -> Option<Rc<Injector>> {
        self.parent.as_ref().and_then(|w| w.upgrade())
    }

    /// Clear all cached instances (singletons and scoped).
    /// Providers remain registered.
    pub fn clear_cache(&self) {
        let mut storage = self.storage.borrow_mut();
        storage.singletons.clear();
        storage.scoped.clear();
    }

    /// Clear only scoped instances.
    pub fn clear_scoped(&self) {
        self.storage.borrow_mut().scoped.clear();
    }

    // ==================== Legacy API ====================

    /// Register a provider (legacy API).
    #[inline]
    pub fn provide<T: Injectable + 'static>(&self, _provider: Provider<T>) {
        self.register_singleton::<T>();
    }

    /// Get a service (legacy API - prefer `resolve`).
    #[inline]
    pub fn get<T: Injectable + Clone + 'static>(&self) -> Option<T> {
        self.resolve::<T>().map(|rc| (*rc).clone())
    }
}

/// Errors that can occur during service resolution.
#[derive(Debug, Clone)]
pub enum ResolutionError {
    /// The requested service was not found.
    NotFound(&'static str),
    /// A circular dependency was detected.
    CircularDependency(&'static str),
    /// Token not found.
    TokenNotFound(String),
}

impl std::fmt::Display for ResolutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(name) => write!(f, "Service not found: {}", name),
            Self::CircularDependency(name) => {
                write!(f, "Circular dependency detected for: {}", name)
            }
            Self::TokenNotFound(name) => write!(f, "Token not found: {}", name),
        }
    }
}

impl std::error::Error for ResolutionError {}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestService {
        value: i32,
    }

    impl Injectable for TestService {
        fn create(_: &Injector) -> Self {
            TestService { value: 42 }
        }
    }

    #[test]
    fn test_singleton_resolution() {
        let injector = Injector::root();
        injector.register_singleton::<TestService>();

        let a = injector.resolve::<TestService>().unwrap();
        let b = injector.resolve::<TestService>().unwrap();

        // Should be the same Rc
        assert!(Rc::ptr_eq(&a, &b));
        assert_eq!(a.value, 42);
    }

    #[test]
    fn test_token_resolution() {
        const API_URL: InjectionToken<String> = InjectionToken::with_id("API_URL", 1000);

        let injector = Injector::root();
        injector.register_token(&API_URL, "https://api.example.com".to_string());

        let url = injector.resolve_token(&API_URL).unwrap();
        assert_eq!(*url, "https://api.example.com");
    }

    #[test]
    fn test_multi_provider() {
        const HANDLERS: MultiToken<String> = MultiToken::with_id("HANDLERS", 2000);

        let injector = Injector::root();
        injector.add_multi(&HANDLERS, "handler1".to_string());
        injector.add_multi(&HANDLERS, "handler2".to_string());

        let handlers = injector.resolve_multi(&HANDLERS);
        assert_eq!(handlers.len(), 2);
    }

    #[test]
    fn test_hierarchical_multi_provider() {
        const HANDLERS: MultiToken<String> = MultiToken::with_id("HANDLERS", 3000);

        let parent = Rc::new(Injector::root());
        parent.add_multi(&HANDLERS, "parent_handler".to_string());

        let child = Injector::child(Rc::clone(&parent));
        child.add_multi(&HANDLERS, "child_handler".to_string());

        let handlers = child.resolve_multi(&HANDLERS);
        assert_eq!(handlers.len(), 2);
    }

    #[test]
    fn test_skip_self() {
        let parent = Rc::new(Injector::root());
        parent.register_value(42i32);

        let child = Injector::child(Rc::clone(&parent));
        child.register_value(100i32);

        // Normal resolve gets child value
        let value = child.resolve::<i32>().unwrap();
        assert_eq!(*value, 100);

        // Skip self gets parent value
        let parent_value = child.resolve_skip_self::<i32>().unwrap();
        assert_eq!(*parent_value, 42);
    }
}
