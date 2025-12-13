//! Configuration for the SSR server.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

/// Configuration for the SSR server.
#[derive(Debug, Clone)]
pub struct SsrConfig {
    /// Address to bind the server to.
    pub bind_addr: SocketAddr,

    /// Directory for static files.
    pub static_dir: Option<PathBuf>,

    /// Whether to enable streaming responses.
    pub streaming_enabled: bool,

    /// Buffer size for streaming responses.
    pub stream_buffer_size: usize,

    /// Request timeout.
    pub request_timeout: Duration,

    /// Maximum request body size.
    pub max_body_size: usize,

    /// Whether to include state serialization for hydration.
    pub include_state: bool,

    /// ID attribute for the root element.
    pub root_id: String,

    /// Custom HTML shell template.
    pub html_shell: Option<String>,

    /// Whether to minify HTML output.
    pub minify_html: bool,

    /// Enable development mode (more verbose errors, no caching).
    pub dev_mode: bool,
}

impl Default for SsrConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:3000".parse().unwrap(),
            static_dir: None,
            streaming_enabled: true,
            stream_buffer_size: 8192,
            request_timeout: Duration::from_secs(30),
            max_body_size: 10 * 1024 * 1024, // 10MB
            include_state: true,
            root_id: "app".to_string(),
            html_shell: None,
            minify_html: false,
            dev_mode: false,
        }
    }
}

impl SsrConfig {
    /// Create a new configuration with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the bind address.
    pub fn bind(mut self, addr: &str) -> Self {
        self.bind_addr = addr.parse().expect("Invalid bind address");
        self
    }

    /// Set the bind address from a SocketAddr.
    pub fn bind_addr(mut self, addr: SocketAddr) -> Self {
        self.bind_addr = addr;
        self
    }

    /// Set the static files directory.
    pub fn with_static_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.static_dir = Some(path.into());
        self
    }

    /// Enable or disable streaming responses.
    pub fn streaming(mut self, enabled: bool) -> Self {
        self.streaming_enabled = enabled;
        self
    }

    /// Set the stream buffer size.
    pub fn stream_buffer_size(mut self, size: usize) -> Self {
        self.stream_buffer_size = size;
        self
    }

    /// Set the request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Set the maximum request body size.
    pub fn max_body_size(mut self, size: usize) -> Self {
        self.max_body_size = size;
        self
    }

    /// Enable or disable state serialization for hydration.
    pub fn with_state(mut self, enabled: bool) -> Self {
        self.include_state = enabled;
        self
    }

    /// Set the root element ID.
    pub fn root_id(mut self, id: impl Into<String>) -> Self {
        self.root_id = id.into();
        self
    }

    /// Set a custom HTML shell template.
    ///
    /// The template should contain `{content}` and optionally `{state}` placeholders.
    pub fn html_shell(mut self, template: impl Into<String>) -> Self {
        self.html_shell = Some(template.into());
        self
    }

    /// Enable or disable HTML minification.
    pub fn minify(mut self, enabled: bool) -> Self {
        self.minify_html = enabled;
        self
    }

    /// Enable development mode.
    pub fn dev(mut self, enabled: bool) -> Self {
        self.dev_mode = enabled;
        self
    }
}

