//! Route guards for protecting routes.

use super::Route;

/// Result of a guard check.
pub enum GuardResult {
    /// Allow navigation to proceed.
    Allow,
    /// Block navigation.
    Deny,
    /// Redirect to a different route.
    Redirect(String),
}

/// Trait for guards that run before a route is activated.
pub trait CanActivate {
    /// Check if the route can be activated.
    fn can_activate(&self, route: &Route) -> GuardResult;
}

/// Trait for guards that run before leaving a route.
pub trait CanDeactivate<T> {
    /// Check if the route can be deactivated.
    fn can_deactivate(&self, component: &T, route: &Route) -> GuardResult;
}

/// Trait for guards that run before child routes are loaded.
pub trait CanActivateChild {
    /// Check if child routes can be activated.
    fn can_activate_child(&self, route: &Route) -> GuardResult;
}

/// Trait for guards that control lazy loading of routes.
pub trait CanLoad {
    /// Check if the route module can be loaded.
    fn can_load(&self, route: &Route) -> GuardResult;
}

/// A guard that always allows navigation.
pub struct AlwaysAllow;

impl CanActivate for AlwaysAllow {
    fn can_activate(&self, _route: &Route) -> GuardResult {
        GuardResult::Allow
    }
}

/// A guard that always denies navigation.
pub struct AlwaysDeny;

impl CanActivate for AlwaysDeny {
    fn can_activate(&self, _route: &Route) -> GuardResult {
        GuardResult::Deny
    }
}

