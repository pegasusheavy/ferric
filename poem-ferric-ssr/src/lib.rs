//! Poem SSR Integration for Ferric
//!
//! This module provides seamless integration between the Ferric framework's
//! server-side rendering capabilities and the Poem web framework.
//!
//! # Examples
//!
//! ```no_run
//! use poem::{Route, get, handler, web::Html, Server, listener::TcpListener};
//! use poem_ferric_ssr::FerricPoemRenderer;
//! use ferric_ssr::SsrConfig;
//!
//! #[handler]
//! async fn index(renderer: poem::web::Data<&FerricPoemRenderer>) -> Html<String> {
//!     let html = renderer.render_with_hydration("app-root", None).await.unwrap();
//!     Html(html)
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), std::io::Error> {
//!     let config = SsrConfig::default();
//!     let renderer = FerricPoemRenderer::new(config);
//!
//!     let app = Route::new()
//!         .at("/", get(index))
//!         .data(renderer);
//!
//!     Server::new(TcpListener::bind("0.0.0.0:3000"))
//!         .run(app)
//!         .await
//! }
//! ```

use ferric_ssr::{SsrConfig, SsrRenderer, SsrError};
use poem::{IntoResponse, Response, web::Html};
use std::sync::Arc;
use serde_json::Value;

/// Ferric SSR renderer for Poem applications.
///
/// # Examples
///
/// ```
/// use poem_ferric_ssr::FerricPoemRenderer;
/// use ferric_ssr::SsrConfig;
///
/// let config = SsrConfig::default();
/// let renderer = FerricPoemRenderer::new(config);
/// ```
#[derive(Clone)]
pub struct FerricPoemRenderer {
    inner: Arc<SsrRenderer>,
}

impl FerricPoemRenderer {
    /// Creates a new Poem renderer with the given configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use poem_ferric_ssr::FerricPoemRenderer;
    /// use ferric_ssr::SsrConfig;
    ///
    /// let config = SsrConfig::default();
    /// let renderer = FerricPoemRenderer::new(config);
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
    /// use poem_ferric_ssr::FerricPoemRenderer;
    /// use ferric_ssr::SsrConfig;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let renderer = FerricPoemRenderer::new(SsrConfig::default());
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
    /// use poem_ferric_ssr::FerricPoemRenderer;
    /// use ferric_ssr::SsrConfig;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let renderer = FerricPoemRenderer::new(SsrConfig::default());
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

/// SSR response that implements IntoResponse.
pub struct SsrResponse(pub String);

impl IntoResponse for SsrResponse {
    fn into_response(self) -> Response {
        Html(self.0).into_response()
    }
}

/// Caching middleware for SSR.
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_creation() {
        let config = SsrConfig::default();
        let _renderer = FerricPoemRenderer::new(config);
    }

    #[tokio::test]
    async fn test_cache() {
        let cache = middleware::SsrCache::new(std::time::Duration::from_secs(60));

        cache.set("key".to_string(), "value".to_string()).await;
        let result = cache.get("key").await;

        assert_eq!(result, Some("value".to_string()));
    }
}

