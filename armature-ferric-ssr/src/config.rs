//! Configuration for Armature-style SSR

use ferric_ssr::SsrConfig;
use std::time::Duration;

/// Builder for SSR configuration
pub struct SsrConfigBuilder {
    enable_hydration: bool,
    minify_html: bool,
    compression: bool,
    cache_ttl: Duration,
    shell_template: Option<String>,
}

impl Default for SsrConfigBuilder {
    fn default() -> Self {
        Self {
            enable_hydration: true,
            minify_html: false,
            compression: false,
            cache_ttl: Duration::from_secs(60),
            shell_template: None,
        }
    }
}

impl SsrConfigBuilder {
    /// Creates a new configuration builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable or disable hydration
    pub fn enable_hydration(mut self, enable: bool) -> Self {
        self.enable_hydration = enable;
        self
    }

    /// Enable or disable HTML minification
    pub fn minify_html(mut self, minify: bool) -> Self {
        self.minify_html = minify;
        self
    }

    /// Enable or disable compression
    pub fn compression(mut self, compress: bool) -> Self {
        self.compression = compress;
        self
    }

    /// Set cache TTL
    pub fn cache_ttl(mut self, ttl: Duration) -> Self {
        self.cache_ttl = ttl;
        self
    }

    /// Set custom shell template
    pub fn shell_template(mut self, template: String) -> Self {
        self.shell_template = Some(template);
        self
    }

    /// Build the configuration
    pub fn build(self) -> SsrConfig {
        SsrConfig::default() // Use ferric-ssr's default for now
    }
}
