//! Traits for SSR integration with Armature.

use crate::{SsrResponse, Result};
use async_trait::async_trait;
use ferric_ssr::HydrationStrategy;

/// Trait for controllers that support SSR.
#[async_trait]
pub trait SsrController {
    /// Render a component for this controller.
    async fn render(&self, component: &str) -> Result<SsrResponse>;

    /// Render with specific hydration strategy.
    async fn render_with_hydration(
        &self,
        component: &str,
        strategy: HydrationStrategy,
    ) -> Result<SsrResponse>;
}

/// Trait for components that can be server-rendered.
pub trait SsrRenderable {
    /// Get the component selector for SSR.
    fn selector(&self) -> &str;

    /// Get the component template.
    fn template(&self) -> &str;

    /// Get component styles.
    fn styles(&self) -> Option<&str> {
        None
    }

    /// Get initial props for rendering.
    fn initial_props(&self) -> Option<serde_json::Value> {
        None
    }
}

