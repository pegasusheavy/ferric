//! Route data resolvers for pre-fetching data before route activation.

use super::events::ActivatedRouteSnapshot;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Result of a resolver operation.
#[derive(Debug, Clone)]
pub enum ResolveResult<T> {
    /// Data is ready.
    Ready(T),
    /// Resolution is pending (async).
    Pending,
    /// Resolution failed with an error.
    Error(String),
    /// Redirect to another route.
    Redirect(String),
}

impl<T> ResolveResult<T> {
    /// Map the ready value.
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> ResolveResult<U> {
        match self {
            ResolveResult::Ready(v) => ResolveResult::Ready(f(v)),
            ResolveResult::Pending => ResolveResult::Pending,
            ResolveResult::Error(e) => ResolveResult::Error(e),
            ResolveResult::Redirect(url) => ResolveResult::Redirect(url),
        }
    }

    /// Check if ready.
    pub fn is_ready(&self) -> bool {
        matches!(self, ResolveResult::Ready(_))
    }

    /// Get the ready value.
    pub fn value(self) -> Option<T> {
        match self {
            ResolveResult::Ready(v) => Some(v),
            _ => None,
        }
    }
}

/// Trait for synchronous resolvers.
pub trait Resolve {
    /// The type of data this resolver produces.
    type Output;

    /// Resolve data for a route.
    fn resolve(&self, route: &ActivatedRouteSnapshot) -> ResolveResult<Self::Output>;

    /// Get a unique key for caching (optional).
    fn cache_key(&self, _route: &ActivatedRouteSnapshot) -> Option<String> {
        None
    }
}

/// Type-erased resolver for storage in collections.
pub trait ResolverAny {
    /// Resolve and return as Any.
    fn resolve_any(&self, route: &ActivatedRouteSnapshot) -> ResolveResult<Box<dyn Any>>;

    /// Get the resolver's name.
    fn name(&self) -> &str;
}

/// Wrapper to make typed resolvers into type-erased ones.
pub struct ResolverWrapper<R: Resolve> {
    name: String,
    resolver: R,
}

impl<R: Resolve> ResolverWrapper<R>
where
    R::Output: 'static,
{
    /// Create a new wrapper.
    pub fn new(name: &str, resolver: R) -> Self {
        Self {
            name: name.to_string(),
            resolver,
        }
    }
}

impl<R: Resolve> ResolverAny for ResolverWrapper<R>
where
    R::Output: 'static,
{
    fn resolve_any(&self, route: &ActivatedRouteSnapshot) -> ResolveResult<Box<dyn Any>> {
        self.resolver.resolve(route).map(|v| Box::new(v) as Box<dyn Any>)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Registry of data resolvers.
pub struct ResolverRegistry {
    resolvers: RefCell<HashMap<String, Rc<dyn ResolverAny>>>,
}

impl ResolverRegistry {
    /// Create a new registry.
    pub fn new() -> Self {
        Self {
            resolvers: RefCell::new(HashMap::new()),
        }
    }

    /// Register a resolver.
    pub fn register<R: Resolve + 'static>(&self, name: &str, resolver: R)
    where
        R::Output: 'static,
    {
        let wrapper = ResolverWrapper::new(name, resolver);
        self.resolvers
            .borrow_mut()
            .insert(name.to_string(), Rc::new(wrapper));
    }

    /// Get a resolver by name.
    pub fn get(&self, name: &str) -> Option<Rc<dyn ResolverAny>> {
        self.resolvers.borrow().get(name).cloned()
    }

    /// Run all resolvers for a route and return resolved data.
    pub fn resolve_all(
        &self,
        resolver_names: &[String],
        route: &ActivatedRouteSnapshot,
    ) -> HashMap<String, ResolveResult<Box<dyn Any>>> {
        let mut results = HashMap::new();

        for name in resolver_names {
            if let Some(resolver) = self.get(name) {
                results.insert(name.clone(), resolver.resolve_any(route));
            } else {
                results.insert(
                    name.clone(),
                    ResolveResult::Error(format!("Resolver '{}' not found", name)),
                );
            }
        }

        results
    }

    /// Check if all resolved data is ready.
    pub fn all_ready(results: &HashMap<String, ResolveResult<Box<dyn Any>>>) -> bool {
        results.values().all(|r| r.is_ready())
    }

    /// Check if any resolved data has errors.
    pub fn has_errors(results: &HashMap<String, ResolveResult<Box<dyn Any>>>) -> bool {
        results.values().any(|r| matches!(r, ResolveResult::Error(_)))
    }
}

impl Default for ResolverRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple resolver that returns static data.
pub struct StaticResolver<T: Clone> {
    data: T,
}

impl<T: Clone> StaticResolver<T> {
    /// Create a new static resolver.
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

impl<T: Clone + 'static> Resolve for StaticResolver<T> {
    type Output = T;

    fn resolve(&self, _route: &ActivatedRouteSnapshot) -> ResolveResult<Self::Output> {
        ResolveResult::Ready(self.data.clone())
    }
}

/// Resolver that extracts a param from the route.
pub struct ParamResolver {
    param_name: String,
}

impl ParamResolver {
    /// Create a new param resolver.
    pub fn new(param_name: &str) -> Self {
        Self {
            param_name: param_name.to_string(),
        }
    }
}

impl Resolve for ParamResolver {
    type Output = String;

    fn resolve(&self, route: &ActivatedRouteSnapshot) -> ResolveResult<Self::Output> {
        match route.params.get(&self.param_name) {
            Some(value) => ResolveResult::Ready(value.clone()),
            None => ResolveResult::Error(format!("Param '{}' not found", self.param_name)),
        }
    }
}

/// Resolver that extracts a query param.
pub struct QueryParamResolver {
    param_name: String,
    default: Option<String>,
}

impl QueryParamResolver {
    /// Create a new query param resolver.
    pub fn new(param_name: &str) -> Self {
        Self {
            param_name: param_name.to_string(),
            default: None,
        }
    }

    /// Set a default value if param is missing.
    pub fn with_default(mut self, default: &str) -> Self {
        self.default = Some(default.to_string());
        self
    }
}

impl Resolve for QueryParamResolver {
    type Output = String;

    fn resolve(&self, route: &ActivatedRouteSnapshot) -> ResolveResult<Self::Output> {
        match route.query_params.get(&self.param_name) {
            Some(value) => ResolveResult::Ready(value.clone()),
            None => match &self.default {
                Some(d) => ResolveResult::Ready(d.clone()),
                None => ResolveResult::Error(format!("Query param '{}' not found", self.param_name)),
            },
        }
    }
}

/// Resolver that combines multiple resolvers.
pub struct CompositeResolver {
    resolvers: Vec<(String, Rc<dyn ResolverAny>)>,
}

impl CompositeResolver {
    /// Create a new composite resolver.
    pub fn new() -> Self {
        Self {
            resolvers: Vec::new(),
        }
    }

    /// Add a resolver.
    pub fn add<R: Resolve + 'static>(mut self, name: &str, resolver: R) -> Self
    where
        R::Output: 'static,
    {
        let wrapper = ResolverWrapper::new(name, resolver);
        self.resolvers.push((name.to_string(), Rc::new(wrapper)));
        self
    }

    /// Resolve all and return as map.
    pub fn resolve_all(&self, route: &ActivatedRouteSnapshot) -> HashMap<String, ResolveResult<Box<dyn Any>>> {
        self.resolvers
            .iter()
            .map(|(name, resolver)| (name.clone(), resolver.resolve_any(route)))
            .collect()
    }
}

impl Default for CompositeResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Resolved data container for type-safe access.
#[derive(Debug, Clone, Default)]
pub struct ResolvedData {
    data: HashMap<String, String>,
}

impl ResolvedData {
    /// Create from a HashMap.
    pub fn from_map(data: HashMap<String, String>) -> Self {
        Self { data }
    }

    /// Get data by key.
    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    /// Get all data.
    pub fn all(&self) -> &HashMap<String, String> {
        &self.data
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_resolver() {
        let resolver = StaticResolver::new("hello".to_string());
        let route = ActivatedRouteSnapshot::default();

        let result = resolver.resolve(&route);
        assert!(result.is_ready());
        assert_eq!(result.value(), Some("hello".to_string()));
    }

    #[test]
    fn test_param_resolver() {
        let resolver = ParamResolver::new("userId");
        let mut route = ActivatedRouteSnapshot::default();
        route.params.insert("userId".to_string(), "123".to_string());

        let result = resolver.resolve(&route);
        assert!(result.is_ready());
        assert_eq!(result.value(), Some("123".to_string()));
    }

    #[test]
    fn test_param_resolver_missing() {
        let resolver = ParamResolver::new("userId");
        let route = ActivatedRouteSnapshot::default();

        let result = resolver.resolve(&route);
        assert!(matches!(result, ResolveResult::Error(_)));
    }

    #[test]
    fn test_query_param_with_default() {
        let resolver = QueryParamResolver::new("page").with_default("1");
        let route = ActivatedRouteSnapshot::default();

        let result = resolver.resolve(&route);
        assert!(result.is_ready());
        assert_eq!(result.value(), Some("1".to_string()));
    }

    #[test]
    fn test_resolver_registry() {
        let registry = ResolverRegistry::new();
        registry.register("user", StaticResolver::new("John"));

        let route = ActivatedRouteSnapshot::default();
        let results = registry.resolve_all(&["user".to_string()], &route);

        assert!(ResolverRegistry::all_ready(&results));
    }
}
