//! Axum SSR Integration for Ferric
//!
//! This module provides seamless integration between the Ferric framework's
//! server-side rendering capabilities and the Axum web framework.
//!
//! # Examples
//!
//! ```no_run
//! use axum::{Router, routing::get};
//! use axum_ferric_ssr::FerricAxumRenderer;
//! use ferric_ssr::SsrConfig;
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = SsrConfig::default();
//!     let renderer = FerricAxumRenderer::new(config);
//!
//!     let app = Router::new()
//!         .route("/", get(|| async { "Hello, Ferric + Axum!" }))
//!         .with_state(renderer);
//!
//!     let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
//!         .await
//!         .unwrap();
//!
//!     axum::serve(listener, app).await.unwrap();
//! }
//! ```

use axum::{
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode, HeaderMap, header},
    response::{Html, IntoResponse},
    Router,
    routing::get,
};
use ferric_ssr::{SsrConfig, SsrRenderer, RenderResult};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;

/// Ferric SSR renderer for Axum applications.
///
/// # Examples
///
/// ```no_run
/// use axum_ssr::FerricAxumRenderer;
/// use ferric_ssr::SsrConfig;
///
/// let config = SsrConfig::default();
/// let renderer = FerricAxumRenderer::new(config);
/// ```
#[derive(Clone)]
pub struct FerricAxumRenderer {
    inner: Arc<SsrRenderer>,
}

impl FerricAxumRenderer {
    /// Creates a new Axum renderer with the given configuration.
    ///
/// # Examples
///
/// ```
/// use axum_ferric_ssr::FerricAxumRenderer;
/// use ferric_ssr::SsrConfig;
    ///
    /// let config = SsrConfig::default();
    /// let renderer = FerricAxumRenderer::new(config);
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
    /// use axum_ssr::FerricAxumRenderer;
    /// use ferric_ssr::SsrConfig;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let renderer = FerricAxumRenderer::new(SsrConfig::default());
    /// let html = renderer.render_to_string("app-root", None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn render_to_string(
        &self,
        component: &str,
        props: Option<serde_json::Value>,
    ) -> Result<String, ferric_ssr::SsrError> {
        self.inner.render_to_string(component, props).await
    }

    /// Renders a Ferric component with hydration support.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use axum_ssr::FerricAxumRenderer;
    /// use ferric_ssr::SsrConfig;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let renderer = FerricAxumRenderer::new(SsrConfig::default());
    /// let html = renderer.render_with_hydration("app-root", None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn render_with_hydration(
        &self,
        component: &str,
        props: Option<serde_json::Value>,
    ) -> Result<String, ferric_ssr::SsrError> {
        self.inner.render_with_hydration(component, props).await
    }
}

/// Axum handler for rendering Ferric components.
///
/// # Examples
///
/// ```no_run
/// use axum::{Router, routing::get};
/// use axum_ssr::{FerricAxumRenderer, render_handler};
/// use ferric_ssr::SsrConfig;
///
/// # async fn example() {
/// let renderer = FerricAxumRenderer::new(SsrConfig::default());
///
/// let app = Router::new()
///     .route("/", get(render_handler))
///     .with_state(renderer);
/// # }
/// ```
pub async fn render_handler(
    State(renderer): State<FerricAxumRenderer>,
) -> Result<Html<String>, StatusCode> {
    match renderer.render_with_hydration("app-root", None).await {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Creates an Axum router with Ferric SSR support.
///
/// # Examples
///
/// ```no_run
/// use axum_ssr::create_ssr_router;
/// use ferric_ssr::SsrConfig;
///
/// # async fn example() {
/// let config = SsrConfig::default();
/// let app = create_ssr_router(config);
/// # }
/// ```
pub fn create_ssr_router(config: SsrConfig) -> Router {
    let renderer = FerricAxumRenderer::new(config);

    Router::new()
        .route("/", get(render_handler))
        .route("/*path", get(render_handler))
        .layer(
            ServiceBuilder::new()
                .layer(CompressionLayer::new())
        )
        .with_state(renderer)
}

/// Response wrapper for SSR results.
pub struct SsrResponse {
    html: String,
    status: StatusCode,
    headers: HeaderMap,
}

impl SsrResponse {
    /// Creates a new SSR response.
    pub fn new(html: String) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            "text/html; charset=utf-8".parse().unwrap(),
        );

        Self {
            html,
            status: StatusCode::OK,
            headers,
        }
    }

    /// Sets the status code.
    pub fn with_status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    /// Adds a header.
    pub fn with_header(mut self, key: header::HeaderName, value: &str) -> Self {
        self.headers.insert(key, value.parse().unwrap());
        self
    }
}

impl IntoResponse for SsrResponse {
    fn into_response(self) -> Response<Body> {
        let mut response = Html(self.html).into_response();
        *response.status_mut() = self.status;
        *response.headers_mut() = self.headers;
        response
    }
}

/// Middleware for SSR caching.
pub mod middleware {
    use super::*;
    use std::collections::HashMap;
    use tokio::sync::RwLock;

    /// Simple in-memory cache for rendered pages.
    #[derive(Clone)]
    pub struct SsrCache {
        cache: Arc<RwLock<HashMap<String, (String, std::time::Instant)>>>,
        ttl: std::time::Duration,
    }

    impl SsrCache {
        /// Creates a new SSR cache with the given TTL.
        pub fn new(ttl: std::time::Duration) -> Self {
            Self {
                cache: Arc::new(RwLock::new(HashMap::new())),
                ttl,
            }
        }

        /// Gets a cached page if available and not expired.
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

        /// Clears expired entries.
        pub async fn cleanup(&self) {
            let mut cache = self.cache.write().await;
            cache.retain(|_, (_, timestamp)| timestamp.elapsed() < self.ttl);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_creation() {
        let config = SsrConfig::default();
        let _renderer = FerricAxumRenderer::new(config);
    }

    #[test]
    fn test_ssr_response() {
        let response = SsrResponse::new("<html></html>".to_string());
        assert_eq!(response.status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_cache() {
        let cache = middleware::SsrCache::new(std::time::Duration::from_secs(60));

        cache.set("key".to_string(), "value".to_string()).await;
        let result = cache.get("key").await;

        assert_eq!(result, Some("value".to_string()));
    }
}

