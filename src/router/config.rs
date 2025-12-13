//! Route configuration and definition.

use std::collections::HashMap;

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
    /// Child routes.
    pub children: Vec<Route>,
    /// Route guards to run before activation.
    pub can_activate: Vec<String>,
    /// Route guards to run before deactivation.
    pub can_deactivate: Vec<String>,
    /// Data resolvers to run before activation.
    pub resolve: HashMap<String, String>,
    /// Static data associated with this route.
    pub data: HashMap<String, String>,
    /// The title for this route.
    pub title: Option<String>,
}

impl Default for Route {
    fn default() -> Self {
        Self {
            path: String::new(),
            component: None,
            redirect_to: None,
            path_match: PathMatch::Prefix,
            children: Vec::new(),
            can_activate: Vec::new(),
            can_deactivate: Vec::new(),
            resolve: HashMap::new(),
            data: HashMap::new(),
            title: None,
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

    /// Add child routes.
    pub fn children(mut self, children: Vec<Route>) -> Self {
        self.children = children;
        self
    }

    /// Add a guard.
    pub fn can_activate(mut self, guard: &str) -> Self {
        self.can_activate.push(guard.to_string());
        self
    }

    /// Set the route title.
    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
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

