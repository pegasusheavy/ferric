//! Error types for ferric-http

use std::fmt;

/// Result type alias for ferric-http operations
pub type Result<T> = std::result::Result<T, Error>;

/// HTTP client error types
#[derive(Debug, Clone)]
pub enum Error {
    /// Network error (connection failed, timeout, etc.)
    Network(String),

    /// Invalid URL
    InvalidUrl(String),

    /// Request building error
    Request(String),

    /// Response parsing error
    Response(String),

    /// JSON serialization/deserialization error
    #[cfg(feature = "json")]
    Json(String),

    /// HTTP status error (non-2xx response)
    Status {
        code: u16,
        message: String,
    },

    /// Request was aborted
    Aborted,

    /// Timeout error
    Timeout,

    /// Request was cancelled
    Cancelled(Option<String>),

    /// Invalid header name or value
    InvalidHeader(String),

    /// Body encoding error
    Body(String),

    /// Interceptor error
    Interceptor(String),

    /// Retry exhausted
    RetryExhausted {
        attempts: u32,
        last_error: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Network(msg) => write!(f, "Network error: {}", msg),
            Error::InvalidUrl(url) => write!(f, "Invalid URL: {}", url),
            Error::Request(msg) => write!(f, "Request error: {}", msg),
            Error::Response(msg) => write!(f, "Response error: {}", msg),
            #[cfg(feature = "json")]
            Error::Json(msg) => write!(f, "JSON error: {}", msg),
            Error::Status { code, message } => {
                write!(f, "HTTP {} {}", code, message)
            }
            Error::Aborted => write!(f, "Request aborted"),
            Error::Timeout => write!(f, "Request timeout"),
            Error::Cancelled(reason) => match reason {
                Some(r) => write!(f, "Request cancelled: {}", r),
                None => write!(f, "Request cancelled"),
            },
            Error::InvalidHeader(msg) => write!(f, "Invalid header: {}", msg),
            Error::Body(msg) => write!(f, "Body error: {}", msg),
            Error::Interceptor(msg) => write!(f, "Interceptor error: {}", msg),
            Error::RetryExhausted { attempts, last_error } => {
                write!(f, "Retry exhausted after {} attempts: {}", attempts, last_error)
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Self {
        Error::InvalidUrl(err.to_string())
    }
}

#[cfg(feature = "json")]
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err.to_string())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Error::Timeout
        } else if err.is_connect() {
            Error::Network(err.to_string())
        } else if err.is_status() {
            let status = err.status().unwrap();
            Error::Status {
                code: status.as_u16(),
                message: status.canonical_reason().unwrap_or("Unknown").to_string(),
            }
        } else {
            Error::Request(err.to_string())
        }
    }
}

