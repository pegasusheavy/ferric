//! Lazy loading support for route modules.
//!
//! Provides dynamic import capabilities for code-splitting and on-demand
//! loading of route modules.
//!
//! ## Features
//!
//! - **Dynamic imports** - Load route modules on navigation
//! - **Code splitting** - Reduce initial bundle size
//! - **Loading states** - Track loading progress
//! - **Preloading** - Optionally preload modules in advance
//! - **Caching** - Cache loaded modules for instant navigation
//!
//! ## Usage
//!
//! ```ignore
//! use ferric_core::router::{Route, LazyModule};
//!
//! // Define a lazy-loaded route
//! let routes = vec![
//!     Route::new("/")
//!         .component("app-home"),
//!     Route::new("/admin")
//!         .load_children(|| async {
//!             // Dynamically import admin module
//!             Ok(vec![
//!                 Route::new("dashboard").component("admin-dashboard"),
//!                 Route::new("users").component("admin-users"),
//!             ])
//!         }),
//! ];
//! ```

use super::{Route, Routes};
use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

/// Counter for lazy module IDs.
static NEXT_MODULE_ID: AtomicU64 = AtomicU64::new(1);

fn next_module_id() -> u64 {
    NEXT_MODULE_ID.fetch_add(1, Ordering::SeqCst)
}

/// Error type for lazy loading operations.
#[derive(Debug, Clone)]
pub enum LazyLoadError {
    /// The module failed to load.
    LoadFailed(String),
    /// The module path was not found.
    NotFound(String),
    /// The module was cancelled (e.g., superseded by another navigation).
    Cancelled,
    /// JavaScript error during dynamic import.
    JsError(String),
    /// The module returned invalid routes.
    InvalidRoutes(String),
    /// Network error during fetch.
    NetworkError(String),
    /// Timeout while loading module.
    Timeout,
}

impl std::fmt::Display for LazyLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LazyLoadError::LoadFailed(msg) => write!(f, "Module load failed: {}", msg),
            LazyLoadError::NotFound(path) => write!(f, "Module not found: {}", path),
            LazyLoadError::Cancelled => write!(f, "Module load cancelled"),
            LazyLoadError::JsError(msg) => write!(f, "JavaScript error: {}", msg),
            LazyLoadError::InvalidRoutes(msg) => write!(f, "Invalid routes: {}", msg),
            LazyLoadError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            LazyLoadError::Timeout => write!(f, "Module load timed out"),
        }
    }
}

impl std::error::Error for LazyLoadError {}

/// Result type for lazy loading operations.
pub type LazyLoadResult<T> = Result<T, LazyLoadError>;

/// The current state of a lazy-loaded module.
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Default)]
pub enum LoadState {
    /// Module has not been loaded yet.
    #[default]
    NotLoaded,
    /// Module is currently loading.
    Loading,
    /// Module has been successfully loaded.
    Loaded,
    /// Module failed to load.
    Error(String),
}


/// A handle to a lazily-loaded route module.
#[derive(Clone)]
pub struct LazyRouteModule {
    /// Unique ID for this module.
    id: u64,
    /// The module path for dynamic import.
    module_path: String,
    /// Current load state.
    state: Rc<RefCell<LoadState>>,
    /// Cached routes once loaded.
    routes: Rc<RefCell<Option<Routes>>>,
    /// Optional loader function for Rust-based loading.
    loader: Rc<RefCell<Option<BoxedLoader>>>,
}

/// Boxed async loader function type.
type BoxedLoader = Box<dyn Fn() -> Pin<Box<dyn Future<Output = LazyLoadResult<Routes>>>>>;

impl LazyRouteModule {
    /// Create a new lazy module with a JavaScript import path.
    pub fn from_path(module_path: &str) -> Self {
        Self {
            id: next_module_id(),
            module_path: module_path.to_string(),
            state: Rc::new(RefCell::new(LoadState::NotLoaded)),
            routes: Rc::new(RefCell::new(None)),
            loader: Rc::new(RefCell::new(None)),
        }
    }

    /// Create a new lazy module with a Rust loader function.
    pub fn from_loader<F, Fut>(loader: F) -> Self
    where
        F: Fn() -> Fut + 'static,
        Fut: Future<Output = LazyLoadResult<Routes>> + 'static,
    {
        let boxed: BoxedLoader = Box::new(move || Box::pin(loader()));
        Self {
            id: next_module_id(),
            module_path: String::new(),
            state: Rc::new(RefCell::new(LoadState::NotLoaded)),
            routes: Rc::new(RefCell::new(None)),
            loader: Rc::new(RefCell::new(Some(boxed))),
        }
    }

    /// Create from a factory function that returns routes directly.
    pub fn from_factory<F>(factory: F) -> Self
    where
        F: Fn() -> Routes + 'static,
    {
        let boxed: BoxedLoader = Box::new(move || {
            let routes = factory();
            Box::pin(async move { Ok(routes) })
        });
        Self {
            id: next_module_id(),
            module_path: String::new(),
            state: Rc::new(RefCell::new(LoadState::NotLoaded)),
            routes: Rc::new(RefCell::new(None)),
            loader: Rc::new(RefCell::new(Some(boxed))),
        }
    }

    /// Get the unique ID of this module.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Get the module path.
    pub fn module_path(&self) -> &str {
        &self.module_path
    }

    /// Get the current load state.
    pub fn state(&self) -> LoadState {
        self.state.borrow().clone()
    }

    /// Check if the module is loaded.
    pub fn is_loaded(&self) -> bool {
        matches!(*self.state.borrow(), LoadState::Loaded)
    }

    /// Check if the module is currently loading.
    pub fn is_loading(&self) -> bool {
        matches!(*self.state.borrow(), LoadState::Loading)
    }

    /// Get the loaded routes (if available).
    pub fn routes(&self) -> Option<Routes> {
        self.routes.borrow().clone()
    }

    /// Load the module asynchronously.
    pub async fn load(&self) -> LazyLoadResult<Routes> {
        // Check if already loaded
        if let Some(routes) = self.routes.borrow().clone() {
            return Ok(routes);
        }

        // Check if already loading (prevent duplicate loads)
        if self.is_loading() {
            // Wait for existing load to complete
            // In a real implementation, we'd use a proper async semaphore
            return Err(LazyLoadError::LoadFailed(
                "Module already loading".to_string(),
            ));
        }

        // Mark as loading
        *self.state.borrow_mut() = LoadState::Loading;

        // Try Rust loader first
        if let Some(ref loader) = *self.loader.borrow() {
            match loader().await {
                Ok(routes) => {
                    *self.routes.borrow_mut() = Some(routes.clone());
                    *self.state.borrow_mut() = LoadState::Loaded;
                    return Ok(routes);
                }
                Err(e) => {
                    *self.state.borrow_mut() = LoadState::Error(e.to_string());
                    return Err(e);
                }
            }
        }

        // Fall back to JavaScript dynamic import
        if !self.module_path.is_empty() {
            match self.load_js_module().await {
                Ok(routes) => {
                    *self.routes.borrow_mut() = Some(routes.clone());
                    *self.state.borrow_mut() = LoadState::Loaded;
                    return Ok(routes);
                }
                Err(e) => {
                    *self.state.borrow_mut() = LoadState::Error(e.to_string());
                    return Err(e);
                }
            }
        }

        let err = LazyLoadError::LoadFailed("No loader or module path configured".to_string());
        *self.state.borrow_mut() = LoadState::Error(err.to_string());
        Err(err)
    }

    /// Load a JavaScript module using dynamic import.
    async fn load_js_module(&self) -> LazyLoadResult<Routes> {
        // Use wasm-bindgen to call JavaScript's dynamic import
        let promise = js_dynamic_import(&self.module_path);
        let result = JsFuture::from(promise).await;

        match result {
            Ok(module) => {
                // Extract routes from the module
                self.extract_routes_from_module(&module)
            }
            Err(err) => {
                let error_msg = err
                    .as_string()
                    .unwrap_or_else(|| "Unknown JS error".to_string());
                Err(LazyLoadError::JsError(error_msg))
            }
        }
    }

    /// Extract routes from a loaded JavaScript module.
    fn extract_routes_from_module(&self, module: &JsValue) -> LazyLoadResult<Routes> {
        // Look for a 'routes' export or 'default' export
        if let Some(routes_array) = js_sys::Reflect::get(module, &JsValue::from_str("routes"))
            .ok()
            .filter(|v| !v.is_undefined())
        {
            return self.parse_routes_from_js(&routes_array);
        }

        if let Some(default_export) = js_sys::Reflect::get(module, &JsValue::from_str("default"))
            .ok()
            .filter(|v| !v.is_undefined())
        {
            return self.parse_routes_from_js(&default_export);
        }

        Err(LazyLoadError::InvalidRoutes(
            "Module does not export 'routes' or 'default'".to_string(),
        ))
    }

    /// Parse routes from a JavaScript value.
    fn parse_routes_from_js(&self, value: &JsValue) -> LazyLoadResult<Routes> {
        if !js_sys::Array::is_array(value) {
            return Err(LazyLoadError::InvalidRoutes(
                "Routes export is not an array".to_string(),
            ));
        }

        let array = js_sys::Array::from(value);
        let mut routes = Vec::new();

        for i in 0..array.length() {
            let item = array.get(i);
            match self.parse_route_from_js(&item) {
                Ok(route) => routes.push(route),
                Err(e) => return Err(e),
            }
        }

        Ok(routes)
    }

    /// Parse a single route from a JavaScript object.
    fn parse_route_from_js(&self, value: &JsValue) -> LazyLoadResult<Route> {
        if !value.is_object() {
            return Err(LazyLoadError::InvalidRoutes(
                "Route is not an object".to_string(),
            ));
        }

        let path = js_sys::Reflect::get(value, &JsValue::from_str("path"))
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_default();

        let component = js_sys::Reflect::get(value, &JsValue::from_str("component"))
            .ok()
            .and_then(|v| v.as_string());

        let redirect_to = js_sys::Reflect::get(value, &JsValue::from_str("redirectTo"))
            .ok()
            .and_then(|v| v.as_string());

        let title = js_sys::Reflect::get(value, &JsValue::from_str("title"))
            .ok()
            .and_then(|v| v.as_string());

        // Parse children recursively
        let children = js_sys::Reflect::get(value, &JsValue::from_str("children"))
            .ok()
            .filter(js_sys::Array::is_array)
            .map(|v| self.parse_routes_from_js(&v))
            .transpose()?
            .unwrap_or_default();

        let mut route = Route::new(&path);
        if let Some(comp) = component {
            route = route.component(&comp);
        }
        if let Some(redirect) = redirect_to {
            route = route.redirect_to(&redirect);
        }
        if let Some(t) = title {
            route = route.title(&t);
        }
        if !children.is_empty() {
            route = route.children(children);
        }

        Ok(route)
    }

    /// Reset the module to unloaded state (for testing/reloading).
    pub fn reset(&self) {
        *self.state.borrow_mut() = LoadState::NotLoaded;
        *self.routes.borrow_mut() = None;
    }
}

impl std::fmt::Debug for LazyRouteModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LazyRouteModule")
            .field("id", &self.id)
            .field("module_path", &self.module_path)
            .field("state", &self.state.borrow())
            .field("has_routes", &self.routes.borrow().is_some())
            .finish()
    }
}

/// Registry for caching and managing lazy modules.
pub struct LazyModuleRegistry {
    /// Cached modules by path.
    modules: RefCell<HashMap<String, Rc<LazyRouteModule>>>,
    /// Preload configuration.
    preload_config: RefCell<PreloadConfig>,
}

impl LazyModuleRegistry {
    /// Create a new registry.
    pub fn new() -> Self {
        Self {
            modules: RefCell::new(HashMap::new()),
            preload_config: RefCell::new(PreloadConfig::default()),
        }
    }

    /// Register a lazy module.
    pub fn register(&self, path: &str, module: LazyRouteModule) {
        self.modules
            .borrow_mut()
            .insert(path.to_string(), Rc::new(module));
    }

    /// Get a registered module.
    pub fn get(&self, path: &str) -> Option<Rc<LazyRouteModule>> {
        self.modules.borrow().get(path).cloned()
    }

    /// Load a module by path.
    pub async fn load(&self, path: &str) -> LazyLoadResult<Routes> {
        if let Some(module) = self.get(path) {
            module.load().await
        } else {
            // Create and register a new module
            let module = LazyRouteModule::from_path(path);
            let routes = module.load().await?;
            self.register(path, module);
            Ok(routes)
        }
    }

    /// Check if a module is loaded.
    pub fn is_loaded(&self, path: &str) -> bool {
        self.get(path).map(|m| m.is_loaded()).unwrap_or(false)
    }

    /// Preload modules based on configuration.
    pub async fn preload_all(&self) -> Vec<LazyLoadResult<Routes>> {
        let mut results = Vec::new();
        let paths: Vec<String> = self.modules.borrow().keys().cloned().collect();

        for path in paths {
            if let Some(module) = self.get(&path)
                && !module.is_loaded() && self.should_preload(&path) {
                    results.push(module.load().await);
                }
        }

        results
    }

    /// Check if a module should be preloaded.
    fn should_preload(&self, path: &str) -> bool {
        let config = self.preload_config.borrow();
        match config.strategy {
            PreloadStrategy::All => true,
            PreloadStrategy::None => false,
            PreloadStrategy::OnDemand => false,
            PreloadStrategy::Custom(ref paths) => paths.contains(&path.to_string()),
        }
    }

    /// Set preload configuration.
    pub fn set_preload_config(&self, config: PreloadConfig) {
        *self.preload_config.borrow_mut() = config;
    }

    /// Clear all cached modules.
    pub fn clear(&self) {
        self.modules.borrow_mut().clear();
    }

    /// Get all loaded module paths.
    pub fn loaded_paths(&self) -> Vec<String> {
        self.modules
            .borrow()
            .iter()
            .filter(|(_, m)| m.is_loaded())
            .map(|(p, _)| p.clone())
            .collect()
    }
}

impl Default for LazyModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for preloading modules.
#[derive(Debug, Clone)]
pub struct PreloadConfig {
    /// Preloading strategy.
    pub strategy: PreloadStrategy,
    /// Delay before preloading (ms).
    pub delay_ms: u32,
    /// Whether to preload on idle.
    pub preload_on_idle: bool,
}

impl Default for PreloadConfig {
    fn default() -> Self {
        Self {
            strategy: PreloadStrategy::OnDemand,
            delay_ms: 0,
            preload_on_idle: false,
        }
    }
}

impl PreloadConfig {
    /// Create a config that preloads all modules.
    pub fn all() -> Self {
        Self {
            strategy: PreloadStrategy::All,
            ..Default::default()
        }
    }

    /// Create a config that preloads no modules.
    pub fn none() -> Self {
        Self {
            strategy: PreloadStrategy::None,
            ..Default::default()
        }
    }

    /// Create a config that preloads on demand.
    pub fn on_demand() -> Self {
        Self {
            strategy: PreloadStrategy::OnDemand,
            ..Default::default()
        }
    }

    /// Create a config that preloads specific modules.
    pub fn custom(paths: Vec<String>) -> Self {
        Self {
            strategy: PreloadStrategy::Custom(paths),
            ..Default::default()
        }
    }

    /// Set preload delay.
    pub fn with_delay(mut self, delay_ms: u32) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    /// Enable preloading on browser idle.
    pub fn with_idle_preload(mut self) -> Self {
        self.preload_on_idle = true;
        self
    }
}

/// Strategy for preloading lazy modules.
#[derive(Debug, Clone)]
pub enum PreloadStrategy {
    /// Preload all modules immediately.
    All,
    /// Don't preload any modules.
    None,
    /// Load modules only when navigating to them.
    OnDemand,
    /// Preload specific modules.
    Custom(Vec<String>),
}

/// JavaScript dynamic import function.
#[wasm_bindgen(inline_js = r#"
export function js_dynamic_import(path) {
    return import(path);
}
"#)]
extern "C" {
    fn js_dynamic_import(path: &str) -> js_sys::Promise;
}

/// Helper function to create a lazy module from a path.
pub fn lazy(module_path: &str) -> LazyRouteModule {
    LazyRouteModule::from_path(module_path)
}

/// Helper function to create a lazy module from a factory function.
pub fn lazy_factory<F>(factory: F) -> LazyRouteModule
where
    F: Fn() -> Routes + 'static,
{
    LazyRouteModule::from_factory(factory)
}

/// Helper function to create a lazy module from an async loader.
pub fn lazy_async<F, Fut>(loader: F) -> LazyRouteModule
where
    F: Fn() -> Fut + 'static,
    Fut: Future<Output = LazyLoadResult<Routes>> + 'static,
{
    LazyRouteModule::from_loader(loader)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lazy_module_from_path() {
        let module = LazyRouteModule::from_path("./admin/routes.js");

        assert!(!module.is_loaded());
        assert!(!module.is_loading());
        assert_eq!(module.module_path(), "./admin/routes.js");
        assert!(module.routes().is_none());
    }

    #[test]
    fn test_lazy_module_from_factory() {
        let module = LazyRouteModule::from_factory(|| {
            vec![
                Route::new("dashboard").component("admin-dashboard"),
                Route::new("users").component("admin-users"),
            ]
        });

        assert!(!module.is_loaded());
        assert!(module.module_path().is_empty());
    }

    #[test]
    fn test_load_state() {
        assert_eq!(LoadState::default(), LoadState::NotLoaded);
        assert!(matches!(LoadState::Loaded, LoadState::Loaded));
        assert!(matches!(
            LoadState::Error("test".to_string()),
            LoadState::Error(_)
        ));
    }

    #[test]
    fn test_lazy_load_error_display() {
        let err = LazyLoadError::NotFound("/admin".to_string());
        assert!(err.to_string().contains("/admin"));

        let err = LazyLoadError::Timeout;
        assert!(err.to_string().contains("timed out"));
    }

    #[test]
    fn test_lazy_module_registry() {
        let registry = LazyModuleRegistry::new();

        let module = LazyRouteModule::from_path("./admin.js");
        registry.register("/admin", module);

        assert!(registry.get("/admin").is_some());
        assert!(registry.get("/users").is_none());
        assert!(!registry.is_loaded("/admin"));
    }

    #[test]
    fn test_preload_config() {
        let config = PreloadConfig::all();
        assert!(matches!(config.strategy, PreloadStrategy::All));

        let config = PreloadConfig::custom(vec!["/admin".to_string()]).with_delay(100);
        assert!(matches!(config.strategy, PreloadStrategy::Custom(_)));
        assert_eq!(config.delay_ms, 100);
    }

    #[test]
    fn test_lazy_helpers() {
        let module = lazy("./module.js");
        assert_eq!(module.module_path(), "./module.js");

        let module = lazy_factory(|| vec![Route::new("/test")]);
        assert!(module.module_path().is_empty());
    }

    #[test]
    fn test_module_reset() {
        let module = LazyRouteModule::from_path("./test.js");
        // Simulate loaded state
        *module.state.borrow_mut() = LoadState::Loaded;
        *module.routes.borrow_mut() = Some(vec![Route::new("/test")]);

        assert!(module.is_loaded());

        module.reset();

        assert!(!module.is_loaded());
        assert!(module.routes().is_none());
    }
}

