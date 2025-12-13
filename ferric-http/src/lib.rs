//! # Ferric HTTP
//!
//! An ergonomic HTTP client library that works seamlessly on both WASM (browser) and native targets.
//!
//! - On WASM: Uses the browser's Fetch API
//! - On Native: Uses reqwest under the hood
//!
//! ## Features
//!
//! - **Unified API**: Same code works on both browser and server
//! - **Builder Pattern**: Fluent request construction
//! - **JSON Support**: Built-in JSON serialization/deserialization (with `json` feature)
//! - **Async/Await**: Fully async API
//! - **Type Safety**: Strong typing with helpful error types
//! - **Interceptors**: Modify requests/responses with middleware
//! - **Retry Logic**: Configurable retry with exponential backoff
//! - **Progress Events**: Track upload/download progress
//! - **Cancellation**: Cancel in-flight requests
//!
//! ## Quick Start
//!
//! ```ignore
//! use ferric_http::Client;
//!
//! // Create a client
//! let client = Client::new();
//!
//! // Simple GET request
//! let response = client.get("https://api.example.com/users").send().await?;
//! let users: Vec<User> = response.json()?;
//!
//! // POST with JSON body
//! let new_user = User { name: "Alice".into(), email: "alice@example.com".into() };
//! let response = client
//!     .post("https://api.example.com/users")
//!     .json(&new_user)?
//!     .send()
//!     .await?;
//!
//! // With interceptors and retry
//! let client = Client::builder()
//!     .base_url("https://api.example.com")
//!     .interceptor(logging_interceptor())
//!     .interceptor(bearer_interceptor("your-token"))
//!     .timeout(30)
//!     .build()?;
//! ```
//!
//! ## Retry Example
//!
//! ```ignore
//! use ferric_http::{Client, RetryPolicy};
//!
//! let response = client
//!     .get("https://api.example.com/data")
//!     .with_retry(RetryPolicy::exponential().max_retries(3))
//!     .send()
//!     .await?;
//! ```
//!
//! ## Progress Tracking
//!
//! ```ignore
//! use ferric_http::{Client, Progress};
//!
//! let progress = Progress::new()
//!     .on_download(|info| println!("Downloaded: {:.1}%", info.percent()));
//!
//! let response = client
//!     .get("https://example.com/large-file")
//!     .with_progress(progress)
//!     .send()
//!     .await?;
//! ```
//!
//! ## One-Shot Functions
//!
//! For simple requests, use the convenience functions:
//!
//! ```ignore
//! use ferric_http::{get, post_json};
//!
//! // Simple GET
//! let response = get("https://api.example.com/health").await?;
//!
//! // POST with JSON
//! let response = post_json("https://api.example.com/users", &user).await?;
//! ```

mod body;
mod client;
mod error;
mod headers;
mod method;
mod request;
mod response;

// New modules
mod cancel;
mod interceptor;
mod progress;
mod retry;

// Platform-specific implementations
#[cfg(target_arch = "wasm32")]
mod fetch;

#[cfg(not(target_arch = "wasm32"))]
mod reqwest_impl;

// Re-exports
pub use body::Body;
pub use client::{Client, ClientBuilder, RequestHandle};
pub use error::{Error, Result};
pub use headers::Headers;
pub use method::Method;
pub use request::{Credentials, Request, RequestBuilder};
pub use response::Response;

// Interceptors
pub use interceptor::{
    Interceptor, InterceptorChain, InterceptResult,
    RequestInterceptorFn, ResponseInterceptorFn,
    // Built-in interceptors
    auth_interceptor, bearer_interceptor, logging_interceptor,
    header_interceptor, default_headers_interceptor,
    status_validator_interceptor, json_content_type_interceptor,
};

// Progress tracking
pub use progress::{
    Progress, ProgressInfo, ProgressCallback, ProgressTracker, ProgressEvent,
};

// Cancellation
pub use cancel::{
    CancellationToken, CancellationTrigger, CancellationSource,
    CancelledFuture, CancelOnDrop,
    any_cancelled, all_cancelled,
};
#[cfg(not(target_arch = "wasm32"))]
pub use cancel::timeout_token;

// Retry logic
pub use retry::{
    RetryConfig, RetryStrategy, RetryPolicy, RetryState,
    RetryInfo, RetryReason, RetryCallback, RetryExecutor,
};

// Convenience functions
pub use client::{delete, get, patch, post, put};

#[cfg(feature = "json")]
pub use client::post_json;

/// Prelude module for common imports
pub mod prelude {
    pub use crate::{
        Body, Client, ClientBuilder, Credentials, Error, Headers, Method, Request, Response, Result,
    };

    // Interceptors
    pub use crate::{
        Interceptor, InterceptorChain, InterceptResult,
        logging_interceptor, bearer_interceptor, auth_interceptor,
    };

    // Progress
    pub use crate::{Progress, ProgressInfo};

    // Cancellation
    pub use crate::{CancellationToken, CancellationTrigger, CancellationSource};

    // Retry
    pub use crate::{RetryConfig, RetryPolicy, RetryStrategy};

    // Convenience functions
    pub use crate::{delete, get, patch, post, put};

    #[cfg(feature = "json")]
    pub use crate::post_json;
}

