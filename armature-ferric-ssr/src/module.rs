//! SSR module for Armature applications.

use crate::{SsrConfig, SsrService};
use armature_core::module::Module;
use armature_core::provider::Provider;
use std::sync::Arc;

/// Module that provides SSR capabilities.
pub struct SsrModule {
    config: SsrConfig,
}

impl SsrModule {
    /// Create a new SSR module with default configuration.
    pub fn new() -> Self {
        Self {
            config: SsrConfig::default(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(config: SsrConfig) -> Self {
        Self { config }
    }
}

impl Default for SsrModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for SsrModule {
    fn name(&self) -> &str {
        "SsrModule"
    }

    fn providers(&self) -> Vec<Box<dyn Provider>> {
        vec![
            Box::new(SsrServiceProvider {
                config: self.config.clone(),
            }),
        ]
    }
}

/// Provider for SSR service.
struct SsrServiceProvider {
    config: SsrConfig,
}

impl Provider for SsrServiceProvider {
    fn provide(&self) -> Arc<dyn std::any::Any + Send + Sync> {
        Arc::new(SsrService::new(self.config.clone()))
    }

    fn type_name(&self) -> &'static str {
        std::any::type_name::<SsrService>()
    }
}

