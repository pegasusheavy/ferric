//! SSR middleware for Armature.

use armature_core::http::{HttpRequest, HttpResponse};
use armature_core::middleware::Middleware;
use async_trait::async_trait;
use std::sync::Arc;

/// Middleware that adds SSR capabilities to routes.
pub struct SsrMiddleware {
    enabled: bool,
}

impl SsrMiddleware {
    /// Create a new SSR middleware.
    pub fn new() -> Self {
        Self { enabled: true }
    }

    /// Create a disabled SSR middleware.
    pub fn disabled() -> Self {
        Self { enabled: false }
    }
}

impl Default for SsrMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for SsrMiddleware {
    async fn handle(
        &self,
        request: HttpRequest,
        next: Box<dyn Fn(HttpRequest) -> futures::future::BoxFuture<'static, Result<HttpResponse, armature_core::Error>> + Send + Sync>,
    ) -> Result<HttpResponse, armature_core::Error> {
        if !self.enabled {
            return next(request).await;
        }

        // Add SSR headers
        let mut request = request;
        request.headers.insert(
            "X-SSR-Enabled".to_string(),
            "true".to_string(),
        );

        let response = next(request).await?;

        // Add SSR metadata to response
        Ok(response.with_header("X-SSR-Rendered", "true"))
    }
}

/// Middleware for caching SSR responses.
pub struct SsrCacheMiddleware {
    max_age: u32,
}

impl SsrCacheMiddleware {
    /// Create a new cache middleware with the given max age in seconds.
    pub fn new(max_age: u32) -> Self {
        Self { max_age }
    }
}

#[async_trait]
impl Middleware for SsrCacheMiddleware {
    async fn handle(
        &self,
        request: HttpRequest,
        next: Box<dyn Fn(HttpRequest) -> futures::future::BoxFuture<'static, Result<HttpResponse, armature_core::Error>> + Send + Sync>,
    ) -> Result<HttpResponse, armature_core::Error> {
        let response = next(request).await?;

        // Add cache control headers for SSR responses
        if response.headers.get("X-SSR-Rendered").is_some() {
            Ok(response.with_header(
                "Cache-Control",
                format!("public, max-age={}", self.max_age),
            ))
        } else {
            Ok(response)
        }
    }
}

