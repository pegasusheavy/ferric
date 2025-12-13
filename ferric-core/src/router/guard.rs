//! Route guards for protecting routes.
//!
//! Guards are used to control navigation by checking conditions before
//! routes are activated or deactivated.
//!
//! ## Guard Types
//!
//! - `CanActivate` - Check before entering a route
//! - `CanDeactivate` - Check before leaving a route
//! - `CanActivateChild` - Check before entering child routes
//! - `CanLoad` - Check before lazy loading a route module
//!
//! ## Usage
//!
//! ```ignore
//! struct AuthGuard;
//!
//! impl CanActivate for AuthGuard {
//!     fn can_activate(&self, route: &Route, state: &RouterStateSnapshot) -> GuardResult {
//!         if is_logged_in() {
//!             GuardResult::Allow
//!         } else {
//!             GuardResult::Redirect("/login".to_string())
//!         }
//!     }
//! }
//! ```

use super::events::ActivatedRouteSnapshot;
use super::Route;
use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

/// Result of a guard check.
#[derive(Debug, Clone)]
pub enum GuardResult {
    /// Allow navigation to proceed.
    Allow,
    /// Block navigation.
    Deny,
    /// Redirect to a different route.
    Redirect(String),
}

impl GuardResult {
    /// Check if navigation is allowed.
    pub fn is_allowed(&self) -> bool {
        matches!(self, GuardResult::Allow)
    }

    /// Get redirect URL if this is a redirect.
    pub fn redirect_url(&self) -> Option<&str> {
        match self {
            GuardResult::Redirect(url) => Some(url),
            _ => None,
        }
    }
}

/// Snapshot of router state for guard checks.
#[derive(Debug, Clone, Default)]
pub struct RouterStateSnapshot {
    /// URL being navigated to.
    pub url: String,
    /// Root route snapshot.
    pub root: Option<ActivatedRouteSnapshot>,
}

/// Trait for guards that run before a route is activated.
pub trait CanActivate {
    /// Check if the route can be activated.
    fn can_activate(&self, route: &Route, state: &RouterStateSnapshot) -> GuardResult;
}

/// Trait for async guards.
pub trait CanActivateAsync {
    /// Check if the route can be activated (async version).
    fn can_activate_async<'a>(
        &'a self,
        route: &'a Route,
        state: &'a RouterStateSnapshot,
    ) -> Pin<Box<dyn Future<Output = GuardResult> + 'a>>;
}

/// Trait for guards that run before leaving a route.
pub trait CanDeactivate<T> {
    /// Check if the route can be deactivated.
    fn can_deactivate(&self, component: &T, route: &Route, state: &RouterStateSnapshot) -> GuardResult;
}

/// Trait for async deactivation guards.
pub trait CanDeactivateAsync<T> {
    /// Check if the route can be deactivated (async version).
    fn can_deactivate_async<'a>(
        &'a self,
        component: &'a T,
        route: &'a Route,
        state: &'a RouterStateSnapshot,
    ) -> Pin<Box<dyn Future<Output = GuardResult> + 'a>>;
}

/// Trait for guards that run before child routes are loaded.
pub trait CanActivateChild {
    /// Check if child routes can be activated.
    fn can_activate_child(&self, route: &Route, state: &RouterStateSnapshot) -> GuardResult;
}

/// Trait for guards that control lazy loading of routes.
pub trait CanLoad {
    /// Check if the route module can be loaded.
    fn can_load(&self, route: &Route) -> GuardResult;
}

/// Trait for async loading guards.
pub trait CanLoadAsync {
    /// Check if the route module can be loaded (async version).
    fn can_load_async<'a>(&'a self, route: &'a Route) -> Pin<Box<dyn Future<Output = GuardResult> + 'a>>;
}

/// Type-erased guard for storage.
pub trait GuardAny {
    /// Run the guard and return result.
    fn check(&self, route: &Route, state: &RouterStateSnapshot) -> GuardResult;

    /// Get the guard's name.
    fn name(&self) -> &str;
}

/// Wrapper for typed guards.
pub struct GuardWrapper<G> {
    name: String,
    guard: G,
}

impl<G: CanActivate + 'static> GuardWrapper<G> {
    /// Create a new wrapper.
    pub fn new(name: &str, guard: G) -> Self {
        Self {
            name: name.to_string(),
            guard,
        }
    }
}

impl<G: CanActivate + 'static> GuardAny for GuardWrapper<G> {
    fn check(&self, route: &Route, state: &RouterStateSnapshot) -> GuardResult {
        self.guard.can_activate(route, state)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Registry of route guards.
pub struct GuardRegistry {
    guards: RefCell<HashMap<String, Rc<dyn GuardAny>>>,
}

impl GuardRegistry {
    /// Create a new registry.
    pub fn new() -> Self {
        Self {
            guards: RefCell::new(HashMap::new()),
        }
    }

    /// Register a guard.
    pub fn register<G: CanActivate + 'static>(&self, name: &str, guard: G) {
        let wrapper = GuardWrapper::new(name, guard);
        self.guards
            .borrow_mut()
            .insert(name.to_string(), Rc::new(wrapper));
    }

    /// Get a guard by name.
    pub fn get(&self, name: &str) -> Option<Rc<dyn GuardAny>> {
        self.guards.borrow().get(name).cloned()
    }

    /// Run all guards for a route.
    pub fn run_guards(
        &self,
        guard_names: &[String],
        route: &Route,
        state: &RouterStateSnapshot,
    ) -> GuardResult {
        for name in guard_names {
            if let Some(guard) = self.get(name) {
                let result = guard.check(route, state);
                if !result.is_allowed() {
                    return result;
                }
            }
        }
        GuardResult::Allow
    }
}

impl Default for GuardRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Built-in Guards
// ============================================================================

/// A guard that always allows navigation.
pub struct AlwaysAllow;

impl CanActivate for AlwaysAllow {
    fn can_activate(&self, _route: &Route, _state: &RouterStateSnapshot) -> GuardResult {
        GuardResult::Allow
    }
}

/// A guard that always denies navigation.
pub struct AlwaysDeny;

impl CanActivate for AlwaysDeny {
    fn can_activate(&self, _route: &Route, _state: &RouterStateSnapshot) -> GuardResult {
        GuardResult::Deny
    }
}

/// A guard that redirects to a specific URL.
pub struct RedirectGuard {
    url: String,
}

impl RedirectGuard {
    /// Create a new redirect guard.
    pub fn new(url: &str) -> Self {
        Self { url: url.to_string() }
    }
}

impl CanActivate for RedirectGuard {
    fn can_activate(&self, _route: &Route, _state: &RouterStateSnapshot) -> GuardResult {
        GuardResult::Redirect(self.url.clone())
    }
}

/// A guard that checks a condition function.
pub struct FunctionGuard<F>
where
    F: Fn(&Route, &RouterStateSnapshot) -> bool,
{
    check_fn: F,
    redirect_url: Option<String>,
}

impl<F> FunctionGuard<F>
where
    F: Fn(&Route, &RouterStateSnapshot) -> bool,
{
    /// Create a new function guard.
    pub fn new(check_fn: F) -> Self {
        Self {
            check_fn,
            redirect_url: None,
        }
    }

    /// Set redirect URL on failure.
    pub fn with_redirect(mut self, url: &str) -> Self {
        self.redirect_url = Some(url.to_string());
        self
    }
}

impl<F> CanActivate for FunctionGuard<F>
where
    F: Fn(&Route, &RouterStateSnapshot) -> bool,
{
    fn can_activate(&self, route: &Route, state: &RouterStateSnapshot) -> GuardResult {
        if (self.check_fn)(route, state) {
            GuardResult::Allow
        } else if let Some(ref url) = self.redirect_url {
            GuardResult::Redirect(url.clone())
        } else {
            GuardResult::Deny
        }
    }
}

/// A guard that requires a specific route data key.
pub struct RequiresDataGuard {
    key: String,
}

impl RequiresDataGuard {
    /// Create a guard that requires specific route data.
    pub fn new(key: &str) -> Self {
        Self { key: key.to_string() }
    }
}

impl CanActivate for RequiresDataGuard {
    fn can_activate(&self, route: &Route, _state: &RouterStateSnapshot) -> GuardResult {
        if route.data.contains_key(&self.key) {
            GuardResult::Allow
        } else {
            GuardResult::Deny
        }
    }
}

/// A guard that combines multiple guards (all must pass).
pub struct CompositeGuard {
    guards: Vec<Rc<dyn GuardAny>>,
}

impl CompositeGuard {
    /// Create a new composite guard.
    pub fn new() -> Self {
        Self { guards: Vec::new() }
    }

    /// Add a guard.
    pub fn add<G: CanActivate + 'static>(mut self, name: &str, guard: G) -> Self {
        self.guards.push(Rc::new(GuardWrapper::new(name, guard)));
        self
    }
}

impl Default for CompositeGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl CanActivate for CompositeGuard {
    fn can_activate(&self, route: &Route, state: &RouterStateSnapshot) -> GuardResult {
        for guard in &self.guards {
            let result = guard.check(route, state);
            if !result.is_allowed() {
                return result;
            }
        }
        GuardResult::Allow
    }
}

/// A guard that passes if any guard passes.
pub struct AnyGuard {
    guards: Vec<Rc<dyn GuardAny>>,
}

impl AnyGuard {
    /// Create a new any-guard.
    pub fn new() -> Self {
        Self { guards: Vec::new() }
    }

    /// Add a guard.
    pub fn add<G: CanActivate + 'static>(mut self, name: &str, guard: G) -> Self {
        self.guards.push(Rc::new(GuardWrapper::new(name, guard)));
        self
    }
}

impl Default for AnyGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl CanActivate for AnyGuard {
    fn can_activate(&self, route: &Route, state: &RouterStateSnapshot) -> GuardResult {
        for guard in &self.guards {
            let result = guard.check(route, state);
            if result.is_allowed() {
                return GuardResult::Allow;
            }
        }
        GuardResult::Deny
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_always_allow() {
        let guard = AlwaysAllow;
        let route = Route::new("/test");
        let state = RouterStateSnapshot::default();

        assert!(guard.can_activate(&route, &state).is_allowed());
    }

    #[test]
    fn test_always_deny() {
        let guard = AlwaysDeny;
        let route = Route::new("/test");
        let state = RouterStateSnapshot::default();

        assert!(!guard.can_activate(&route, &state).is_allowed());
    }

    #[test]
    fn test_redirect_guard() {
        let guard = RedirectGuard::new("/login");
        let route = Route::new("/admin");
        let state = RouterStateSnapshot::default();

        let result = guard.can_activate(&route, &state);
        assert_eq!(result.redirect_url(), Some("/login"));
    }

    #[test]
    fn test_function_guard() {
        let guard = FunctionGuard::new(|route, _| route.path == "/allowed");

        let allowed_route = Route::new("/allowed");
        let denied_route = Route::new("/denied");
        let state = RouterStateSnapshot::default();

        assert!(guard.can_activate(&allowed_route, &state).is_allowed());
        assert!(!guard.can_activate(&denied_route, &state).is_allowed());
    }

    #[test]
    fn test_composite_guard() {
        let guard = CompositeGuard::new()
            .add("allow", AlwaysAllow)
            .add("allow2", AlwaysAllow);

        let route = Route::new("/test");
        let state = RouterStateSnapshot::default();

        assert!(guard.can_activate(&route, &state).is_allowed());
    }

    #[test]
    fn test_guard_registry() {
        let registry = GuardRegistry::new();
        registry.register("auth", AlwaysAllow);

        let guard = registry.get("auth");
        assert!(guard.is_some());
    }
}
