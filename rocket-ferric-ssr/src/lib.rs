//! Rocket SSR Integration for Ferric
//!
//! This module provides seamless integration between the Ferric framework's
//! server-side rendering capabilities and the Rocket web framework.
//!
//! # Examples
//!
//! ```no_run
//! use rocket::{get, routes, State};
//! use rocket::response::content::RawHtml;
//! use rocket_ferric_ssr::FerricRocketRenderer;
//! use ferric_ssr::SsrConfig;
//!
//! #[get("/")]
//! async fn index(renderer: &State<FerricRocketRenderer>) -> RawHtml<String> {
//!     let html = renderer.render_with_hydration("app-root", None).await.unwrap();
//!     RawHtml(html)
//! }
//!
//! #[rocket::launch]
//! fn rocket() -> _ {
//!     let config = SsrConfig::default();
//!     let renderer = FerricRocketRenderer::new(config);
//!
//!     rocket::build()
//!         .manage(renderer)
//!         .mount("/", routes![index])
//! }
//! ```

use ferric_ssr::{SsrConfig, SsrRenderer, SsrError};
use rocket::{State, response::content::RawHtml};
use std::sync::Arc;
use serde_json::Value;

/// Ferric SSR renderer for Rocket applications.
///
/// # Examples
///
/// ```
/// use rocket_ferric_ssr::FerricRocketRenderer;
/// use ferric_ssr::SsrConfig;
///
/// let config = SsrConfig::default();
/// let renderer = FerricRocketRenderer::new(config);
/// ```
#[derive(Clone)]
pub struct FerricRocketRenderer {
    inner: Arc<SsrRenderer>,
}

impl FerricRocketRenderer {
    /// Creates a new Rocket renderer with the given configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use rocket_ferric_ssr::FerricRocketRenderer;
    /// use ferric_ssr::SsrConfig;
    ///
    /// let config = SsrConfig::default();
    /// let renderer = FerricRocketRenderer::new(config);
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
    /// use rocket_ferric_ssr::FerricRocketRenderer;
    /// use ferric_ssr::SsrConfig;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let renderer = FerricRocketRenderer::new(SsrConfig::default());
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
    /// use rocket_ferric_ssr::FerricRocketRenderer;
    /// use ferric_ssr::SsrConfig;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let renderer = FerricRocketRenderer::new(SsrConfig::default());
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

/// Macro to create SSR routes easily.
///
/// # Examples
///
/// ```no_run
/// use rocket::{get, State};
/// use rocket::response::content::RawHtml;
/// use rocket_ferric_ssr::FerricRocketRenderer;
///
/// #[get("/")]
/// async fn index(renderer: &State<FerricRocketRenderer>) -> RawHtml<String> {
///     let html = renderer.render_with_hydration("app-root", None).await.unwrap();
///     RawHtml(html)
/// }
/// ```
#[macro_export]
macro_rules! ssr_route {
    ($renderer:expr, $component:expr) => {
        $renderer.render_with_hydration($component, None).await
    };
    ($renderer:expr, $component:expr, $props:expr) => {
        $renderer.render_with_hydration($component, Some($props)).await
    };
}

/// Caching support for SSR.
pub mod cache {
    use super::*;
    use std::collections::HashMap;
    use tokio::sync::RwLock;

    /// Cache for SSR-rendered pages.
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
        let _renderer = FerricRocketRenderer::new(config);
    }

    #[tokio::test]
    async fn test_cache() {
        let cache = cache::SsrCache::new(std::time::Duration::from_secs(60));

        cache.set("key".to_string(), "value".to_string()).await;
        let result = cache.get("key").await;

        assert_eq!(result, Some("value".to_string()));
    }
}

