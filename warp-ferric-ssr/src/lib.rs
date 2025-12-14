//! Warp SSR Integration for Ferric
//!
//! This module provides seamless integration between the Ferric framework's
//! server-side rendering capabilities and the Warp web framework.
//!
//! # Examples
//!
//! ```no_run
//! use warp::Filter;
//! use warp_ferric_ssr::FerricWarpRenderer;
//! use ferric_ssr::SsrConfig;
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = SsrConfig::default();
//!     let renderer = FerricWarpRenderer::new(config);
//!
//!     let routes = warp::path::end()
//!         .and(warp::any().map(move || renderer.clone()))
//!         .and_then(render_handler);
//!
//!     warp::serve(routes).run(([0, 0, 0, 0], 3000)).await;
//! }
//!
//! async fn render_handler(renderer: FerricWarpRenderer) -> Result<impl warp::Reply, warp::Rejection> {
//!     let html = renderer.render_with_hydration("app-root", None).await
//!         .map_err(|_| warp::reject::not_found())?;
//!     Ok(warp::reply::html(html))
//! }
//! ```

use ferric_ssr::{SsrConfig, SsrRenderer, SsrError};
use std::sync::Arc;
use warp::{Filter, Rejection, Reply, reject, reply};
use serde_json::Value;

/// Ferric SSR renderer for Warp applications.
///
/// # Examples
///
/// ```
/// use warp_ferric_ssr::FerricWarpRenderer;
/// use ferric_ssr::SsrConfig;
///
/// let config = SsrConfig::default();
/// let renderer = FerricWarpRenderer::new(config);
/// ```
#[derive(Clone)]
pub struct FerricWarpRenderer {
    inner: Arc<SsrRenderer>,
}

impl FerricWarpRenderer {
    /// Creates a new Warp renderer with the given configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use warp_ferric_ssr::FerricWarpRenderer;
    /// use ferric_ssr::SsrConfig;
    ///
    /// let config = SsrConfig::default();
    /// let renderer = FerricWarpRenderer::new(config);
    /// ```
    pub fn new(config: SsrConfig) -> Self {
        Self {
            inner: Arc::new(SsrRenderer::new(config)),
        }
    }

    /// Renders a Ferric component to HTML.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use warp_ferric_ssr::FerricWarpRenderer;
    /// use ferric_ssr::SsrConfig;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let renderer = FerricWarpRenderer::new(SsrConfig::default());
    /// let html = renderer.render_to_string("app-root", None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn render_to_string(
        &self,
        component: &str,
        props: Option<Value>,
    ) -> Result<String, SsrError> {
        self.inner.render_to_string(component, props).await
    }

    /// Renders a Ferric component with hydration support.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use warp_ferric_ssr::FerricWarpRenderer;
    /// use ferric_ssr::SsrConfig;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let renderer = FerricWarpRenderer::new(SsrConfig::default());
    /// let html = renderer.render_with_hydration("app-root", None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn render_with_hydration(
        &self,
        component: &str,
        props: Option<Value>,
    ) -> Result<String, SsrError> {
        self.inner.render_with_hydration(component, props).await
    }
}

/// Creates a Warp filter that provides the Ferric renderer.
///
/// # Examples
///
/// ```no_run
/// use warp::Filter;
/// use warp_ferric_ssr::{with_renderer, FerricWarpRenderer};
/// use ferric_ssr::SsrConfig;
///
/// # async fn example() {
/// let renderer = FerricWarpRenderer::new(SsrConfig::default());
///
/// let routes = warp::path::end()
///     .and(with_renderer(renderer))
///     .and_then(|renderer: FerricWarpRenderer| async move {
///         let html = renderer.render_with_hydration("app-root", None).await
///             .map_err(|_| warp::reject::not_found())?;
///         Ok::<_, warp::Rejection>(warp::reply::html(html))
///     });
/// # }
/// ```
pub fn with_renderer(
    renderer: FerricWarpRenderer,
) -> impl Filter<Extract = (FerricWarpRenderer,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || renderer.clone())
}

/// Creates a complete Warp SSR route.
///
/// # Examples
///
/// ```no_run
/// use warp_ferric_ssr::create_ssr_route;
/// use ferric_ssr::SsrConfig;
///
/// # async fn example() {
/// let config = SsrConfig::default();
/// let routes = create_ssr_route(config);
/// warp::serve(routes).run(([0, 0, 0, 0], 3000)).await;
/// # }
/// ```
pub fn create_ssr_route(
    config: SsrConfig,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let renderer = FerricWarpRenderer::new(config);

    warp::path::end()
        .and(with_renderer(renderer))
        .and_then(default_handler)
}

/// Default SSR handler for Warp.
pub async fn default_handler(renderer: FerricWarpRenderer) -> Result<impl Reply, Rejection> {
    match renderer.render_with_hydration("app-root", None).await {
        Ok(html) => Ok(reply::html(html)),
        Err(_) => Err(reject::not_found()),
    }
}

/// SSR middleware for caching.
pub mod middleware {
    use super::*;
    use std::collections::HashMap;
    use tokio::sync::RwLock;

    /// Cache for SSR-rendered pages.
    #[derive(Clone)]
    pub struct SsrCache {
        cache: Arc<RwLock<HashMap<String, (String, std::time::Instant)>>>,
        ttl: std::time::Duration,
    }

    impl SsrCache {
        /// Creates a new cache with the given TTL.
        pub fn new(ttl: std::time::Duration) -> Self {
            Self {
                cache: Arc::new(RwLock::new(HashMap::new())),
                ttl,
            }
        }

        /// Gets a cached page.
        pub async fn get(&self, key: &str) -> Option<String> {
            let cache = self.cache.read().await;
            if let Some((html, timestamp)) = cache.get(key) {
                if timestamp.elapsed() < self.ttl {
                    return Some(html.clone());
                }
            }
            None
        }

        /// Caches a rendered page.
        pub async fn set(&self, key: String, html: String) {
            let mut cache = self.cache.write().await;
            cache.insert(key, (html, std::time::Instant::now()));
        }
    }

    /// Creates a caching filter for Warp.
    pub fn with_cache(
        cache: SsrCache,
    ) -> impl Filter<Extract = (SsrCache,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || cache.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_creation() {
        let config = SsrConfig::default();
        let _renderer = FerricWarpRenderer::new(config);
    }

    #[tokio::test]
    async fn test_cache() {
        let cache = middleware::SsrCache::new(std::time::Duration::from_secs(60));

        cache.set("key".to_string(), "value".to_string()).await;
        let result = cache.get("key").await;

        assert_eq!(result, Some("value".to_string()));
    }
}

