//! Error types for SSR operations.

use std::fmt;

/// Result type for SSR operations.
pub type SsrResult<T> = Result<T, SsrError>;

/// Errors that can occur during server-side rendering.
#[derive(Debug)]
pub enum SsrError {
    /// Error during component rendering.
    RenderError(String),

    /// Error serializing state.
    SerializationError(String),

    /// HTTP-related error.
    HttpError(String),

    /// I/O error.
    IoError(std::io::Error),

    /// Hyper error.
    HyperError(hyper::Error),

    /// Configuration error.
    ConfigError(String),

    /// Template error.
    TemplateError(String),
}

impl fmt::Display for SsrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SsrError::RenderError(msg) => write!(f, "Render error: {}", msg),
            SsrError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            SsrError::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            SsrError::IoError(err) => write!(f, "I/O error: {}", err),
            SsrError::HyperError(err) => write!(f, "Hyper error: {}", err),
            SsrError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            SsrError::TemplateError(msg) => write!(f, "Template error: {}", msg),
        }
    }
}

impl std::error::Error for SsrError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SsrError::IoError(err) => Some(err),
            SsrError::HyperError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for SsrError {
    fn from(err: std::io::Error) -> Self {
        SsrError::IoError(err)
    }
}

impl From<hyper::Error> for SsrError {
    fn from(err: hyper::Error) -> Self {
        SsrError::HyperError(err)
    }
}

impl From<serde_json::Error> for SsrError {
    fn from(err: serde_json::Error) -> Self {
        SsrError::SerializationError(err.to_string())
    }
}

