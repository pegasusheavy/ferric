//! Component rendering to HTML strings.

use crate::error::{SsrError, SsrResult};
use crate::hydration::{HydrationConfig, HydrationStrategy};
use crate::state::{SerializedState, StateSerializer};
use html_escape::encode_text;
use std::collections::HashMap;

/// Context for rendering operations.
#[derive(Debug, Clone, Default)]
pub struct RenderContext {
    /// State to be serialized for hydration.
    state: HashMap<String, serde_json::Value>,

    /// Current rendering depth (for detecting infinite loops).
    depth: usize,

    /// Maximum rendering depth.
    max_depth: usize,

    /// Whether to escape HTML in text content.
    escape_html: bool,
}

impl RenderContext {
    /// Create a new render context.
    pub fn new() -> Self {
        Self {
            state: HashMap::new(),
            depth: 0,
            max_depth: 100,
            escape_html: true,
        }
    }

    /// Set the maximum rendering depth.
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Disable HTML escaping (use with caution).
    pub fn without_escaping(mut self) -> Self {
        self.escape_html = false;
        self
    }

    /// Add state to be serialized for hydration.
    pub fn add_state<T: serde::Serialize>(&mut self, key: &str, value: &T) -> SsrResult<()> {
        let json = serde_json::to_value(value)?;
        self.state.insert(key.to_string(), json);
        Ok(())
    }

    /// Get the accumulated state.
    pub fn state(&self) -> &HashMap<String, serde_json::Value> {
        &self.state
    }

    /// Increment depth and check for overflow.
    pub fn enter(&mut self) -> SsrResult<()> {
        self.depth += 1;
        if self.depth > self.max_depth {
            return Err(SsrError::RenderError(format!(
                "Maximum rendering depth ({}) exceeded",
                self.max_depth
            )));
        }
        Ok(())
    }

    /// Decrement depth.
    pub fn exit(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    /// Escape text if escaping is enabled.
    pub fn escape_text(&self, text: &str) -> String {
        if self.escape_html {
            encode_text(text).to_string()
        } else {
            text.to_string()
        }
    }
}

/// Trait for types that can be rendered to HTML.
pub trait Renderable {
    /// Render this component to an HTML string.
    fn render(&self, ctx: &mut RenderContext) -> SsrResult<String>;

    /// Get the component's display name (for debugging).
    fn display_name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

/// HTML renderer for building HTML output.
#[derive(Debug, Default)]
pub struct HtmlRenderer {
    buffer: String,
    indent_level: usize,
    pretty_print: bool,
}

impl HtmlRenderer {
    /// Create a new HTML renderer.
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            indent_level: 0,
            pretty_print: false,
        }
    }

    /// Enable pretty printing with indentation.
    pub fn pretty(mut self) -> Self {
        self.pretty_print = true;
        self
    }

    /// Write raw HTML (no escaping).
    pub fn raw(&mut self, html: &str) -> &mut Self {
        self.buffer.push_str(html);
        self
    }

    /// Write escaped text content.
    pub fn text(&mut self, text: &str) -> &mut Self {
        self.buffer.push_str(&encode_text(text));
        self
    }

    /// Open an HTML tag.
    pub fn open_tag(&mut self, tag: &str) -> &mut Self {
        self.write_indent();
        self.buffer.push('<');
        self.buffer.push_str(tag);
        self
    }

    /// Add an attribute to the current tag.
    pub fn attr(&mut self, name: &str, value: &str) -> &mut Self {
        self.buffer.push(' ');
        self.buffer.push_str(name);
        self.buffer.push_str("=\"");
        self.buffer.push_str(&encode_text(value));
        self.buffer.push('"');
        self
    }

    /// Add a boolean attribute (present = true).
    pub fn bool_attr(&mut self, name: &str, value: bool) -> &mut Self {
        if value {
            self.buffer.push(' ');
            self.buffer.push_str(name);
        }
        self
    }

    /// Close the opening tag.
    pub fn close_open(&mut self) -> &mut Self {
        self.buffer.push('>');
        if self.pretty_print {
            self.buffer.push('\n');
        }
        self.indent_level += 1;
        self
    }

    /// Close a self-closing tag.
    pub fn self_close(&mut self) -> &mut Self {
        self.buffer.push_str(" />");
        if self.pretty_print {
            self.buffer.push('\n');
        }
        self
    }

    /// Write a closing tag.
    pub fn close_tag(&mut self, tag: &str) -> &mut Self {
        self.indent_level = self.indent_level.saturating_sub(1);
        self.write_indent();
        self.buffer.push_str("</");
        self.buffer.push_str(tag);
        self.buffer.push('>');
        if self.pretty_print {
            self.buffer.push('\n');
        }
        self
    }

    /// Write an element with text content.
    pub fn element(&mut self, tag: &str, attrs: &[(&str, &str)], content: &str) -> &mut Self {
        self.open_tag(tag);
        for (name, value) in attrs {
            self.attr(name, value);
        }
        self.buffer.push('>');
        self.text(content);
        self.buffer.push_str("</");
        self.buffer.push_str(tag);
        self.buffer.push('>');
        if self.pretty_print {
            self.buffer.push('\n');
        }
        self
    }

    /// Write a void element (no closing tag).
    pub fn void_element(&mut self, tag: &str, attrs: &[(&str, &str)]) -> &mut Self {
        self.open_tag(tag);
        for (name, value) in attrs {
            self.attr(name, value);
        }
        self.buffer.push('>');
        if self.pretty_print {
            self.buffer.push('\n');
        }
        self
    }

    /// Write indentation if pretty printing.
    fn write_indent(&mut self) {
        if self.pretty_print {
            for _ in 0..self.indent_level {
                self.buffer.push_str("  ");
            }
        }
    }

    /// Get the rendered HTML string.
    pub fn finish(self) -> String {
        self.buffer
    }

    /// Get the current buffer as a string slice.
    pub fn as_str(&self) -> &str {
        &self.buffer
    }
}

/// Render a component to an HTML string.
pub fn render_to_string<T: Renderable>(component: &T) -> SsrResult<String> {
    let mut ctx = RenderContext::new();
    component.render(&mut ctx)
}

/// Render a component to an HTML string with state serialization.
pub fn render_to_string_with_state<T: Renderable>(
    component: &T,
) -> SsrResult<(String, SerializedState)> {
    let mut ctx = RenderContext::new();
    let html = component.render(&mut ctx)?;
    let state = StateSerializer::serialize_map(ctx.state())?;
    Ok((html, state))
}

/// Render a component with hydration support.
pub fn render_with_hydration<T: Renderable>(
    component: &T,
    strategy: &dyn HydrationStrategy,
    component_id: &str,
) -> SsrResult<(String, SerializedState)> {
    let mut ctx = RenderContext::new();
    let html = component.render(&mut ctx)?;
    let state = StateSerializer::serialize_map(ctx.state())?;

    // Add hydration markers to the HTML if component should be hydrated
    let html_with_markers = if strategy.should_hydrate(component_id) {
        let markers = strategy.generate_markers(component_id);
        wrap_with_markers(&html, &markers)
    } else {
        html
    };

    Ok((html_with_markers, state))
}

/// Wrap HTML content with hydration markers.
fn wrap_with_markers(html: &str, markers: &crate::hydration::HydrationMarkers) -> String {
    if markers.hydrate {
        let attrs = markers.to_html_attrs();
        // Find the first element and add attributes to it
        // This is a simple implementation - in production you'd use a proper HTML parser
        if let Some(pos) = html.find('>') {
            let (start, end) = html.split_at(pos);
            format!("{} {}{}", start, attrs, end)
        } else {
            html.to_string()
        }
    } else {
        html.to_string()
    }
}

/// Wrap rendered content in an HTML document shell.
pub fn wrap_in_shell(content: &str, state: Option<&SerializedState>, config: &ShellConfig) -> String {
    let state_script = state
        .map(|s| {
            format!(
                r#"<script id="{}" type="application/json">{}</script>"#,
                config.state_script_id,
                s.as_json()
            )
        })
        .unwrap_or_default();

    let hydration_script = config.hydration_script.as_deref().unwrap_or("");

    if let Some(template) = &config.custom_template {
        template
            .replace("{content}", content)
            .replace("{state}", &state_script)
            .replace("{title}", &config.title)
            .replace("{hydration}", hydration_script)
    } else {
        format!(
            r#"<!DOCTYPE html>
<html lang="{}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    {}
</head>
<body>
    <div id="{}">{}</div>
    {}
    {}
    {}
</body>
</html>"#,
            config.lang,
            encode_text(&config.title),
            config.head_extra,
            config.root_id,
            content,
            state_script,
            hydration_script,
            config.body_extra
        )
    }
}

/// Configuration for the HTML shell.
#[derive(Debug, Clone)]
pub struct ShellConfig {
    /// Document language.
    pub lang: String,
    /// Document title.
    pub title: String,
    /// Root element ID.
    pub root_id: String,
    /// ID for the state script tag.
    pub state_script_id: String,
    /// Extra content for the head.
    pub head_extra: String,
    /// Extra content for the body.
    pub body_extra: String,
    /// Custom template.
    pub custom_template: Option<String>,
    /// Hydration script (generated by HydrationStrategy).
    pub hydration_script: Option<String>,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            lang: "en".to_string(),
            title: "Ferric App".to_string(),
            root_id: "app".to_string(),
            state_script_id: "__FERRIC_STATE__".to_string(),
            head_extra: String::new(),
            body_extra: String::new(),
            custom_template: None,
            hydration_script: None,
        }
    }
}

impl ShellConfig {
    /// Create a new shell config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the document title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set the root element ID.
    pub fn with_root_id(mut self, id: impl Into<String>) -> Self {
        self.root_id = id.into();
        self
    }

    /// Set the hydration script.
    pub fn with_hydration(mut self, strategy: &dyn HydrationStrategy) -> Self {
        let config = HydrationConfig::new()
            .with_root_id(&self.root_id)
            .with_state_script_id(&self.state_script_id);
        self.hydration_script = Some(strategy.generate_hydration_script(&config));
        self
    }

    /// Add extra content to the head.
    pub fn with_head_extra(mut self, content: impl Into<String>) -> Self {
        self.head_extra = content.into();
        self
    }

    /// Add extra content to the body.
    pub fn with_body_extra(mut self, content: impl Into<String>) -> Self {
        self.body_extra = content.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestComponent {
        message: String,
    }

    impl Renderable for TestComponent {
        fn render(&self, ctx: &mut RenderContext) -> SsrResult<String> {
            ctx.enter()?;
            let mut renderer = HtmlRenderer::new();
            renderer
                .open_tag("div")
                .attr("class", "test")
                .close_open()
                .text(&self.message)
                .close_tag("div");
            ctx.exit();
            Ok(renderer.finish())
        }
    }

    #[test]
    fn test_render_to_string() {
        let component = TestComponent {
            message: "Hello, World!".to_string(),
        };
        let html = render_to_string(&component).unwrap();
        assert!(html.contains("Hello, World!"));
        assert!(html.contains("class=\"test\""));
    }

    #[test]
    fn test_html_escaping() {
        let component = TestComponent {
            message: "<script>alert('xss')</script>".to_string(),
        };
        let html = render_to_string(&component).unwrap();
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_html_renderer() {
        let mut renderer = HtmlRenderer::new();
        renderer
            .open_tag("a")
            .attr("href", "https://example.com")
            .close_open()
            .text("Click me")
            .close_tag("a");

        let html = renderer.finish();
        assert_eq!(html, r#"<a href="https://example.com">Click me</a>"#);
    }
}

