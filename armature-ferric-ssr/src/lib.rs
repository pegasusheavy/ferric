//! # Armature-Style Ferric SSR
//!
//! Server-Side Rendering module with an Armature-inspired API for Ferric framework.
//!
//! This module provides an Angular/NestJS-inspired API for SSR, demonstrating how
//! Ferric's SSR capabilities can be wrapped with decorator-style APIs similar to
//! frameworks like Armature, Actix, or Rocket.
//!
//! ## Features
//!
//! - **Decorator-style API**: Familiar patterns for Angular/NestJS developers
//! - **Automatic Hydration**: Configure hydration strategies per route
//! - **Performance**: Efficient rendering with Hyper
//! - **Type-safe**: Full Rust type safety
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use armature_ferric_ssr::prelude::*;
//!
//! #[derive(Clone)]
//! struct AppController {
//!     ssr_service: Arc<SsrService>,
//! }
//!
//! impl AppController {
//!     async fn index(&self) -> SsrResponse {
//!         self.ssr_service
//!             .render_component("app-root")
//!             .with_hydration(HydrationStrategy::Full)
//!             .await
//!     }
//! }
//! ```

use ferric_ssr::{SsrRenderer, SsrError, SsrConfig};
use std::sync::Arc;
use serde_json::Value;
use hyper::{Body, Request, Response, StatusCode};
use hyper::header;
use std::collections::HashMap;

pub mod config;
pub mod error;
pub mod response;
pub mod service;

pub use config::*;
pub use error::*;
pub use response::*;
pub use service::*;

// Re-export ferric-ssr types
pub use ferric_ssr::{
    HydrationStrategy, RenderConfig,
};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        SsrService, SsrResponse, SsrConfigBuilder,
        ArmatureStyleError,
    };

    pub use ferric_ssr::{
        HydrationStrategy, RenderConfig, SsrConfig,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_creation() {
        let config = SsrConfig::default();
        let _service = SsrService::new(config);
    }
}
