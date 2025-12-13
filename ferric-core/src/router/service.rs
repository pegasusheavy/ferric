//! Router service for programmatic navigation.
//!
//! The Router is the main entry point for navigation in Ferric applications.
//!
//! ## Usage
//!
//! ```ignore
//! // Configure routes
//! let routes = vec![
//!     Route::new("/").component("app-home"),
//!     Route::new("/users/:id").component("app-user-detail"),
//!     Route::new("/admin")
//!         .can_activate("AuthGuard")
//!         .children(vec![
//!             Route::new("dashboard").component("admin-dashboard"),
//!             Route::new("users").component("admin-users"),
//!         ]),
//! ];
//!
//! // Create router
//! let router = Router::new(routes);
//!
//! // Navigate programmatically
//! router.navigate("/users/123")?;
//!
//! // Navigate with query params
//! router.navigate_by_url("/search?q=rust&page=2")?;
//!
//! // Subscribe to events
//! router.events().subscribe(|event| {
//!     if event.is_navigation_end() {
//!         println!("Navigation complete: {}", event.url());
//!     }
//! });
//! ```

use super::events::*;
use super::guard::{CanActivate, CanDeactivate, CanLoad, GuardResult};
use super::lazy::{LazyModuleRegistry, LazyLoadError, LazyRouteModule, PreloadConfig};
use super::params::{Params, QueryParams, ParsedUrl, extract_params};
use super::resolver::{ResolverRegistry, ResolveResult};
use super::{Route, Routes, PathMatch};
use crate::di::{Injectable, Injector};
use crate::reactive::{signal, Signal, computed, Computed};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

/// Counter for navigation IDs.
static NEXT_NAV_ID: AtomicU64 = AtomicU64::new(1);

fn next_navigation_id() -> NavigationId {
    NEXT_NAV_ID.fetch_add(1, Ordering::SeqCst)
}

/// The router service provides navigation and URL management.
#[derive(Clone)]
pub struct Router {
    inner: Rc<RefCell<RouterInner>>,
    /// Event emitter for router events.
    events: Rc<RouterEvents>,
    /// Resolver registry.
    resolvers: Rc<ResolverRegistry>,
    /// URL serializer.
    url_serializer: Rc<dyn UrlSerializer>,
    /// Lazy module registry for caching loaded modules.
    lazy_modules: Rc<LazyModuleRegistry>,
}

struct RouterInner {
    /// Configured routes.
    routes: Routes,
    /// Current activated route (reactive).
    current_route: Signal<Option<ActivatedRoute>>,
    /// Current URL (reactive).
    current_url: Signal<String>,
    /// Base href for the application.
    base_href: String,
    /// Navigation state history.
    state_history: Vec<NavigationState>,
    /// Whether navigation is in progress.
    navigating: Signal<bool>,
    /// Last navigation ID.
    last_navigation_id: NavigationId,
    /// Named outlets registry.
    outlets: HashMap<String, Rc<RefCell<Option<web_sys::Element>>>>,
}

impl Router {
    /// Create a new router with the given routes.
    pub fn new(routes: Routes) -> Self {
        Self {
            inner: Rc::new(RefCell::new(RouterInner {
                routes,
                current_route: signal(None),
                current_url: signal(String::from("/")),
                base_href: String::from("/"),
                state_history: Vec::new(),
                navigating: signal(false),
                last_navigation_id: 0,
                outlets: HashMap::new(),
            })),
            events: Rc::new(RouterEvents::new()),
            resolvers: Rc::new(ResolverRegistry::new()),
            url_serializer: Rc::new(DefaultUrlSerializer),
            lazy_modules: Rc::new(LazyModuleRegistry::new()),
        }
    }

    /// Create with a custom URL serializer.
    pub fn with_serializer<S: UrlSerializer + 'static>(mut self, serializer: S) -> Self {
        self.url_serializer = Rc::new(serializer);
        self
    }

    /// Configure preloading for lazy modules.
    pub fn with_preload_config(self, config: PreloadConfig) -> Self {
        self.lazy_modules.set_preload_config(config);
        self
    }

    /// Get the lazy module registry.
    pub fn lazy_modules(&self) -> &LazyModuleRegistry {
        &self.lazy_modules
    }

    /// Set the base href.
    pub fn set_base_href(&self, base: &str) {
        self.inner.borrow_mut().base_href = base.to_string();
    }

    /// Get the base href.
    pub fn base_href(&self) -> String {
        self.inner.borrow().base_href.clone()
    }

    /// Get the event emitter.
    pub fn events(&self) -> &RouterEvents {
        &self.events
    }

    /// Get the resolver registry.
    pub fn resolvers(&self) -> &ResolverRegistry {
        &self.resolvers
    }

    /// Navigate to a URL.
    pub fn navigate(&self, url: &str) -> Result<bool, JsValue> {
        self.navigate_internal(url, NavigationTrigger::Imperative, None)
    }

    /// Navigate to a URL with extras.
    pub fn navigate_by_url(&self, url: &str) -> Result<bool, JsValue> {
        self.navigate_internal(url, NavigationTrigger::Imperative, None)
    }

    /// Navigate with navigation extras.
    pub fn navigate_with_extras(
        &self,
        url: &str,
        extras: NavigationExtras,
    ) -> Result<bool, JsValue> {
        self.navigate_internal(url, NavigationTrigger::Imperative, Some(extras))
    }

    /// Navigate using commands (like Angular's router.navigate(['/users', userId])).
    pub fn navigate_by_commands(&self, commands: &[&str]) -> Result<bool, JsValue> {
        let url = commands.join("/");
        let url = if url.starts_with('/') { url } else { format!("/{}", url) };
        self.navigate(&url)
    }

    /// Navigate with query parameters.
    pub fn navigate_with_params(
        &self,
        url: &str,
        params: &HashMap<String, String>,
    ) -> Result<bool, JsValue> {
        let query_string: String = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        let full_url = if query_string.is_empty() {
            url.to_string()
        } else {
            format!("{}?{}", url, query_string)
        };

        self.navigate(&full_url)
    }

    /// Internal navigation implementation.
    fn navigate_internal(
        &self,
        url: &str,
        trigger: NavigationTrigger,
        extras: Option<NavigationExtras>,
    ) -> Result<bool, JsValue> {
        let nav_id = next_navigation_id();
        let parsed = self.url_serializer.parse(url);
        let full_url = url.to_string();

        // Mark as navigating
        self.inner.borrow().navigating.set(true);
        self.inner.borrow_mut().last_navigation_id = nav_id;

        // Emit NavigationStart
        self.events.emit(Event::NavigationStart(NavigationStart {
            id: nav_id,
            url: full_url.clone(),
            trigger,
            state: extras.as_ref().and_then(|e| e.state.clone()),
        }));

        // Match routes
        let matched = self.match_route_tree(&parsed.path);

        if matched.is_none() {
            // No route matched
            self.events.emit(Event::NavigationError(NavigationError {
                id: nav_id,
                url: full_url.clone(),
                error: "No matching route found".to_string(),
            }));
            self.inner.borrow().navigating.set(false);
            return Ok(false);
        }

        let (route, params) = matched.unwrap();

        // Emit RoutesRecognized
        let snapshot = self.create_snapshot(&route, &params, &parsed);
        self.events.emit(Event::RoutesRecognized(RoutesRecognized {
            id: nav_id,
            url: full_url.clone(),
            state: RouterState {
                root: Some(snapshot.clone()),
                url: full_url.clone(),
            },
        }));

        // Run guards
        self.events.emit(Event::GuardsCheckStart(GuardsCheckStart {
            id: nav_id,
            url: full_url.clone(),
            state: RouterState {
                root: Some(snapshot.clone()),
                url: full_url.clone(),
            },
        }));

        // TODO: Actually run guards here
        let guards_passed = true;

        self.events.emit(Event::GuardsCheckEnd(GuardsCheckEnd {
            id: nav_id,
            url: full_url.clone(),
            should_activate: guards_passed,
        }));

        if !guards_passed {
            self.events.emit(Event::NavigationCancel(NavigationCancel {
                id: nav_id,
                url: full_url.clone(),
                reason: CancelReason::GuardRejected,
            }));
            self.inner.borrow().navigating.set(false);
            return Ok(false);
        }

        // Run resolvers
        self.events.emit(Event::ResolveStart(ResolveStart {
            id: nav_id,
            url: full_url.clone(),
            state: RouterState {
                root: Some(snapshot.clone()),
                url: full_url.clone(),
            },
        }));

        // TODO: Actually run resolvers here

        self.events.emit(Event::ResolveEnd(ResolveEnd {
            id: nav_id,
            url: full_url.clone(),
            state: RouterState {
                root: Some(snapshot.clone()),
                url: full_url.clone(),
            },
        }));

        // Activate route
        self.events.emit(Event::ActivationStart(ActivationStart {
            id: nav_id,
            url: full_url.clone(),
            snapshot: snapshot.clone(),
        }));

        // Create ActivatedRoute
        let activated = ActivatedRoute::from_snapshot(&snapshot, &route);

        // Update state
        {
            let inner = self.inner.borrow();
            inner.current_route.set(Some(activated));
            inner.current_url.set(full_url.clone());
        }

        // Update browser history
        if !matches!(trigger, NavigationTrigger::PopState) {
            let window = web_sys::window().expect("no global window exists");
            let history = window.history()?;

            let replace = extras
                .as_ref()
                .map(|e| e.replace_url)
                .unwrap_or(false);

            if replace {
                history.replace_state_with_url(&JsValue::NULL, "", Some(&full_url))?;
            } else {
                history.push_state_with_url(&JsValue::NULL, "", Some(&full_url))?;
            }
        }

        // Update document title if set
        if let Some(title) = &route.title {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    document.set_title(title);
                }
            }
        }

        self.events.emit(Event::ActivationEnd(ActivationEnd {
            id: nav_id,
            url: full_url.clone(),
            snapshot: snapshot.clone(),
        }));

        // Navigation complete
        self.events.emit(Event::NavigationEnd(NavigationEnd {
            id: nav_id,
            url: full_url.clone(),
            url_after_redirects: full_url,
        }));

        self.inner.borrow().navigating.set(false);
        Ok(true)
    }

    /// Match routes in the route tree.
    fn match_route_tree(&self, path: &str) -> Option<(Route, HashMap<String, String>)> {
        let inner = self.inner.borrow();
        self.match_routes_recursive(&inner.routes, path, "")
    }

    /// Load lazy children for a route if needed.
    /// Returns the updated route with loaded children.
    pub async fn load_lazy_children(
        &self,
        route: &Route,
        nav_id: NavigationId,
        url: &str,
    ) -> Result<Route, LazyLoadError> {
        if let Some(ref lazy_module) = route.load_children {
            if !lazy_module.is_loaded() {
                // Emit load start event
                self.events.emit(Event::RouteConfigLoadStart(RouteConfigLoadStart {
                    id: nav_id,
                    url: url.to_string(),
                    route: route.clone(),
                }));

                // Load the module
                let loaded_routes = lazy_module.load().await?;

                // Create a new route with loaded children
                let mut updated_route = route.clone();
                updated_route.children = loaded_routes;

                // Emit load end event
                self.events.emit(Event::RouteConfigLoadEnd(RouteConfigLoadEnd {
                    id: nav_id,
                    url: url.to_string(),
                    route: updated_route.clone(),
                }));

                return Ok(updated_route);
            } else if let Some(routes) = lazy_module.routes() {
                // Already loaded, just use cached routes
                let mut updated_route = route.clone();
                updated_route.children = routes;
                return Ok(updated_route);
            }
        }

        // No lazy loading needed
        Ok(route.clone())
    }

    /// Navigate asynchronously with lazy loading support.
    pub fn navigate_async(&self, url: &str) {
        let router = self.clone();
        let url = url.to_string();

        spawn_local(async move {
            let _ = router.navigate_with_lazy_load(&url).await;
        });
    }

    /// Internal async navigation with lazy loading.
    async fn navigate_with_lazy_load(&self, url: &str) -> Result<bool, JsValue> {
        let nav_id = next_navigation_id();
        let parsed = self.url_serializer.parse(url);
        let full_url = url.to_string();

        // Mark as navigating
        self.inner.borrow().navigating.set(true);
        self.inner.borrow_mut().last_navigation_id = nav_id;

        // Emit NavigationStart
        self.events.emit(Event::NavigationStart(NavigationStart {
            id: nav_id,
            url: full_url.clone(),
            trigger: NavigationTrigger::Imperative,
            state: None,
        }));

        // First pass: find route that might need lazy loading
        let matched = self.match_route_tree(&parsed.path);

        if matched.is_none() {
            self.events.emit(Event::NavigationError(NavigationError {
                id: nav_id,
                url: full_url.clone(),
                error: "No matching route found".to_string(),
            }));
            self.inner.borrow().navigating.set(false);
            return Ok(false);
        }

        let (mut route, mut params) = matched.unwrap();

        // Check if route needs lazy loading
        if route.needs_loading() {
            match self.load_lazy_children(&route, nav_id, &full_url).await {
                Ok(updated_route) => {
                    // Re-match with loaded children
                    let updated_routes = vec![updated_route];
                    if let Some((new_route, new_params)) = self.match_routes_recursive(&updated_routes, &parsed.path, "") {
                        route = new_route;
                        params = new_params;
                    }
                }
                Err(e) => {
                    self.events.emit(Event::NavigationError(NavigationError {
                        id: nav_id,
                        url: full_url.clone(),
                        error: e.to_string(),
                    }));
                    self.inner.borrow().navigating.set(false);
                    return Ok(false);
                }
            }
        }

        // Continue with standard navigation flow
        self.complete_navigation(nav_id, &full_url, &route, &params, &parsed)
    }

    /// Complete navigation after any lazy loading.
    fn complete_navigation(
        &self,
        nav_id: NavigationId,
        full_url: &str,
        route: &Route,
        params: &HashMap<String, String>,
        parsed: &ParsedUrl,
    ) -> Result<bool, JsValue> {
        // Emit RoutesRecognized
        let snapshot = self.create_snapshot(route, params, parsed);
        self.events.emit(Event::RoutesRecognized(RoutesRecognized {
            id: nav_id,
            url: full_url.to_string(),
            state: RouterState {
                root: Some(snapshot.clone()),
                url: full_url.to_string(),
            },
        }));

        // Run guards
        self.events.emit(Event::GuardsCheckStart(GuardsCheckStart {
            id: nav_id,
            url: full_url.to_string(),
            state: RouterState {
                root: Some(snapshot.clone()),
                url: full_url.to_string(),
            },
        }));

        // TODO: Actually run guards here
        let guards_passed = true;

        self.events.emit(Event::GuardsCheckEnd(GuardsCheckEnd {
            id: nav_id,
            url: full_url.to_string(),
            should_activate: guards_passed,
        }));

        if !guards_passed {
            self.events.emit(Event::NavigationCancel(NavigationCancel {
                id: nav_id,
                url: full_url.to_string(),
                reason: CancelReason::GuardRejected,
            }));
            self.inner.borrow().navigating.set(false);
            return Ok(false);
        }

        // Run resolvers
        self.events.emit(Event::ResolveStart(ResolveStart {
            id: nav_id,
            url: full_url.to_string(),
            state: RouterState {
                root: Some(snapshot.clone()),
                url: full_url.to_string(),
            },
        }));

        // TODO: Actually run resolvers here

        self.events.emit(Event::ResolveEnd(ResolveEnd {
            id: nav_id,
            url: full_url.to_string(),
            state: RouterState {
                root: Some(snapshot.clone()),
                url: full_url.to_string(),
            },
        }));

        // Activate route
        self.events.emit(Event::ActivationStart(ActivationStart {
            id: nav_id,
            url: full_url.to_string(),
            snapshot: snapshot.clone(),
        }));

        // Create ActivatedRoute
        let activated = ActivatedRoute::from_snapshot(&snapshot, route);

        // Update state
        {
            let inner = self.inner.borrow();
            inner.current_route.set(Some(activated));
            inner.current_url.set(full_url.to_string());
        }

        // Update browser history
        let window = web_sys::window().expect("no global window exists");
        let history = window.history()?;
        history.push_state_with_url(&JsValue::NULL, "", Some(full_url))?;

        // Update document title if set
        if let Some(title) = &route.title {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    document.set_title(title);
                }
            }
        }

        self.events.emit(Event::ActivationEnd(ActivationEnd {
            id: nav_id,
            url: full_url.to_string(),
            snapshot: snapshot.clone(),
        }));

        // Navigation complete
        self.events.emit(Event::NavigationEnd(NavigationEnd {
            id: nav_id,
            url: full_url.to_string(),
            url_after_redirects: full_url.to_string(),
        }));

        self.inner.borrow().navigating.set(false);
        Ok(true)
    }

    /// Preload a lazy route's children.
    pub fn preload(&self, path: &str) {
        let router = self.clone();
        let path = path.to_string();

        spawn_local(async move {
            let matched = router.match_route_tree(&path);
            if let Some((route, _)) = matched {
                if route.needs_loading() {
                    let _ = router.load_lazy_children(&route, 0, &path).await;
                }
            }
        });
    }

    /// Preload all lazy routes based on configuration.
    pub fn preload_all(&self) {
        let router = self.clone();

        spawn_local(async move {
            router.lazy_modules.preload_all().await;
        });
    }

    /// Recursively match routes.
    fn match_routes_recursive(
        &self,
        routes: &[Route],
        path: &str,
        parent_path: &str,
    ) -> Option<(Route, HashMap<String, String>)> {
        for route in routes {
            let full_pattern = if parent_path.is_empty() {
                route.path.clone()
            } else if route.path.is_empty() {
                parent_path.to_string()
            } else {
                format!("{}/{}", parent_path.trim_end_matches('/'), route.path.trim_start_matches('/'))
            };

            // Check for redirect
            if let Some(ref redirect) = route.redirect_to {
                if self.path_matches(&full_pattern, path, route.path_match) {
                    // Handle redirect - return the redirect target
                    return self.match_routes_recursive(routes, redirect, "");
                }
            }

            // Try to match this route
            if let Some(params) = extract_params(&full_pattern, path) {
                // Check path match strategy
                let matches = match route.path_match {
                    PathMatch::Full => self.paths_equal(&full_pattern, path, &params),
                    PathMatch::Prefix => true,
                };

                if matches {
                    // If this route has children, try to match them
                    if !route.children.is_empty() {
                        if let Some(child_match) =
                            self.match_routes_recursive(&route.children, path, &full_pattern)
                        {
                            return Some(child_match);
                        }
                    }

                    // Return this route if it has a component
                    if route.component.is_some() {
                        return Some((route.clone(), params));
                    }
                }
            }
        }

        None
    }

    /// Check if a pattern matches a path.
    fn path_matches(&self, pattern: &str, path: &str, strategy: PathMatch) -> bool {
        match strategy {
            PathMatch::Full => extract_params(pattern, path)
                .map(|_| self.paths_equal(pattern, path, &HashMap::new()))
                .unwrap_or(false),
            PathMatch::Prefix => extract_params(pattern, path).is_some(),
        }
    }

    /// Check if paths are equal (accounting for params).
    fn paths_equal(&self, pattern: &str, path: &str, _params: &HashMap<String, String>) -> bool {
        let pattern_len = pattern.split('/').filter(|s| !s.is_empty()).count();
        let path_len = path.split('/').filter(|s| !s.is_empty()).count();
        pattern_len == path_len
    }

    /// Create a route snapshot.
    fn create_snapshot(
        &self,
        route: &Route,
        params: &HashMap<String, String>,
        parsed: &ParsedUrl,
    ) -> ActivatedRouteSnapshot {
        ActivatedRouteSnapshot {
            path: route.path.clone(),
            params: params.clone(),
            query_params: parsed
                .query_params
                .iter()
                .filter_map(|(k, v)| v.first().map(|val| (k.clone(), val.clone())))
                .collect(),
            fragment: parsed.fragment.clone(),
            data: route.data.clone(),
            resolved_data: HashMap::new(),
            component: route.component.clone(),
            children: Vec::new(),
            outlet: "primary".to_string(),
        }
    }

    /// Navigate back in history.
    pub fn back(&self) -> Result<(), JsValue> {
        let window = web_sys::window().expect("no global window exists");
        let history = window.history()?;
        history.back()
    }

    /// Navigate forward in history.
    pub fn forward(&self) -> Result<(), JsValue> {
        let window = web_sys::window().expect("no global window exists");
        let history = window.history()?;
        history.forward()
    }

    /// Get the current URL.
    pub fn url(&self) -> String {
        self.inner.borrow().current_url.get()
    }

    /// Get the current URL as a reactive signal.
    pub fn url_signal(&self) -> Signal<String> {
        self.inner.borrow().current_url.clone()
    }

    /// Get the current activated route.
    pub fn current_route(&self) -> Option<ActivatedRoute> {
        self.inner.borrow().current_route.get()
    }

    /// Get the current route as a reactive signal.
    pub fn current_route_signal(&self) -> Signal<Option<ActivatedRoute>> {
        self.inner.borrow().current_route.clone()
    }

    /// Check if navigation is in progress.
    pub fn is_navigating(&self) -> bool {
        self.inner.borrow().navigating.get()
    }

    /// Create a RouterLink for use in templates.
    pub fn create_link(&self, path: &str) -> RouterLink {
        RouterLink {
            path: path.to_string(),
            query_params: HashMap::new(),
            fragment: None,
            router: self.clone(),
        }
    }

    /// Register a named outlet.
    pub fn register_outlet(&self, name: &str, element: web_sys::Element) {
        self.inner
            .borrow_mut()
            .outlets
            .insert(name.to_string(), Rc::new(RefCell::new(Some(element))));
    }

    /// Unregister a named outlet.
    pub fn unregister_outlet(&self, name: &str) {
        self.inner.borrow_mut().outlets.remove(name);
    }

    /// Initialize router with popstate listener.
    pub fn init(&self) -> Result<(), JsValue> {
        let window = web_sys::window().expect("no global window exists");
        let router = self.clone();

        let closure = Closure::wrap(Box::new(move |_event: web_sys::PopStateEvent| {
            if let Some(window) = web_sys::window() {
                if let Ok(pathname) = window.location().pathname() {
                    let search = window.location().search().unwrap_or_default();
                    let hash = window.location().hash().unwrap_or_default();
                    let full_url = format!("{}{}{}", pathname, search, hash);
                    let _ = router.navigate_internal(&full_url, NavigationTrigger::PopState, None);
                }
            }
        }) as Box<dyn FnMut(_)>);

        window.add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref())?;
        closure.forget();

        // Handle initial navigation
        if let Ok(pathname) = window.location().pathname() {
            let search = window.location().search().unwrap_or_default();
            let hash = window.location().hash().unwrap_or_default();
            let full_url = format!("{}{}{}", pathname, search, hash);
            let _ = self.navigate_internal(&full_url, NavigationTrigger::Initial, None);
        }

        Ok(())
    }
}

impl Injectable for Router {
    fn create(_injector: &Injector) -> Self {
        Self::new(Vec::new())
    }
}

/// Information about the currently activated route.
#[derive(Debug, Clone)]
pub struct ActivatedRoute {
    /// The route configuration.
    pub route: Route,
    /// URL parameters (reactive).
    pub params: Params,
    /// Query parameters (reactive).
    pub query_params: QueryParams,
    /// The URL fragment.
    pub fragment: Signal<Option<String>>,
    /// Static data from the route configuration.
    pub data: HashMap<String, String>,
    /// Resolved data from resolvers.
    pub resolved_data: HashMap<String, String>,
    /// Parent route (if nested).
    pub parent: Option<Box<ActivatedRoute>>,
    /// Child routes.
    pub children: Vec<ActivatedRoute>,
    /// Outlet name.
    pub outlet: String,
}

impl ActivatedRoute {
    /// Create from a snapshot.
    fn from_snapshot(snapshot: &ActivatedRouteSnapshot, route: &Route) -> Self {
        Self {
            route: route.clone(),
            params: Params::from_map(snapshot.params.clone()),
            query_params: QueryParams::from_query_string(
                &snapshot
                    .query_params
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("&"),
            ),
            fragment: signal(snapshot.fragment.clone()),
            data: snapshot.data.clone(),
            resolved_data: snapshot.resolved_data.clone(),
            parent: None,
            children: Vec::new(),
            outlet: snapshot.outlet.clone(),
        }
    }

    /// Get a parameter by name.
    pub fn param(&self, name: &str) -> Option<String> {
        self.params.get(name)
    }

    /// Get a query parameter by name.
    pub fn query_param(&self, name: &str) -> Option<String> {
        self.query_params.get(name)
    }

    /// Get static route data.
    pub fn get_data(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    /// Get resolved data.
    pub fn get_resolved(&self, key: &str) -> Option<&String> {
        self.resolved_data.get(key)
    }
}

/// Navigation extras for customizing navigation behavior.
#[derive(Debug, Clone, Default)]
pub struct NavigationExtras {
    /// Replace the current URL instead of pushing.
    pub replace_url: bool,
    /// Skip location change (useful for guards).
    pub skip_location_change: bool,
    /// Query params to merge.
    pub query_params: Option<HashMap<String, String>>,
    /// How to handle query params.
    pub query_params_handling: QueryParamsHandling,
    /// Fragment to set.
    pub fragment: Option<String>,
    /// State to pass with navigation.
    pub state: Option<NavigationState>,
    /// Preserve fragment from current URL.
    pub preserve_fragment: bool,
}

/// How to handle query params during navigation.
#[derive(Debug, Clone, Copy, Default)]
pub enum QueryParamsHandling {
    /// Replace all query params.
    #[default]
    Replace,
    /// Merge with existing query params.
    Merge,
    /// Preserve existing query params.
    Preserve,
}

/// A router link for declarative navigation.
#[derive(Clone)]
pub struct RouterLink {
    path: String,
    query_params: HashMap<String, String>,
    fragment: Option<String>,
    router: Router,
}

impl RouterLink {
    /// Add a query parameter.
    pub fn query_param(mut self, key: &str, value: &str) -> Self {
        self.query_params.insert(key.to_string(), value.to_string());
        self
    }

    /// Set the fragment.
    pub fn fragment(mut self, fragment: &str) -> Self {
        self.fragment = Some(fragment.to_string());
        self
    }

    /// Get the href for this link.
    pub fn href(&self) -> String {
        let mut url = self.path.clone();

        if !self.query_params.is_empty() {
            let query: String = self
                .query_params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            url.push_str(&format!("?{}", query));
        }

        if let Some(ref frag) = self.fragment {
            url.push_str(&format!("#{}", frag));
        }

        url
    }

    /// Navigate using this link.
    pub fn navigate(&self) -> Result<bool, JsValue> {
        self.router.navigate(&self.href())
    }

    /// Check if this link is active.
    pub fn is_active(&self) -> bool {
        let current = self.router.url();
        current.starts_with(&self.path)
    }

    /// Check if this link is exactly active.
    pub fn is_exact_active(&self) -> bool {
        let current = self.router.url();
        let current_path = current.split('?').next().unwrap_or(&current);
        current_path == self.path
    }
}

/// URL serialization strategy.
pub trait UrlSerializer {
    /// Parse a URL string.
    fn parse(&self, url: &str) -> ParsedUrl;

    /// Serialize a URL.
    fn serialize(&self, parsed: &ParsedUrl) -> String;
}

/// Default URL serializer (path-based routing).
pub struct DefaultUrlSerializer;

impl UrlSerializer for DefaultUrlSerializer {
    fn parse(&self, url: &str) -> ParsedUrl {
        ParsedUrl::parse(url)
    }

    fn serialize(&self, parsed: &ParsedUrl) -> String {
        parsed.to_string()
    }
}

/// Hash-based URL serializer (for older browsers or specific requirements).
pub struct HashUrlSerializer;

impl UrlSerializer for HashUrlSerializer {
    fn parse(&self, url: &str) -> ParsedUrl {
        // Extract path from hash
        let hash_part = url.split('#').nth(1).unwrap_or("");
        ParsedUrl::parse(hash_part)
    }

    fn serialize(&self, parsed: &ParsedUrl) -> String {
        format!("/#{}", parsed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_creation() {
        let routes = vec![
            Route::new("/").component("app-home"),
            Route::new("/about").component("app-about"),
        ];

        let router = Router::new(routes);
        assert_eq!(router.url(), "/");
    }

    #[test]
    fn test_router_link() {
        let router = Router::new(vec![]);
        let link = router.create_link("/users/123")
            .query_param("tab", "posts")
            .fragment("section-1");

        assert_eq!(link.href(), "/users/123?tab=posts#section-1");
    }

    #[test]
    fn test_hash_url_serializer() {
        let serializer = HashUrlSerializer;
        let parsed = serializer.parse("/#/users/123?sort=name");

        assert_eq!(parsed.path, "/users/123");
        assert!(parsed.query_params.contains_key("sort"));
    }
}
