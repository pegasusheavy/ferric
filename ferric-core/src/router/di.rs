//! Dependency Injection integration for Router.

use super::{Router, Routes};
use crate::di::{InjectionToken, Injector, Injectable};

/// Injection tokens for Router configuration.
pub mod tokens {
    use super::*;

    /// Base href for the application.
    pub const ROUTER_BASE_HREF: InjectionToken<String> =
        InjectionToken::with_id("ROUTER_BASE_HREF", 2001);

    /// Enable HTML5 history mode (vs hash mode).
    pub const ROUTER_USE_HASH: InjectionToken<bool> =
        InjectionToken::with_id("ROUTER_USE_HASH", 2002);

    /// Scroll restoration strategy.
    pub const ROUTER_SCROLL_RESTORATION: InjectionToken<String> =
        InjectionToken::with_id("ROUTER_SCROLL_RESTORATION", 2003);

    /// Enable/disable route tracing for debugging.
    pub const ROUTER_ENABLE_TRACING: InjectionToken<bool> =
        InjectionToken::with_id("ROUTER_ENABLE_TRACING", 2004);

    /// Initial navigation URL.
    pub const ROUTER_INITIAL_NAVIGATION: InjectionToken<String> =
        InjectionToken::with_id("ROUTER_INITIAL_NAVIGATION", 2005);

    /// Routes configuration.
    pub const ROUTER_ROUTES: InjectionToken<Routes> =
        InjectionToken::with_id("ROUTER_ROUTES", 2006);
}

/// Helper to create a router from DI configuration.
pub fn create_router_from_injector(injector: &Injector) -> Router {
    // Get routes from DI or use empty
    let routes = injector
        .resolve_token(&tokens::ROUTER_ROUTES)
        .map(|r| (*r).clone())
        .unwrap_or_default();

    let router = Router::new(routes);

    // Configure base href
    if let Some(base_href) = injector.resolve_token(&tokens::ROUTER_BASE_HREF) {
        router.set_base_href(&base_href);
    }

    router
}

/// Router module for dependency injection setup.
pub struct RouterModule;

impl RouterModule {
    /// Provide router with routes.
    pub fn provide_with_routes(injector: &Injector, routes: Routes) {
        injector.register_token(&tokens::ROUTER_ROUTES, routes);
        injector.register_singleton::<Router>();
    }

    /// Provide router with base href.
    pub fn provide_with_base_href(injector: &Injector, base_href: impl Into<String>) {
        injector.register_token(&tokens::ROUTER_BASE_HREF, base_href.into());
        injector.register_singleton::<Router>();
    }

    /// Provide router with full configuration.
    pub fn provide_with_config<F>(injector: &Injector, configure: F)
    where
        F: Fn(&Injector),
    {
        configure(injector);
        injector.register_singleton::<Router>();
    }

    /// Provide default router.
    pub fn provide_default(injector: &Injector) {
        injector.register_singleton::<Router>();
    }
}

/// Injectable wrapper for Router with DI configuration.
#[derive(Clone)]
pub struct InjectableRouter {
    router: Router,
}

impl InjectableRouter {
    pub fn new(router: Router) -> Self {
        Self { router }
    }

    pub fn router(&self) -> &Router {
        &self.router
    }
}

impl std::ops::Deref for InjectableRouter {
    type Target = Router;

    fn deref(&self) -> &Self::Target {
        &self.router
    }
}

impl Injectable for InjectableRouter {
    fn create(injector: &Injector) -> Self {
        Self::new(create_router_from_injector(injector))
    }
}

