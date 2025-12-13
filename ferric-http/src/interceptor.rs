//! Request and response interceptors.
//!
//! Interceptors allow you to modify requests before they are sent and
//! responses before they are returned to the caller.
//!
//! ## Usage
//!
//! ```ignore
//! use ferric_http::{Client, Interceptor, InterceptorChain};
//!
//! // Create an auth interceptor
//! let auth = Interceptor::request(|req| {
//!     req.header("Authorization", format!("Bearer {}", get_token()))
//! });
//!
//! // Create a logging interceptor
//! let logging = Interceptor::full(
//!     |req| { println!("Request: {} {}", req.method(), req.url()); req },
//!     |res| { println!("Response: {}", res.status()); res },
//! );
//!
//! // Use with client
//! let client = Client::builder()
//!     .interceptor(auth)
//!     .interceptor(logging)
//!     .build()?;
//! ```

use crate::{Request, Response, Result, Error};
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Type alias for a request interceptor function.
pub type RequestInterceptorFn = Arc<dyn Fn(Request) -> InterceptResult<Request> + Send + Sync>;

/// Type alias for a response interceptor function.
pub type ResponseInterceptorFn = Arc<dyn Fn(Response) -> InterceptResult<Response> + Send + Sync>;

/// Type alias for an async request interceptor function.
pub type AsyncRequestInterceptorFn = Arc<
    dyn Fn(Request) -> Pin<Box<dyn Future<Output = InterceptResult<Request>> + Send>>
        + Send
        + Sync,
>;

/// Type alias for an async response interceptor function.
pub type AsyncResponseInterceptorFn = Arc<
    dyn Fn(Response) -> Pin<Box<dyn Future<Output = InterceptResult<Response>> + Send>>
        + Send
        + Sync,
>;

/// Result of an interception operation.
#[derive(Debug)]
pub enum InterceptResult<T> {
    /// Continue with the (possibly modified) value.
    Continue(T),
    /// Abort the request/response processing.
    Abort(Error),
    /// Skip remaining interceptors and return immediately.
    Skip(T),
}

impl<T> InterceptResult<T> {
    /// Map the contained value.
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> InterceptResult<U> {
        match self {
            InterceptResult::Continue(v) => InterceptResult::Continue(f(v)),
            InterceptResult::Abort(e) => InterceptResult::Abort(e),
            InterceptResult::Skip(v) => InterceptResult::Skip(f(v)),
        }
    }

    /// Get the value if Continue or Skip.
    pub fn into_value(self) -> Result<T> {
        match self {
            InterceptResult::Continue(v) | InterceptResult::Skip(v) => Ok(v),
            InterceptResult::Abort(e) => Err(e),
        }
    }

    /// Check if this is a Continue result.
    pub fn is_continue(&self) -> bool {
        matches!(self, InterceptResult::Continue(_))
    }

    /// Check if this is an Abort result.
    pub fn is_abort(&self) -> bool {
        matches!(self, InterceptResult::Abort(_))
    }
}

/// An interceptor that can modify requests and/or responses.
#[derive(Clone)]
pub struct Interceptor {
    name: String,
    request_fn: Option<RequestInterceptorFn>,
    response_fn: Option<ResponseInterceptorFn>,
}

impl Interceptor {
    /// Create a request-only interceptor.
    pub fn request<F>(name: &str, f: F) -> Self
    where
        F: Fn(Request) -> Request + Send + Sync + 'static,
    {
        Self {
            name: name.to_string(),
            request_fn: Some(Arc::new(move |req| InterceptResult::Continue(f(req)))),
            response_fn: None,
        }
    }

    /// Create a request interceptor that can abort.
    pub fn request_with_result<F>(name: &str, f: F) -> Self
    where
        F: Fn(Request) -> InterceptResult<Request> + Send + Sync + 'static,
    {
        Self {
            name: name.to_string(),
            request_fn: Some(Arc::new(f)),
            response_fn: None,
        }
    }

    /// Create a response-only interceptor.
    pub fn response<F>(name: &str, f: F) -> Self
    where
        F: Fn(Response) -> Response + Send + Sync + 'static,
    {
        Self {
            name: name.to_string(),
            request_fn: None,
            response_fn: Some(Arc::new(move |res| InterceptResult::Continue(f(res)))),
        }
    }

    /// Create a response interceptor that can abort.
    pub fn response_with_result<F>(name: &str, f: F) -> Self
    where
        F: Fn(Response) -> InterceptResult<Response> + Send + Sync + 'static,
    {
        Self {
            name: name.to_string(),
            request_fn: None,
            response_fn: Some(Arc::new(f)),
        }
    }

    /// Create an interceptor for both request and response.
    pub fn full<RF, SF>(name: &str, request_fn: RF, response_fn: SF) -> Self
    where
        RF: Fn(Request) -> Request + Send + Sync + 'static,
        SF: Fn(Response) -> Response + Send + Sync + 'static,
    {
        Self {
            name: name.to_string(),
            request_fn: Some(Arc::new(move |req| InterceptResult::Continue(request_fn(req)))),
            response_fn: Some(Arc::new(move |res| InterceptResult::Continue(response_fn(res)))),
        }
    }

    /// Get the interceptor name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Intercept a request.
    pub fn intercept_request(&self, request: Request) -> InterceptResult<Request> {
        match &self.request_fn {
            Some(f) => f(request),
            None => InterceptResult::Continue(request),
        }
    }

    /// Intercept a response.
    pub fn intercept_response(&self, response: Response) -> InterceptResult<Response> {
        match &self.response_fn {
            Some(f) => f(response),
            None => InterceptResult::Continue(response),
        }
    }
}

impl fmt::Debug for Interceptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Interceptor")
            .field("name", &self.name)
            .field("has_request_fn", &self.request_fn.is_some())
            .field("has_response_fn", &self.response_fn.is_some())
            .finish()
    }
}

/// Chain of interceptors to be applied in order.
#[derive(Clone, Default)]
pub struct InterceptorChain {
    interceptors: Vec<Interceptor>,
}

impl InterceptorChain {
    /// Create a new empty chain.
    pub fn new() -> Self {
        Self {
            interceptors: Vec::new(),
        }
    }

    /// Add an interceptor to the chain.
    pub fn add(&mut self, interceptor: Interceptor) {
        self.interceptors.push(interceptor);
    }

    /// Add an interceptor and return self for chaining.
    pub fn with(mut self, interceptor: Interceptor) -> Self {
        self.add(interceptor);
        self
    }

    /// Get the number of interceptors.
    pub fn len(&self) -> usize {
        self.interceptors.len()
    }

    /// Check if the chain is empty.
    pub fn is_empty(&self) -> bool {
        self.interceptors.is_empty()
    }

    /// Process a request through all interceptors.
    pub fn process_request(&self, mut request: Request) -> Result<Request> {
        for interceptor in &self.interceptors {
            match interceptor.intercept_request(request) {
                InterceptResult::Continue(r) => request = r,
                InterceptResult::Skip(r) => return Ok(r),
                InterceptResult::Abort(e) => return Err(e),
            }
        }
        Ok(request)
    }

    /// Process a response through all interceptors (in reverse order).
    pub fn process_response(&self, mut response: Response) -> Result<Response> {
        for interceptor in self.interceptors.iter().rev() {
            match interceptor.intercept_response(response) {
                InterceptResult::Continue(r) => response = r,
                InterceptResult::Skip(r) => return Ok(r),
                InterceptResult::Abort(e) => return Err(e),
            }
        }
        Ok(response)
    }

    /// Get all interceptor names.
    pub fn names(&self) -> Vec<&str> {
        self.interceptors.iter().map(|i| i.name()).collect()
    }
}

impl fmt::Debug for InterceptorChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InterceptorChain")
            .field("interceptors", &self.names())
            .finish()
    }
}

// ============================================================================
// Built-in Interceptors
// ============================================================================

/// Create an authentication interceptor.
pub fn auth_interceptor<F>(token_provider: F) -> Interceptor
where
    F: Fn() -> String + Send + Sync + 'static,
{
    Interceptor::request("auth", move |mut req| {
        let token = token_provider();
        req.headers.bearer_auth(token);
        req
    })
}

/// Create a static bearer token interceptor.
pub fn bearer_interceptor(token: impl Into<String>) -> Interceptor {
    let token = token.into();
    Interceptor::request("bearer", move |mut req| {
        req.headers.bearer_auth(&token);
        req
    })
}

/// Create a logging interceptor.
pub fn logging_interceptor() -> Interceptor {
    Interceptor::full(
        "logging",
        |req| {
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(
                &format!("HTTP Request: {} {}", req.method(), req.url()).into(),
            );
            #[cfg(not(target_arch = "wasm32"))]
            println!("HTTP Request: {} {}", req.method(), req.url());
            req
        },
        |res| {
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(&format!("HTTP Response: {}", res.status()).into());
            #[cfg(not(target_arch = "wasm32"))]
            println!("HTTP Response: {}", res.status());
            res
        },
    )
}

/// Create a header-adding interceptor.
pub fn header_interceptor(key: impl Into<String>, value: impl Into<String>) -> Interceptor {
    let key = key.into();
    let value = value.into();
    Interceptor::request("header", move |mut req| {
        req.headers.insert(&key, &value);
        req
    })
}

/// Create an interceptor that adds default headers.
pub fn default_headers_interceptor(headers: Vec<(String, String)>) -> Interceptor {
    Interceptor::request("default_headers", move |mut req| {
        for (k, v) in &headers {
            if !req.headers.contains(k) {
                req.headers.insert(k, v);
            }
        }
        req
    })
}

/// Create an interceptor that validates response status.
pub fn status_validator_interceptor() -> Interceptor {
    Interceptor::response_with_result("status_validator", |res| {
        if res.status() >= 400 {
            InterceptResult::Abort(Error::Status {
                code: res.status(),
                message: format!("HTTP error: {}", res.status()),
            })
        } else {
            InterceptResult::Continue(res)
        }
    })
}

/// Create a content-type enforcing interceptor.
pub fn json_content_type_interceptor() -> Interceptor {
    Interceptor::request("json_content_type", |mut req| {
        if !req.headers.contains("content-type") {
            req.headers.content_type("application/json");
        }
        if !req.headers.contains("accept") {
            req.headers.insert("accept", "application/json");
        }
        req
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Method, Headers};

    fn make_test_request() -> Request {
        Request {
            method: Method::Get,
            url: url::Url::parse("https://example.com/test").unwrap(),
            headers: Headers::new(),
            body: crate::Body::Empty,
            timeout_ms: None,
            credentials: crate::Credentials::default(),
        }
    }

    #[test]
    fn test_request_interceptor() {
        let interceptor = Interceptor::request("test", |mut req| {
            req.headers.insert("X-Test", "value");
            req
        });

        let request = make_test_request();
        let result = interceptor.intercept_request(request);

        match result {
            InterceptResult::Continue(req) => {
                assert!(req.headers.contains("x-test"));
            }
            _ => panic!("Expected Continue"),
        }
    }

    #[test]
    fn test_interceptor_chain() {
        let chain = InterceptorChain::new()
            .with(Interceptor::request("first", |mut req| {
                req.headers.insert("X-First", "1");
                req
            }))
            .with(Interceptor::request("second", |mut req| {
                req.headers.insert("X-Second", "2");
                req
            }));

        assert_eq!(chain.len(), 2);

        let request = make_test_request();
        let result = chain.process_request(request).unwrap();

        assert!(result.headers.contains("x-first"));
        assert!(result.headers.contains("x-second"));
    }

    #[test]
    fn test_abort_interceptor() {
        let chain = InterceptorChain::new()
            .with(Interceptor::request_with_result("abort", |_req| {
                InterceptResult::Abort(Error::Request("Aborted".to_string()))
            }))
            .with(Interceptor::request("never_called", |mut req| {
                req.headers.insert("X-Never", "called");
                req
            }));

        let request = make_test_request();
        let result = chain.process_request(request);

        assert!(result.is_err());
    }
}


//!
//! Interceptors allow you to modify requests before they are sent and
//! responses before they are returned to the caller.
//!
//! ## Usage
//!
//! ```ignore
//! use ferric_http::{Client, Interceptor, InterceptorChain};
//!
//! // Create an auth interceptor
//! let auth = Interceptor::request(|req| {
//!     req.header("Authorization", format!("Bearer {}", get_token()))
//! });
//!
//! // Create a logging interceptor
//! let logging = Interceptor::full(
//!     |req| { println!("Request: {} {}", req.method(), req.url()); req },
//!     |res| { println!("Response: {}", res.status()); res },
//! );
//!
//! // Use with client
//! let client = Client::builder()
//!     .interceptor(auth)
//!     .interceptor(logging)
//!     .build()?;
//! ```

use crate::{Request, Response, Result, Error};
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Type alias for a request interceptor function.
pub type RequestInterceptorFn = Arc<dyn Fn(Request) -> InterceptResult<Request> + Send + Sync>;

/// Type alias for a response interceptor function.
pub type ResponseInterceptorFn = Arc<dyn Fn(Response) -> InterceptResult<Response> + Send + Sync>;

/// Type alias for an async request interceptor function.
pub type AsyncRequestInterceptorFn = Arc<
    dyn Fn(Request) -> Pin<Box<dyn Future<Output = InterceptResult<Request>> + Send>>
        + Send
        + Sync,
>;

/// Type alias for an async response interceptor function.
pub type AsyncResponseInterceptorFn = Arc<
    dyn Fn(Response) -> Pin<Box<dyn Future<Output = InterceptResult<Response>> + Send>>
        + Send
        + Sync,
>;

/// Result of an interception operation.
#[derive(Debug)]
pub enum InterceptResult<T> {
    /// Continue with the (possibly modified) value.
    Continue(T),
    /// Abort the request/response processing.
    Abort(Error),
    /// Skip remaining interceptors and return immediately.
    Skip(T),
}

impl<T> InterceptResult<T> {
    /// Map the contained value.
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> InterceptResult<U> {
        match self {
            InterceptResult::Continue(v) => InterceptResult::Continue(f(v)),
            InterceptResult::Abort(e) => InterceptResult::Abort(e),
            InterceptResult::Skip(v) => InterceptResult::Skip(f(v)),
        }
    }

    /// Get the value if Continue or Skip.
    pub fn into_value(self) -> Result<T> {
        match self {
            InterceptResult::Continue(v) | InterceptResult::Skip(v) => Ok(v),
            InterceptResult::Abort(e) => Err(e),
        }
    }

    /// Check if this is a Continue result.
    pub fn is_continue(&self) -> bool {
        matches!(self, InterceptResult::Continue(_))
    }

    /// Check if this is an Abort result.
    pub fn is_abort(&self) -> bool {
        matches!(self, InterceptResult::Abort(_))
    }
}

/// An interceptor that can modify requests and/or responses.
#[derive(Clone)]
pub struct Interceptor {
    name: String,
    request_fn: Option<RequestInterceptorFn>,
    response_fn: Option<ResponseInterceptorFn>,
}

impl Interceptor {
    /// Create a request-only interceptor.
    pub fn request<F>(name: &str, f: F) -> Self
    where
        F: Fn(Request) -> Request + Send + Sync + 'static,
    {
        Self {
            name: name.to_string(),
            request_fn: Some(Arc::new(move |req| InterceptResult::Continue(f(req)))),
            response_fn: None,
        }
    }

    /// Create a request interceptor that can abort.
    pub fn request_with_result<F>(name: &str, f: F) -> Self
    where
        F: Fn(Request) -> InterceptResult<Request> + Send + Sync + 'static,
    {
        Self {
            name: name.to_string(),
            request_fn: Some(Arc::new(f)),
            response_fn: None,
        }
    }

    /// Create a response-only interceptor.
    pub fn response<F>(name: &str, f: F) -> Self
    where
        F: Fn(Response) -> Response + Send + Sync + 'static,
    {
        Self {
            name: name.to_string(),
            request_fn: None,
            response_fn: Some(Arc::new(move |res| InterceptResult::Continue(f(res)))),
        }
    }

    /// Create a response interceptor that can abort.
    pub fn response_with_result<F>(name: &str, f: F) -> Self
    where
        F: Fn(Response) -> InterceptResult<Response> + Send + Sync + 'static,
    {
        Self {
            name: name.to_string(),
            request_fn: None,
            response_fn: Some(Arc::new(f)),
        }
    }

    /// Create an interceptor for both request and response.
    pub fn full<RF, SF>(name: &str, request_fn: RF, response_fn: SF) -> Self
    where
        RF: Fn(Request) -> Request + Send + Sync + 'static,
        SF: Fn(Response) -> Response + Send + Sync + 'static,
    {
        Self {
            name: name.to_string(),
            request_fn: Some(Arc::new(move |req| InterceptResult::Continue(request_fn(req)))),
            response_fn: Some(Arc::new(move |res| InterceptResult::Continue(response_fn(res)))),
        }
    }

    /// Get the interceptor name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Intercept a request.
    pub fn intercept_request(&self, request: Request) -> InterceptResult<Request> {
        match &self.request_fn {
            Some(f) => f(request),
            None => InterceptResult::Continue(request),
        }
    }

    /// Intercept a response.
    pub fn intercept_response(&self, response: Response) -> InterceptResult<Response> {
        match &self.response_fn {
            Some(f) => f(response),
            None => InterceptResult::Continue(response),
        }
    }
}

impl fmt::Debug for Interceptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Interceptor")
            .field("name", &self.name)
            .field("has_request_fn", &self.request_fn.is_some())
            .field("has_response_fn", &self.response_fn.is_some())
            .finish()
    }
}

/// Chain of interceptors to be applied in order.
#[derive(Clone, Default)]
pub struct InterceptorChain {
    interceptors: Vec<Interceptor>,
}

impl InterceptorChain {
    /// Create a new empty chain.
    pub fn new() -> Self {
        Self {
            interceptors: Vec::new(),
        }
    }

    /// Add an interceptor to the chain.
    pub fn add(&mut self, interceptor: Interceptor) {
        self.interceptors.push(interceptor);
    }

    /// Add an interceptor and return self for chaining.
    pub fn with(mut self, interceptor: Interceptor) -> Self {
        self.add(interceptor);
        self
    }

    /// Get the number of interceptors.
    pub fn len(&self) -> usize {
        self.interceptors.len()
    }

    /// Check if the chain is empty.
    pub fn is_empty(&self) -> bool {
        self.interceptors.is_empty()
    }

    /// Process a request through all interceptors.
    pub fn process_request(&self, mut request: Request) -> Result<Request> {
        for interceptor in &self.interceptors {
            match interceptor.intercept_request(request) {
                InterceptResult::Continue(r) => request = r,
                InterceptResult::Skip(r) => return Ok(r),
                InterceptResult::Abort(e) => return Err(e),
            }
        }
        Ok(request)
    }

    /// Process a response through all interceptors (in reverse order).
    pub fn process_response(&self, mut response: Response) -> Result<Response> {
        for interceptor in self.interceptors.iter().rev() {
            match interceptor.intercept_response(response) {
                InterceptResult::Continue(r) => response = r,
                InterceptResult::Skip(r) => return Ok(r),
                InterceptResult::Abort(e) => return Err(e),
            }
        }
        Ok(response)
    }

    /// Get all interceptor names.
    pub fn names(&self) -> Vec<&str> {
        self.interceptors.iter().map(|i| i.name()).collect()
    }
}

impl fmt::Debug for InterceptorChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InterceptorChain")
            .field("interceptors", &self.names())
            .finish()
    }
}

// ============================================================================
// Built-in Interceptors
// ============================================================================

/// Create an authentication interceptor.
pub fn auth_interceptor<F>(token_provider: F) -> Interceptor
where
    F: Fn() -> String + Send + Sync + 'static,
{
    Interceptor::request("auth", move |mut req| {
        let token = token_provider();
        req.headers.bearer_auth(token);
        req
    })
}

/// Create a static bearer token interceptor.
pub fn bearer_interceptor(token: impl Into<String>) -> Interceptor {
    let token = token.into();
    Interceptor::request("bearer", move |mut req| {
        req.headers.bearer_auth(&token);
        req
    })
}

/// Create a logging interceptor.
pub fn logging_interceptor() -> Interceptor {
    Interceptor::full(
        "logging",
        |req| {
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(
                &format!("HTTP Request: {} {}", req.method(), req.url()).into(),
            );
            #[cfg(not(target_arch = "wasm32"))]
            println!("HTTP Request: {} {}", req.method(), req.url());
            req
        },
        |res| {
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(&format!("HTTP Response: {}", res.status()).into());
            #[cfg(not(target_arch = "wasm32"))]
            println!("HTTP Response: {}", res.status());
            res
        },
    )
}

/// Create a header-adding interceptor.
pub fn header_interceptor(key: impl Into<String>, value: impl Into<String>) -> Interceptor {
    let key = key.into();
    let value = value.into();
    Interceptor::request("header", move |mut req| {
        req.headers.insert(&key, &value);
        req
    })
}

/// Create an interceptor that adds default headers.
pub fn default_headers_interceptor(headers: Vec<(String, String)>) -> Interceptor {
    Interceptor::request("default_headers", move |mut req| {
        for (k, v) in &headers {
            if !req.headers.contains(k) {
                req.headers.insert(k, v);
            }
        }
        req
    })
}

/// Create an interceptor that validates response status.
pub fn status_validator_interceptor() -> Interceptor {
    Interceptor::response_with_result("status_validator", |res| {
        if res.status() >= 400 {
            InterceptResult::Abort(Error::Status {
                code: res.status(),
                message: format!("HTTP error: {}", res.status()),
            })
        } else {
            InterceptResult::Continue(res)
        }
    })
}

/// Create a content-type enforcing interceptor.
pub fn json_content_type_interceptor() -> Interceptor {
    Interceptor::request("json_content_type", |mut req| {
        if !req.headers.contains("content-type") {
            req.headers.content_type("application/json");
        }
        if !req.headers.contains("accept") {
            req.headers.insert("accept", "application/json");
        }
        req
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Method, Headers};

    fn make_test_request() -> Request {
        Request {
            method: Method::Get,
            url: url::Url::parse("https://example.com/test").unwrap(),
            headers: Headers::new(),
            body: crate::Body::Empty,
            timeout_ms: None,
            credentials: crate::Credentials::default(),
        }
    }

    #[test]
    fn test_request_interceptor() {
        let interceptor = Interceptor::request("test", |mut req| {
            req.headers.insert("X-Test", "value");
            req
        });

        let request = make_test_request();
        let result = interceptor.intercept_request(request);

        match result {
            InterceptResult::Continue(req) => {
                assert!(req.headers.contains("x-test"));
            }
            _ => panic!("Expected Continue"),
        }
    }

    #[test]
    fn test_interceptor_chain() {
        let chain = InterceptorChain::new()
            .with(Interceptor::request("first", |mut req| {
                req.headers.insert("X-First", "1");
                req
            }))
            .with(Interceptor::request("second", |mut req| {
                req.headers.insert("X-Second", "2");
                req
            }));

        assert_eq!(chain.len(), 2);

        let request = make_test_request();
        let result = chain.process_request(request).unwrap();

        assert!(result.headers.contains("x-first"));
        assert!(result.headers.contains("x-second"));
    }

    #[test]
    fn test_abort_interceptor() {
        let chain = InterceptorChain::new()
            .with(Interceptor::request_with_result("abort", |_req| {
                InterceptResult::Abort(Error::Request("Aborted".to_string()))
            }))
            .with(Interceptor::request("never_called", |mut req| {
                req.headers.insert("X-Never", "called");
                req
            }));

        let request = make_test_request();
        let result = chain.process_request(request);

        assert!(result.is_err());
    }
}

