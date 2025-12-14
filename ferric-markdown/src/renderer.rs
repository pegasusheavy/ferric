//! HTML rendering from parsed markdown.

use crate::config::{MarkdownConfig, RenderOptions};
use crate::parser::MarkdownParser;
use pulldown_cmark::html;

/// Renders markdown to HTML.
pub struct MarkdownRenderer {
    parser: MarkdownParser,
    options: RenderOptions,
}

impl MarkdownRenderer {
    /// Create a new renderer with the given configuration.
    pub fn new(config: &MarkdownConfig) -> Self {
        let parser = MarkdownParser::with_options(
            config.enable_tables,
            config.enable_footnotes,
            config.enable_strikethrough,
            config.enable_tasklists,
        );

        Self {
            parser,
            options: RenderOptions::from(config),
        }
    }

    /// Create a renderer with custom options.
    pub fn with_options(options: &RenderOptions) -> Self {
        Self {
            parser: MarkdownParser::new(),
            options: options.clone(),
        }
    }

    /// Render markdown to HTML string.
    pub fn render(&self, markdown: &str) -> String {
        let parser = self.parser.parse(markdown);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        html_output
    }

    /// Render markdown and wrap in a container div.
    pub fn render_wrapped(&self, markdown: &str) -> String {
        let html = self.render(markdown);
        format!(
            r#"<div class="{}markdown">{}</div>"#,
            self.options.css_class_prefix, html
        )
    }

    /// Render inline markdown (no block elements).
    pub fn render_inline(&self, markdown: &str) -> String {
        // Remove paragraph tags for inline rendering
        let html = self.render(markdown);
        html.trim_start_matches("<p>")
            .trim_end_matches("</p>")
            .to_string()
    }
}

impl Default for MarkdownRenderer {
    fn default() -> Self {
        Self::new(&MarkdownConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_rendering() {
        let renderer = MarkdownRenderer::default();
        let html = renderer.render("# Hello");
        assert!(html.contains("<h1>"));
        assert!(html.contains("Hello"));
    }

    #[test]
    fn test_bold_rendering() {
        let renderer = MarkdownRenderer::default();
        let html = renderer.render("**bold**");
        assert!(html.contains("<strong>"));
        assert!(html.contains("bold"));
    }

    #[test]
    fn test_list_rendering() {
        let renderer = MarkdownRenderer::default();
        let html = renderer.render("- Item 1\n- Item 2");
        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>"));
    }

    #[test]
    fn test_inline_rendering() {
        let renderer = MarkdownRenderer::default();
        let html = renderer.render_inline("**bold**");
        assert!(!html.contains("<p>"));
        assert!(html.contains("<strong>"));
    }
}

