//! Error types for Armature-style SSR

use std::fmt;

/// Error type for Armature-style SSR operations
#[derive(Debug, Clone)]
pub enum ArmatureStyleError {
    /// Rendering failed
    RenderFailed(String),
    /// Component not found
    ComponentNotFound(String),
    /// Configuration error
    ConfigError(String),
    /// Internal error
    Internal(String),
}

impl fmt::Display for ArmatureStyleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RenderFailed(msg) => write!(f, "Render failed: {}", msg),
            Self::ComponentNotFound(name) => write!(f, "Component not found: {}", name),
            Self::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            Self::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for ArmatureStyleError {}

impl From<ferric_ssr::SsrError> for ArmatureStyleError {
    fn from(err: ferric_ssr::SsrError) -> Self {
        Self::RenderFailed(err.to_string())
    }
}
