//! Route configuration and definition.

use super::lazy::LazyRouteModule;
use std::collections::HashMap;
use std::rc::Rc;

/// Configuration for a single route.
#[derive(Debug, Clone)]
pub struct Route {
    /// The path pattern for this route (e.g., "/users/:id").
    pub path: String,
    /// The component selector to render for this route.
    pub component: Option<String>,
    /// Redirect to another path.
    pub redirect_to: Option<String>,
    /// Path matching strategy.
    pub path_match: PathMatch,
    /// Child routes (static).
    pub children: Vec<Route>,
    /// Lazy-loaded child routes module.
    pub load_children: Option<Rc<LazyRouteModule>>,
    /// Route guards to run before activation.
    pub can_activate: Vec<String>,
    /// Route guards to run before deactivation.
    pub can_deactivate: Vec<String>,
    /// Guard to check before loading lazy children.
    pub can_load: Vec<String>,
    /// Data resolvers to run before activation.
    pub resolve: HashMap<String, String>,
    /// Static data associated with this route.
    pub data: HashMap<String, String>,
    /// The title for this route.
    pub title: Option<String>,
    /// Named outlet for this route (default: "primary").
    pub outlet: String,
    /// Whether to run guards and resolvers on every navigation (not just first).
    pub run_guards_and_resolvers: RunGuardsAndResolvers,
}

/// When to run guards and resolvers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RunGuardsAndResolvers {
    /// Run on param changes only.
    #[default]
    ParamsChange,
    /// Run on param or query param changes.
    ParamsOrQueryParamsChange,
    /// Run on any URL change.
    PathParamsChange,
    /// Always run on navigation.
    Always,
}

impl Default for Route {
    fn default() -> Self {
        Self {
            path: String::new(),
            component: None,
            redirect_to: None,
            path_match: PathMatch::Prefix,
            children: Vec::new(),
            load_children: None,
            can_activate: Vec::new(),
            can_deactivate: Vec::new(),
            can_load: Vec::new(),
            resolve: HashMap::new(),
            data: HashMap::new(),
            title: None,
            outlet: "primary".to_string(),
            run_guards_and_resolvers: RunGuardsAndResolvers::default(),
        }
    }
}

impl Route {
    /// Create a new route with the given path.
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            ..Default::default()
        }
    }

    /// Set the component for this route.
    pub fn component(mut self, component: &str) -> Self {
        self.component = Some(component.to_string());
        self
    }

    /// Set a redirect for this route.
    pub fn redirect_to(mut self, path: &str) -> Self {
        self.redirect_to = Some(path.to_string());
        self
    }

    /// Set the path matching strategy.
    pub fn path_match(mut self, strategy: PathMatch) -> Self {
        self.path_match = strategy;
        self
    }

    /// Add child routes (static).
    pub fn children(mut self, children: Vec<Route>) -> Self {
        self.children = children;
        self
    }

    /// Set lazy-loaded children using a module path.
    ///
    /// ## Example
    ///
    /// ```ignore
    /// Route::new("/admin")
    ///     .load_children_path("./admin/routes.js")
    /// ```
    pub fn load_children_path(mut self, module_path: &str) -> Self {
        self.load_children = Some(Rc::new(LazyRouteModule::from_path(module_path)));
        self
    }

    /// Set lazy-loaded children using a factory function.
    ///
    /// ## Example
    ///
    /// ```ignore
    /// Route::new("/admin")
    ///     .load_children_factory(|| vec![
    ///         Route::new("dashboard").component("admin-dashboard"),
    ///         Route::new("users").component("admin-users"),
    ///     ])
    /// ```
    pub fn load_children_factory<F>(mut self, factory: F) -> Self
    where
        F: Fn() -> Routes + 'static,
    {
        self.load_children = Some(Rc::new(LazyRouteModule::from_factory(factory)));
        self
    }

    /// Set lazy-loaded children using a LazyRouteModule directly.
    pub fn load_children_module(mut self, module: LazyRouteModule) -> Self {
        self.load_children = Some(Rc::new(module));
        self
    }

    /// Add an activation guard.
    pub fn can_activate(mut self, guard: &str) -> Self {
        self.can_activate.push(guard.to_string());
        self
    }

    /// Add a deactivation guard.
    pub fn can_deactivate(mut self, guard: &str) -> Self {
        self.can_deactivate.push(guard.to_string());
        self
    }

    /// Add a load guard (for lazy-loaded routes).
    pub fn can_load(mut self, guard: &str) -> Self {
        self.can_load.push(guard.to_string());
        self
    }

    /// Set the route title.
    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    /// Add static data to this route.
    pub fn data(mut self, key: &str, value: &str) -> Self {
        self.data.insert(key.to_string(), value.to_string());
        self
    }

    /// Add a resolver to this route.
    pub fn resolve(mut self, key: &str, resolver: &str) -> Self {
        self.resolve.insert(key.to_string(), resolver.to_string());
        self
    }

    /// Set the outlet name for this route.
    pub fn outlet(mut self, outlet: &str) -> Self {
        self.outlet = outlet.to_string();
        self
    }

    /// Set when to run guards and resolvers.
    pub fn run_guards_and_resolvers(mut self, strategy: RunGuardsAndResolvers) -> Self {
        self.run_guards_and_resolvers = strategy;
        self
    }

    /// Check if this route has lazy-loaded children.
    pub fn has_lazy_children(&self) -> bool {
        self.load_children.is_some()
    }

    /// Check if this route is a lazy route that needs loading.
    pub fn needs_loading(&self) -> bool {
        self.load_children
            .as_ref()
            .map(|m| !m.is_loaded())
            .unwrap_or(false)
    }

    /// Get the lazy module if configured.
    pub fn lazy_module(&self) -> Option<&Rc<LazyRouteModule>> {
        self.load_children.as_ref()
    }
}

/// Path matching strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathMatch {
    /// Match the beginning of the URL path.
    Prefix,
    /// Match the entire URL path.
    Full,
}

/// A collection of routes.
pub type Routes = Vec<Route>;

