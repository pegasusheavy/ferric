//! Unified HTTP client that works on both WASM and native targets

use crate::{Body, Credentials, Headers, Method, Request, Response, Result};

/// HTTP client for making requests
///
/// Works seamlessly on both WASM (using Fetch API) and native (using reqwest).
///
/// # Example
///
/// ```ignore
/// use ferric_http::{Client, Method};
///
/// let client = Client::new();
///
/// // Simple GET request
/// let response = client.get("https://api.example.com/data").await?;
/// let text = response.text()?;
///
/// // POST with JSON body
/// let response = client
///     .post("https://api.example.com/users")
///     .json(&user)?
///     .send()
///     .await?;
/// ```
#[derive(Debug, Clone)]
pub struct Client {
    base_url: Option<url::Url>,
    default_headers: Headers,
    default_timeout_ms: Option<u32>,
    default_credentials: Credentials,
    #[cfg(not(target_arch = "wasm32"))]
    inner: reqwest::Client,
}

impl Client {
    /// Create a new HTTP client
    pub fn new() -> Self {
        Self {
            base_url: None,
            default_headers: Headers::new(),
            default_timeout_ms: None,
            default_credentials: Credentials::default(),
            #[cfg(not(target_arch = "wasm32"))]
            inner: reqwest::Client::new(),
        }
    }

    /// Create a client builder for configuration
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// Set the base URL for relative paths
    pub fn with_base_url(mut self, url: impl AsRef<str>) -> Result<Self> {
        self.base_url = Some(url::Url::parse(url.as_ref())?);
        Ok(self)
    }

    /// Create a GET request
    pub fn get(&self, url: impl AsRef<str>) -> RequestHandle {
        self.request(Method::Get, url)
    }

    /// Create a POST request
    pub fn post(&self, url: impl AsRef<str>) -> RequestHandle {
        self.request(Method::Post, url)
    }

    /// Create a PUT request
    pub fn put(&self, url: impl AsRef<str>) -> RequestHandle {
        self.request(Method::Put, url)
    }

    /// Create a DELETE request
    pub fn delete(&self, url: impl AsRef<str>) -> RequestHandle {
        self.request(Method::Delete, url)
    }

    /// Create a PATCH request
    pub fn patch(&self, url: impl AsRef<str>) -> RequestHandle {
        self.request(Method::Patch, url)
    }

    /// Create a HEAD request
    pub fn head(&self, url: impl AsRef<str>) -> RequestHandle {
        self.request(Method::Head, url)
    }

    /// Create a request with the specified method
    pub fn request(&self, method: Method, url: impl AsRef<str>) -> RequestHandle {
        RequestHandle {
            client: self.clone(),
            method,
            url: url.as_ref().to_string(),
            headers: self.default_headers.clone(),
            body: Body::Empty,
            timeout_ms: self.default_timeout_ms,
            credentials: self.default_credentials,
            query_params: Vec::new(),
        }
    }

    /// Execute a pre-built request
    pub async fn execute(&self, request: Request) -> Result<Response> {
        #[cfg(target_arch = "wasm32")]
        {
            crate::fetch::execute(request).await
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            crate::reqwest_impl::execute_with_client(&self.inner, request).await
        }
    }

    /// Resolve a URL against the base URL
    fn resolve_url(&self, url: &str) -> Result<url::Url> {
        if let Some(base) = &self.base_url {
            // Check if URL is already absolute
            if url.starts_with("http://") || url.starts_with("https://") {
                Ok(url::Url::parse(url)?)
            } else {
                Ok(base.join(url)?)
            }
        } else {
            Ok(url::Url::parse(url)?)
        }
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for configuring a Client
#[derive(Debug, Default)]
pub struct ClientBuilder {
    base_url: Option<String>,
    default_headers: Headers,
    default_timeout_ms: Option<u32>,
    default_credentials: Credentials,
}

impl ClientBuilder {
    /// Create a new client builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the base URL for all requests
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Add a default header to all requests
    pub fn default_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.insert(key, value);
        self
    }

    /// Set the default timeout for all requests (in milliseconds)
    pub fn timeout_ms(mut self, ms: u32) -> Self {
        self.default_timeout_ms = Some(ms);
        self
    }

    /// Set the default timeout for all requests (in seconds)
    pub fn timeout(mut self, secs: u32) -> Self {
        self.default_timeout_ms = Some(secs * 1000);
        self
    }

    /// Set the default credentials mode
    pub fn credentials(mut self, mode: Credentials) -> Self {
        self.default_credentials = mode;
        self
    }

    /// Set bearer token authentication for all requests
    pub fn bearer_auth(mut self, token: impl Into<String>) -> Self {
        self.default_headers.bearer_auth(token);
        self
    }

    /// Build the client
    pub fn build(self) -> Result<Client> {
        let base_url = if let Some(url) = self.base_url {
            Some(url::Url::parse(&url)?)
        } else {
            None
        };

        #[cfg(not(target_arch = "wasm32"))]
        let inner = {
            let mut builder = reqwest::Client::builder();
            if let Some(timeout) = self.default_timeout_ms {
                builder = builder.timeout(std::time::Duration::from_millis(timeout as u64));
            }
            builder.build().map_err(|e| crate::Error::Request(e.to_string()))?
        };

        Ok(Client {
            base_url,
            default_headers: self.default_headers,
            default_timeout_ms: self.default_timeout_ms,
            default_credentials: self.default_credentials,
            #[cfg(not(target_arch = "wasm32"))]
            inner,
        })
    }
}

/// Handle for building and sending a request
#[derive(Debug, Clone)]
pub struct RequestHandle {
    client: Client,
    method: Method,
    url: String,
    headers: Headers,
    body: Body,
    timeout_ms: Option<u32>,
    credentials: Credentials,
    query_params: Vec<(String, String)>,
}

impl RequestHandle {
    /// Set a header
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key, value);
        self
    }

    /// Set multiple headers
    pub fn headers(mut self, headers: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>) -> Self {
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

    /// Add a query parameter
    pub fn query(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query_params.push((key.into(), value.into()));
        self
    }

    /// Add multiple query parameters
    pub fn query_pairs(mut self, pairs: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>) -> Self {
        for (k, v) in pairs {
            self.query_params.push((k.into(), v.into()));
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

    /// Build the request without sending
    pub fn build(self) -> Result<Request> {
        let mut url = self.client.resolve_url(&self.url)?;

        // Add query parameters
        if !self.query_params.is_empty() {
            let mut query = url.query_pairs_mut();
            for (k, v) in &self.query_params {
                query.append_pair(k, v);
            }
        }

        Ok(Request {
            method: self.method,
            url,
            headers: self.headers,
            body: self.body,
            timeout_ms: self.timeout_ms,
            credentials: self.credentials,
        })
    }

    /// Send the request and return the response
    pub async fn send(self) -> Result<Response> {
        #[cfg(not(target_arch = "wasm32"))]
        let inner_client = self.client.inner.clone();

        let request = self.build()?;

        #[cfg(target_arch = "wasm32")]
        {
            crate::fetch::execute(request).await
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            crate::reqwest_impl::execute_with_client(&inner_client, request).await
        }
    }
}

// Convenience functions for one-shot requests

/// Make a GET request
pub async fn get(url: impl AsRef<str>) -> Result<Response> {
    Client::new().get(url).send().await
}

/// Make a POST request
pub async fn post(url: impl AsRef<str>) -> Result<Response> {
    Client::new().post(url).send().await
}

/// Make a POST request with a JSON body (requires `json` feature)
#[cfg(feature = "json")]
pub async fn post_json<T: serde::Serialize>(url: impl AsRef<str>, body: &T) -> Result<Response> {
    Client::new().post(url).json(body)?.send().await
}

/// Make a PUT request
pub async fn put(url: impl AsRef<str>) -> Result<Response> {
    Client::new().put(url).send().await
}

/// Make a DELETE request
pub async fn delete(url: impl AsRef<str>) -> Result<Response> {
    Client::new().delete(url).send().await
}

/// Make a PATCH request
pub async fn patch(url: impl AsRef<str>) -> Result<Response> {
    Client::new().patch(url).send().await
}

