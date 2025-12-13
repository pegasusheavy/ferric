//! Layout component that wraps page content.

use ferric_ssr::prelude::*;

/// Main layout wrapper that provides consistent structure across pages.
pub struct Layout<T: Renderable> {
    title: String,
    content: T,
}

impl<T: Renderable> Layout<T> {
    /// Create a new layout with a title and content component.
    pub fn new(title: impl Into<String>, content: T) -> Self {
        Self {
            title: title.into(),
            content,
        }
    }
}

impl<T: Renderable> Renderable for Layout<T> {
    fn render(&self, ctx: &mut RenderContext) -> SsrResult<String> {
        ctx.enter()?;

        let content = self.content.render(ctx)?;

        let mut html = HtmlRenderer::new().pretty();

        // Document start
        html.raw("<!DOCTYPE html>\n");
        html.open_tag("html").attr("lang", "en").close_open();

        // Head
        html.open_tag("head").close_open();
        html.void_element("meta", &[("charset", "UTF-8")]);
        html.void_element(
            "meta",
            &[
                ("name", "viewport"),
                ("content", "width=device-width, initial-scale=1.0"),
            ],
        );
        html.element("title", &[], &self.title);
        html.open_tag("style").close_open();
        html.raw(GLOBAL_STYLES);
        html.close_tag("style");
        html.close_tag("head");

        // Body
        html.open_tag("body").close_open();

        // Navigation
        html.open_tag("nav").close_open();
        html.open_tag("div").attr("class", "nav-content").close_open();
        html.open_tag("a")
            .attr("href", "/")
            .attr("class", "logo")
            .close_open();
        html.text("🦀 Ferric SSR");
        html.close_tag("a");
        html.open_tag("div").attr("class", "nav-links").close_open();
        html.element("a", &[("href", "/")], "Home");
        html.element("a", &[("href", "/about")], "About");
        html.element("a", &[("href", "/counter")], "Counter");
        html.element("a", &[("href", "/todos")], "Todos");
        html.element("a", &[("href", "/user")], "User");
        html.element("a", &[("href", "/api/data")], "API");
        html.close_tag("div");
        html.close_tag("div");
        html.close_tag("nav");

        // Main content
        html.open_tag("main").close_open();
        html.raw(&content);
        html.close_tag("main");

        // Footer
        html.open_tag("footer").close_open();
        html.element("p", &[], "Built with Ferric SSR • Rust + Hyper");
        html.close_tag("footer");

        html.close_tag("body");
        html.close_tag("html");

        ctx.exit();
        Ok(html.finish())
    }
}

const GLOBAL_STYLES: &str = r#"
* {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
    line-height: 1.6;
    color: #1a1a2e;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    min-height: 100vh;
}

nav {
    background: rgba(255, 255, 255, 0.1);
    backdrop-filter: blur(10px);
    padding: 1rem 2rem;
    position: sticky;
    top: 0;
    z-index: 100;
}

.nav-content {
    max-width: 1200px;
    margin: 0 auto;
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.logo {
    font-size: 1.5rem;
    font-weight: bold;
    color: white;
    text-decoration: none;
}

.nav-links {
    display: flex;
    gap: 1.5rem;
}

.nav-links a {
    color: rgba(255, 255, 255, 0.9);
    text-decoration: none;
    font-weight: 500;
    transition: color 0.2s;
}

.nav-links a:hover {
    color: white;
    text-decoration: underline;
}

main {
    max-width: 800px;
    margin: 2rem auto;
    padding: 2rem;
    background: white;
    border-radius: 16px;
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
}

footer {
    text-align: center;
    padding: 2rem;
    color: rgba(255, 255, 255, 0.8);
}

h1 {
    font-size: 2.5rem;
    color: #1a1a2e;
    margin-bottom: 1rem;
}

h2 {
    font-size: 1.5rem;
    color: #4a4a6a;
    margin-bottom: 0.5rem;
}

h3 {
    font-size: 1.25rem;
    color: #667eea;
    margin-bottom: 0.5rem;
}

p {
    color: #666;
    margin-bottom: 1rem;
}

code {
    background: #f0f0f5;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
    font-family: 'SF Mono', Monaco, 'Courier New', monospace;
    font-size: 0.9em;
}

pre {
    background: #1a1a2e;
    color: #e0e0e0;
    padding: 1rem;
    border-radius: 8px;
    overflow-x: auto;
    margin: 1rem 0;
}

pre code {
    background: none;
    padding: 0;
}
"#;

