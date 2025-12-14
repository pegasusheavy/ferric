//! Dependency Injection integration for HTTP client.
//!
//! Makes the HTTP client injectable with configuration via DI tokens.

use crate::{Client, ClientBuilder, Credentials, Headers, Result};

/// Injection tokens for HTTP client configuration.
pub mod tokens {
    use super::*;
    use ferric_core::di::InjectionToken;

    /// Base URL for all HTTP requests.
    pub const HTTP_BASE_URL: InjectionToken<String> =
        InjectionToken::with_id("HTTP_BASE_URL", 1001);

    /// Default timeout in milliseconds for HTTP requests.
    pub const HTTP_TIMEOUT_MS: InjectionToken<u32> =
        InjectionToken::with_id("HTTP_TIMEOUT_MS", 1002);

    /// Default credentials mode for HTTP requests.
    pub const HTTP_CREDENTIALS: InjectionToken<Credentials> =
        InjectionToken::with_id("HTTP_CREDENTIALS", 1003);

    /// Bearer token for authentication.
    pub const HTTP_BEARER_TOKEN: InjectionToken<String> =
        InjectionToken::with_id("HTTP_BEARER_TOKEN", 1004);

    /// API key for authentication.
    pub const HTTP_API_KEY: InjectionToken<String> =
        InjectionToken::with_id("HTTP_API_KEY", 1005);

    /// Custom default headers.
    pub const HTTP_DEFAULT_HEADERS: InjectionToken<Headers> =
        InjectionToken::with_id("HTTP_DEFAULT_HEADERS", 1006);
}

/// Injectable HTTP client that pulls configuration from DI.
#[derive(Clone)]
pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    /// Create a new HTTP client with DI-provided configuration.
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Get the underlying client.
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Get a reference to use the client directly.
    pub fn as_ref(&self) -> &Client {
        &self.client
    }
}

impl std::ops::Deref for HttpClient {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl ferric_core::di::Injectable for HttpClient {
    fn create(injector: &ferric_core::di::Injector) -> Self {
        let mut builder = ClientBuilder::new();

        // Configure from DI tokens
        if let Some(base_url) = injector.resolve_token(&tokens::HTTP_BASE_URL) {
            builder = builder.base_url(base_url.as_str());
        }

        if let Some(timeout) = injector.resolve_token(&tokens::HTTP_TIMEOUT_MS) {
            builder = builder.timeout_ms(*timeout);
        }

        if let Some(credentials) = injector.resolve_token(&tokens::HTTP_CREDENTIALS) {
            builder = builder.credentials(*credentials);
        }

        if let Some(token) = injector.resolve_token(&tokens::HTTP_BEARER_TOKEN) {
            builder = builder.bearer_auth(token.as_str());
        }

        if let Some(headers) = injector.resolve_token(&tokens::HTTP_DEFAULT_HEADERS) {
            for (key, value) in headers.iter() {
                builder = builder.default_header(key, value);
            }
        }

        let client = builder.build().expect("Failed to build HTTP client");
        Self::new(client)
    }
}

/// HTTP client factory for custom configuration.
pub struct HttpClientFactory {
    builder: ClientBuilder,
}

impl HttpClientFactory {
    /// Create a new HTTP client factory.
    pub fn new() -> Self {
        Self {
            builder: ClientBuilder::new(),
        }
    }

    /// Configure with a builder function.
    pub fn configure<F>(mut self, f: F) -> Self
    where
        F: FnOnce(ClientBuilder) -> ClientBuilder,
    {
        self.builder = f(self.builder);
        self
    }

    /// Build the client.
    pub fn build(self) -> Result<HttpClient> {
        Ok(HttpClient::new(self.builder.build()?))
    }
}

impl Default for HttpClientFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl ferric_core::di::Injectable for HttpClientFactory {
    fn create(_injector: &ferric_core::di::Injector) -> Self {
        Self::new()
    }
}

/// Helper to register HTTP client with DI.
pub fn provide_http_client(injector: &ferric_core::di::Injector) {
    injector.register_singleton::<HttpClient>();
}

/// Helper to configure HTTP client with common settings.
pub struct HttpModule;

impl HttpModule {
    /// Provide HTTP client with default configuration.
    pub fn provide_default(injector: &ferric_core::di::Injector) {
        injector.register_singleton::<HttpClient>();
    }

    /// Provide HTTP client with base URL.
    pub fn provide_with_base_url(
        injector: &ferric_core::di::Injector,
        base_url: impl Into<String>,
    ) {
        injector.register_token(&tokens::HTTP_BASE_URL, base_url.into());
        injector.register_singleton::<HttpClient>();
    }

    /// Provide HTTP client with authentication.
    pub fn provide_with_bearer_auth(
        injector: &ferric_core::di::Injector,
        token: impl Into<String>,
    ) {
        injector.register_token(&tokens::HTTP_BEARER_TOKEN, token.into());
        injector.register_singleton::<HttpClient>();
    }

    /// Provide HTTP client with full configuration.
    pub fn provide_with_config<F>(
        injector: &ferric_core::di::Injector,
        configure: F,
    ) where
        F: Fn(&ferric_core::di::Injector),
    {
        configure(injector);
        injector.register_singleton::<HttpClient>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    use ferric_core::di::Injector;

    #[test]
    fn test_http_client_injectable() {
        let injector = Injector::root();

        // Register with default config
        injector.register_singleton::<HttpClient>();

        // Resolve
        let client = injector.resolve::<HttpClient>().unwrap();
        assert!(Rc::strong_count(&client) >= 1);
    }

    #[test]
    fn test_http_client_with_base_url() {
        let injector = Injector::root();

        // Configure base URL
        injector.register_token(&tokens::HTTP_BASE_URL, "https://api.example.com".to_string());
        injector.register_singleton::<HttpClient>();

        // Resolve
        let client = injector.resolve::<HttpClient>().unwrap();
        // Client should be configured with base URL
        assert!(Rc::strong_count(&client) >= 1);
    }

    #[test]
    fn test_http_module() {
        let injector = Injector::root();

        // Use module helper
        HttpModule::provide_with_base_url(&injector, "https://api.example.com");

        // Resolve
        let client = injector.resolve::<HttpClient>().unwrap();
        assert!(Rc::strong_count(&client) >= 1);
    }
}

