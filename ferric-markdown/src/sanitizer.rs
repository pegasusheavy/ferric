//! HTML sanitization for security.

use crate::config::{MarkdownConfig, RenderOptions};
use ammonia::Builder;

/// HTML sanitizer to prevent XSS attacks.
#[derive(Default)]
pub struct HtmlSanitizer {
    allow_dangerous_html: bool,
}

impl HtmlSanitizer {
    /// Create a new sanitizer from configuration.
    pub fn new(config: &MarkdownConfig) -> Self {
        Self {
            allow_dangerous_html: config.allow_dangerous_html,
        }
    }

    /// Create a sanitizer from render options.
    pub fn with_options(options: &RenderOptions) -> Self {
        Self {
            allow_dangerous_html: options.allow_dangerous_html,
        }
    }

    /// Sanitize HTML to prevent XSS.
    ///
    /// This uses ammonia to clean the HTML and remove dangerous content.
    pub fn sanitize(&self, html: &str) -> String {
        if self.allow_dangerous_html {
            // Bypass sanitization (unsafe)
            html.to_string()
        } else {
            // Use ammonia for safe HTML
            let mut builder = Builder::default();

            // Allow common markdown elements
            builder
                .add_tags(&["mark", "del", "ins"])
                .add_tag_attributes("input", &["type", "checked", "disabled"])
                .link_rel(Some("noopener noreferrer"));

            builder.clean(html).to_string()
        }
    }

    /// Sanitize with strict settings (minimal allowed HTML).
    pub fn sanitize_strict(&self, html: &str) -> String {
        let builder = Builder::default();
        builder.clean(html).to_string()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_safe_html() {
        let sanitizer = HtmlSanitizer::default();
        let html = "<p>Hello <strong>World</strong></p>";
        let result = sanitizer.sanitize(html);
        assert!(result.contains("<p>"));
        assert!(result.contains("<strong>"));
    }

    #[test]
    fn test_sanitize_dangerous_html() {
        let sanitizer = HtmlSanitizer::default();
        let html = "<p>Hello</p><script>alert('xss')</script>";
        let result = sanitizer.sanitize(html);
        assert!(!result.contains("<script>"));
        assert!(result.contains("<p>"));
    }

    #[test]
    fn test_sanitize_onclick() {
        let sanitizer = HtmlSanitizer::default();
        let html = "<a href=\"#\" onclick=\"alert('xss')\">Click</a>";
        let result = sanitizer.sanitize(html);
        assert!(!result.contains("onclick"));
    }
}

