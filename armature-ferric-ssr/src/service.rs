//! SSR service for Armature-style API

use crate::{SsrResponse, ArmatureStyleError};
use ferric_ssr::{SsrRenderer, SsrConfig, HydrationStrategy};
use std::sync::Arc;
use serde_json::Value;

/// SSR service for rendering Ferric components
pub struct SsrService {
    renderer: Arc<SsrRenderer>,
}

impl SsrService {
    /// Create a new SSR service
    pub fn new(config: SsrConfig) -> Self {
        Self {
            renderer: Arc::new(SsrRenderer::new(config)),
        }
    }

    /// Start building a render request
    pub fn render_component(&self, selector: &str) -> SsrBuilder {
        SsrBuilder {
            renderer: Arc::clone(&self.renderer),
            selector: selector.to_string(),
            props: None,
            hydration: HydrationStrategy::Full,
        }
    }

    /// Render component directly to string
    pub async fn render_to_string(
        &self,
        selector: &str,
        props: Option<Value>,
    ) -> Result<String, ArmatureStyleError> {
        self.renderer
            .render_to_string(selector, props)
            .await
            .map_err(Into::into)
    }
}

/// Builder for SSR rendering
pub struct SsrBuilder {
    renderer: Arc<SsrRenderer>,
    selector: String,
    props: Option<Value>,
    hydration: HydrationStrategy,
}

impl SsrBuilder {
    /// Set props for the component
    pub fn with_props(mut self, props: Value) -> Self {
        self.props = Some(props);
        self
    }

    /// Set hydration strategy
    pub fn with_hydration(mut self, strategy: HydrationStrategy) -> Self {
        self.hydration = strategy;
        self
    }

    /// Execute the render
    pub async fn await(self) -> Result<SsrResponse, ArmatureStyleError> {
        let html = self.renderer
            .render_with_hydration(&self.selector, self.props)
            .await?;

        Ok(SsrResponse::new(html))
    }
}
