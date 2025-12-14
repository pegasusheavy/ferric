//! Request builder for HTTP requests

use crate::{Body, Headers, Method, Result};
use url::Url;

/// HTTP request configuration
#[derive(Debug, Clone)]
pub struct Request {
    pub(crate) method: Method,
    pub(crate) url: Url,
    pub(crate) headers: Headers,
    pub(crate) body: Body,
    pub(crate) timeout_ms: Option<u32>,
    pub(crate) credentials: Credentials,
}

/// Credentials mode for requests
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Credentials {
    /// Don't send or receive cookies
    Omit,
    /// Send cookies for same-origin requests only (default)
    #[default]
    SameOrigin,
    /// Always send cookies, even for cross-origin requests
    Include,
}

impl Request {
    /// Create a new request builder
    pub fn new(method: Method, url: impl AsRef<str>) -> Result<Self> {
        let url = Url::parse(url.as_ref())?;
        Ok(Self {
            method,
            url,
            headers: Headers::new(),
            body: Body::Empty,
            timeout_ms: None,
            credentials: Credentials::default(),
        })
    }

    /// Create a GET request
    pub fn get(url: impl AsRef<str>) -> Result<Self> {
        Self::new(Method::Get, url)
    }

    /// Create a POST request
    pub fn post(url: impl AsRef<str>) -> Result<Self> {
        Self::new(Method::Post, url)
    }

    /// Create a PUT request
    pub fn put(url: impl AsRef<str>) -> Result<Self> {
        Self::new(Method::Put, url)
    }

    /// Create a DELETE request
    pub fn delete(url: impl AsRef<str>) -> Result<Self> {
        Self::new(Method::Delete, url)
    }

    /// Create a PATCH request
    pub fn patch(url: impl AsRef<str>) -> Result<Self> {
        Self::new(Method::Patch, url)
    }

    /// Create a HEAD request
    pub fn head(url: impl AsRef<str>) -> Result<Self> {
        Self::new(Method::Head, url)
    }

    /// Set a header
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key, value);
        self
    }

    /// Set multiple headers
    pub fn headers(mut self, headers: Headers) -> Self {
        for (k, v) in headers {
            self.headers.insert(k, v);
        }
        self
    }

    /// Set the request body
    pub fn body(mut self, body: impl Into<Body>) -> Self {
        self.body = body.into();
        self
    }

    /// Set a JSON body (requires `json` feature)
    #[cfg(feature = "json")]
    pub fn json<T: serde::Serialize>(mut self, value: &T) -> Result<Self> {
        self.body = Body::json(value)?;
        self.headers.content_type("application/json");
        Ok(self)
    }

    /// Set form-encoded body
    pub fn form(mut self, fields: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>) -> Self {
        self.body = Body::form(fields);
        self.headers.content_type("application/x-www-form-urlencoded");
        self
    }

    /// Set a query parameter
    pub fn query(mut self, key: &str, value: &str) -> Self {
        self.url.query_pairs_mut().append_pair(key, value);
        self
    }

    /// Set multiple query parameters
    pub fn query_pairs(mut self, pairs: impl IntoIterator<Item = (impl AsRef<str>, impl AsRef<str>)>) -> Self {
        {
            let mut query = self.url.query_pairs_mut();
            for (k, v) in pairs {
                query.append_pair(k.as_ref(), v.as_ref());
            }
        }
        self
    }

    /// Set request timeout in milliseconds
    pub fn timeout_ms(mut self, ms: u32) -> Self {
        self.timeout_ms = Some(ms);
        self
    }

    /// Set request timeout in seconds
    pub fn timeout(self, secs: u32) -> Self {
        self.timeout_ms(secs * 1000)
    }

    /// Set credentials mode
    pub fn credentials(mut self, mode: Credentials) -> Self {
        self.credentials = mode;
        self
    }

    /// Set bearer token authentication
    pub fn bearer_auth(mut self, token: impl Into<String>) -> Self {
        self.headers.bearer_auth(token);
        self
    }

    /// Set basic authentication
    pub fn basic_auth(mut self, username: &str, password: Option<&str>) -> Self {
        self.headers.basic_auth(username, password);
        self
    }

    /// Get the URL
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Get the method
    pub fn method(&self) -> Method {
        self.method
    }

    /// Get the headers
    pub fn get_headers(&self) -> &Headers {
        &self.headers
    }
}

/// Builder for constructing requests with a base URL
#[derive(Debug, Clone)]
pub struct RequestBuilder {
    base_url: Option<Url>,
    default_headers: Headers,
    default_timeout_ms: Option<u32>,
    default_credentials: Credentials,
}

impl RequestBuilder {
    /// Create a new request builder
    pub fn new() -> Self {
        Self {
            base_url: None,
            default_headers: Headers::new(),
            default_timeout_ms: None,
            default_credentials: Credentials::default(),
        }
    }

    /// Set the base URL for relative paths
    pub fn base_url(mut self, url: impl AsRef<str>) -> Result<Self> {
        self.base_url = Some(Url::parse(url.as_ref())?);
        Ok(self)
    }

    /// Add a default header
    pub fn default_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.insert(key, value);
        self
    }

    /// Set default timeout in milliseconds
    pub fn default_timeout_ms(mut self, ms: u32) -> Self {
        self.default_timeout_ms = Some(ms);
        self
    }

    /// Set default credentials mode
    pub fn default_credentials(mut self, mode: Credentials) -> Self {
        self.default_credentials = mode;
        self
    }

    /// Build a request with the configured defaults
    pub fn build(&self, method: Method, path: impl AsRef<str>) -> Result<Request> {
        let url = if let Some(base) = &self.base_url {
            base.join(path.as_ref())?
        } else {
            Url::parse(path.as_ref())?
        };

        let mut request = Request {
            method,
            url,
            headers: self.default_headers.clone(),
            body: Body::Empty,
            timeout_ms: self.default_timeout_ms,
            credentials: self.default_credentials,
        };

        // Set Content-Type based on body if not already set
        if let Some(ct) = request.body.content_type()
            && !request.headers.contains("content-type") {
                request.headers.content_type(ct);
            }

        Ok(request)
    }

    /// Build a GET request
    pub fn get(&self, path: impl AsRef<str>) -> Result<Request> {
        self.build(Method::Get, path)
    }

    /// Build a POST request
    pub fn post(&self, path: impl AsRef<str>) -> Result<Request> {
        self.build(Method::Post, path)
    }

    /// Build a PUT request
    pub fn put(&self, path: impl AsRef<str>) -> Result<Request> {
        self.build(Method::Put, path)
    }

    /// Build a DELETE request
    pub fn delete(&self, path: impl AsRef<str>) -> Result<Request> {
        self.build(Method::Delete, path)
    }

    /// Build a PATCH request
    pub fn patch(&self, path: impl AsRef<str>) -> Result<Request> {
        self.build(Method::Patch, path)
    }
}

impl Default for RequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

